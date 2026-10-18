use crate::game::Game;
use crate::grid::{Grid, desired_text};
use std::fmt::Write;

/// `Serializer.serializeGlobalInfoFor`
pub fn init_lines(grid: &Grid, player: usize, out: &mut String) {
	out.clear();
	let _ = writeln!(out, "{player}\n{}\n{}", grid.width, grid.height);
	for tile in &grid.tiles {
		let _ = writeln!(out, "{} {}", tile.zone, tile.kind);
	}
	let _ = writeln!(out, "{}", grid.towns.len());
	for town in &grid.towns {
		let _ = writeln!(
			out,
			"{} {} {} {}",
			town.id,
			town.coord.x,
			town.coord.y,
			desired_text(&town.desired)
		);
	}
}

/// `Serializer.serializeFrameInfoFor`
pub fn turn_lines(game: &Game, player: usize, out: &mut String) {
	out.clear();
	let _ = writeln!(
		out,
		"{}\n{}",
		game.players[player].score,
		game.players[1 - player].score
	);
	let mut connections: Vec<String> = Vec::new();
	for tile in &game.grid.tiles {
		let zone = &game.grid.zones[tile.zone as usize];
		let _ = write!(
			out,
			"{} {} {} ",
			tile.track,
			zone.instability,
			u8::from(zone.inked)
		);
		if tile.connections.is_empty() {
			out.push('x');
		} else {
			connections.clear();
			connections.extend(
				tile.connections
					.iter()
					.map(|(from, to)| format!("{from}-{to}")),
			);
			connections.sort_unstable();
			for (i, pair) in connections.iter().enumerate() {
				if i > 0 {
					out.push(',');
				}
				out.push_str(pair);
			}
		}
		out.push('\n');
	}
}
