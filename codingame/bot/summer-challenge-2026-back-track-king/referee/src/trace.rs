use crate::game::{Game, TurnReport};
use crate::map::{NO_TOWN, TERRAIN_CHAR, format_map};
use std::fmt::Write;

#[derive(Default)]
pub struct Trace {
	pub text: String,
	pub board: bool,
}

impl Trace {
	pub fn with_board(board: bool) -> Self {
		Trace {
			text: String::new(),
			board,
		}
	}

	pub fn record(&mut self, game: &Game, report: &TurnReport) {
		let _ = writeln!(
			self.text,
			"turn {} score {} {} gained {} {}",
			game.state.turn,
			game.state.score[0],
			game.state.score[1],
			report.gained[0],
			report.gained[1]
		);
		for side in 0..2 {
			let cells: Vec<String> = report.placed[side]
				.iter()
				.map(|&cell| {
					let (x, y) = game.map.xy(cell);
					format!("{x},{y}")
				})
				.collect();
			let _ = writeln!(self.text, "  side{side} placed [{}]", cells.join(" "));
			for refused in &report.skipped[side] {
				let _ = writeln!(self.text, "  side{side} refused {refused}");
			}
			if let Some(message) = &report.messages[side] {
				let _ = writeln!(self.text, "  side{side} says {message}");
			}
		}
		if !report.inked.is_empty() {
			let _ = writeln!(self.text, "  inked {:?}", report.inked);
		}
		if self.board {
			self.text.push_str(&render(game));
		}
	}
}

pub fn render(game: &Game) -> String {
	let map = &game.map;
	let mut out = String::with_capacity(map.cells() + map.height);
	for y in 0..map.height {
		for x in 0..map.width {
			let cell = map.idx(x, y);
			out.push(if map.town_at[cell] != NO_TOWN {
				(b'A' + map.town_at[cell] as u8) as char
			} else if game.state.inked[map.region_of(cell)] {
				':'
			} else {
				match game.state.owner[cell] {
					0 => '0',
					1 => '1',
					2 => '*',
					_ => TERRAIN_CHAR[map.terrain[cell] as usize] as char,
				}
			});
		}
		out.push('\n');
	}
	out
}

pub fn replay_text(
	header: &[(&str, String)],
	game: &Game,
	trace: &Trace,
	verdict: String,
) -> String {
	let mut out = String::new();
	for (key, value) in header {
		let _ = writeln!(out, "{key} {value}");
	}
	out.push_str(&format_map(&game.map));
	out.push_str(&trace.text);
	let _ = writeln!(out, "result {verdict}");
	out
}
