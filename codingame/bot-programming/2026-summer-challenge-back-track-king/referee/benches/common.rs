#![allow(dead_code)]

use btk::driver::make_driver;
use btk::game::{DEFAULT_LEAGUE, Game};
use btk::gridmaker;
use btk::proto::{init_lines, turn_lines};
use std::path::PathBuf;
use std::time::Duration;

pub const SMALL: i64 = 4;
pub const MEDIUM: i64 = 5;
pub const LARGE: i64 = 2;

pub fn bot(name: &str) -> String {
	let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", "target", "release", name]
		.iter()
		.collect();
	assert!(
		path.is_file(),
		"{} is missing, run `cargo build --release` before benching",
		path.display()
	);
	path.to_string_lossy().into_owned()
}

pub fn fresh(seed: i64) -> Game {
	Game::new(gridmaker::make(seed), DEFAULT_LEAGUE)
}

pub fn after(seed: i64, turns: i32) -> Game {
	let mut game = fresh(seed);
	if turns == 0 {
		return game;
	}
	let command = bot("fuzz");
	let timeout = Duration::from_secs(10);
	let mut sides = [
		make_driver(&command, timeout).unwrap(),
		make_driver(&command, timeout).unwrap(),
	];
	let mut frame = String::new();
	for (player, side) in sides.iter_mut().enumerate() {
		init_lines(&game.grid, player, &mut frame);
		side.send(&frame).unwrap();
	}
	for _ in 0..turns {
		if game.ended {
			break;
		}
		game.reset_turn_data();
		for (player, side) in sides.iter_mut().enumerate() {
			turn_lines(&game, player, &mut frame);
			side.send(&frame).unwrap();
		}
		let answers = [sides[0].answer().unwrap(), sides[1].answer().unwrap()];
		for (player, answer) in answers.iter().enumerate() {
			game.take_commands(player, answer);
		}
		game.perform_update();
	}
	game
}
