use std::process::ExitCode;
use std::time::Duration;

use referee_core::{Game, Interactive, Step};
use serde::Deserialize;

#[derive(Deserialize)]
struct Test {
	width: i64,
	height: i64,
	jumps: u32,
	batman: (i64, i64),
	bombs: (i64, i64),
}

struct Building {
	width: i64,
	height: i64,
	jumps_left: u32,
	initial_jumps: u32,
	batman: (i64, i64),
	bombs: (i64, i64),
	device: &'static str,
}

impl Building {
	fn squared_distance(&self, (x, y): (i64, i64)) -> i64 {
		let (bomb_x, bomb_y) = self.bombs;
		(x - bomb_x).pow(2) + (y - bomb_y).pow(2)
	}

	fn parse_window(&self, line: &str) -> Option<(i64, i64)> {
		let mut fields = line.split(' ');
		let x = fields.next()?.parse().ok()?;
		let y = fields.next()?.parse().ok()?;
		let inside = (0..self.width).contains(&x) && (0..self.height).contains(&y);
		(fields.next().is_none() && inside).then_some((x, y))
	}
}

impl Game for Building {
	type Test = Test;

	const FIRST_TURN: Duration = Duration::from_millis(1000);
	const TURN: Duration = Duration::from_millis(150);

	fn new(test: Test) -> Self {
		Self {
			width: test.width,
			height: test.height,
			jumps_left: test.jumps,
			initial_jumps: test.jumps,
			batman: test.batman,
			bombs: test.bombs,
			device: "UNKNOWN",
		}
	}

	fn initial_input(&self) -> String {
		let (width, height, jumps) = (self.width, self.height, self.initial_jumps);
		let (x, y) = self.batman;
		format!("{width} {height}\n{jumps}\n{x} {y}\n")
	}

	fn turn_input(&self) -> String {
		let device = self.device;
		format!("{device}\n")
	}

	fn play(&mut self, action: &[String]) -> Step {
		let line = action[0].as_str();
		let Some(window) = self.parse_window(line) else {
			return Step::Lose(format!("invalid window {line:?}"));
		};
		let before = self.squared_distance(self.batman);
		let after = self.squared_distance(window);
		self.batman = window;
		self.jumps_left -= 1;
		if window == self.bombs {
			return Step::Win;
		}
		if self.jumps_left == 0 {
			let (x, y) = window;
			return Step::Lose(format!("out of jumps at ({x},{y})"));
		}
		self.device = match after.cmp(&before) {
			std::cmp::Ordering::Less => "WARMER",
			std::cmp::Ordering::Greater => "COLDER",
			std::cmp::Ordering::Equal => "SAME",
		};
		Step::Continue
	}
}

fn main() -> ExitCode {
	referee_core::main(Interactive::<Building>::default())
}
