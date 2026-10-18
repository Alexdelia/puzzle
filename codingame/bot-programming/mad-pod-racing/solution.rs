#![allow(dead_code)]

use mpr::bot_search::{Budget, SearchBot};
use mpr::cgio::run_cg_bot;

fn main() {
	run_cg_bot(SearchBot::new(Budget::TimeMs(65), 0x5EED));
}

mod mpr {
	pub mod bot_search {
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
	}
	pub mod bot_simple {
		use super::geom::{V2, v2};
		use super::race::{Action, BotAction, Driver, Obs, PodObs, Track};

		pub const DRIFT_COMPENSATION: f64 = 3.0;
		pub const BRAKE_DIST: f64 = 1200.0;
		pub const BRAKE_MIN_FACTOR: f64 = 0.3;
		pub const GIVE_UP_ANGLE: f64 = 90.0;
		pub const BOOST_MIN_DIST: f64 = 4500.0;
		pub const BOOST_MAX_ANGLE: f64 = 3.0;
		pub const SHIELD_DIST: f64 = 820.0;
		pub const SHIELD_MIN_CLOSING: f64 = 250.0;

		pub struct SimpleBot {
			cps: Vec<V2>,
		}

		impl SimpleBot {
			pub fn new() -> Self {
				SimpleBot { cps: Vec::new() }
			}
		}

		impl Default for SimpleBot {
			fn default() -> Self {
				Self::new()
			}
		}

		fn norm_deg180(mut a: f64) -> f64 {
			while a > 180.0 {
				a -= 360.0;
			}
			while a < -180.0 {
				a += 360.0;
			}
			a
		}

		fn pos_of(p: &PodObs) -> V2 {
			v2(p.x as f64, p.y as f64)
		}

		fn vel_of(p: &PodObs) -> V2 {
			v2(p.vx as f64, p.vy as f64)
		}

		fn next_turn_pos(p: &PodObs) -> V2 {
			pos_of(p) + vel_of(p)
		}

		impl SimpleBot {
			fn pod_action(&self, obs: &Obs, idx: usize) -> Action {
				let me = &obs.pods[idx];
				let pos = pos_of(me);
				let vel = vel_of(me);
				let cp = self.cps[me.next_cp];

				let target = cp - vel * DRIFT_COMPENSATION;
				let dist_cp = pos.dist(cp);

				let desired_deg = (target.y - pos.y).atan2(target.x - pos.x).to_degrees();
				let rel = if obs.turn == 0 || me.angle < 0 {
					0.0
				} else {
					norm_deg180(desired_deg - me.angle as f64)
				};

				let mut act = if rel.abs() > GIVE_UP_ANGLE {
					BotAction::Thrust(0)
				} else {
					let angle_factor = 1.0 - (rel.abs() / GIVE_UP_ANGLE);
					let dist_factor = if dist_cp > BRAKE_DIST {
						1.0
					} else {
						BRAKE_MIN_FACTOR + (1.0 - BRAKE_MIN_FACTOR) * (dist_cp / BRAKE_DIST)
					};
					let f = angle_factor.min(dist_factor.max(angle_factor * dist_factor));
					BotAction::Thrust((100.0 * f).round() as i64)
				};

				if (obs.turn == 0 || rel.abs() < BOOST_MAX_ANGLE) && dist_cp > BOOST_MIN_DIST {
					act = BotAction::Boost;
				}

				for enemy in &obs.pods[2..4] {
					let my_next = next_turn_pos(me);
					let enemy_next = next_turn_pos(enemy);
					let closing = (vel - vel_of(enemy)).norm();
					if my_next.dist(enemy_next) < SHIELD_DIST && closing > SHIELD_MIN_CLOSING {
						act = BotAction::Shield;
					}
				}

				Action {
					x: target.x.round() as i64,
					y: target.y.round() as i64,
					act,
				}
			}
		}

		impl Driver for SimpleBot {
			fn init(&mut self, track: &Track) {
				self.cps = track.cps.clone();
			}

			fn act(&mut self, obs: &Obs) -> Option<[Action; 2]> {
				Some([self.pod_action(obs, 0), self.pod_action(obs, 1)])
			}
		}
	}
	pub mod cgio {
		use super::geom::v2;
		use super::race::{Driver, Obs, PodObs, Track};
		use std::io::{BufRead, Write};

		fn read_line(input: &mut impl BufRead) -> Option<String> {
			let mut line = String::new();
			if input.read_line(&mut line).ok()? == 0 {
				return None;
			}
			Some(line)
		}

		fn read_ints(input: &mut impl BufRead) -> Option<Vec<i64>> {
			let line = read_line(input)?;
			Some(
				line.split_whitespace()
					.filter_map(|t| t.parse().ok())
					.collect(),
			)
		}

		pub fn read_track(input: &mut impl BufRead) -> Option<Track> {
			let laps = read_ints(input)?[0] as usize;
			let ncp = read_ints(input)?[0] as usize;
			let mut cps = Vec::with_capacity(ncp);
			for _ in 0..ncp {
				let c = read_ints(input)?;
				cps.push(v2(c[0] as f64, c[1] as f64));
			}
			Some(Track::new(cps, laps))
		}

		pub fn read_obs(input: &mut impl BufRead, turn: usize) -> Option<Obs> {
			let mut pods = [PodObs {
				x: 0,
				y: 0,
				vx: 0,
				vy: 0,
				angle: 0,
				next_cp: 0,
			}; 4];
			for pod in &mut pods {
				let v = read_ints(input)?;
				if v.len() < 6 {
					return None;
				}
				*pod = PodObs {
					x: v[0],
					y: v[1],
					vx: v[2],
					vy: v[3],
					angle: v[4],
					next_cp: v[5] as usize,
				};
			}
			Some(Obs { turn, pods })
		}

		pub fn run_cg_bot(mut driver: impl Driver) {
			let stdin = std::io::stdin();
			let mut input = stdin.lock();
			let stdout = std::io::stdout();
			let mut out = stdout.lock();

			let track = read_track(&mut input).expect("init input");
			driver.init(&track);

			let mut turn = 0;
			while let Some(obs) = read_obs(&mut input, turn) {
				let actions = driver.act(&obs).expect("driver action");
				writeln!(out, "{}\n{}", actions[0], actions[1]).unwrap();
				out.flush().unwrap();
				turn += 1;
			}
		}
	}
	pub mod geom {
		use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

		#[derive(Clone, Copy, Debug, Default, PartialEq)]
		pub struct V2 {
			pub x: f64,
			pub y: f64,
		}

		pub const fn v2(x: f64, y: f64) -> V2 {
			V2 { x, y }
		}

		impl V2 {
			pub fn dot(self, o: V2) -> f64 {
				self.x * o.x + self.y * o.y
			}

			pub fn norm2(self) -> f64 {
				self.x * self.x + self.y * self.y
			}

			pub fn norm(self) -> f64 {
				self.norm2().sqrt()
			}

			pub fn dist2(self, o: V2) -> f64 {
				let x = o.x - self.x;
				let y = o.y - self.y;
				x * x + y * y
			}

			pub fn dist(self, o: V2) -> f64 {
				self.dist2(o).sqrt()
			}
		}

		impl Add for V2 {
			type Output = V2;
			fn add(self, o: V2) -> V2 {
				v2(self.x + o.x, self.y + o.y)
			}
		}

		impl AddAssign for V2 {
			fn add_assign(&mut self, o: V2) {
				self.x += o.x;
				self.y += o.y;
			}
		}

		impl Sub for V2 {
			type Output = V2;
			fn sub(self, o: V2) -> V2 {
				v2(self.x - o.x, self.y - o.y)
			}
		}

		impl SubAssign for V2 {
			fn sub_assign(&mut self, o: V2) {
				self.x -= o.x;
				self.y -= o.y;
			}
		}

		impl Mul<f64> for V2 {
			type Output = V2;
			fn mul(self, s: f64) -> V2 {
				v2(self.x * s, self.y * s)
			}
		}

		impl Neg for V2 {
			type Output = V2;
			fn neg(self) -> V2 {
				v2(-self.x, -self.y)
			}
		}
	}
	pub mod maps {
		use super::geom::{V2, v2};

		pub const CHECKPOINT_JITTER: i64 = 30;

		pub const AGADE_MAPS: [&[V2]; 13] = [
			&[
				v2(12460.0, 1350.0),
				v2(10540.0, 5980.0),
				v2(3580.0, 5180.0),
				v2(13580.0, 7600.0),
			],
			&[
				v2(3600.0, 5280.0),
				v2(13840.0, 5080.0),
				v2(10680.0, 2280.0),
				v2(8700.0, 7460.0),
				v2(7200.0, 2160.0),
			],
			&[
				v2(4560.0, 2180.0),
				v2(7350.0, 4940.0),
				v2(3320.0, 7230.0),
				v2(14580.0, 7700.0),
				v2(10560.0, 5060.0),
				v2(13100.0, 2320.0),
			],
			&[v2(5010.0, 5260.0), v2(11480.0, 6080.0), v2(9100.0, 1840.0)],
			&[
				v2(14660.0, 1410.0),
				v2(3450.0, 7220.0),
				v2(9420.0, 7240.0),
				v2(5970.0, 4240.0),
			],
			&[
				v2(3640.0, 4420.0),
				v2(8000.0, 7900.0),
				v2(13300.0, 5540.0),
				v2(9560.0, 1400.0),
			],
			&[
				v2(4100.0, 7420.0),
				v2(13500.0, 2340.0),
				v2(12940.0, 7220.0),
				v2(5640.0, 2580.0),
			],
			&[
				v2(14520.0, 7780.0),
				v2(6320.0, 4290.0),
				v2(7800.0, 860.0),
				v2(7660.0, 5970.0),
				v2(3140.0, 7540.0),
				v2(9520.0, 4380.0),
			],
			&[
				v2(10040.0, 5970.0),
				v2(13920.0, 1940.0),
				v2(8020.0, 3260.0),
				v2(2670.0, 7020.0),
			],
			&[v2(7500.0, 6940.0), v2(6000.0, 5360.0), v2(11300.0, 2820.0)],
			&[
				v2(4060.0, 4660.0),
				v2(13040.0, 1900.0),
				v2(6560.0, 7840.0),
				v2(7480.0, 1360.0),
				v2(12700.0, 7100.0),
			],
			&[
				v2(3020.0, 5190.0),
				v2(6280.0, 7760.0),
				v2(14100.0, 7760.0),
				v2(13880.0, 1220.0),
				v2(10240.0, 4920.0),
				v2(6100.0, 2200.0),
			],
			&[
				v2(10323.0, 3366.0),
				v2(11203.0, 5425.0),
				v2(7259.0, 6656.0),
				v2(5425.0, 2838.0),
			],
		];

		#[derive(Clone)]
		pub struct Rng(pub u64);

		impl Rng {
			pub fn new(seed: u64) -> Self {
				Rng(seed)
			}

			pub fn next_u64(&mut self) -> u64 {
				self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
				let mut z = self.0;
				z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
				z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
				z ^ (z >> 31)
			}

			pub fn below(&mut self, n: u64) -> u64 {
				self.next_u64() % n
			}

			pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
				lo + self.below((hi - lo + 1) as u64) as i64
			}

			pub fn f01(&mut self) -> f64 {
				(self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
			}

			pub fn chance(&mut self, p: f64) -> bool {
				self.f01() < p
			}
		}

		pub fn agade_map(rng: &mut Rng) -> Vec<V2> {
			let base = AGADE_MAPS[rng.below(AGADE_MAPS.len() as u64) as usize];
			let mut cps: Vec<V2> = base
				.iter()
				.map(|c| {
					v2(
						c.x + rng.range(-CHECKPOINT_JITTER, CHECKPOINT_JITTER) as f64,
						c.y + rng.range(-CHECKPOINT_JITTER, CHECKPOINT_JITTER) as f64,
					)
				})
				.collect();
			for i in (1..cps.len()).rev() {
				let j = rng.below(i as u64 + 1) as usize;
				cps.swap(i, j);
			}
			cps
		}

		pub fn random_map(rng: &mut Rng) -> Vec<V2> {
			let n = rng.range(3, 8) as usize;
			let mut cps: Vec<V2> = Vec::with_capacity(n);
			while cps.len() < n {
				let c = v2(rng.range(1500, 14500) as f64, rng.range(1000, 8000) as f64);
				if cps.iter().all(|o| o.dist(c) > 2000.0) {
					cps.push(c);
				}
			}
			cps
		}

		pub fn parse_map(spec: &str) -> Option<Vec<V2>> {
			let mut cps = Vec::new();
			for part in spec.split(';') {
				let part = part.trim();
				if part.is_empty() {
					continue;
				}
				let mut nums = part
					.split(|c: char| c == ',' || c.is_whitespace())
					.filter(|s| !s.is_empty());
				let x: f64 = nums.next()?.parse().ok()?;
				let y: f64 = nums.next()?.parse().ok()?;
				cps.push(v2(x, y));
			}
			if cps.len() >= 2 { Some(cps) } else { None }
		}

		pub fn format_map(cps: &[V2]) -> String {
			cps.iter()
				.map(|c| format!("{} {}", c.x as i64, c.y as i64))
				.collect::<Vec<_>>()
				.join(";")
		}
	}
	pub mod race {
		use super::geom::{V2, v2};
		use super::sim::{
			Cmd, CmdKind, MAX_THRUST, MAX_TURNS, RAD_TO_DEG, State, expand_cps, init_state,
			player_of, progress_winner, round_cg, step,
		};

		#[derive(Clone, Debug)]
		pub struct Track {
			pub cps: Vec<V2>,
			pub laps: usize,
		}

		impl Track {
			pub fn new(cps: Vec<V2>, laps: usize) -> Self {
				Track { cps, laps }
			}

			pub fn expanded(&self) -> Vec<V2> {
				expand_cps(&self.cps, self.laps)
			}
		}

		#[derive(Clone, Copy, Debug)]
		pub struct PodObs {
			pub x: i64,
			pub y: i64,
			pub vx: i64,
			pub vy: i64,
			pub angle: i64,
			pub next_cp: usize,
		}

		#[derive(Clone, Copy, Debug)]
		pub struct Obs {
			pub turn: usize,
			pub pods: [PodObs; 4],
		}

		#[derive(Clone, Copy, Debug, PartialEq)]
		pub enum BotAction {
			Thrust(i64),
			Boost,
			Shield,
		}

		#[derive(Clone, Copy, Debug, PartialEq)]
		pub struct Action {
			pub x: i64,
			pub y: i64,
			pub act: BotAction,
		}

		impl Action {
			pub fn to_cmd(self) -> Cmd {
				Cmd {
					target: v2(self.x as f64, self.y as f64),
					kind: match self.act {
						BotAction::Thrust(t) => CmdKind::Thrust(t as f64),
						BotAction::Boost => CmdKind::Boost,
						BotAction::Shield => CmdKind::Shield,
					},
				}
			}

			pub fn valid(self) -> bool {
				match self.act {
					BotAction::Thrust(t) => (0..=MAX_THRUST).contains(&t),
					_ => true,
				}
			}
		}

		impl std::fmt::Display for Action {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match self.act {
					BotAction::Thrust(t) => write!(f, "{} {} {}", self.x, self.y, t),
					BotAction::Boost => write!(f, "{} {} BOOST", self.x, self.y),
					BotAction::Shield => write!(f, "{} {} SHIELD", self.x, self.y),
				}
			}
		}

		pub trait Driver {
			fn init(&mut self, _track: &Track) {}
			fn act(&mut self, obs: &Obs) -> Option<[Action; 2]>;
		}

		pub fn pod_obs(state: &State, pod: usize, ncp_raw: usize) -> PodObs {
			let p = &state.pods[pod];
			let deg = round_cg(p.angle * RAD_TO_DEG) as i64;
			let angle = if state.turn == 0 {
				deg
			} else {
				deg.rem_euclid(360)
			};
			PodObs {
				x: p.pos.x as i64,
				y: p.pos.y as i64,
				vx: p.vel.x as i64,
				vy: p.vel.y as i64,
				angle,
				next_cp: p.next % ncp_raw,
			}
		}

		pub fn build_obs(state: &State, player: usize, ncp_raw: usize) -> Obs {
			let order = if player == 0 {
				[0, 1, 2, 3]
			} else {
				[2, 3, 0, 1]
			};
			Obs {
				turn: state.turn,
				pods: order.map(|i| pod_obs(state, i, ncp_raw)),
			}
		}

		#[derive(Clone, Copy, Debug, PartialEq)]
		pub enum EndReason {
			Finished,
			Timeout,
			InvalidAction,
			ProgressTiebreak,
		}

		#[derive(Clone, Copy, Debug)]
		pub struct RaceResult {
			pub winner: usize,
			pub reason: EndReason,
			pub turns: usize,
		}

		pub fn run_race(
			track: &Track,
			drivers: [&mut dyn Driver; 2],
			mut on_turn: Option<&mut dyn FnMut(&State, &[Cmd; 4])>,
		) -> RaceResult {
			let cps = track.expanded();
			let ncp_raw = track.cps.len();
			let mut state = init_state(&cps);
			let [d0, d1] = drivers;
			d0.init(track);
			d1.init(track);

			for _ in 0..MAX_TURNS {
				let mut actions = [[Action {
					x: 0,
					y: 0,
					act: BotAction::Thrust(0),
				}; 2]; 2];
				for (player, driver) in [(0, &mut *d0), (1, &mut *d1)] {
					let obs = build_obs(&state, player, ncp_raw);
					match driver.act(&obs) {
						Some(acts) if acts.iter().all(|a| a.valid()) => actions[player] = acts,
						_ => {
							return RaceResult {
								winner: 1 - player,
								reason: EndReason::InvalidAction,
								turns: state.turn,
							};
						}
					}
				}
				let cmds = [
					actions[0][0].to_cmd(),
					actions[0][1].to_cmd(),
					actions[1][0].to_cmd(),
					actions[1][1].to_cmd(),
				];
				step(&cps, &mut state, &cmds);
				if let Some(f) = on_turn.as_deref_mut() {
					f(&state, &cmds);
				}
				if let Some((winner, reason)) = check_outcome(&state) {
					return RaceResult {
						winner,
						reason,
						turns: state.turn,
					};
				}
			}

			RaceResult {
				winner: progress_winner(&cps, &state),
				reason: EndReason::ProgressTiebreak,
				turns: MAX_TURNS,
			}
		}

		pub fn check_outcome(state: &State) -> Option<(usize, EndReason)> {
			if state.timeout[0] <= 0 {
				return Some((1, EndReason::Timeout));
			}
			if state.timeout[1] <= 0 {
				return Some((0, EndReason::Timeout));
			}
			for i in 0..4 {
				if state.pods[i].won {
					return Some((player_of(i), EndReason::Finished));
				}
			}
			None
		}
	}
	pub mod sim {
		use super::geom::{V2, v2};

		pub const WIDTH: f64 = 16000.0;
		pub const HEIGHT: f64 = 9000.0;
		pub const POD_RADIUS: f64 = 400.0;
		pub const POD_RSQ: f64 = 800.0 * 800.0;
		pub const CP_RADIUS: f64 = 600.0;
		pub const CP_RSQ: f64 = 600.0 * 600.0;
		pub const FRICTION: f64 = 0.85;
		pub const MAX_ROTATE: f64 = std::f64::consts::PI / 10.0;
		pub const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;
		pub const RAD_TO_DEG: f64 = 180.0 / std::f64::consts::PI;
		pub const TAU: f64 = std::f64::consts::TAU;
		pub const SEPARATION_EPSILON: f64 = 0.00001;
		pub const MIN_IMPULSE: f64 = 120.0;
		pub const BOOST_THRUST: f64 = 650.0;
		pub const USED_BOOST_THRUST: f64 = 200.0;
		pub const MAX_THRUST: i64 = 200;
		pub const SHIELD_TIMER: u8 = 4;
		pub const TIMEOUT_TURNS: i32 = 100;
		pub const MAX_TURNS: usize = 500;
		pub const NO_COLLISION: f64 = 10.0;

		#[derive(Clone, Copy, Debug, PartialEq)]
		pub struct Pod {
			pub pos: V2,
			pub vel: V2,
			pub angle: f64,
			pub next: usize,
			pub shield: u8,
			pub boosted: bool,
			pub won: bool,
		}

		#[derive(Clone, Copy, Debug, PartialEq)]
		pub enum CmdKind {
			Thrust(f64),
			Boost,
			Shield,
		}

		#[derive(Clone, Copy, Debug, PartialEq)]
		pub struct Cmd {
			pub target: V2,
			pub kind: CmdKind,
		}

		#[derive(Clone, Copy, Debug, PartialEq)]
		pub struct State {
			pub pods: [Pod; 4],
			pub timeout: [i32; 2],
			pub turn: usize,
		}

		#[derive(Clone, Copy, Debug, Default)]
		pub struct StepEvents {
			pub bounces: u32,
		}

		pub fn player_of(pod: usize) -> usize {
			pod / 2
		}

		pub fn round_cg(x: f64) -> f64 {
			(x + 0.5).floor()
		}

		pub fn get_angle(from: V2, to: V2) -> f64 {
			(to.y - from.y).atan2(to.x - from.x)
		}

		impl Pod {
			pub fn diff_angle(&self, target: V2) -> f64 {
				let a = get_angle(self.pos, target);
				let da = (a - self.angle) % TAU;
				(2.0 * da) % TAU - da
			}

			pub fn rotate_toward(&mut self, target: V2) {
				let mut a = get_angle(self.pos, target);
				let d = self.diff_angle(target);
				if d < -MAX_ROTATE {
					a = self.angle - MAX_ROTATE;
				}
				if d > MAX_ROTATE {
					a = self.angle + MAX_ROTATE;
				}
				self.angle = a;
			}

			pub fn rotate_free(&mut self, target: V2) {
				self.angle = 0.0;
				let mut a = self.diff_angle(target);
				while a < 0.0 {
					a += TAU;
				}
				while a > TAU {
					a -= TAU;
				}
				self.angle = a;
			}

			pub fn apply_thrust(&mut self, thrust: f64) {
				let (sin, cos) = self.angle.sin_cos();
				self.vel.x += cos * thrust;
				self.vel.y += sin * thrust;
			}

			fn end_turn(&mut self) {
				self.vel.x = (self.vel.x * FRICTION).trunc();
				self.vel.y = (self.vel.y * FRICTION).trunc();
				self.pos.x = round_cg(self.pos.x);
				self.pos.y = round_cg(self.pos.y);
				if self.shield > 0 {
					self.shield -= 1;
				}
			}
		}

		pub fn collide_time(a: &Pod, b: &Pod, rsq: f64) -> f64 {
			let p = b.pos - a.pos;
			let p_len2 = p.dot(p);
			if p_len2 <= rsq {
				return 0.0;
			}
			let v = b.vel - a.vel;
			let dot = p.dot(v);
			if dot > 0.0 {
				return NO_COLLISION;
			}
			let v_len2 = v.dot(v);
			let disc = dot * dot - v_len2 * (p_len2 - rsq);
			if disc < 0.0 {
				return NO_COLLISION;
			}
			(-dot - disc.sqrt()) / v_len2
		}

		pub fn cp_hit(seg_start: V2, seg_end: V2, cp: V2) -> bool {
			let d = seg_end - seg_start;
			let pd2 = d.dot(d);
			let mut pp = seg_start;
			if pd2 != 0.0 {
				let u = (cp - seg_start).dot(d) / pd2;
				if u > 1.0 {
					pp = seg_end;
				} else if u > 0.0 {
					pp = v2(seg_start.x + u * d.x, seg_start.y + u * d.y);
				}
			}
			let dx = pp.x - cp.x;
			let dy = pp.y - cp.y;
			dx * dx + dy * dy < CP_RSQ
		}

		fn bounce(pods: &mut [Pod; 4], i: usize, j: usize) {
			let mut a = pods[i];
			let mut b = pods[j];

			let mut normal = b.pos - a.pos;
			let dd = normal.norm();
			normal.x /= dd;
			normal.y /= dd;

			let relv = a.vel - b.vel;

			let m1 = if a.shield == SHIELD_TIMER { 0.1 } else { 1.0 };
			let m2 = if b.shield == SHIELD_TIMER { 0.1 } else { 1.0 };

			let mut force = normal.dot(relv) / (m1 + m2);
			if force < MIN_IMPULSE {
				force += MIN_IMPULSE;
			} else {
				force += force;
			}

			let impulse = normal * -force;
			a.vel.x += impulse.x * m1;
			a.vel.y += impulse.y * m1;
			b.vel.x += -impulse.x * m2;
			b.vel.y += -impulse.y * m2;

			if dd <= 800.0 {
				let gap = dd - 800.0;
				a.pos.x += normal.x * -(-gap / 2.0 + SEPARATION_EPSILON);
				a.pos.y += normal.y * -(-gap / 2.0 + SEPARATION_EPSILON);
				b.pos.x += normal.x * (-gap / 2.0 + SEPARATION_EPSILON);
				b.pos.y += normal.y * (-gap / 2.0 + SEPARATION_EPSILON);
			}

			pods[i] = a;
			pods[j] = b;
		}

		fn forward(pods: &mut [Pod; 4], t: f64) {
			for p in pods {
				p.pos.x += p.vel.x * t;
				p.pos.y += p.vel.y * t;
			}
		}

		fn pass_checkpoint(state: &mut State, total_cps: usize, pod: usize) {
			let p = &mut state.pods[pod];
			p.next += 1;
			if p.next >= total_cps {
				p.next = total_cps - 1;
				p.won = true;
			}
			state.timeout[player_of(pod)] = TIMEOUT_TURNS;
		}

		fn move_pods(cps: &[V2], state: &mut State) -> StepEvents {
			let mut events = StepEvents::default();
			let mut t = 1.0;
			let mut seg_start = [
				state.pods[0].pos,
				state.pods[1].pos,
				state.pods[2].pos,
				state.pods[3].pos,
			];
			while t > 0.0 {
				let mut first = t;
				let mut cli = 0usize;
				let mut clj = 0usize;
				for i in (1..4).rev() {
					for j in (0..i).rev() {
						let tx = collide_time(&state.pods[i], &state.pods[j], POD_RSQ);
						if tx <= first {
							first = tx;
							cli = i;
							clj = j;
						}
					}
				}
				forward(&mut state.pods, first);
				t -= first;
				if cli != clj {
					bounce(&mut state.pods, cli, clj);
					events.bounces += 1;
				}
				if t > 0.0 {
					for i in 0..4 {
						if cp_hit(seg_start[i], state.pods[i].pos, cps[state.pods[i].next]) {
							pass_checkpoint(state, cps.len(), i);
						}
					}
					seg_start = [
						state.pods[0].pos,
						state.pods[1].pos,
						state.pods[2].pos,
						state.pods[3].pos,
					];
				}
			}
			for i in 0..4 {
				state.pods[i].end_turn();
				if cp_hit(seg_start[i], state.pods[i].pos, cps[state.pods[i].next]) {
					pass_checkpoint(state, cps.len(), i);
				}
			}
			state.timeout[0] -= 1;
			state.timeout[1] -= 1;
			events
		}

		pub fn apply_cmd(turn: usize, pod: &mut Pod, cmd: &Cmd) {
			let mut thrust = match cmd.kind {
				CmdKind::Thrust(v) => v,
				CmdKind::Boost => {
					if !pod.boosted {
						pod.boosted = true;
						BOOST_THRUST
					} else {
						USED_BOOST_THRUST
					}
				}
				CmdKind::Shield => {
					pod.shield = SHIELD_TIMER;
					0.0
				}
			};
			if pod.shield > 0 {
				thrust = 0.0;
			}
			if cmd.target == pod.pos {
				return;
			}
			if turn == 0 {
				pod.rotate_free(cmd.target);
			} else {
				pod.rotate_toward(cmd.target);
			}
			pod.apply_thrust(thrust);
		}

		pub fn step(cps: &[V2], state: &mut State, cmds: &[Cmd; 4]) -> StepEvents {
			for i in 0..4 {
				apply_cmd(state.turn, &mut state.pods[i], &cmds[i]);
			}
			let events = move_pods(cps, state);
			state.turn += 1;
			events
		}

		pub const START_OFFSETS: [V2; 4] = [
			v2(500.0, -500.0),
			v2(-500.0, 500.0),
			v2(1500.0, -1500.0),
			v2(-1500.0, 1500.0),
		];

		pub fn init_state(cps: &[V2]) -> State {
			let mut dir = cps[1] - cps[0];
			let dd = cps[1].dist(cps[0]);
			dir.x /= dd;
			dir.y /= dd;

			let make_pod = |i: usize| Pod {
				pos: v2(
					round_cg(cps[0].x + dir.y * START_OFFSETS[i].x),
					round_cg(cps[0].y + dir.x * START_OFFSETS[i].y),
				),
				vel: v2(0.0, 0.0),
				angle: -DEG_TO_RAD,
				next: 1,
				shield: 0,
				boosted: false,
				won: false,
			};

			State {
				pods: [make_pod(0), make_pod(1), make_pod(2), make_pod(3)],
				timeout: [TIMEOUT_TURNS, TIMEOUT_TURNS],
				turn: 0,
			}
		}

		pub fn expand_cps(raw: &[V2], laps: usize) -> Vec<V2> {
			let mut cps = Vec::with_capacity(raw.len() * laps + 1);
			for _ in 0..laps {
				cps.extend_from_slice(raw);
			}
			cps.push(raw[0]);
			cps
		}

		pub fn progress_score(cps: &[V2], pod: &Pod) -> f64 {
			pod.next as f64 * 1000000.0 - pod.pos.dist(cps[pod.next])
		}

		pub fn progress_winner(cps: &[V2], state: &State) -> usize {
			let mut winner = 0;
			let mut best = 0.0;
			for i in 0..4 {
				let score = progress_score(cps, &state.pods[i]);
				if score > best {
					best = score;
					winner = i;
				}
			}
			player_of(winner)
		}
	}
}
