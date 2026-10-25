use std::fmt::Write as _;

mod runner;
use std::process::ExitCode;
use std::time::Duration;

use referee_core::{Game, Step};

use crate::runner::ClosedStdin;
use serde::Deserialize;

#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum Side {
	Top,
	Left,
	Right,
}

impl Side {
	fn name(self) -> &'static str {
		match self {
			Side::Top => "TOP",
			Side::Left => "LEFT",
			Side::Right => "RIGHT",
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Mover {
	x: i32,
	y: i32,
	side: Side,
}

#[derive(Deserialize)]
struct RockSpawn {
	turn: usize,
	x: i32,
	y: i32,
	side: Side,
}

#[derive(Deserialize)]
struct Test {
	grid: Vec<Vec<i32>>,
	exit: i32,
	indy: (i32, i32),
	rocks: Vec<RockSpawn>,
}

enum Move {
	To(Mover),
	Stuck,
	Wall,
	Outside,
}

struct Fall {
	grid: Vec<Vec<i32>>,
	exit: i32,
	indy: Mover,
	spawns: Vec<RockSpawn>,
	rocks: Vec<Option<Mover>>,
	turn: usize,
}

fn exit_side(room: i32, entry: Side) -> Option<(i32, i32)> {
	use Side::{Left, Right, Top};
	const DOWN: Option<(i32, i32)> = Some((0, 1));
	const WEST: Option<(i32, i32)> = Some((-1, 0));
	const EAST: Option<(i32, i32)> = Some((1, 0));
	match (room.abs(), entry) {
		(1, _) | (3 | 7 | 9, Top) | (4 | 7 | 8 | 12, Right) | (5 | 8 | 9 | 13, Left) => DOWN,
		(2 | 6, Right) | (4 | 10, Top) => WEST,
		(2 | 6, Left) | (5 | 11, Top) => EAST,
		_ => None,
	}
}

fn has_opening(room: i32, side: Side) -> bool {
	use Side::{Left, Right, Top};
	matches!(
		(room.abs(), side),
		(1 | 4 | 5 | 6, _)
			| (3 | 7 | 9 | 10 | 11, Top)
			| (2 | 7 | 8 | 12, Right)
			| (2 | 8 | 9 | 13, Left)
	)
}

fn rotate_right(room: i32) -> i32 {
	match room {
		2 => 3,
		3 => 2,
		4 => 5,
		5 => 4,
		9 => 6,
		13 => 10,
		6..=12 => room + 1,
		_ => room,
	}
}

fn rotate_left(room: i32) -> i32 {
	(0..3).fold(room, |room, _| rotate_right(room))
}

impl Fall {
	fn width(&self) -> i32 {
		self.grid[0].len() as i32
	}

	fn height(&self) -> i32 {
		self.grid.len() as i32
	}

	fn room(&self, x: i32, y: i32) -> i32 {
		self.grid[y as usize][x as usize]
	}

	fn advance(&self, mover: Mover) -> Move {
		let Some((dx, dy)) = exit_side(self.room(mover.x, mover.y), mover.side) else {
			return Move::Stuck;
		};
		let (x, y) = (mover.x + dx, mover.y + dy);
		if !(0..self.width()).contains(&x) || !(0..self.height()).contains(&y) {
			return Move::Outside;
		}
		let side = match (dx, dy) {
			(-1, 0) => Side::Right,
			(1, 0) => Side::Left,
			_ => Side::Top,
		};
		if !has_opening(self.room(x, y), side) {
			return Move::Wall;
		}
		Move::To(Mover { x, y, side })
	}

	fn rotate(&mut self, action: &str) -> Result<(), String> {
		let mut words = action.split(' ');
		let (Some(x), Some(y), Some(direction), None) =
			(words.next(), words.next(), words.next(), words.next())
		else {
			return Err(format!("invalid command {action:?}"));
		};
		let (Ok(x), Ok(y)) = (x.parse::<i32>(), y.parse::<i32>()) else {
			return Err(format!("invalid command {action:?}"));
		};
		if !(0..self.width()).contains(&x) || !(0..self.height()).contains(&y) {
			return Err(format!("cell ({x},{y}) outside the grid"));
		}
		let room = self.room(x, y);
		let occupied = (x, y) == (self.indy.x, self.indy.y)
			|| self
				.rocks
				.iter()
				.flatten()
				.any(|rock| (rock.x, rock.y) == (x, y));
		if room < 0 || occupied || (x, y) == (self.exit, self.height() - 1) {
			return Err(format!("cell ({x},{y}) cannot be rotated"));
		}
		let rotated = match direction {
			"LEFT" => rotate_left(room),
			"RIGHT" => rotate_right(room),
			_ => return Err(format!("invalid command {action:?}")),
		};
		self.grid[y as usize][x as usize] = rotated;
		Ok(())
	}

	fn move_rocks(&mut self) {
		let current = &self.rocks;
		let mut next = Vec::with_capacity(current.len());
		for rock in current.iter().copied() {
			let Some(rock) = rock else {
				next.push(None);
				continue;
			};
			let shared = current
				.iter()
				.flatten()
				.filter(|other| other.x == rock.x && other.y == rock.y)
				.count();
			if shared > 1 {
				next.push(None);
				continue;
			}
			next.push(match self.advance(rock) {
				Move::To(next) => Some(next),
				Move::Stuck | Move::Wall | Move::Outside => None,
			});
		}
		self.rocks = next;
	}

	fn spawn_rocks(&mut self) {
		for (slot, spawn) in self.rocks.iter_mut().zip(&self.spawns) {
			if spawn.turn == self.turn {
				*slot = Some(Mover {
					x: spawn.x,
					y: spawn.y,
					side: spawn.side,
				});
			}
		}
	}

	fn crushed(&self) -> bool {
		self.rocks
			.iter()
			.flatten()
			.any(|rock| (rock.x, rock.y) == (self.indy.x, self.indy.y))
	}
}

impl Game for Fall {
	type Test = Test;

	const FIRST_TURN: Duration = Duration::from_millis(1000);
	const TURN: Duration = Duration::from_millis(150);

	fn new(test: Test) -> Self {
		let mut fall = Self {
			rocks: vec![None; test.rocks.len()],
			grid: test.grid,
			exit: test.exit,
			indy: Mover {
				x: test.indy.0,
				y: test.indy.1,
				side: Side::Top,
			},
			spawns: test.rocks,
			turn: 1,
		};
		fall.spawn_rocks();
		fall
	}

	fn initial_input(&self) -> String {
		let (width, height, exit) = (self.width(), self.height(), self.exit);
		let mut out = String::with_capacity(16 + (self.height() * self.width()) as usize * 4);
		let _ = writeln!(out, "{width} {height}");
		for row in &self.grid {
			let mut first = true;
			for cell in row {
				if !first {
					out.push(' ');
				}
				first = false;
				let _ = write!(out, "{cell}");
			}
			out.push('\n');
		}
		let _ = writeln!(out, "{exit}");
		out
	}

	fn turn_input(&self) -> String {
		let Mover { x, y, side } = self.indy;
		let side = side.name();
		let count = self.rocks.iter().flatten().count();
		let mut out = String::with_capacity(24 + count * 16);
		let _ = writeln!(out, "{x} {y} {side}");
		let _ = writeln!(out, "{count}");
		for rock in self.rocks.iter().flatten() {
			let (rock_x, rock_y, rock_side) = (rock.x, rock.y, rock.side.name());
			let _ = writeln!(out, "{rock_x} {rock_y} {rock_side}");
		}
		out
	}

	fn play(&mut self, action: &[String]) -> Step {
		let action = action[0].as_str();
		if action != "WAIT"
			&& let Err(reason) = self.rotate(action)
		{
			return Step::Lose(reason);
		}
		if self.crushed() {
			let (x, y) = (self.indy.x, self.indy.y);
			return Step::Lose(format!("Indy crushed by a rock at ({x},{y})"));
		}
		let (x, y) = (self.indy.x, self.indy.y);
		self.indy = match self.advance(self.indy) {
			Move::To(next) => next,
			Move::Stuck => return Step::Lose(format!("Indy stopped at ({x},{y})")),
			Move::Wall | Move::Outside => {
				return Step::Lose(format!("Indy crashed on a wall leaving ({x},{y})"));
			}
		};
		if (self.indy.x, self.indy.y) == (self.exit, self.height() - 1) {
			return Step::Win;
		}
		self.move_rocks();
		self.turn += 1;
		self.spawn_rocks();
		Step::Continue
	}
}

fn main() -> ExitCode {
	referee_core::main(ClosedStdin::<Fall>::default())
}
