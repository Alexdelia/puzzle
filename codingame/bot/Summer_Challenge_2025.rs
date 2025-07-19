use core::str::FromStr;
use std::{collections::HashMap, io};

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

type Id = u8;
type Coord = (usize, usize);

struct Env {
	player_id: Id,

	ally: HashMap<Id, Agent>,
	foe: HashMap<Id, Agent>,

	w: usize,
	h: usize,
	grid: Vec<Vec<Cell>>,
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
}

enum Cell {
	Empty,
	Agent,
	Cover50,
	Cover75,
}

impl Env {
	fn parse() -> Self {
		let mut input = String::new();

		io::stdin().read_line(&mut input).unwrap();
		let player_id = parse_input!(input, Id);

		io::stdin().read_line(&mut input).unwrap();
		let total_agent_count = parse_input!(input, usize);
		let player_agent_count = total_agent_count / 2;

		let mut ally = HashMap::with_capacity(player_agent_count);
		let mut foe = HashMap::with_capacity(player_agent_count);

		for _ in 0..total_agent_count {
			io::stdin().read_line(&mut input).unwrap();

			let agent = Agent::parse(&input, player_id);

			if agent.ally {
				ally.insert(agent.id, agent);
			} else {
				foe.insert(agent.id, agent);
			}
		}

		io::stdin().read_line(&mut input).unwrap();
		let inputs = input.split_whitespace().collect::<Vec<_>>();
		let w = parse_input!(inputs[0], usize);
		let h = parse_input!(inputs[1], usize);

		let grid = parse_grid(&mut input, w, h);

		Self {
			player_id,
			ally,
			foe,
			w,
			h,
			grid,
		}
	}

	fn update(&mut self) {
		let mut input = String::new();
		io::stdin().read_line(&mut input).unwrap();

		let total_alive_agent_count = parse_input!(input, usize);

		for _ in 0..total_alive_agent_count {
			io::stdin().read_line(&mut input).unwrap();

			let inputs = input.split(" ").collect::<Vec<_>>();
			let id = parse_input!(inputs[0], Id);

			let is_ally = parse_input!(inputs[1], Id) == self.player_id;
			let set = if is_ally {
				&mut self.ally
			} else {
				&mut self.foe
			};
			let agent = set.get_mut(&id).expect("agent does not exist");

			self.grid[agent.pos.1][agent.pos.0] = Cell::Empty;

			agent.update(&inputs);

			self.grid[agent.pos.1][agent.pos.0] = Cell::Agent;
		}

		// player agent count
		io::stdin().read_line(&mut input).unwrap();
	}
}

impl Agent {
	fn parse(input: &str, player_id: Id) -> Self {
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

fn parse_grid(buf: &mut String, _w: usize, h: usize) -> Vec<Vec<Cell>> {
	let mut grid = Vec::with_capacity(h);

	for _ in 0..h {
		io::stdin().read_line(buf).unwrap();

		grid.push(
			buf.split_whitespace()
				.skip(2)
				.step_by(3)
				.map(|s| s.parse::<Cell>().unwrap())
				.collect::<Vec<Cell>>(),
		);
	}

	grid
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

fn main() {
	let mut e = Env::parse();

	loop {
		e.update();

		for agent in e.ally.values() {
			println!("{id}; HUNKER_DOWN", id = agent.id);
		}
	}
}
