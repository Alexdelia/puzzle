use std::process::ExitCode;
use std::time::Duration;

use referee_core::{Game, Interactive, Step};
use serde::Deserialize;

const TURN_LIMIT: usize = 50;
const LANES: i32 = 4;

#[derive(Deserialize)]
struct Test {
	required: usize,
	road: Vec<String>,
	speed: i32,
	lanes: Vec<i32>,
}

struct Bike {
	x: i32,
	y: i32,
	alive: bool,
}

struct Bridge {
	required: usize,
	road: Vec<String>,
	speed: i32,
	bikes: Vec<Bike>,
	turn: usize,
}

impl Bridge {
	fn length(&self) -> i32 {
		self.road[0].len() as i32
	}

	fn hole(&self, x: i32, y: i32) -> bool {
		self.road[y as usize].as_bytes().get(x as usize) == Some(&b'0')
	}

	fn straight_fall(&self, x: i32, y: i32) -> Option<(i32, i32)> {
		(x + 1..=x + self.speed)
			.find(|&cell| self.hole(cell, y))
			.map(|cell| (cell, y))
	}

	fn turn_fall(&self, x: i32, y: i32, lane_shift: i32) -> Option<(i32, i32)> {
		(1..=self.speed).find_map(|step| {
			let cell = x + step;
			if step < self.speed && self.hole(cell, y) {
				Some((cell, y))
			} else if self.hole(cell, y + lane_shift) {
				Some((cell, y + lane_shift))
			} else {
				None
			}
		})
	}

	fn alive(&self) -> impl Iterator<Item = &Bike> {
		self.bikes.iter().filter(|bike| bike.alive)
	}
}

impl Game for Bridge {
	type Test = Test;

	const FIRST_TURN: Duration = Duration::from_millis(1000);
	const TURN: Duration = Duration::from_millis(150);

	fn new(test: Test) -> Self {
		Self {
			required: test.required,
			road: test.road,
			speed: test.speed,
			bikes: test
				.lanes
				.iter()
				.map(|&y| Bike {
					x: 0,
					y,
					alive: true,
				})
				.collect(),
			turn: 0,
		}
	}

	fn initial_input(&self) -> String {
		let count = self.bikes.len();
		let required = self.required;
		let road: String = self.road.iter().map(|lane| format!("{lane}\n")).collect();
		format!("{count}\n{required}\n{road}")
	}

	fn turn_input(&self) -> String {
		let speed = self.speed;
		let mut input = format!("{speed}\n");
		for bike in &self.bikes {
			let (x, y, alive) = (bike.x, bike.y, u8::from(bike.alive));
			input += &format!("{x} {y} {alive}\n");
		}
		input
	}

	fn play(&mut self, action: &[String]) -> Step {
		self.turn += 1;
		let command = action[0].as_str();
		let mut lane_shift = 0;
		match command {
			"SPEED" => self.speed += 1,
			"SLOW" => self.speed = (self.speed - 1).max(0),
			"UP" => lane_shift = -1,
			"DOWN" => lane_shift = 1,
			"JUMP" | "WAIT" => {}
			_ => return Step::Lose(format!("invalid command {command:?}")),
		}
		if self
			.alive()
			.any(|bike| !(0..LANES).contains(&(bike.y + lane_shift)))
		{
			lane_shift = 0;
		}
		let falls: Vec<Option<(i32, i32)>> = self
			.bikes
			.iter()
			.map(|bike| {
				let (x, y) = (bike.x, bike.y);
				if !bike.alive {
					None
				} else if command == "JUMP" {
					self.hole(x + self.speed, y).then_some((x + self.speed, y))
				} else if lane_shift == 0 {
					self.straight_fall(x, y)
				} else {
					self.turn_fall(x, y, lane_shift)
				}
			})
			.collect();
		let speed = self.speed;
		for (bike, fall) in self.bikes.iter_mut().zip(falls) {
			if !bike.alive {
				continue;
			}
			match fall {
				Some((x, y)) => {
					bike.x = x;
					bike.y = y;
					bike.alive = false;
				}
				None => {
					bike.x += speed;
					bike.y += lane_shift;
				}
			}
		}
		let survivors = self.alive().count();
		let required = self.required;
		if survivors < required {
			return Step::Lose(format!("{survivors} bikes left, {required} required"));
		}
		if self.turn >= TURN_LIMIT {
			return Step::Lose(format!("bridge not crossed in {TURN_LIMIT} turns"));
		}
		if self.alive().any(|bike| bike.x > self.length()) {
			return Step::Win;
		}
		Step::Continue
	}
}

fn main() -> ExitCode {
	referee_core::main(Interactive::<Bridge>::default())
}
