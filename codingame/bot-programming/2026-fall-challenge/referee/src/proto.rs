use crate::game::Game;
use std::fmt::Write;

pub const NEW_PATHS_LEAGUE: i32 = 3;

pub fn init_lines(game: &Game, out: &mut String) {
	let board = &game.board;
	writeln!(out, "{}", board.len()).unwrap();
	for (id, p) in board.points.iter().enumerate() {
		writeln!(out, "{id} {} {}", p.x, p.y).unwrap();
	}
}

pub fn linkable(game: &Game) -> Vec<(usize, usize)> {
	if game.league < NEW_PATHS_LEAGUE {
		return Vec::new();
	}
	let board = &game.board;
	let mut links = Vec::new();
	for a in 0..board.len() {
		for b in a + 1..board.len() {
			if !board.connected(a, b) && board.can_connect(a, b) {
				links.push((a, b));
			}
		}
	}
	links
}

pub fn turn_lines(game: &Game, player: usize, out: &mut String) {
	let board = &game.board;
	let opponent = 1 - player;
	let [mine, theirs] = [player, opponent].map(|p| game.players[p].score);
	writeln!(out, "{mine} {theirs}").unwrap();
	for (id, units) in board.units.iter().enumerate() {
		writeln!(out, "{id} {} {}", units[player], units[opponent]).unwrap();
	}
	let edges = board.edges();
	writeln!(out, "{}", edges.len()).unwrap();
	for (a, b) in edges {
		writeln!(out, "{a} {b}").unwrap();
	}
	writeln!(out, "{}", board.triangles.len()).unwrap();
	for &t in &board.triangles {
		let triangle = board.triangle(t);
		let [a, b, c] = triangle.nodes;
		let owner = match triangle.owner {
			None => -1,
			Some(owner) if owner == player => 0,
			Some(_) => 1,
		};
		let [me, them] = [player, opponent].map(|p| u8::from(triangle.can_capture[p]));
		writeln!(out, "{a} {b} {c} {owner} {me} {them}").unwrap();
	}
	let links = linkable(game);
	writeln!(out, "{}", links.len()).unwrap();
	for (a, b) in links {
		writeln!(out, "{a} {b}").unwrap();
	}
}

pub fn input(game: &Game, player: usize, initial: bool) -> String {
	let mut out = String::new();
	if initial {
		init_lines(game, &mut out);
	}
	turn_lines(game, player, &mut out);
	out
}
