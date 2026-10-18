use super::geom::{V2, v2};
use super::maps::Rng;
use super::race::{Action, BotAction, Driver, Obs, Track};
use super::sim::{
	Cmd, CmdKind, DEG_TO_RAD, MAX_ROTATE, Pod, State, TIMEOUT_TURNS, player_of, step,
};
use std::time::Instant;

pub const DEPTH: usize = 6;
pub const CP_WEIGHT: f64 = 30000.0;
pub const WIN_SCORE: f64 = 1e9;
pub const WIN_EARLIER_BONUS: f64 = 1e6;
pub const BLOCKER_CP_WEIGHT: f64 = 0.7;
pub const BLOCKER_RAM_WEIGHT: f64 = 0.3;
pub const RESTART_CHANCE: f64 = 0.05;
pub const TARGET_PROJECTION: f64 = 10000.0;

const SHIELD_GENE: i8 = -1;
const BOOST_GENE: i8 = 101;

#[derive(Clone, Copy)]
struct Gene {
	delta: f64,
	thrust: i8,
}

impl Gene {
	fn straight() -> Gene {
		Gene {
			delta: 0.0,
			thrust: 100,
		}
	}
}

#[derive(Clone, Copy)]
struct Plan {
	genes: [[Gene; DEPTH]; 2],
}

impl Plan {
	fn straight() -> Plan {
		Plan {
			genes: [[Gene::straight(); DEPTH]; 2],
		}
	}

	fn shift(&self) -> Plan {
		let mut p = *self;
		for pod in 0..2 {
			for d in 0..DEPTH - 1 {
				p.genes[pod][d] = p.genes[pod][d + 1];
			}
			p.genes[pod][DEPTH - 1] = Gene::straight();
		}
		p
	}
}

#[derive(Clone, Copy)]
pub enum Budget {
	Rollouts(usize),
	TimeMs(u64),
}

pub struct SearchBot {
	budget: Budget,
	first_turn_ms: u64,
	cps: Vec<V2>,
	expanded: Vec<V2>,
	ncp: usize,
	prev_next_cp: Option<[usize; 4]>,
	expanded_next: [usize; 4],
	timeout: [i32; 2],
	my_shield: [u8; 2],
	team_boost_used: bool,
	prev_plan: Plan,
	rng: Rng,
}

impl SearchBot {
	pub fn new(budget: Budget, seed: u64) -> Self {
		SearchBot {
			budget,
			first_turn_ms: 500,
			cps: Vec::new(),
			expanded: Vec::new(),
			ncp: 1,
			prev_next_cp: None,
			expanded_next: [1; 4],
			timeout: [TIMEOUT_TURNS; 2],
			my_shield: [0; 2],
			team_boost_used: false,
			prev_plan: Plan::straight(),
			rng: Rng::new(seed),
		}
	}

	fn reconstruct(&mut self, obs: &Obs) -> State {
		for s in &mut self.my_shield {
			if *s > 0 {
				*s -= 1;
			}
		}
		if let Some(prev) = self.prev_next_cp {
			let mut passed = [false; 2];
			for i in 0..4 {
				let d = (obs.pods[i].next_cp + self.ncp - prev[i]) % self.ncp;
				self.expanded_next[i] += d;
				if d > 0 {
					passed[player_of(i)] = true;
				}
			}
			for p in 0..2 {
				if passed[p] {
					self.timeout[p] = TIMEOUT_TURNS;
				}
				self.timeout[p] -= 1;
			}
		}
		self.prev_next_cp = Some([
			obs.pods[0].next_cp,
			obs.pods[1].next_cp,
			obs.pods[2].next_cp,
			obs.pods[3].next_cp,
		]);

		let make_pod = |i: usize, shield: u8, boosted: bool| {
			let p = &obs.pods[i];
			Pod {
				pos: v2(p.x as f64, p.y as f64),
				vel: v2(p.vx as f64, p.vy as f64),
				angle: if p.angle < 0 {
					-DEG_TO_RAD
				} else {
					p.angle as f64 * DEG_TO_RAD
				},
				next: self.expanded_next[i].min(self.expanded.len() - 1),
				shield,
				boosted,
				won: false,
			}
		};
		State {
			pods: [
				make_pod(0, self.my_shield[0], self.team_boost_used),
				make_pod(1, self.my_shield[1], self.team_boost_used),
				make_pod(2, 0, true),
				make_pod(3, 0, true),
			],
			timeout: self.timeout,
			turn: obs.turn,
		}
	}

	fn progress(&self, pod: &Pod) -> f64 {
		pod.next as f64 * CP_WEIGHT - pod.pos.dist(self.expanded[pod.next])
	}

	fn racer_index(&self, state: &State, player: usize) -> usize {
		let base = player * 2;
		if self.progress(&state.pods[base]) >= self.progress(&state.pods[base + 1]) {
			base
		} else {
			base + 1
		}
	}

	fn heuristic_cmd(&self, pod: &Pod) -> Cmd {
		let cp = self.expanded[pod.next];
		let target = cp - pod.vel * 3.0;
		let thrust = if pod.diff_angle(target).abs() > std::f64::consts::FRAC_PI_2 {
			0.0
		} else {
			100.0
		};
		Cmd {
			target,
			kind: CmdKind::Thrust(thrust),
		}
	}

	fn gene_cmd(pod: &Pod, g: Gene) -> Cmd {
		let angle = pod.angle + g.delta;
		let target = v2(
			pod.pos.x + angle.cos() * TARGET_PROJECTION,
			pod.pos.y + angle.sin() * TARGET_PROJECTION,
		);
		let kind = match g.thrust {
			SHIELD_GENE => CmdKind::Shield,
			BOOST_GENE => {
				if pod.boosted {
					CmdKind::Thrust(100.0)
				} else {
					CmdKind::Boost
				}
			}
			t => CmdKind::Thrust(t as f64),
		};
		Cmd { target, kind }
	}

	fn eval(&self, state: &State, roles: &Roles) -> f64 {
		let my_racer = &state.pods[roles.my_racer];
		let en_racer = &state.pods[roles.en_racer];
		let blocker = &state.pods[roles.my_blocker];

		let mut score = self.progress(my_racer) - self.progress(en_racer);

		let block_point = self.expanded[en_racer.next];
		score -= BLOCKER_CP_WEIGHT * blocker.pos.dist(block_point);
		score -= BLOCKER_RAM_WEIGHT * blocker.pos.dist(en_racer.pos);

		score
	}

	fn rollout(&self, base: &State, plan: &Plan, roles: &Roles) -> f64 {
		let mut st = *base;
		for d in 0..DEPTH {
			let cmds = [
				Self::gene_cmd(&st.pods[0], plan.genes[0][d]),
				Self::gene_cmd(&st.pods[1], plan.genes[1][d]),
				self.heuristic_cmd(&st.pods[2]),
				self.heuristic_cmd(&st.pods[3]),
			];
			step(&self.expanded, &mut st, &cmds);

			let my_won = st.pods[0].won || st.pods[1].won;
			let en_won = st.pods[2].won || st.pods[3].won;
			if my_won {
				return WIN_SCORE - d as f64 * WIN_EARLIER_BONUS;
			}
			if en_won || st.timeout[0] <= 0 {
				return -WIN_SCORE + d as f64 * WIN_EARLIER_BONUS;
			}
			if st.timeout[1] <= 0 {
				return WIN_SCORE - d as f64 * WIN_EARLIER_BONUS;
			}
		}
		self.eval(&st, roles)
	}

	fn random_delta(&mut self) -> f64 {
		match self.rng.below(5) {
			0 => -MAX_ROTATE,
			1 => MAX_ROTATE,
			2 => 0.0,
			_ => (self.rng.f01() * 2.0 - 1.0) * MAX_ROTATE,
		}
	}

	fn random_thrust(&mut self) -> i8 {
		match self.rng.below(12) {
			0 => 0,
			1 => 50,
			2 => SHIELD_GENE,
			3 => {
				if self.team_boost_used {
					100
				} else {
					BOOST_GENE
				}
			}
			_ => 100,
		}
	}

	fn random_plan(&mut self) -> Plan {
		let mut p = Plan::straight();
		for pod in 0..2 {
			for d in 0..DEPTH {
				p.genes[pod][d] = Gene {
					delta: self.random_delta(),
					thrust: self.random_thrust(),
				};
			}
		}
		p
	}

	fn mutate(&mut self, plan: &Plan) -> Plan {
		let mut p = *plan;
		let edits = 1 + self.rng.below(3);
		for _ in 0..edits {
			let pod = self.rng.below(2) as usize;
			let d = self.rng.below(DEPTH as u64) as usize;
			match self.rng.below(3) {
				0 => p.genes[pod][d].delta = self.random_delta(),
				1 => p.genes[pod][d].thrust = self.random_thrust(),
				_ => {
					p.genes[pod][d] = Gene {
						delta: self.random_delta(),
						thrust: self.random_thrust(),
					}
				}
			}
		}
		p
	}

	fn sanitize(&self, plan: &mut Plan) {
		if self.team_boost_used {
			for pod in 0..2 {
				for g in &mut plan.genes[pod] {
					if g.thrust == BOOST_GENE {
						g.thrust = 100;
					}
				}
			}
		}
	}

	fn search(&mut self, state: &State) -> Plan {
		let roles = Roles::assign(self, state);
		let start = Instant::now();
		let budget_ms = if state.turn == 0 {
			self.first_turn_ms
		} else {
			match self.budget {
				Budget::TimeMs(ms) => ms,
				Budget::Rollouts(_) => 0,
			}
		};

		let mut best = self.prev_plan.shift();
		self.sanitize(&mut best);
		let mut best_score = self.rollout(state, &best, &roles);

		for cand in [Plan::straight(), self.random_plan()] {
			let s = self.rollout(state, &cand, &roles);
			if s > best_score {
				best_score = s;
				best = cand;
			}
		}

		let mut done = 3usize;
		loop {
			match self.budget {
				Budget::Rollouts(n) => {
					if done >= n {
						break;
					}
				}
				Budget::TimeMs(_) => {
					if done % 64 == 0 && start.elapsed().as_millis() as u64 >= budget_ms {
						break;
					}
				}
			}
			let cand = if self.rng.chance(RESTART_CHANCE) {
				self.random_plan()
			} else {
				self.mutate(&best)
			};
			let s = self.rollout(state, &cand, &roles);
			if s > best_score {
				best_score = s;
				best = cand;
			}
			done += 1;
		}
		best
	}
}

struct Roles {
	my_racer: usize,
	my_blocker: usize,
	en_racer: usize,
}

impl Roles {
	fn assign(bot: &SearchBot, state: &State) -> Roles {
		let my_racer = bot.racer_index(state, 0);
		Roles {
			my_racer,
			my_blocker: 1 - my_racer,
			en_racer: bot.racer_index(state, 1),
		}
	}
}

impl Driver for SearchBot {
	fn init(&mut self, track: &Track) {
		self.cps = track.cps.clone();
		self.expanded = track.expanded();
		self.ncp = track.cps.len();
	}

	fn act(&mut self, obs: &Obs) -> Option<[Action; 2]> {
		let state = self.reconstruct(obs);
		let plan = self.search(&state);
		self.prev_plan = plan;

		let mut actions = [Action {
			x: 0,
			y: 0,
			act: BotAction::Thrust(0),
		}; 2];
		for pod in 0..2 {
			let g = plan.genes[pod][0];
			let cmd = Self::gene_cmd(&state.pods[pod], g);
			let act = match cmd.kind {
				CmdKind::Shield => {
					self.my_shield[pod] = 4;
					BotAction::Shield
				}
				CmdKind::Boost => {
					self.team_boost_used = true;
					BotAction::Boost
				}
				CmdKind::Thrust(t) => BotAction::Thrust(t as i64),
			};
			actions[pod] = Action {
				x: cmd.target.x.round() as i64,
				y: cmd.target.y.round() as i64,
				act,
			};
		}
		Some(actions)
	}
}
