use crate::arena::{EndReason, MatchResult};
use crate::game::{Game, TurnReport};
use crate::grid::{TERRAIN_CHAR, coord};
use crate::proto::{init_lines, turn_lines};
use std::fmt::Write;

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Detail {
	/// Nothing but the result line.
	Quiet,
	/// Board, answers, errors and a hash of each player's input per turn.
	Digest,
	/// Every protocol line the players were given, verbatim.
	Full,
}

pub struct Trace {
	pub text: String,
	pub detail: Detail,
	seen_errors: usize,
}

impl Trace {
	pub fn new(detail: Detail) -> Self {
		Trace {
			text: String::new(),
			detail,
			seen_errors: 0,
		}
	}

	pub fn header(&mut self, seed: i64, game: &Game) {
		if self.detail == Detail::Quiet {
			return;
		}
		let _ = writeln!(self.text, "seed {seed}");
		let _ = writeln!(self.text, "league {}", game.league);
		let mut lines = String::new();
		for player in 0..2 {
			init_lines(&game.grid, player, &mut lines);
			self.protocol(&format!("init{player}"), &lines);
		}
		for player in 0..2 {
			turn_lines(game, player, &mut lines);
			self.protocol(&format!("frame{player}"), &lines);
		}
	}

	pub fn turn(&mut self, game: &Game, report: &TurnReport, answers: &[String; 2], turn: usize) {
		if self.detail == Detail::Quiet {
			return;
		}
		let _ = writeln!(self.text, "turn {turn}");
		let mut lines = String::new();
		for player in 0..2 {
			turn_lines(game, player, &mut lines);
			self.protocol(&format!("frame{player}"), &lines);
		}
		for (player, answer) in answers.iter().enumerate() {
			let _ = writeln!(self.text, "out{player} {answer}");
			if let Some(message) = &game.players[player].message {
				let _ = writeln!(self.text, "say{player} {message}");
			}
		}
		for line in &game.summary[self.seen_errors..] {
			let _ = writeln!(self.text, "err {line}");
		}
		self.seen_errors = game.summary.len();
		let _ = writeln!(
			self.text,
			"score {} {}",
			game.players[0].score, game.players[1].score
		);
		if !report.inked.is_empty() {
			let inked: Vec<String> = report.inked.iter().map(|id| id.to_string()).collect();
			let _ = writeln!(self.text, "inked {}", inked.join(" "));
		}
		self.text.push_str(&render(game));
	}

	pub fn footer(&mut self, game: &Game, result: &MatchResult) {
		if self.detail == Detail::Quiet {
			return;
		}
		let _ = writeln!(
			self.text,
			"end reason {} score {} {} turns {} text0 {} text1 {}",
			reason_text(result.reason),
			result.score[0],
			result.score[1],
			result.turns,
			result.texts[0],
			result.texts[1]
		);
		for (key, value) in metadata(game) {
			let _ = writeln!(self.text, "meta {key} {value}");
		}
	}

	fn protocol(&mut self, label: &str, text: &str) {
		match self.detail {
			Detail::Quiet => {}
			Detail::Digest => {
				let _ = writeln!(
					self.text,
					"{label} {:016x} {}",
					fnv1a(text.as_bytes()),
					text.len()
				);
			}
			Detail::Full => {
				let _ = writeln!(self.text, "{label} {} lines", text.lines().count());
				self.text.push_str(text);
			}
		}
	}
}

pub fn reason_text(reason: EndReason) -> &'static str {
	match reason {
		EndReason::GameOver => "game-over",
		EndReason::Timeout => "timeout",
		EndReason::Invalid => "invalid",
		EndReason::Broken => "broken",
	}
}

pub fn render(game: &Game) -> String {
	let grid = &game.grid;
	let mut out = String::with_capacity(grid.cells() + grid.height as usize);
	for y in 0..grid.height {
		for x in 0..grid.width {
			let tile = grid.tile(coord(x, y));
			out.push(if tile.is_town() {
				town_char(tile.town as usize)
			} else if grid.zones[tile.zone as usize].inked {
				':'
			} else {
				match tile.track {
					0 => '0',
					1 => '1',
					2 => '*',
					_ => TERRAIN_CHAR[tile.kind as usize] as char,
				}
			});
		}
		out.push('\n');
	}
	out
}

fn town_char(id: usize) -> char {
	if id < 26 {
		(b'A' + id as u8) as char
	} else {
		'#'
	}
}

pub fn metadata(game: &Game) -> Vec<(String, String)> {
	let stats = &game.stats;
	let mut out = Vec::new();
	for player in 0..2 {
		for (key, value) in [
			("tracksPlaced", stats.placed_tracks[player]),
			("tracksPlacedOnPlains", stats.on_plains[player]),
			("tracksPlacedOnRiver", stats.on_river[player]),
			("tracksPlacedOnMountains", stats.on_mountains[player]),
			("zonesInked", stats.zones_inked[player]),
			("ownTracksInkedOut", stats.own_tracks_inked_out[player]),
			("enemyTracksInkedOut", stats.enemy_tracks_inked_out[player]),
			(
				"extraTilesInConnection",
				stats.extra_tiles_in_connection[player],
			),
			("autobuildCalled", stats.autobuild_called[player]),
		] {
			out.push((format!("{key}_{player}"), value.to_string()));
		}
		out.push((
			format!("averageTrackOwnershipPercentagePerActiveConnection_{player}"),
			format!("{:.6}", stats.average_ownership(player)),
		));
	}
	out
}

pub fn fnv1a(bytes: &[u8]) -> u64 {
	let mut hash = 0xcbf2_9ce4_8422_2325u64;
	for &byte in bytes {
		hash ^= byte as u64;
		hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
	}
	hash
}

pub fn result_line(result: &MatchResult, seed: i64) -> String {
	let outcome = match result.winner {
		Some(player) => format!("p{} wins", player + 1),
		None => "draw".into(),
	};
	let mut line = format!(
		"seed {seed} score {} {} turns {} {outcome} ({})",
		result.score[0],
		result.score[1],
		result.turns,
		reason_text(result.reason)
	);
	if let (Some(player), Some(note)) = (result.culprit, &result.note) {
		let _ = write!(line, ": p{} {note}", player + 1);
	}
	line
}
