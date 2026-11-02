mod common;

use common::{golden_logs, replay, seed_of};
use tryangle::mapgen::{generate, options};

#[test]
fn boards_match_the_official_generator() {
	for (name, log) in golden_logs() {
		let board = options(&generate(seed_of(&log)));
		assert_eq!(
			log.option("nodes"),
			Some(board.nodes.as_str()),
			"{name} nodes"
		);
		assert_eq!(
			log.option("edges"),
			Some(board.edges.as_str()),
			"{name} edges"
		);
		assert_eq!(
			log.option("units"),
			Some(board.units.as_str()),
			"{name} units"
		);
	}
}

#[test]
fn frames_match_codingame_logs() {
	for (name, log) in golden_logs() {
		let ours = replay(&log).trace.frames_text();
		let theirs = log.frames_text();
		for (line, (want, got)) in theirs.lines().zip(ours.lines()).enumerate() {
			assert_eq!(want, got, "{name} line {}", line + 1);
		}
		assert_eq!(
			theirs.lines().count(),
			ours.lines().count(),
			"{name} frame count"
		);
	}
}

#[test]
fn scores_and_tooltips_match_codingame_logs() {
	for (name, log) in golden_logs() {
		let replayed = replay(&log);
		assert_eq!(log.scores, Some(replayed.result.score), "{name} scores");
		assert_eq!(
			log.tooltips,
			replayed.trace.tooltips_line(),
			"{name} tooltips"
		);
	}
}

#[test]
fn every_recorded_answer_is_consumed() {
	for (name, log) in golden_logs() {
		let mut replayed = replay(&log);
		for (player, side) in replayed.sides.iter_mut().enumerate() {
			assert_eq!(
				side.answers.next(),
				None,
				"{name} player {player} answers left over"
			);
		}
	}
}

#[test]
fn turn_input_matches_what_the_bot_read() {
	let mut checked = 0;
	for (name, log) in golden_logs() {
		for player in 0..2 {
			let echoed = log.stdin_of(player);
			if echoed.iter().all(Vec::is_empty) {
				continue;
			}
			let replayed = replay(&log);
			let sent = &replayed.sides[player].received;
			assert_eq!(
				sent.len(),
				echoed.len(),
				"{name} player {player} input turns"
			);
			for (turn, (ours, theirs)) in sent.iter().zip(&echoed).enumerate() {
				let ours: Vec<&str> = ours.lines().collect();
				assert_eq!(
					ours,
					*theirs,
					"{name} player {player} input of turn {}",
					turn + 1
				);
			}
			checked += 1;
		}
	}
	assert!(checked > 0, "no golden log echoes its input on stderr");
}
