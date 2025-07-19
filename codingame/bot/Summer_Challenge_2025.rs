use core::{fmt::Display, str::FromStr};
use std::{
	collections::{BTreeMap, HashMap, HashSet},
	io,
};

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

fn read_line() -> String {
	let mut input = String::new();
	io::stdin().read_line(&mut input).unwrap();
	input
}

type Id = u8;
type Coord = (usize, usize);
type Grid = Vec<Vec<Cell>>;

struct Env {
	player_id: Id,

	ally: HashMap<Id, Agent>,
	foe: HashMap<Id, Agent>,

	w: usize,
	h: usize,
	grid: Grid,

	covered_cells: Vec<Coord>,
}

type ShootCooldown = u8;
type Wetness = u8;

struct Agent {
	id: Id,
	ally: bool,
	pos: Coord,
	wet: Wetness,

	total_shoot_cooldown: ShootCooldown,
	current_shoot_cooldown: ShootCooldown,
	optimal_range: usize,
	shoot_power: Wetness,
	slash_bombs: u8,

	actions: AgentActions,
}

struct AgentActions {
	r#move: Option<Coord>,
	move_priority: MovePriorityQueue,
	shoot: Option<Id>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
	Empty,
	Cover50,
	Cover75,
}

impl Env {
	fn parse() -> Self {
		let player_id = parse_input!(read_line(), Id);

		let total_agent_count = parse_input!(read_line(), usize);
		let player_agent_count = total_agent_count / 2;

		let mut ally = HashMap::with_capacity(player_agent_count);
		let mut foe = HashMap::with_capacity(player_agent_count);

		for _ in 0..total_agent_count {
			let agent = Agent::parse(player_id);

			if agent.ally {
				ally.insert(agent.id, agent);
			} else {
				foe.insert(agent.id, agent);
			}
		}

		let input = read_line();
		let inputs = input.split_whitespace().collect::<Vec<_>>();
		let w = parse_input!(inputs[0], usize);
		let h = parse_input!(inputs[1], usize);

		let grid = Self::_parse_grid(w, h);

		let covered_cells = Self::_get_covered_cells(w, h, &grid);

		Self {
			player_id,
			ally,
			foe,
			w,
			h,
			grid,
			covered_cells,
		}
	}

	fn _parse_grid(_w: usize, h: usize) -> Vec<Vec<Cell>> {
		let mut grid = Vec::with_capacity(h);

		for _ in 0..h {
			let input = read_line();

			grid.push(
				input
					.split_whitespace()
					.skip(2)
					.step_by(3)
					.map(|s| s.parse::<Cell>().unwrap())
					.collect::<Vec<Cell>>(),
			);
		}

		grid
	}

	fn _get_covered_cells(w: usize, h: usize, grid: &Grid) -> Vec<Coord> {
		let mut covered_cells = HashSet::new();

		for y in 0..h {
			for x in 0..w {
				if !grid[y][x].is_cover() {
					continue;
				}

				if y > 0 && !grid[y - 1][x].is_cover() {
					covered_cells.insert((x, y - 1));
				}
				if y < h - 1 && !grid[y + 1][x].is_cover() {
					covered_cells.insert((x, y + 1));
				}
				if x > 0 && !grid[y][x - 1].is_cover() {
					covered_cells.insert((x - 1, y));
				}
				if x < w - 1 && !grid[y][x + 1].is_cover() {
					covered_cells.insert((x + 1, y));
				}
			}
		}

		covered_cells.into_iter().collect::<Vec<Coord>>()
	}

	fn update(&mut self) {
		let total_alive_agent_count = parse_input!(read_line(), usize);

		for _ in 0..total_alive_agent_count {
			let input = read_line();
			let inputs = input.split(" ").collect::<Vec<_>>();
			let id = parse_input!(inputs[0], Id);

			let agent = self
				.ally
				.get_mut(&id)
				.unwrap_or(self.foe.get_mut(&id).expect("agent does not exist"));

			agent.update(&inputs);
		}
		// TODO: remove dead agents

		let _player_agent_count = read_line();
	}

	fn compute_move_priority(&mut self) {
		for agent in self.ally.values_mut() {
			agent.actions.move_priority.clear();
		}

		for cell in &self.covered_cells {
			let total_max_incoming_damage = self.compute_max_incoming_damage(*cell);

			for agent in self.ally.values_mut() {
				// TODO: path finding because obstacles will increase distance
				let distance = dist(agent.pos, *cell);

				let priority = MovePriority {
					distance,
					total_max_incoming_damage,
				};

				agent.actions.move_priority.insert(priority, *cell);
			}
		}
	}

	// TODO: compute max incoming damage for all covered cells once per turn
	fn compute_max_incoming_damage(&self, pos: Coord) -> usize {
		let mut sum: usize = 0;
		for agent in self.foe.values() {
			sum += compute_damage(&self.grid, agent, agent.pos, pos) as usize;
		}
		sum
	}
}

impl Agent {
	fn parse(player_id: Id) -> Self {
		let input = read_line();
		let inputs: Vec<&str> = input.split_whitespace().collect();

		Self {
			id: parse_input!(inputs[0], Id),
			ally: parse_input!(inputs[1], Id) == player_id,
			pos: (0, 0),
			wet: 0,

			total_shoot_cooldown: parse_input!(inputs[2], ShootCooldown),
			current_shoot_cooldown: 0,
			optimal_range: parse_input!(inputs[3], usize),
			shoot_power: parse_input!(inputs[4], Wetness),
			slash_bombs: parse_input!(inputs[5], u8),

			actions: AgentActions {
				r#move: None,
				move_priority: BTreeMap::new(),
				shoot: None,
			},
		}
	}

	fn update(&mut self, inputs: &[&str]) {
		self.pos = (
			parse_input!(inputs[1], usize),
			parse_input!(inputs[2], usize),
		);
		self.current_shoot_cooldown = parse_input!(inputs[3], ShootCooldown);
		self.slash_bombs = parse_input!(inputs[4], u8);
		self.wet = parse_input!(inputs[5], Wetness);
	}
}

impl FromStr for Cell {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"0" => Ok(Cell::Empty),
			"1" => Ok(Cell::Cover50),
			"2" => Ok(Cell::Cover75),
			_ => Err(()),
		}
	}
}

impl Cell {
	fn is_cover(&self) -> bool {
		matches!(self, Cell::Cover50 | Cell::Cover75)
	}

	fn damage_reduction_factor(&self) -> f32 {
		match self {
			Cell::Empty => 1.0,
			Cell::Cover50 => 0.5,
			Cell::Cover75 => 0.25,
		}
	}
}

enum Action {
	Move(Coord),
	Shoot(Id),
	// Throw,
	SelfCover25,
	Message(String),
}

impl Display for Action {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Action::Move((x, y)) => write!(f, "MOVE {x} {y}"),
			Action::Shoot(id) => write!(f, "SHOOT {id}"),
			// Action::Throw => write!(f, "THROW"),
			Action::SelfCover25 => write!(f, "HUNKER_DOWN"),
			Action::Message(msg) => write!(f, "MESSAGE {msg}"),
		}
	}
}

type MovePriorityQueue = BTreeMap<MovePriority, Coord>;

struct MovePriority {
	distance: usize,
	total_max_incoming_damage: usize,
}

impl Ord for MovePriority {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.distance
			.max(1)
			.cmp(&other.distance.max(1))
			.then_with(|| {
				self.total_max_incoming_damage
					.cmp(&other.total_max_incoming_damage)
			})
	}
}

impl PartialOrd for MovePriority {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl PartialEq for MovePriority {
	fn eq(&self, other: &Self) -> bool {
		self.distance.max(1) == other.distance.max(1)
			&& self.total_max_incoming_damage == other.total_max_incoming_damage
	}
}

impl Eq for MovePriority {}

fn compute_damage(grid: &Grid, agent: &Agent, from: Coord, to: Coord) -> Wetness {
	if agent.current_shoot_cooldown > 0 {
		return 0;
	}

	let mut base = agent.shoot_power;

	let distance = dist(from, to);

	if distance > agent.optimal_range * 2 {
		return 0;
	} else if distance > agent.optimal_range {
		base /= 2;
	}

	if distance <= 2 {
		return base;
	}

	let dx = from.0 as isize - to.0 as isize;
	let dy = from.1 as isize - to.1 as isize;

	let vertical_cover = if dx > 0 {
		grid[to.1][to.0 - 1]
	} else {
		grid[to.1][to.0 + 1]
	};
	let horizontal_cover = if dy > 0 {
		grid[to.1 - 1][to.0]
	} else {
		grid[to.1 + 1][to.0]
	};

	let damage_reduction_factor = vertical_cover
		.damage_reduction_factor()
		.min(horizontal_cover.damage_reduction_factor());

	return (base as f32 * damage_reduction_factor) as Wetness;
}

fn dist(src: Coord, dst: Coord) -> usize {
	((src.0 as isize - dst.0 as isize).abs() + (src.1 as isize - dst.1 as isize).abs()) as usize
}

fn main() {
	let mut e = Env::parse();

	loop {
		e.update();

		e.compute_move_priority();

		// TODO: check that 2+ agents don't overkill a single enemy and then waste a shoot
		// TODO: check that 2+ agents don't move to the same cell

		for ally in e.ally.values_mut() {
			let mut actions = Vec::<Action>::new();

			if let Some(target) = ally.actions.move_priority.first_entry() {
				let pos = *target.get();
				actions.push(Action::Move(pos));
				ally.pos = pos;
			}

			let mut most_damage_id = None;
			let mut most_damage_value = 0;
			for foe in e.foe.values() {
				let damage = compute_damage(&e.grid, ally, ally.pos, foe.pos);
				if damage > most_damage_value {
					most_damage_value = damage;
					most_damage_id = Some(foe.id);
				}
			}
			if let Some(id) = most_damage_id {
				actions.push(Action::Shoot(id));
			}

			let actions = actions
				.into_iter()
				.map(|a| a.to_string())
				.collect::<Vec<_>>();

			println!("{id};{actions}", id = ally.id, actions = actions.join(";"));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_priority_queue_basic() {
		let mut queue = MovePriorityQueue::new();
		queue.insert(
			MovePriority {
				distance: 2,
				total_max_incoming_damage: 3,
			},
			(1, 1),
		);
		queue.insert(
			MovePriority {
				distance: 1,
				total_max_incoming_damage: 5,
			},
			(2, 2),
		);

		assert_eq!(queue.len(), 2);
		assert_eq!(queue.keys().next().unwrap().distance, 1);
	}

	#[test]
	fn test_priority_queue_equal_distances() {
		let mut queue = MovePriorityQueue::new();
		queue.insert(
			MovePriority {
				distance: 1,
				total_max_incoming_damage: 3,
			},
			(0, 0),
		);
		queue.insert(
			MovePriority {
				distance: 1,
				total_max_incoming_damage: 2,
			},
			(1, 1),
		);

		assert_eq!(queue.len(), 2);
		let first = queue.keys().next().unwrap();
		assert_eq!(first.distance, 1);
		assert_eq!(first.total_max_incoming_damage, 2);
	}

	#[test]
	fn test_priority_queue_technically_equal_distance() {
		let mut queue = MovePriorityQueue::new();
		queue.insert(
			MovePriority {
				distance: 1,
				total_max_incoming_damage: 2,
			},
			(0, 0),
		);
		queue.insert(
			MovePriority {
				distance: 0,
				total_max_incoming_damage: 3,
			},
			(1, 1),
		);

		assert_eq!(queue.len(), 2);
		let first = queue.keys().next().unwrap();
		assert_eq!(first.distance, 1);
		assert_eq!(first.total_max_incoming_damage, 2);
	}
}
