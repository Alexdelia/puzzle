use std::cmp::Ordering;
use std::process::ExitCode;
use std::time::Duration;

use referee_core::{Game, Interactive, Step};
use serde::Deserialize;

#[derive(Deserialize)]
struct Test {
	size: (i32, i32),
	jumps: u32,
	batman: (i32, i32),
	bomb: (i32, i32),
}

struct Building {
	size: (i32, i32),
	jumps_left: u32,
	batman: (i32, i32),
	bomb: (i32, i32),
}

fn parse_jump(line: &str) -> Option<(i32, i32)> {
	let (x, y) = line.trim_end().split_once(' ')?;
	Some((x.parse().ok()?, y.parse().ok()?))
}

impl Building {
	fn direction(&self) -> String {
		let vertical = match self.bomb.1.cmp(&self.batman.1) {
			Ordering::Less => "U",
			Ordering::Equal => "",
			Ordering::Greater => "D",
		};
		let horizontal = match self.bomb.0.cmp(&self.batman.0) {
			Ordering::Less => "L",
			Ordering::Equal => "",
			Ordering::Greater => "R",
		};
		format!("{vertical}{horizontal}")
	}
}

impl Game for Building {
	type Test = Test;

	const FIRST_TURN: Duration = Duration::from_millis(1000);
	const TURN: Duration = Duration::from_millis(150);

	fn new(test: Test) -> Self {
		Self {
			size: test.size,
			jumps_left: test.jumps,
			batman: test.batman,
			bomb: test.bomb,
		}
	}

	fn initial_input(&self) -> String {
		let (width, height) = self.size;
		let jumps = self.jumps_left;
		let (x, y) = self.batman;
		format!("{width} {height}\n{jumps}\n{x} {y}\n")
	}

	fn turn_input(&self) -> String {
		let direction = self.direction();
		format!("{direction}\n")
	}

	fn play(&mut self, action: &[String]) -> Step {
		let line = action[0].as_str();
		let Some((x, y)) = parse_jump(line) else {
			return Step::Lose(format!("expected 'x y' but found {line:?}"));
		};
		let (width, height) = self.size;
		if !(0..width).contains(&x) {
			return Step::Lose(format!("expected 0 <= x < {width} but found x = {x}"));
		}
		if !(0..height).contains(&y) {
			return Step::Lose(format!("expected 0 <= y < {height} but found y = {y}"));
		}
		self.batman = (x, y);
		self.jumps_left -= 1;
		if self.batman == self.bomb {
			return Step::Win;
		}
		if self.jumps_left == 0 {
			return Step::Lose(format!("bombs exploded, Batman at ({x},{y})"));
		}
		Step::Continue
	}
}

fn main() -> ExitCode {
	referee_core::main(Interactive::<Building>::default())
}
