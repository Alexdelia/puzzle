use crate::map::{Cell, Map};
use std::fmt::{self, Write};

pub const SAFE: i8 = -2;
pub const NO_FIRE: i8 = -1;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
	Wait,
	Cut(usize),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Fault {
	BadOutput(String),
	OutOfMap(i64, i64),
	CutOnCooldown(u32),
	CutSafe(usize, usize),
	CutOnFire(usize, usize),
	CutBurnt(usize, usize),
}

impl fmt::Display for Fault {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Fault::BadOutput(line) => write!(f, "bad output {line:?}"),
			Fault::OutOfMap(x, y) => write!(f, "cut outside map at {x} {y}"),
			Fault::CutOnCooldown(cooldown) => write!(f, "cut with cooldown {cooldown}"),
			Fault::CutSafe(x, y) => write!(f, "cut safe cell {x} {y}"),
			Fault::CutOnFire(x, y) => write!(f, "cut burning cell {x} {y}"),
			Fault::CutBurnt(x, y) => write!(f, "cut burnt cell {x} {y}"),
		}
	}
}

impl Action {
	pub fn parse(line: &str, map: &Map) -> Result<Action, Fault> {
		let line = line.trim();
		if line == "WAIT" {
			return Ok(Action::Wait);
		}
		let bad_output = || Fault::BadOutput(line.to_string());
		let numbers = line
			.split_whitespace()
			.map(|token| token.parse::<i64>().map_err(|_| bad_output()))
			.collect::<Result<Vec<_>, _>>()?;
		let [x, y] = numbers[..] else {
			return Err(bad_output());
		};
		if x < 0 || y < 0 || x as usize >= map.width || y as usize >= map.height {
			return Err(Fault::OutOfMap(x, y));
		}
		Ok(Action::Cut(map.index(x as usize, y as usize)))
	}

	pub fn display(self, map: &Map) -> String {
		match self {
			Action::Wait => "WAIT".to_string(),
			Action::Cut(i) => {
				let (x, y) = map.coords(i);
				format!("{x} {y}")
			}
		}
	}
}

#[derive(Clone)]
pub struct Game<'a> {
	map: &'a Map,
	progress: Vec<i8>,
	burning: Vec<usize>,
	burnt_out: Vec<usize>,
	cooldown: u32,
	turn: u32,
	burnt_value: u32,
	cut_value: u32,
}

impl<'a> Game<'a> {
	pub fn new(map: &'a Map) -> Game<'a> {
		let mut game = Game {
			map,
			progress: map
				.cells
				.iter()
				.map(|&cell| if cell == Cell::Safe { SAFE } else { NO_FIRE })
				.collect(),
			burning: Vec::new(),
			burnt_out: Vec::new(),
			cooldown: 0,
			turn: 0,
			burnt_value: 0,
			cut_value: 0,
		};
		game.ignite(map.start);
		game
	}

	pub fn map(&self) -> &'a Map {
		self.map
	}

	pub fn progress(&self) -> &[i8] {
		&self.progress
	}

	pub fn cooldown(&self) -> u32 {
		self.cooldown
	}

	pub fn turn(&self) -> u32 {
		self.turn
	}

	pub fn active_fires(&self) -> usize {
		self.burning.len()
	}

	pub fn is_over(&self) -> bool {
		self.burning.is_empty()
	}

	pub fn burnt_value(&self) -> u32 {
		self.burnt_value
	}

	pub fn cut_value(&self) -> u32 {
		self.cut_value
	}

	pub fn remaining_value(&self) -> u32 {
		self.map.total_value() - self.burnt_value - self.cut_value
	}

	pub fn check(&self, action: Action) -> Result<(), Fault> {
		let Action::Cut(i) = action else {
			return Ok(());
		};
		let (x, y) = self.map.coords(i);
		match self.progress[i] {
			_ if self.cooldown > 0 => Err(Fault::CutOnCooldown(self.cooldown)),
			SAFE => Err(Fault::CutSafe(x, y)),
			NO_FIRE => Ok(()),
			p if p == self.map.params(i).fire_duration => Err(Fault::CutBurnt(x, y)),
			_ => Err(Fault::CutOnFire(x, y)),
		}
	}

	pub fn play(&mut self, action: Action) -> Result<(), Fault> {
		self.check(action)?;
		if let Action::Cut(i) = action {
			let params = self.map.params(i);
			self.progress[i] = SAFE;
			self.cooldown = params.cut_duration;
			self.cut_value += params.value;
		}
		self.spread();
		Ok(())
	}

	pub fn burn_out(&mut self) {
		while !self.is_over() {
			self.spread();
		}
	}

	pub fn turn_input(&self) -> String {
		let mut input = format!("{cooldown}\n", cooldown = self.cooldown);
		for row in self.progress.chunks(self.map.width) {
			for (x, p) in row.iter().enumerate() {
				let separator = if x + 1 == row.len() { '\n' } else { ' ' };
				write!(input, "{p}{separator}").unwrap();
			}
		}
		input
	}

	fn spread(&mut self) {
		let Game {
			map,
			progress,
			burning,
			burnt_out,
			..
		} = self;
		burning.retain(|&i| {
			progress[i] += 1;
			let still_burning = progress[i] < map.params(i).fire_duration;
			if !still_burning {
				burnt_out.push(i);
			}
			still_burning
		});
		for i in std::mem::take(&mut self.burnt_out) {
			for n in self.map.neighbours(i) {
				if self.progress[n] == NO_FIRE {
					self.ignite(n);
				}
			}
		}
		self.cooldown = self.cooldown.saturating_sub(1);
		self.turn += 1;
	}

	fn ignite(&mut self, i: usize) {
		let params = self.map.params(i);
		self.progress[i] = 0;
		self.burnt_value += params.value;
		if params.fire_duration > 0 {
			self.burning.push(i);
		}
	}
}

pub fn score_plan(map: &Map, cuts: &[usize]) -> Result<u32, (usize, Fault)> {
	let mut game = Game::new(map);
	for (k, &i) in cuts.iter().enumerate() {
		while game.cooldown() > 0 {
			game.play(Action::Wait).unwrap();
		}
		game.play(Action::Cut(i)).map_err(|fault| (k, fault))?;
	}
	game.burn_out();
	Ok(game.remaining_value())
}
