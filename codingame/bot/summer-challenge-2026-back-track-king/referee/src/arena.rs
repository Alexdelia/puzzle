use crate::driver::Driver;
use crate::game::{ENGINE_MAX_TURNS, Game, TurnReport};
use crate::proto::{init_lines, turn_lines};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndReason {
	/// The referee ended the game: 100 turns played, or no connection left to build.
	GameOver,
	/// A player answered too late, or not at all.
	Timeout,
	/// A player sent something the official parser rejects.
	Invalid,
	/// The bot process could not be talked to.
	Broken,
}

#[derive(Clone, Debug)]
pub struct MatchResult {
	pub score: [i32; 2],
	pub winner: Option<usize>,
	pub reason: EndReason,
	pub turns: i32,
	pub culprit: Option<usize>,
	pub note: Option<String>,
	pub texts: [String; 2],
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

pub type TurnSink<'a> = &'a mut dyn FnMut(&Game, &TurnReport, &[String; 2], usize);

pub fn run_match(
	game: &mut Game,
	drivers: [&mut dyn Driver; 2],
	limits: Option<TimeLimits>,
	mut sink: Option<TurnSink>,
) -> MatchResult {
	let [d0, d1] = drivers;
	let mut slowest = [Duration::ZERO; 2];
	let mut frames = [String::new(), String::new()];
	let mut answers = [String::new(), String::new()];

	for (player, driver) in [(0usize, &mut *d0), (1, &mut *d1)] {
		init_lines(&game.grid, player, &mut frames[player]);
		if let Err(why) = driver.send(&frames[player]) {
			return verdict(game, EndReason::Broken, Some(player), Some(why), slowest);
		}
	}

	let mut turn = 1;
	while turn <= ENGINE_MAX_TURNS && !game.ended && game.active_players() > 0 {
		game.reset_turn_data();
		answers[0].clear();
		answers[1].clear();
		let mut silent: [Option<(EndReason, String)>; 2] = [None, None];

		for player in 0..2 {
			if !game.players[player].active {
				continue;
			}
			turn_lines(game, player, &mut frames[player]);
			let driver = if player == 0 { &mut *d0 } else { &mut *d1 };
			if let Err(why) = driver.send(&frames[player]) {
				silent[player] = Some((EndReason::Broken, why));
			}
		}

		for player in 0..2 {
			if !game.players[player].active || silent[player].is_some() {
				continue;
			}
			let driver = if player == 0 { &mut *d0 } else { &mut *d1 };
			match driver.answer() {
				Ok(line) => answers[player] = line,
				Err(why) => {
					let reason = if why.contains("no answer within") {
						EndReason::Timeout
					} else {
						EndReason::Broken
					};
					silent[player] = Some((reason, why));
					continue;
				}
			}
			let took = driver.elapsed();
			slowest[player] = slowest[player].max(took);
			if let Some(limits) = limits {
				let cap = if turn == 1 { limits.first } else { limits.rest };
				if took > cap {
					silent[player] = Some((
						EndReason::Timeout,
						format!("answered in {took:?}, over the {cap:?} budget"),
					));
				}
			}
		}

		for player in 0..2 {
			if !game.players[player].active {
				continue;
			}
			if silent[player].is_some() {
				game.timed_out(player);
			} else {
				game.take_commands(player, &answers[player]);
			}
		}

		let report = game.perform_update();
		if game.active_players() < 2 {
			game.ended = true;
		}
		record_turn(&mut sink, game, &report, &answers, turn);

		if let Some(player) = (0..2).find(|&player| silent[player].is_some()) {
			let (reason, why) = silent[player].clone().unwrap();
			return verdict(game, reason, Some(player), Some(why), slowest);
		}
		if let Some(player) = (0..2).find(|&player| !game.players[player].active) {
			let why = game.players[player]
				.deactivated_by
				.clone()
				.unwrap_or_default();
			return verdict(game, EndReason::Invalid, Some(player), Some(why), slowest);
		}
		turn += 1;
	}

	verdict(game, EndReason::GameOver, None, None, slowest)
}

fn record_turn(
	sink: &mut Option<TurnSink>,
	game: &Game,
	report: &TurnReport,
	answers: &[String; 2],
	turn: i32,
) {
	if let Some(record) = sink.as_deref_mut() {
		record(game, report, answers, turn as usize);
	}
}

fn verdict(
	game: &mut Game,
	reason: EndReason,
	culprit: Option<usize>,
	note: Option<String>,
	slowest: [Duration; 2],
) -> MatchResult {
	let texts = game.on_end();
	let score = [game.players[0].score, game.players[1].score];
	MatchResult {
		winner: match score[0].cmp(&score[1]) {
			std::cmp::Ordering::Greater => Some(0),
			std::cmp::Ordering::Less => Some(1),
			std::cmp::Ordering::Equal => None,
		},
		score,
		reason,
		turns: game.turn,
		culprit,
		note,
		texts,
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
