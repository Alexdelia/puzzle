use std::{fmt::Display, io, str::FromStr};

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

// type Turn = u16;
// const MAX_TURN: Turn = 300;

const FIRST_TURN_MS_LIMIT: u64 = 1000;
const TURN_MS_LIMIT: u64 = 50;

type Axis = u8;
// const MAX_H = 11;
// const MAX_W = MAX_H * 2;

type Coord = (Axis, Axis);

// TODO: find max troll id
type TrollId = u8;

// TODO: find max resource amount
type Resource = u16;

// TODO: check if can store resource in uint
// (128/6 ~= 21; 2^21 = 2_097_152)
// (64/6 ~= 10; 2^10 = 1_024)
enum ResourceKind {
	Plum = 0,
	Lemon = 1,
	Apple = 2,
	Banana = 3,
	Iron = 4,
	Wood = 5,
}

impl FromStr for ResourceKind {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"PLUM" => Ok(Self::Plum),
			"LEMON" => Ok(Self::Lemon),
			"APPLE" => Ok(Self::Apple),
			"BANANA" => Ok(Self::Banana),
			"IRON" => Ok(Self::Iron),
			"WOOD" => Ok(Self::Wood),
			_ => Err(()),
		}
	}
}

#[derive(Clone, Copy, Default)]
struct PlayerInventory {
	plum: Resource,
	lemon: Resource,
	apple: Resource,
	banana: Resource,
	iron: Resource,
	wood: Resource,
}

struct Tree {
	kind: ResourceKind,
	pos: Coord,
	size: i32,   // TODO: find best type
	health: i32, // TODO: find best type
	fruit: u8,
	cooldown: i32, // TODO: find best type
}

impl Tree {
	const MAX_FRUIT: u8 = 3;
}

struct Troll {
	id: TrollId,
	my_troll: bool, // TODO: check if necessary to store
	pos: Coord,
	movement_speed: i32,    // TODO: find best type
	carry_capacity: i32,    // TODO: find best type
	harvest_power: i32,     // TODO: find best type
	chop_power: i32,        // TODO: find best type
	carry: PlayerInventory, // TODO: check if necessary to store
}

struct Env {
	w: Axis,
	h: Axis,

	grass_grid: Vec<Vec<bool>>, // TODO: optimize to 11*22

	my_shack: Coord,
	op_shack: Coord,
}

struct TurnState {
	my_inventory: PlayerInventory,
	op_inventory: PlayerInventory,

	my_troll_list: Vec<Troll>,
	op_troll_list: Vec<Troll>,

	tree_list: Vec<Tree>,
}

enum Action {
	Move(TrollId, Coord),
	Harvest(TrollId),
	Drop(TrollId),
}

impl Display for Action {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Action::Move(id, (x, y)) => write!(f, "MOVE {id} {x} {y}"),
			Action::Harvest(id) => write!(f, "HARVEST {id}"),
			Action::Drop(id) => write!(f, "DROP {id}"),
		}
	}
}

/* grid:
N = player id shack ?
. = grass
*/
impl Env {
	fn read() -> Self {
		let mut s = String::new();

		io::stdin().read_line(&mut s).unwrap();
		let input = s.split(" ").collect::<Vec<_>>();
		let w = parse_input!(input[0], Axis);
		let h = parse_input!(input[1], Axis);

		let mut my_shack = Coord::default();
		let mut op_shack = Coord::default();

		let grass_grid = Vec::with_capacity(h as usize);
		for y in 0..h {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();

			for (x, c) in s.trim_matches('\n').chars().enumerate() {
				match c {
					'.' => {
						// TODO: store grass
					}
					'0' => {
						my_shack = (x as Axis, y);
					}
					'1' => {
						op_shack = (x as Axis, y);
					}
					_ => panic!("invalid grid character '{c}' at ({x}, {y})"),
				}
			}
		}

		Self {
			w,
			h,
			grass_grid,
			my_shack,
			op_shack,
		}
	}
}

impl Tree {
	fn read() -> Self {
		let mut s = String::new();
		io::stdin().read_line(&mut s).unwrap();
		let input = s.split(" ").collect::<Vec<_>>();

		Self {
			kind: parse_input!(input[0], ResourceKind),
			pos: (parse_input!(input[1], Axis), parse_input!(input[2], Axis)),
			size: parse_input!(input[3], i32),
			health: parse_input!(input[4], i32),
			fruit: parse_input!(input[5], u8),
			cooldown: parse_input!(input[6], i32),
		}
	}
}

impl PlayerInventory {
	fn read() -> Self {
		let mut s = String::new();
		io::stdin().read_line(&mut s).unwrap();
		let input = s.split(" ").collect::<Vec<_>>();

		Self::parse(&input)
	}

	fn parse(input: &[&str]) -> Self {
		Self {
			plum: input[0].trim().parse().expect("failed to parse plum"),
			lemon: input[1].trim().parse().expect("failed to parse lemon"),
			apple: input[2].trim().parse().expect("failed to parse apple"),
			banana: input[3].trim().parse().expect("failed to parse banana"),
			iron: input[4].trim().parse().expect("failed to parse iron"),
			wood: input[5].trim().parse().expect("failed to parse wood"),
		}
	}
}

impl Troll {
	fn read() -> Self {
		let mut s = String::new();
		io::stdin().read_line(&mut s).unwrap();
		let input = s.split(" ").collect::<Vec<_>>();

		Self {
			id: parse_input!(input[0], TrollId),
			my_troll: parse_input!(input[0], u8) == 0,
			pos: (parse_input!(input[2], Axis), parse_input!(input[3], Axis)),
			movement_speed: parse_input!(input[4], i32),
			carry_capacity: parse_input!(input[5], i32),
			harvest_power: parse_input!(input[6], i32),
			chop_power: parse_input!(input[7], i32),
			carry: PlayerInventory::parse(&input[8..]),
		}
	}
}

impl TurnState {
	fn read() -> Self {
		let mut s = String::new();

		let my_inventory = PlayerInventory::read();
		let op_inventory = PlayerInventory::read();

		io::stdin().read_line(&mut s).unwrap();
		let tree_count = parse_input!(s, usize);

		let mut tree_list = Vec::with_capacity(tree_count);
		for _ in 0..tree_count {
			tree_list.push(Tree::read());
		}

		s.clear();
		io::stdin().read_line(&mut s).unwrap();
		let troll_count = parse_input!(s, usize);
		let player_troll_count = troll_count / 2;

		// TODO: check if not better to allocate on stack with [Troll; MAX_TROLL_COUNT]
		let mut my_troll_list = Vec::with_capacity(player_troll_count);
		let mut op_troll_list = Vec::with_capacity(player_troll_count);
		for _ in 0..troll_count {
			let troll = Troll::read();
			if troll.my_troll {
				my_troll_list.push(troll);
			} else {
				op_troll_list.push(troll);
			}
		}

		Self {
			my_inventory,
			op_inventory,
			my_troll_list,
			op_troll_list,
			tree_list,
		}
	}
}

// TODO: cache??
/// manhattan distance
fn dist(a: Coord, b: Coord) -> u8 {
	(a.0 as i8 - b.0 as i8).abs() as u8 + (a.1 as i8 - b.1 as i8).abs() as u8
}

fn drop_to_shack(troll: &Troll, env: &Env) -> Action {
	if (troll.pos.0 == env.my_shack.0 && (troll.pos.1 as i8 - env.my_shack.1 as i8).abs() == 1)
		|| (troll.pos.1 == env.my_shack.1 && (troll.pos.0 as i8 - env.my_shack.0 as i8).abs() == 1)
	{
		Action::Drop(troll.id)
	} else {
		Action::Move(troll.id, env.my_shack)
	}
}

fn harvest_tree(troll: &Troll, tree: &Tree) -> Action {
	if troll.pos.0 == tree.pos.0 && troll.pos.1 == tree.pos.1 {
		Action::Harvest(troll.id)
	} else {
		Action::Move(troll.id, tree.pos)
	}
}

fn find_best_tree<'a>(_env: &'a Env, state: &'a TurnState, troll: &'a Troll) -> Option<&'a Tree> {
	let mut best_tree = None;
	let mut best_tree_dist = u8::MAX;

	for tree in &state.tree_list {
		if tree.fruit == 0 {
			continue;
		}

		let Some(_best) = best_tree else {
			best_tree = Some(tree);
			continue;
		};

		let tree_dist = dist(troll.pos, tree.pos);
		if tree_dist < best_tree_dist {
			best_tree = Some(tree);
			best_tree_dist = tree_dist;
		}
	}

	best_tree
}

fn solve(env: &Env, state: &TurnState) -> Vec<Action> {
	// TODO: check if not better to allocate on stack with [Action; MAX_TROLL_COUNT]
	let mut action_list = Vec::with_capacity(state.my_troll_list.len());

	for troll in &state.my_troll_list {
		if troll.carry.plum > 0
			|| troll.carry.lemon > 0
			|| troll.carry.apple > 0
			|| troll.carry.banana > 0
			|| troll.carry.iron > 0
			|| troll.carry.wood > 0
		{
			action_list.push(drop_to_shack(troll, env));
			continue;
		}

		if let Some(best_tree) = find_best_tree(env, state, troll) {
			action_list.push(harvest_tree(troll, best_tree));
		} else {
			action_list.push(drop_to_shack(troll, env));
		}
	}

	action_list
}

fn main() {
	let env = Env::read();

	loop {
		let state = TurnState::read();

		let action_list = solve(&env, &state);
		println!(
			"{}",
			action_list
				.into_iter()
				.map(|a| a.to_string())
				.collect::<Vec<_>>()
				.join(";")
		);
	}
}
