#![allow(dead_code)]

use btk::arena::run_match;
use btk::driver::make_driver;
use btk::game::{Game, Rules};
use btk::mapgen::{GenParams, generate};
use std::path::PathBuf;
use std::time::Duration;

pub const SMALL: (usize, usize, usize) = (21, 14, 4);
pub const MEDIUM: (usize, usize, usize) = (25, 17, 8);
pub const LARGE: (usize, usize, usize) = (30, 20, 12);

pub fn shape((width, height, towns): (usize, usize, usize)) -> GenParams {
	GenParams {
		width: Some(width),
		height: Some(height),
		towns: Some(towns),
	}
}

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

pub fn fresh(seed: u64, size: (usize, usize, usize)) -> Game {
	Game::new(generate(seed, &shape(size)), Rules::default())
}

pub fn after(seed: u64, size: (usize, usize, usize), turns: usize) -> Game {
	let mut game = fresh(seed, size);
	if turns == 0 {
		return game;
	}
	game.rules.max_turns = turns;
	let command = bot("greedy");
	let timeout = Duration::from_secs(10);
	let mut d0 = make_driver(&command, timeout).unwrap();
	let mut d1 = make_driver(&command, timeout).unwrap();
	run_match(&mut game, [d0.as_mut(), d1.as_mut()], None, None);
	game.rules.max_turns = Rules::default().max_turns;
	game
}
