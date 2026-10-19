use std::process::ExitCode;
use std::time::Duration;

use referee_core::{Game, Interactive, Step};
use serde::Deserialize;

const MOUNTAINS: i32 = 8;
const START_ALTITUDE: u32 = 10;

#[derive(Deserialize)]
struct Test {
	mountains: Vec<Vec<u32>>,
}

struct Mountain {
	heights: Vec<u32>,
	shots: usize,
}

impl Mountain {
	fn height(&self) -> u32 {
		self.heights.get(self.shots).copied().unwrap_or(0)
	}
}

struct Descent {
	mountains: Vec<Mountain>,
	altitude: u32,
}

impl Game for Descent {
	type Test = Test;

	const FIRST_TURN: Duration = Duration::from_millis(1000);
	const TURN: Duration = Duration::from_millis(150);

	fn new(test: Test) -> Self {
		Self {
			mountains: test
				.mountains
				.into_iter()
				.map(|heights| Mountain { heights, shots: 0 })
				.collect(),
			altitude: START_ALTITUDE,
		}
	}

	fn initial_input(&self) -> String {
		String::new()
	}

	fn turn_input(&self) -> String {
		self.mountains
			.iter()
			.map(|mountain| format!("{height}\n", height = mountain.height()))
			.collect()
	}

	fn play(&mut self, action: &[String]) -> Step {
		let command = action[0].strip_suffix('\r').unwrap_or(&action[0]);
		let target = match command.parse::<i32>() {
			Ok(index) if (0..MOUNTAINS).contains(&index) => index as usize,
			_ => return Step::Lose(format!("invalid command {command:?}")),
		};
		self.altitude = self.altitude.saturating_sub(1);
		let altitude = self.altitude;
		if let Some(index) = self
			.mountains
			.iter()
			.position(|mountain| mountain.height() > altitude)
		{
			return Step::Lose(format!(
				"crashed on mountain {index} at altitude {altitude}"
			));
		}
		self.mountains[target].shots += 1;
		if self.mountains.iter().all(|mountain| mountain.height() == 0) {
			return Step::Win;
		}
		Step::Continue
	}
}

fn main() -> ExitCode {
	referee_core::main(Interactive::<Descent>::default())
}
