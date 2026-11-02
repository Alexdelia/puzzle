use crate::driver::Driver;
use crate::game::{Answer, Game, Phase};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndReason {
	/// The turn limit ended the game.
	TurnLimit,
	/// The referee deactivated a player: domination, no units left, invalid command or timeout.
	Deactivated,
}

#[derive(Clone, Debug)]
pub struct MatchResult {
	pub score: [i32; 2],
	pub winner: Option<usize>,
	pub reason: EndReason,
	pub turns: i32,
	pub frames: i32,
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

#[derive(Clone, Debug, Default)]
pub struct FrameRecord {
	pub turn: i32,
	pub asked: [bool; 2],
	pub answers: [Option<String>; 2],
	pub summary: Vec<String>,
	pub tooltips: Vec<String>,
}

pub type FrameSink<'a> = &'a mut dyn FnMut(&Game, &FrameRecord);

pub fn run_match(
	game: &mut Game,
	drivers: [&mut dyn Driver; 2],
	limits: Option<TimeLimits>,
	mut sink: Option<FrameSink>,
) -> MatchResult {
	let mut slowest = [Duration::ZERO; 2];
	let mut answered_once = [false; 2];
	let mut input_turns = 0;
	let mut notes: [Option<String>; 2] = [None, None];

	while !game.is_over() {
		let phase = game.begin_frame();
		let mut record = FrameRecord {
			turn: game.turn,
			..FrameRecord::default()
		};
		if let Phase::Ask(inputs) = &phase {
			input_turns += 1;
			let asked = phase.asked();
			record.asked = asked;
			let mut failed = [false; 2];
			for (player, text) in inputs.iter().enumerate() {
				let Some(text) = text else { continue };
				if let Err(why) = drivers[player].send(text) {
					failed[player] = true;
					notes[player] = Some(why);
				}
			}
			let mut answers: [Option<Answer>; 2] = [None, None];
			for player in (0..2).filter(|&p| asked[p] && !failed[p]) {
				match drivers[player].answer() {
					Ok(line) => {
						let took = drivers[player].elapsed();
						slowest[player] = slowest[player].max(took);
						let cap = limits.map(|l| {
							if answered_once[player] {
								l.rest
							} else {
								l.first
							}
						});
						answered_once[player] = true;
						if let Some(cap) = cap.filter(|&cap| took > cap) {
							notes[player] =
								Some(format!("answered in {took:?}, over the {cap:?} budget"));
							answers[player] = Some(Answer::Timeout);
						} else {
							record.answers[player] = Some(line.clone());
							answers[player] = Some(Answer::Line(line));
						}
					}
					Err(why) => {
						notes[player] = Some(why);
						answers[player] = Some(Answer::Timeout);
					}
				}
			}
			game.end_frame(asked, answers);
		}
		record.summary = game.summary.clone();
		record.tooltips = game.tooltips.clone();
		if let Some(sink) = sink.as_deref_mut() {
			sink(game, &record);
		}
	}

	let score = game.scores();
	let culprit = (0..2).find(|&p| !game.players[p].active);
	MatchResult {
		score,
		winner: match score[0].cmp(&score[1]) {
			std::cmp::Ordering::Greater => Some(0),
			std::cmp::Ordering::Less => Some(1),
			std::cmp::Ordering::Equal => None,
		},
		reason: if culprit.is_some() {
			EndReason::Deactivated
		} else {
			EndReason::TurnLimit
		},
		turns: input_turns,
		frames: game.turn,
		note: culprit.map(|p| {
			let why = game.players[p].deactivated_by.clone().unwrap_or_default();
			match &notes[p] {
				Some(detail) => format!("{why} ({detail})"),
				None => why,
			}
		}),
		culprit,
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
