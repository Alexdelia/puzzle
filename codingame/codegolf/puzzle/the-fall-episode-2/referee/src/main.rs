mod runner;

use std::process::ExitCode;
use std::time::Duration;

use referee_core::{Game, Step};
use runner::ClosedStdin;
use serde::Deserialize;

#[derive(Clone, Copy, PartialEq, Deserialize)]
enum Entry {
	#[serde(rename = "TOP")]
	Top,
	#[serde(rename = "LEFT")]
	Left,
	#[serde(rename = "RIGHT")]
	Right,
}

impl Entry {
	fn name(self) -> &'static str {
		match self {
			Entry::Top => "TOP",
			Entry::Left => "LEFT",
			Entry::Right => "RIGHT",
		}
	}
}

#[derive(Clone, Copy, PartialEq)]
struct Mover {
	x: i32,
	y: i32,
	entry: Entry,
}

#[derive(Deserialize)]
struct Rock {
	turn: usize,
	x: i32,
	y: i32,
	entry: Entry,
}

#[derive(Deserialize)]
struct Test {
	grid: Vec<Vec<i32>>,
	exit: i32,
	indy: (i32, i32, Entry),
	rocks: Vec<Rock>,
}

struct Fall {
	initial: String,
	kinds: Vec<Vec<u8>>,
	locked: Vec<Vec<bool>>,
	width: i32,
	height: i32,
	exit: (i32, i32),
	indy: Mover,
	schedule: Vec<Rock>,
	rocks: Vec<(usize, Mover)>,
	turn: usize,
}

fn exit_offset(kind: u8, entry: Entry) -> Option<(i32, i32)> {
	use Entry::{Left, Right, Top};
	const DOWN: (i32, i32) = (0, 1);
	const WEST: (i32, i32) = (-1, 0);
	const EAST: (i32, i32) = (1, 0);
	Some(match (kind, entry) {
		(1, _) => DOWN,
		(2 | 6, Left) => EAST,
		(2 | 6, Right) => WEST,
		(3 | 7 | 9, Top) => DOWN,
		(4, Top) => WEST,
		(4 | 7 | 8 | 12, Right) => DOWN,
		(5, Top) => EAST,
		(5 | 8 | 9 | 13, Left) => DOWN,
		(10, Top) => WEST,
		(11, Top) => EAST,
		_ => return None,
	})
}

fn clockwise(kind: u8) -> u8 {
	match kind {
		2 => 3,
		3 => 2,
		4 => 5,
		5 => 4,
		6..=8 | 10..=12 => kind + 1,
		9 => 6,
		13 => 10,
		_ => kind,
	}
}

fn counterclockwise(kind: u8) -> u8 {
	(0..3).fold(kind, |kind, _| clockwise(kind))
}

impl Fall {
	fn inside(&self, x: i32, y: i32) -> bool {
		(0..self.width).contains(&x) && (0..self.height).contains(&y)
	}

	fn kind(&self, x: i32, y: i32) -> u8 {
		self.kinds[y as usize][x as usize]
	}

	fn advance(&self, mover: Mover) -> Option<Mover> {
		let (offset_x, offset_y) = exit_offset(self.kind(mover.x, mover.y), mover.entry)?;
		let (x, y) = (mover.x + offset_x, mover.y + offset_y);
		let entry = match (offset_x, offset_y) {
			(1, _) => Entry::Left,
			(-1, _) => Entry::Right,
			_ => Entry::Top,
		};
		if !self.inside(x, y) {
			return None;
		}
		exit_offset(self.kind(x, y), entry)?;
		Some(Mover { x, y, entry })
	}

	fn spawn(&mut self) {
		let turn = self.turn;
		self.rocks.extend(
			self.schedule
				.iter()
				.enumerate()
				.filter(|(_, rock)| rock.turn == turn)
				.map(|(index, rock)| {
					let mover = Mover {
						x: rock.x,
						y: rock.y,
						entry: rock.entry,
					};
					(index, mover)
				}),
		);
		self.rocks.sort_by_key(|&(index, _)| index);
	}

	fn rotate(&mut self, action: &str) -> Result<String, String> {
		let words: Vec<&str> = action.split(' ').collect();
		let [x, y, direction] = words[..] else {
			return Err(format!("invalid action {action:?}"));
		};
		let (Ok(x), Ok(y)) = (x.parse::<i32>(), y.parse::<i32>()) else {
			return Err(format!("invalid action {action:?}"));
		};
		let turn: fn(u8) -> u8 = match direction {
			"RIGHT" => clockwise,
			"LEFT" => counterclockwise,
			_ => return Err(format!("invalid action {action:?}")),
		};
		if !self.inside(x, y) {
			return Err(format!("cell ({x},{y}) outside the grid"));
		}
		let occupied = (self.indy.x, self.indy.y) == (x, y)
			|| self
				.rocks
				.iter()
				.any(|(_, rock)| (rock.x, rock.y) == (x, y));
		if self.locked[y as usize][x as usize] || occupied || (x, y) == self.exit {
			return Err(format!("cell ({x},{y}) cannot be rotated"));
		}
		let cell = &mut self.kinds[y as usize][x as usize];
		*cell = turn(*cell);
		Ok(format!("rotated ({x},{y}) {direction}"))
	}
}

impl Game for Fall {
	type Test = Test;

	const FIRST_TURN: Duration = Duration::from_millis(1000);
	const TURN: Duration = Duration::from_millis(150);

	fn new(test: Test) -> Self {
		let height = test.grid.len() as i32;
		let width = test.grid[0].len() as i32;
		let rows: Vec<String> = test
			.grid
			.iter()
			.map(|row| row.iter().map(i32::to_string).collect::<Vec<_>>().join(" "))
			.collect();
		let rows = rows.join("\n");
		let exit_x = test.exit;
		let initial = format!("{width} {height}\n{rows}\n{exit_x}\n");
		let (x, y, entry) = test.indy;
		let mut fall = Self {
			initial,
			kinds: test
				.grid
				.iter()
				.map(|row| row.iter().map(|kind| kind.unsigned_abs() as u8).collect())
				.collect(),
			locked: test
				.grid
				.iter()
				.map(|row| row.iter().map(|kind| *kind < 0).collect())
				.collect(),
			width,
			height,
			exit: (exit_x, height - 1),
			indy: Mover { x, y, entry },
			schedule: test.rocks,
			rocks: Vec::new(),
			turn: 1,
		};
		fall.spawn();
		fall
	}

	fn initial_input(&self) -> String {
		self.initial.clone()
	}

	fn turn_input(&self) -> String {
		let Mover { x, y, entry } = self.indy;
		let entry = entry.name();
		let count = self.rocks.len();
		let mut input = format!("{x} {y} {entry}\n{count}\n");
		for (_, Mover { x, y, entry }) in &self.rocks {
			let entry = entry.name();
			input += &format!("{x} {y} {entry}\n");
		}
		input
	}

	fn play(&mut self, action: &[String]) -> Step {
		let action = action[0].as_str();
		if action != "WAIT"
			&& let Err(reason) = self.rotate(action)
		{
			return Step::Lose(reason);
		}
		let (from_x, from_y) = (self.indy.x, self.indy.y);
		let Some(indy) = self.advance(self.indy) else {
			return Step::Lose(format!("Indy crashed leaving ({from_x},{from_y})"));
		};
		self.indy = indy;
		let hit = |x: i32, y: i32| (x, y) == (indy.x, indy.y);
		let mut crushed = false;
		let mut rocks = Vec::with_capacity(self.rocks.len());
		for &(index, rock) in &self.rocks {
			match self.advance(rock) {
				Some(next) => {
					crushed |= hit(next.x, next.y);
					rocks.push((index, next));
				}
				None => crushed |= hit(rock.x, rock.y),
			}
		}
		self.rocks = rocks;
		let (x, y) = (indy.x, indy.y);
		if crushed {
			return Step::Lose(format!("Indy hit by a rock at ({x},{y})"));
		}
		if (x, y) == self.exit {
			return Step::Win;
		}
		self.turn += 1;
		self.spawn();
		if self.rocks.iter().any(|(_, rock)| hit(rock.x, rock.y)) {
			return Step::Lose(format!("rock appeared on Indy at ({x},{y})"));
		}
		Step::Continue
	}
}

fn main() -> ExitCode {
	referee_core::main(ClosedStdin::<Fall>::default())
}
