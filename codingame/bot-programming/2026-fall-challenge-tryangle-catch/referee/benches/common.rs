#![allow(dead_code)]

use tryangle::game::{Answer, DEFAULT_LEAGUE, Game, Phase};
use tryangle::mapgen;

pub const SEEDS: [i64; 3] = [1, 2, 3];

pub struct Wanderer {
	state: u64,
}

impl Wanderer {
	pub fn new(seed: u64) -> Self {
		Wanderer {
			state: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
		}
	}

	fn next(&mut self, bound: usize) -> usize {
		self.state ^= self.state << 13;
		self.state ^= self.state >> 7;
		self.state ^= self.state << 17;
		(self.state % bound as u64) as usize
	}

	pub fn commands(&mut self, game: &Game, player: usize) -> String {
		let board = &game.board;
		let mut commands = Vec::new();
		for node in 0..board.len() {
			let units = board.units[node][player];
			let around = &board.neighbors[node];
			if units > 0 && !around.is_empty() && self.next(3) > 0 {
				let to = around[self.next(around.len())];
				commands.push(format!(
					"MOVE {node} {to} {}",
					1 + self.next(units as usize)
				));
			}
		}
		for &t in &board.triangles {
			let triangle = board.triangle(t);
			if triangle.owner == Some(player) && self.next(2) == 0 {
				let [a, b, c] = triangle.nodes;
				commands.push(format!("SPAWN {a} {b} {c}"));
			}
		}
		if commands.is_empty() {
			"WAIT".into()
		} else {
			commands.join(";")
		}
	}
}

pub fn fresh(seed: i64) -> Game {
	Game::new(mapgen::generate(seed), DEFAULT_LEAGUE)
}

pub fn step(game: &mut Game, wanderers: &mut [Wanderer; 2]) {
	let phase = game.begin_frame();
	if let Phase::Ask(_) = phase {
		let asked = phase.asked();
		let answers =
			[0, 1].map(|p| asked[p].then(|| Answer::Line(wanderers[p].commands(game, p))));
		game.end_frame(asked, answers);
	}
}

pub fn after(seed: i64, frames: i32) -> Game {
	let mut game = fresh(seed);
	let mut wanderers = [Wanderer::new(seed as u64), Wanderer::new(seed as u64 + 7)];
	while game.turn < frames && !game.is_over() {
		step(&mut game, &mut wanderers);
	}
	game
}

pub fn play_out(seed: i64) -> Game {
	after(seed, i32::MAX)
}
