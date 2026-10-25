mod runner;

use std::process::ExitCode;
use std::time::Duration;

use referee_core::{Game, Step};
use runner::ClosedStdin;
use serde::Deserialize;

const FUSE: u32 = 3;
const RANGE: i32 = 3;
const NODE: u8 = b'@';
const PASSIVE: u8 = b'#';
const EMPTY: u8 = b'.';

#[derive(Deserialize)]
struct Test {
	rounds: u32,
	bombs: u32,
	grid: Vec<String>,
}

struct Bomb {
	x: i32,
	y: i32,
	fuse: u32,
}

struct Firewall {
	width: i32,
	height: i32,
	grid: Vec<Vec<u8>>,
	rounds: u32,
	bombs: u32,
	armed: Vec<Bomb>,
}

enum Action {
	Wait,
	Place(i32, i32),
}

fn parse_coordinate(token: &str) -> Option<i32> {
	if token.is_empty() || !token.bytes().all(|byte| byte.is_ascii_digit()) {
		return None;
	}
	Some(token.parse().unwrap_or(i32::MAX))
}

fn parse_action(line: &str) -> Option<Action> {
	if line.eq_ignore_ascii_case("WAIT") {
		return Some(Action::Wait);
	}
	let (x, y) = line.split_once(' ')?;
	Some(Action::Place(parse_coordinate(x)?, parse_coordinate(y)?))
}

impl Firewall {
	fn contains(&self, x: i32, y: i32) -> bool {
		(0..self.width).contains(&x) && (0..self.height).contains(&y)
	}

	fn cell(&self, x: i32, y: i32) -> u8 {
		self.grid[y as usize][x as usize]
	}

	fn blast(&self, x: i32, y: i32) -> Vec<(i32, i32)> {
		let mut cells = vec![(x, y)];
		for (step_x, step_y) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
			for distance in 1..=RANGE {
				let (cell_x, cell_y) = (x + step_x * distance, y + step_y * distance);
				if !self.contains(cell_x, cell_y) || self.cell(cell_x, cell_y) == PASSIVE {
					break;
				}
				cells.push((cell_x, cell_y));
			}
		}
		cells
	}

	fn detonate(&mut self) {
		for bomb in &mut self.armed {
			bomb.fuse -= 1;
		}
		let mut exploding: Vec<(i32, i32)> = Vec::new();
		while let Some(index) = self.armed.iter().position(|bomb| bomb.fuse == 0) {
			let bomb = self.armed.swap_remove(index);
			exploding.push((bomb.x, bomb.y));
		}
		while let Some((x, y)) = exploding.pop() {
			for (cell_x, cell_y) in self.blast(x, y) {
				if self.cell(cell_x, cell_y) == NODE {
					self.grid[cell_y as usize][cell_x as usize] = EMPTY;
				}
				while let Some(index) = self
					.armed
					.iter()
					.position(|bomb| (bomb.x, bomb.y) == (cell_x, cell_y))
				{
					self.armed.swap_remove(index);
					exploding.push((cell_x, cell_y));
				}
			}
		}
	}

	fn place(&mut self, x: i32, y: i32) -> Result<(), String> {
		if self.bombs == 0 {
			return Ok(());
		}
		let (width, height) = (self.width, self.height);
		if !(0..width).contains(&x) {
			return Err(format!("expected 0<=x<{width}, found x={x}"));
		}
		if !(0..height).contains(&y) {
			return Err(format!("expected 0<=y<{height}, found y={y}"));
		}
		if self.cell(x, y) != EMPTY {
			return Err(format!("bomb placed on a node at ({x}, {y})"));
		}
		self.armed.push(Bomb { x, y, fuse: FUSE });
		self.bombs -= 1;
		Ok(())
	}

	fn nodes_left(&self) -> bool {
		self.grid.iter().flatten().any(|&cell| cell == NODE)
	}
}

impl Game for Firewall {
	type Test = Test;

	const FIRST_TURN: Duration = Duration::from_millis(1000);
	const TURN: Duration = Duration::from_millis(140);

	fn new(test: Test) -> Self {
		let grid: Vec<Vec<u8>> = test.grid.into_iter().map(String::into_bytes).collect();
		Self {
			width: grid[0].len() as i32,
			height: grid.len() as i32,
			grid,
			rounds: test.rounds,
			bombs: test.bombs,
			armed: Vec::new(),
		}
	}

	fn initial_input(&self) -> String {
		let (width, height) = (self.width, self.height);
		let mut input = format!("{width} {height}\n");
		for row in &self.grid {
			input.push_str(&String::from_utf8_lossy(row));
			input.push('\n');
		}
		input
	}

	fn turn_input(&self) -> String {
		let (rounds, bombs) = (self.rounds, self.bombs);
		format!("{rounds} {bombs}\n")
	}

	fn play(&mut self, action: &[String]) -> Step {
		let line = action[0].as_str();
		let Some(action) = parse_action(line) else {
			return Step::Lose(format!("invalid action {line:?}"));
		};
		self.detonate();
		if let Action::Place(x, y) = action
			&& let Err(reason) = self.place(x, y)
		{
			return Step::Lose(reason);
		}
		if !self.nodes_left() {
			return Step::Win;
		}
		if self.bombs == 0 && self.armed.is_empty() {
			return Step::Lose("no bombs left".to_string());
		}
		if self.rounds == 1 {
			return Step::Lose("maximum amount of rounds reached".to_string());
		}
		self.rounds -= 1;
		Step::Continue
	}
}

fn main() -> ExitCode {
	referee_core::main(ClosedStdin::<Firewall>::default())
}
