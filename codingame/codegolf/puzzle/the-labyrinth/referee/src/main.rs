use std::process::ExitCode;
use std::time::Duration;

use referee_core::{Game, Interactive, Step};
use serde::Deserialize;

const FUEL: u32 = 1200;
const SCAN_RADIUS: usize = 2;
const UNKNOWN: u8 = b'?';
const WALL: u8 = b'#';
const START: u8 = b'T';
const CONTROL_ROOM: u8 = b'C';

#[derive(Deserialize)]
struct Test {
	alarm: u32,
	start: (usize, usize),
	maze: Vec<String>,
}

struct Labyrinth {
	maze: Vec<Vec<u8>>,
	scanned: Vec<Vec<u8>>,
	alarm: u32,
	countdown: Option<u32>,
	position: (usize, usize),
	fuel: u32,
}

fn direction_offset(direction: &str) -> Option<(isize, isize)> {
	Some(match direction {
		"UP" => (-1, 0),
		"DOWN" => (1, 0),
		"LEFT" => (0, -1),
		"RIGHT" => (0, 1),
		_ => return None,
	})
}

impl Labyrinth {
	fn rows(&self) -> usize {
		self.maze.len()
	}

	fn columns(&self) -> usize {
		self.maze[0].len()
	}

	fn scan(&mut self) {
		let (row, column) = self.position;
		let rows = row.saturating_sub(SCAN_RADIUS)..(row + SCAN_RADIUS + 1).min(self.rows());
		let columns =
			column.saturating_sub(SCAN_RADIUS)..(column + SCAN_RADIUS + 1).min(self.columns());
		for scanned_row in rows {
			let source = &self.maze[scanned_row][columns.clone()];
			self.scanned[scanned_row][columns.clone()].copy_from_slice(source);
		}
	}

	fn neighbour(&self, (offset_row, offset_column): (isize, isize)) -> Option<(usize, usize)> {
		let row = self.position.0.checked_add_signed(offset_row)?;
		let column = self.position.1.checked_add_signed(offset_column)?;
		(row < self.rows() && column < self.columns()).then_some((row, column))
	}
}

impl Game for Labyrinth {
	type Test = Test;

	const FIRST_TURN: Duration = Duration::from_millis(1000);
	const TURN: Duration = Duration::from_millis(200);

	fn new(test: Test) -> Self {
		let maze: Vec<Vec<u8>> = test.maze.into_iter().map(String::into_bytes).collect();
		let scanned = vec![vec![UNKNOWN; maze[0].len()]; maze.len()];
		let mut labyrinth = Self {
			maze,
			scanned,
			alarm: test.alarm,
			countdown: None,
			position: test.start,
			fuel: FUEL,
		};
		labyrinth.scan();
		labyrinth
	}

	fn initial_input(&self) -> String {
		let rows = self.rows();
		let columns = self.columns();
		let alarm = self.alarm;
		format!("{rows} {columns} {alarm}\n")
	}

	fn turn_input(&self) -> String {
		let (row, column) = self.position;
		let mut input = format!("{row} {column}\n").into_bytes();
		for line in &self.scanned {
			input.extend_from_slice(line);
			input.push(b'\n');
		}
		String::from_utf8(input).expect("maze is ASCII")
	}

	fn play(&mut self, action: &[String]) -> Step {
		let direction = action[0].as_str();
		let Some(offset) = direction_offset(direction) else {
			return Step::Lose(format!("invalid move {direction:?}"));
		};
		let Some((row, column)) = self.neighbour(offset) else {
			let (row, column) = self.position;
			return Step::Lose(format!("left the maze from ({row},{column})"));
		};
		let cell = self.maze[row][column];
		if cell == WALL {
			return Step::Lose(format!("hit a wall at ({row},{column})"));
		}
		self.position = (row, column);
		self.fuel -= 1;
		self.scan();
		match self.countdown.as_mut() {
			Some(countdown) => *countdown -= 1,
			None if cell == CONTROL_ROOM => self.countdown = Some(self.alarm),
			None => {}
		}
		if self.countdown.is_some() && cell == START {
			return Step::Win;
		}
		if self.countdown == Some(0) {
			return Step::Lose(format!("alarm went off at ({row},{column})"));
		}
		if self.fuel == 0 {
			return Step::Lose(format!("out of fuel at ({row},{column})"));
		}
		Step::Continue
	}
}

fn main() -> ExitCode {
	referee_core::main(Interactive::<Labyrinth>::default())
}
