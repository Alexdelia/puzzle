use crate::game::Game;
use crate::grid::{TERRAIN_CHAR, coord};

pub const SIDE0: char = 'o';
pub const SIDE1: char = 'x';
pub const NEUTRAL: char = '*';
pub const INKED: char = ':';

pub fn town_char(id: usize) -> char {
	if id < 26 {
		(b'A' + id as u8) as char
	} else {
		'#'
	}
}

pub fn owner_char(owner: i8) -> Option<char> {
	match owner {
		0 => Some(SIDE0),
		1 => Some(SIDE1),
		2 => Some(NEUTRAL),
		_ => None,
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
				INKED
			} else {
				owner_char(tile.track).unwrap_or(TERRAIN_CHAR[tile.kind as usize] as char)
			});
		}
		out.push('\n');
	}
	out
}

pub fn as_seen_by(board: &str, me: usize) -> String {
	if me == 0 {
		return board.to_string();
	}
	board
		.chars()
		.map(|glyph| match glyph {
			SIDE0 => SIDE1,
			SIDE1 => SIDE0,
			other => other,
		})
		.collect()
}
