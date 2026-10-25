mod runner;

use std::process::ExitCode;
use std::time::Duration;

use referee_core::{Game, Step};
use runner::ClosedStdin;
use serde::Deserialize;

const FUSE: u32 = 3;
const RANGE: i32 = 3;
const DIRECTIONS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

#[derive(Deserialize)]
struct Test {
	rounds: u32,
	bombs: u32,
	grid: Vec<String>,
	nodes: Vec<(i32, i32, i32, i32)>,
}

struct Node {
	position: (i32, i32),
	velocity: (i32, i32),
}

struct Bomb {
	position: (i32, i32),
	fuse: u32,
}

struct Vox {
	width: i32,
	height: i32,
	walls: Vec<Vec<bool>>,
	rounds: u32,
	bombs: u32,
	nodes: Vec<Node>,
	armed: Vec<Bomb>,
}

enum Action {
	Wait,
	Place(i32, i32),
}

fn parse_action(line: &str) -> Option<Action> {
	if line == "WAIT" {
		return Some(Action::Wait);
	}
	let (x, y) = line.split_once(' ')?;
	let is_number = |text: &str| !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit());
	if !is_number(x) || !is_number(y) {
		return None;
	}
	Some(Action::Place(
		x.parse().unwrap_or(i32::MAX),
		y.parse().unwrap_or(i32::MAX),
	))
}

impl Vox {
	fn is_open(&self, (x, y): (i32, i32)) -> bool {
		(0..self.width).contains(&x)
			&& (0..self.height).contains(&y)
			&& !self.walls[y as usize][x as usize]
	}

	fn has_node(&self, position: (i32, i32)) -> bool {
		self.nodes.iter().any(|node| node.position == position)
	}

	fn place(&mut self, x: i32, y: i32) -> Result<(), String> {
		let (width, height) = (self.width, self.height);
		if !(0..width).contains(&x) {
			return Err(format!("x={x} outside 0<=x<{width}"));
		}
		if !(0..height).contains(&y) {
			return Err(format!("y={y} outside 0<=y<{height}"));
		}
		if !self.is_open((x, y)) || self.has_node((x, y)) {
			return Err(format!("bomb placed on a node at ({x}, {y})"));
		}
		if self.bombs > 0 {
			self.bombs -= 1;
			self.armed.push(Bomb {
				position: (x, y),
				fuse: FUSE,
			});
		}
		Ok(())
	}

	fn move_nodes(&mut self) {
		let mut nodes = std::mem::take(&mut self.nodes);
		for node in &mut nodes {
			let (dx, dy) = node.velocity;
			if (dx, dy) == (0, 0) {
				continue;
			}
			let (x, y) = node.position;
			if self.is_open((x + dx, y + dy)) {
				node.position = (x + dx, y + dy);
				continue;
			}
			node.velocity = (-dx, -dy);
			if self.is_open((x - dx, y - dy)) {
				node.position = (x - dx, y - dy);
			}
		}
		self.nodes = nodes;
	}

	fn blast(&self, (x, y): (i32, i32)) -> Vec<(i32, i32)> {
		let mut cells = vec![(x, y)];
		for (dx, dy) in DIRECTIONS {
			for distance in 1..=RANGE {
				let cell = (x + dx * distance, y + dy * distance);
				if !self.is_open(cell) {
					break;
				}
				cells.push(cell);
			}
		}
		cells
	}

	fn explode(&mut self) {
		for bomb in &mut self.armed {
			bomb.fuse -= 1;
		}
		let mut burning = Vec::new();
		let mut pending = Vec::new();
		for bomb in std::mem::take(&mut self.armed) {
			if bomb.fuse == 0 {
				pending.push(bomb.position);
			} else {
				self.armed.push(bomb);
			}
		}
		while let Some(origin) = pending.pop() {
			let cells = self.blast(origin);
			let (triggered, intact) = std::mem::take(&mut self.armed)
				.into_iter()
				.partition(|bomb| cells.contains(&bomb.position));
			self.armed = intact;
			pending.extend(triggered.into_iter().map(|bomb: Bomb| bomb.position));
			burning.extend(cells);
		}
		self.nodes.retain(|node| !burning.contains(&node.position));
	}
}

impl Game for Vox {
	type Test = Test;

	const FIRST_TURN: Duration = Duration::from_millis(1000);
	const TURN: Duration = Duration::from_millis(150);

	fn new(test: Test) -> Self {
		let walls: Vec<Vec<bool>> = test
			.grid
			.iter()
			.map(|row| row.bytes().map(|cell| cell == b'#').collect())
			.collect();
		Self {
			width: walls[0].len() as i32,
			height: walls.len() as i32,
			walls,
			rounds: test.rounds,
			bombs: test.bombs,
			nodes: test
				.nodes
				.into_iter()
				.map(|(x, y, dx, dy)| Node {
					position: (x, y),
					velocity: (dx, dy),
				})
				.collect(),
			armed: Vec::new(),
		}
	}

	fn initial_input(&self) -> String {
		let (width, height) = (self.width, self.height);
		format!("{width} {height}\n")
	}

	fn turn_input(&self) -> String {
		let (rounds, bombs) = (self.rounds, self.bombs);
		let mut input = format!("{rounds} {bombs}\n");
		for y in 0..self.height {
			for x in 0..self.width {
				input.push(if self.walls[y as usize][x as usize] {
					'#'
				} else if self.has_node((x, y)) {
					'@'
				} else {
					'.'
				});
			}
			input.push('\n');
		}
		input
	}

	fn play(&mut self, action: &[String]) -> Step {
		if self.nodes.is_empty() {
			return Step::Win;
		}
		let line = action[0].as_str();
		match parse_action(line) {
			None => return Step::Lose(format!("invalid action {line:?}")),
			Some(Action::Wait) => {}
			Some(Action::Place(x, y)) => {
				if let Err(reason) = self.place(x, y) {
					return Step::Lose(reason);
				}
			}
		}
		if self.bombs == 0 && self.armed.is_empty() {
			return Step::Lose("no bombs left".to_string());
		}
		self.move_nodes();
		self.explode();
		self.rounds -= 1;
		if self.rounds == 0 {
			return Step::Lose("maximum amount of rounds reached".to_string());
		}
		Step::Continue
	}
}

fn main() -> ExitCode {
	referee_core::main(ClosedStdin::<Vox>::default())
}
