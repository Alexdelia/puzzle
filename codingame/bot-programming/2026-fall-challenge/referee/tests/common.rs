#![allow(dead_code)]

use std::path::PathBuf;
use tryangle::arena::{FrameRecord, MatchResult, run_match};
use tryangle::cglog::{CgLog, CgTrace};
use tryangle::driver::Driver;
use tryangle::game::{DEFAULT_LEAGUE, Game};
use tryangle::mapgen;

pub struct Scripted {
	pub answers: std::vec::IntoIter<String>,
	pub received: Vec<String>,
}

impl Scripted {
	pub fn new(answers: Vec<String>) -> Self {
		Scripted {
			answers: answers.into_iter(),
			received: Vec::new(),
		}
	}
}

impl Driver for Scripted {
	fn send(&mut self, text: &str) -> Result<(), String> {
		self.received.push(text.to_string());
		Ok(())
	}

	fn answer(&mut self) -> Result<String, String> {
		self.answers
			.next()
			.ok_or_else(|| "the recorded game has no more answers".into())
	}
}

pub fn golden_logs() -> Vec<(String, CgLog)> {
	let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/cg");
	let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
		.expect("golden log directory")
		.map(|entry| entry.unwrap().path())
		.collect();
	paths.sort();
	assert!(!paths.is_empty(), "no golden logs");
	paths
		.into_iter()
		.map(|path| {
			let name = path.file_name().unwrap().to_string_lossy().into_owned();
			let log = CgLog::parse(&std::fs::read_to_string(&path).unwrap())
				.unwrap_or_else(|e| panic!("{name}: {e}"));
			(name, log)
		})
		.collect()
}

pub fn seed_of(log: &CgLog) -> i64 {
	log.option("seed")
		.expect("seed option")
		.parse()
		.expect("numeric seed")
}

pub struct Replayed {
	pub result: MatchResult,
	pub trace: CgTrace,
	pub sides: [Scripted; 2],
}

pub fn replay(log: &CgLog) -> Replayed {
	let mut game = Game::new(mapgen::generate(seed_of(log)), DEFAULT_LEAGUE);
	let mut sides = [0, 1].map(|player| Scripted::new(log.answers(player)));
	let mut trace = CgTrace::default();
	let result = {
		let [side0, side1] = sides.each_mut();
		let mut record = |_: &Game, frame: &FrameRecord| trace.push(frame);
		run_match(&mut game, [side0, side1], None, Some(&mut record))
	};
	Replayed {
		result,
		trace,
		sides,
	}
}
