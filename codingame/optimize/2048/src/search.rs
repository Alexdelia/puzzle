use std::collections::HashMap;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use lib2048::err;
use lib2048::game::{Board, Move, Score, Seed};
use lib2048::io::{FILE_RESULT, read::read, write::write};

// linear congruential generator
pub const R_A: u128 = 1664525;
pub const R_C: u128 = 1013904223;
pub const R_M: u128 = 1 << 32;

type Games = HashMap<Seed, (Board, Seed, Score)>;

fn play(mut board: Board, mut seed: Seed) -> (Vec<Move>, Score) {
	let mut am: Vec<Move>;
	let mut sm = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_millis();

	while !board.is_over() {
		am = vec![Move::Up, Move::Down, Move::Left, Move::Right];
		while !board.play(am.remove(sm as usize % am.len())) {
			sm = (R_A * sm + R_C) % R_M;
			if am.is_empty() {
				panic!("Board.is_over() has been checked but no move change the board");
			}
		}

		seed = board.spawn_tile(seed);
	}

	(board.moves.into(), board.score)
}

fn parse() -> Option<Games> {
	if std::env::args().len() < 2 {
		eprintln!(
			"usage: \x1b[1m{} [\x1b[35;1m<seed>\x1b[0m\x1b[1m]\t\x1b[3m(one seed per arg)\x1b[0m",
			std::env::args().next().unwrap()
		);
		return None;
	}

	let mut games: Games = HashMap::new();

	for arg in std::env::args().skip(1) {
		match arg.parse::<Seed>() {
			Ok(seed) => {
				let mut b = Board::new();
				let cur_seed = b.spawn_tile(seed);
				let cur_seed = b.spawn_tile(cur_seed);
				games.insert(seed, (b, cur_seed, 0));
			}
			Err(e) => {
				err!("{}", e);
				return None;
			}
		}
	}

	Some(games)
}

fn main() -> ExitCode {
	let mut games: Games = parse().unwrap_or_else(|| {
		std::process::exit(1);
	});

	{
		let bs = read(FILE_RESULT).unwrap_or_else(|| {
			std::process::exit(2);
		});

		for s in bs {
			if let Some((_, _, score)) = games.get_mut(&s.0) {
				*score = s.1;
			}
		}
	}

	let mut c: usize = 0;

	loop {
		for (seed, (board, cur_seed, score)) in games.iter_mut() {
			let (moves, new_score) = play(board.clone(), *cur_seed);
			if new_score > *score {
				*score = new_score;
				print!("\x1b[33;1m{}\x1b[0m:\t", seed);
				write(FILE_RESULT, *seed, *score, &moves);
				println!(
					"\x1b[32;1m{} \x1b[35;1m{} \x1b[31;1m{}\x1b[0m",
					*score,
					moves.len(),
					c
				);
			}
		}

		c += 1;
	}
}
