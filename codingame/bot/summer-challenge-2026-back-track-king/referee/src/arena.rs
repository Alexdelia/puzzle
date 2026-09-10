use crate::driver::{Driver, View};
use crate::game::{Game, TurnReport, validate_actions};
use std::time::Duration;

pub type TurnSink<'a> = &'a mut dyn FnMut(&Game, &TurnReport);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndReason {
	TurnLimit,
	Foul,
	NoAnswer,
	TooSlow,
}

#[derive(Clone, Debug)]
pub struct MatchResult {
	pub score: [u32; 2],
	pub winner: Option<usize>,
	pub reason: EndReason,
	pub turns: usize,
	pub culprit: Option<usize>,
	pub note: Option<String>,
	pub slowest: [Duration; 2],
}

#[derive(Clone, Copy, Debug)]
pub struct TimeLimits {
	pub first: Duration,
	pub rest: Duration,
}

impl Default for TimeLimits {
	fn default() -> Self {
		TimeLimits {
			first: Duration::from_millis(1000),
			rest: Duration::from_millis(50),
		}
	}
}

pub fn run_match(
	game: &mut Game,
	drivers: [&mut dyn Driver; 2],
	limits: Option<TimeLimits>,
	mut sink: Option<TurnSink>,
) -> MatchResult {
	let [d0, d1] = drivers;
	let mut slowest = [Duration::ZERO; 2];
	for (player, driver) in [(0usize, &mut *d0), (1, &mut *d1)] {
		if let Err(why) = driver.init(&game.map, player) {
			return verdict(game, 1 - player, EndReason::NoAnswer, why, slowest);
		}
	}

	while !game.over() {
		let mut actions = [Vec::new(), Vec::new()];
		for (player, driver) in [(0usize, &mut *d0), (1, &mut *d1)] {
			let view = view_of(game, player);
			if let Err(why) = driver.observe(&view) {
				return verdict(game, 1 - player, EndReason::NoAnswer, why, slowest);
			}
		}
		for (player, driver) in [(0usize, &mut *d0), (1, &mut *d1)] {
			let view = view_of(game, player);
			match driver
				.decide(&view)
				.and_then(|list| validate_actions(&list).map(|_| list))
			{
				Ok(list) => actions[player] = list,
				Err(why) => return verdict(game, 1 - player, EndReason::NoAnswer, why, slowest),
			}
			let took = driver.elapsed();
			slowest[player] = slowest[player].max(took);
			if let Some(limits) = limits {
				let cap = if game.state.turn == 0 {
					limits.first
				} else {
					limits.rest
				};
				if took > cap {
					let why = format!("answered in {took:?}, limit {cap:?}");
					return verdict(game, 1 - player, EndReason::TooSlow, why, slowest);
				}
			}
		}
		match game.step([&actions[0], &actions[1]]) {
			Ok(report) => {
				if let Some(f) = sink.as_deref_mut() {
					f(game, &report);
				}
			}
			Err(foul) => {
				let why = foul.reason.clone();
				return verdict(game, 1 - foul.player, EndReason::Foul, why, slowest);
			}
		}
	}

	let score = game.state.score;
	MatchResult {
		score,
		winner: match score[0].cmp(&score[1]) {
			std::cmp::Ordering::Greater => Some(0),
			std::cmp::Ordering::Less => Some(1),
			std::cmp::Ordering::Equal => None,
		},
		reason: EndReason::TurnLimit,
		turns: game.state.turn,
		culprit: None,
		note: None,
		slowest,
	}
}

fn view_of(game: &Game, player: usize) -> View<'_> {
	View {
		map: &game.map,
		state: &game.state,
		conn: &game.conn,
		player,
	}
}

fn verdict(
	game: &Game,
	winner: usize,
	reason: EndReason,
	note: String,
	slowest: [Duration; 2],
) -> MatchResult {
	MatchResult {
		score: game.state.score,
		winner: Some(winner),
		reason,
		turns: game.state.turn,
		culprit: Some(1 - winner),
		note: Some(note),
		slowest,
	}
}

pub fn wilson_ci(wins: f64, n: f64) -> (f64, f64) {
	if n == 0.0 {
		return (0.0, 1.0);
	}
	let z = 1.96;
	let p = wins / n;
	let z2 = z * z;
	let denom = 1.0 + z2 / n;
	let center = (p + z2 / (2.0 * n)) / denom;
	let half = z * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt() / denom;
	(center - half, center + half)
}

pub fn elo_diff(p: f64) -> f64 {
	let p = p.clamp(0.001, 0.999);
	-400.0 * (1.0 / p - 1.0).log10()
}
