use super::geom::{V2, v2};
use super::sim::{
	Cmd, CmdKind, MAX_THRUST, MAX_TURNS, RAD_TO_DEG, State, expand_cps, init_state, player_of,
	progress_winner, round_cg, step,
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
