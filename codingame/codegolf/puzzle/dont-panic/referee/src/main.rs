use std::process::ExitCode;
use std::time::Duration;

use referee_core::{Game, Interactive, Step};
use serde::Deserialize;

const SPAWN_INTERVAL: u32 = 3;

#[derive(Deserialize)]
struct Test {
	floors: u32,
	width: i32,
	rounds: u32,
	exit: (i32, i32),
	clones: u32,
	generator: i32,
	elevators: Vec<(i32, i32)>,
}

struct Clone {
	floor: i32,
	pos: i32,
	direction: i32,
	blocked: bool,
}

struct Drive {
	test: Test,
	clones: Vec<Clone>,
	spawned: u32,
	turn: u32,
}

impl Drive {
	fn blocked_at(&self, floor: i32, pos: i32) -> bool {
		self.clones
			.iter()
			.any(|clone| clone.blocked && clone.floor == floor && clone.pos == pos)
	}

	fn has_elevator(&self, floor: i32, pos: i32) -> bool {
		self.test.elevators.contains(&(floor, pos))
	}

	fn spawn(&mut self) {
		let generator = self.test.generator;
		let direction = if self.blocked_at(0, generator) { -1 } else { 1 };
		self.clones.push(Clone {
			floor: 0,
			pos: generator,
			direction,
			blocked: false,
		});
		self.spawned += 1;
	}

	fn leading(&mut self) -> Option<&mut Clone> {
		self.clones.iter_mut().find(|clone| !clone.blocked)
	}

	fn advance(&mut self, index: usize) {
		let Clone {
			floor,
			pos,
			direction,
			..
		} = self.clones[index];
		let (floor, pos, direction) = if self.has_elevator(floor, pos) {
			let floor = floor + 1;
			let direction = if self.blocked_at(floor, pos) {
				-direction
			} else {
				direction
			};
			(floor, pos, direction)
		} else {
			let direction = if self.blocked_at(floor, pos + direction) {
				-direction
			} else {
				direction
			};
			(floor, pos + direction, direction)
		};
		let clone = &mut self.clones[index];
		clone.floor = floor;
		clone.pos = pos;
		clone.direction = direction;
	}
}

impl Game for Drive {
	type Test = Test;

	const FIRST_TURN: Duration = Duration::from_millis(1000);
	const TURN: Duration = Duration::from_millis(150);

	fn new(test: Test) -> Self {
		let mut drive = Self {
			test,
			clones: Vec::new(),
			spawned: 0,
			turn: 1,
		};
		drive.spawn();
		drive
	}

	fn initial_input(&self) -> String {
		let Test {
			floors,
			width,
			rounds,
			exit: (exit_floor, exit_pos),
			clones,
			ref elevators,
			..
		} = self.test;
		let count = elevators.len();
		let mut input =
			format!("{floors} {width} {rounds} {exit_floor} {exit_pos} {clones} 0 {count}\n");
		for (floor, pos) in elevators {
			input += &format!("{floor} {pos}\n");
		}
		input
	}

	fn turn_input(&self) -> String {
		match self.clones.iter().find(|clone| !clone.blocked) {
			Some(clone) => {
				let (floor, pos) = (clone.floor, clone.pos);
				let direction = if clone.direction > 0 { "RIGHT" } else { "LEFT" };
				format!("{floor} {pos} {direction}\n")
			}
			None => "-1 -1 NONE\n".to_string(),
		}
	}

	fn play(&mut self, action: &[String]) -> Step {
		match action[0].as_str() {
			"WAIT" => {}
			"BLOCK" => {
				if let Some(clone) = self.leading() {
					clone.blocked = true;
				}
			}
			other => return Step::Lose(format!("invalid action {other:?}")),
		}
		let exit = self.test.exit;
		if self
			.clones
			.iter()
			.any(|clone| (clone.floor, clone.pos) == exit)
		{
			return Step::Win;
		}
		for index in 0..self.clones.len() {
			if !self.clones[index].blocked {
				self.advance(index);
			}
		}
		let width = self.test.width;
		self.clones.retain(|clone| (0..width).contains(&clone.pos));
		if self.turn == self.test.rounds {
			return Step::Lose("max rounds count reached".to_string());
		}
		self.turn += 1;
		if self.turn % SPAWN_INTERVAL == 1 && self.spawned < self.test.clones {
			self.spawn();
		}
		if self.spawned == self.test.clones && self.clones.iter().all(|clone| clone.blocked) {
			return Step::Lose("no more clones".to_string());
		}
		Step::Continue
	}
}

fn main() -> ExitCode {
	referee_core::main(Interactive::<Drive>::default())
}
