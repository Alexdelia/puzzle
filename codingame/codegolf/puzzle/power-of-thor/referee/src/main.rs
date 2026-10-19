use std::process::ExitCode;
use std::time::Duration;

use referee_core::{Game, Interactive, Step};
use serde::Deserialize;

const WIDTH: i32 = 40;
const HEIGHT: i32 = 18;

#[derive(Deserialize)]
struct Test {
	light: (i32, i32),
	thor: (i32, i32),
	energy: u32,
}

struct Thor {
	light: (i32, i32),
	position: (i32, i32),
	energy: u32,
}

fn direction_offset(direction: &str) -> Option<(i32, i32)> {
	Some(match direction {
		"N" => (0, -1),
		"NE" => (1, -1),
		"E" => (1, 0),
		"SE" => (1, 1),
		"S" => (0, 1),
		"SW" => (-1, 1),
		"W" => (-1, 0),
		"NW" => (-1, -1),
		_ => return None,
	})
}

impl Game for Thor {
	type Test = Test;

	const FIRST_TURN: Duration = Duration::from_millis(1000);
	const TURN: Duration = Duration::from_millis(150);

	fn new(test: Test) -> Self {
		Self {
			light: test.light,
			position: test.thor,
			energy: test.energy,
		}
	}

	fn initial_input(&self) -> String {
		let (light_x, light_y) = self.light;
		let (thor_x, thor_y) = self.position;
		format!("{light_x} {light_y} {thor_x} {thor_y}\n")
	}

	fn turn_input(&self) -> String {
		let energy = self.energy;
		format!("{energy}\n")
	}

	fn play(&mut self, action: &[String]) -> Step {
		let direction = action[0].as_str();
		let Some((offset_x, offset_y)) = direction_offset(direction) else {
			return Step::Lose(format!("invalid move {direction:?}"));
		};
		let (x, y) = (self.position.0 + offset_x, self.position.1 + offset_y);
		self.position = (x, y);
		self.energy -= 1;
		if !(0..WIDTH).contains(&x) || !(0..HEIGHT).contains(&y) {
			return Step::Lose(format!("Thor left the map at ({x},{y})"));
		}
		if self.position == self.light {
			return Step::Win;
		}
		if self.energy == 0 {
			return Step::Lose(format!("out of energy at ({x},{y})"));
		}
		Step::Continue
	}
}

fn main() -> ExitCode {
	referee_core::main(Interactive::<Thor>::default())
}
