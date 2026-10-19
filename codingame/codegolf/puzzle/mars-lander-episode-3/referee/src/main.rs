use std::process::ExitCode;
use std::time::Duration;

use referee_core::{Game, Interactive, Step};
use serde::Deserialize;

const GRAVITY: f64 = 3.711;
const WIDTH: f64 = 7000.0;
const HEIGHT: f64 = 3000.0;
const ROTATE_RANGE: std::ops::RangeInclusive<i32> = -90..=90;
const POWER_RANGE: std::ops::RangeInclusive<i32> = 0..=4;
const ROTATE_STEP: i32 = 15;
const MAX_LANDING_H_SPEED: f64 = 20.0;
const MAX_LANDING_V_SPEED: f64 = 40.0;

type Point = (f64, f64);

#[derive(Deserialize)]
struct Test {
	surface: Vec<(i32, i32)>,
	x: i32,
	y: i32,
	h_speed: i32,
	v_speed: i32,
	fuel: i32,
	rotate: i32,
	power: i32,
}

struct Lander {
	surface: Vec<(i32, i32)>,
	x: f64,
	y: f64,
	h_speed: f64,
	v_speed: f64,
	fuel: i32,
	rotate: i32,
	power: i32,
}

enum Outcome {
	Flying,
	Landed,
	Crashed(String),
}

fn cross(a: Point, b: Point) -> f64 {
	a.0 * b.1 - a.1 * b.0
}

fn hit_time(from: Point, to: Point, start: Point, end: Point) -> Option<f64> {
	let path = (to.0 - from.0, to.1 - from.1);
	let segment = (end.0 - start.0, end.1 - start.1);
	let denominator = cross(path, segment);
	if denominator == 0.0 {
		return None;
	}
	let offset = (start.0 - from.0, start.1 - from.1);
	let path_fraction = cross(offset, segment) / denominator;
	let segment_fraction = cross(offset, path) / denominator;
	((0.0..=1.0).contains(&path_fraction) && (0.0..=1.0).contains(&segment_fraction))
		.then_some(path_fraction)
}

fn parse_command(line: &str) -> Result<(i32, i32), String> {
	let values: Vec<i32> = line
		.split_whitespace()
		.map(str::parse)
		.collect::<Result<_, _>>()
		.map_err(|_| format!("invalid command {line:?}"))?;
	let [rotate, power] = values[..] else {
		return Err(format!("invalid command {line:?}"));
	};
	if !ROTATE_RANGE.contains(&rotate) {
		return Err(format!("rotation {rotate} outside -90..90"));
	}
	if !POWER_RANGE.contains(&power) {
		return Err(format!("power {power} outside 0..4"));
	}
	Ok((rotate, power))
}

impl Lander {
	fn display(&self) -> [i64; 7] {
		[
			self.x.round() as i64,
			self.y.round() as i64,
			self.h_speed.round() as i64,
			self.v_speed.round() as i64,
			i64::from(self.fuel),
			i64::from(self.rotate),
			i64::from(self.power),
		]
	}

	fn first_hit(&self, from: Point, to: Point) -> Option<(f64, bool)> {
		self.surface
			.windows(2)
			.filter_map(|pair| {
				let start = (f64::from(pair[0].0), f64::from(pair[0].1));
				let end = (f64::from(pair[1].0), f64::from(pair[1].1));
				hit_time(from, to, start, end).map(|time| (time, pair[0].1 == pair[1].1))
			})
			.min_by(|a, b| a.0.total_cmp(&b.0))
	}

	fn advance(&mut self, rotate: i32, power: i32) -> Outcome {
		self.rotate += (rotate - self.rotate).clamp(-ROTATE_STEP, ROTATE_STEP);
		self.power = if self.fuel == 0 {
			0
		} else {
			self.power + (power - self.power).clamp(-1, 1)
		};
		self.fuel = (self.fuel - self.power).max(0);
		let angle = f64::from(self.rotate).to_radians();
		let thrust = f64::from(self.power);
		let h_acceleration = -thrust * angle.sin();
		let v_acceleration = thrust * angle.cos() - GRAVITY;
		let from = (self.x, self.y);
		let to = (
			self.x + self.h_speed + 0.5 * h_acceleration,
			self.y + self.v_speed + 0.5 * v_acceleration,
		);
		self.h_speed += h_acceleration;
		self.v_speed += v_acceleration;
		(self.x, self.y) = to;
		if let Some((time, flat)) = self.first_hit(from, to) {
			self.x = from.0 + time * (to.0 - from.0);
			self.y = from.1 + time * (to.1 - from.1);
			let (h_speed, v_speed, rotate) = (self.h_speed, self.v_speed, self.rotate);
			if !flat {
				return Outcome::Crashed("crashed on non-flat ground".to_string());
			}
			if rotate != 0
				|| h_speed.abs() > MAX_LANDING_H_SPEED
				|| v_speed.abs() > MAX_LANDING_V_SPEED
			{
				return Outcome::Crashed(format!(
					"bad landing: rotate {rotate}, hSpeed {h_speed:.3}, vSpeed {v_speed:.3}"
				));
			}
			return Outcome::Landed;
		}
		if !(0.0..WIDTH).contains(&self.x) || !(0.0..HEIGHT).contains(&self.y) {
			return Outcome::Crashed("left the zone".to_string());
		}
		Outcome::Flying
	}
}

impl Game for Lander {
	type Test = Test;

	const FIRST_TURN: Duration = Duration::from_millis(1000);
	const TURN: Duration = Duration::from_millis(150);

	fn new(test: Test) -> Self {
		Self {
			surface: test.surface,
			x: f64::from(test.x),
			y: f64::from(test.y),
			h_speed: f64::from(test.h_speed),
			v_speed: f64::from(test.v_speed),
			fuel: test.fuel,
			rotate: test.rotate,
			power: test.power,
		}
	}

	fn initial_input(&self) -> String {
		let count = self.surface.len();
		let mut input = format!("{count}\n");
		for (x, y) in &self.surface {
			input += &format!("{x} {y}\n");
		}
		input
	}

	fn turn_input(&self) -> String {
		let [x, y, h_speed, v_speed, fuel, rotate, power] = self.display();
		format!("{x} {y} {h_speed} {v_speed} {fuel} {rotate} {power}\n")
	}

	fn play(&mut self, action: &[String]) -> Step {
		let (rotate, power) = match parse_command(&action[0]) {
			Ok(command) => command,
			Err(reason) => return Step::Lose(reason),
		};
		match self.advance(rotate, power) {
			Outcome::Flying => Step::Continue,
			Outcome::Landed => Step::Win,
			Outcome::Crashed(reason) => {
				let [x, y, ..] = self.display();
				Step::Lose(format!("{reason} at ({x},{y})"))
			}
		}
	}
}

fn main() -> ExitCode {
	referee_core::main(Interactive::<Lander>::default())
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs;
	use std::path::Path;

	fn numbers(line: &str) -> Vec<i64> {
		line.split(|c: char| !(c.is_ascii_digit() || c == '-'))
			.filter(|token| !token.is_empty() && *token != "-")
			.map(|token| token.parse().unwrap())
			.collect()
	}

	fn replay(log: &str, lander: &mut Lander) -> usize {
		let mut states = Vec::new();
		let mut commands = Vec::new();
		let mut position_line = Vec::new();
		for line in log.lines().map(str::trim) {
			if line.starts_with("X=") {
				position_line = numbers(line);
			} else if line.starts_with("Fuel=") {
				let mut state = position_line.clone();
				state.extend(&numbers(line)[..3]);
				states.push(state);
			} else if let Some(command) = line.strip_prefix("stdout: ") {
				commands.push(command.to_string());
			}
		}
		assert_eq!(lander.display().to_vec(), states[0]);
		let mut checked = 0;
		for (command, expected) in commands.iter().zip(&states[1..]) {
			let Ok((rotate, power)) = parse_command(command) else {
				break;
			};
			let outcome = lander.advance(rotate, power);
			let shown_fields = if matches!(outcome, Outcome::Flying) {
				0
			} else {
				2
			};
			assert_eq!(
				lander.display()[shown_fields..],
				expected[shown_fields..],
				"frame {checked}"
			);
			checked += 1;
			if shown_fields > 0 {
				break;
			}
		}
		checked
	}

	#[test]
	fn replays_match_codingame_frames() {
		let referee_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
		for entry in fs::read_dir(referee_dir.join("replays")).unwrap() {
			let path = entry.unwrap().path();
			let name = path.file_name().unwrap().to_string_lossy().to_string();
			let fixture = referee_dir.join(format!("tests/{prefix}.json", prefix = &name[..2]));
			let test: Test = serde_json::from_str(&fs::read_to_string(fixture).unwrap()).unwrap();
			let mut lander = Lander::new(test);
			let checked = replay(&fs::read_to_string(&path).unwrap(), &mut lander);
			assert!(checked > 10, "{name}: only {checked} frames");
		}
	}
}
