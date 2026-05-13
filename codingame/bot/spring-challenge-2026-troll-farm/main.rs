use std::{fmt::Display, io, str::FromStr};

// TODO: maybe start to chop only when op chop

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
const MAX_H: Axis = 11;
const MAX_W: Axis = MAX_H * 2;

type Coord = (Axis, Axis);

// TODO: find max troll id
type TrollId = u8;

// TODO: find max resource amount
type Resource = u16;

type MoveSpeed = Axis;
type CarryCapacity = Resource;
type HarvestPower = CarryCapacity;
type ChopPower = Resource;

/// 0..=4
type TreeSize = u8;
/// 0..=20
type TreeHealth = u8;
/// 0..=3
type TreeFruit = u8;
/// 0..=9
type TreeCooldown = u8;

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

impl Display for ResourceKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Plum => write!(f, "PLUM"),
			Self::Lemon => write!(f, "LEMON"),
			Self::Apple => write!(f, "APPLE"),
			Self::Banana => write!(f, "BANANA"),
			Self::Iron => write!(f, "IRON"),
			Self::Wood => write!(f, "WOOD"),
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
	is_next_to_water: bool,
	size: TreeSize,
	health: TreeHealth,
	fruit: TreeFruit,
	cooldown: TreeCooldown,
}

struct Troll {
	id: TrollId,
	my_troll: bool, // TODO: check if necessary to store
	pos: Coord,
	move_speed: MoveSpeed,
	carry_capacity: CarryCapacity,
	harvest_power: HarvestPower,
	chop_power: ChopPower,
	carry: PlayerInventory,
}

struct Env {
	grid: Grid,

	my_shack: Coord,
	op_shack: Coord,

	grass_next_to_water_list: Vec<Coord>,
	water_list: Vec<Coord>,
	rock_list: Vec<Coord>,
	iron_list: Vec<Coord>,
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
	Plant(TrollId, ResourceKind),
	Pick(TrollId, ResourceKind),
	Train(MoveSpeed, CarryCapacity, HarvestPower, ChopPower),
	Chop(TrollId),
	Mine(TrollId),
}

impl Display for Action {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Action::Move(id, (x, y)) => write!(f, "MOVE {id} {x} {y}"),
			Action::Harvest(id) => write!(f, "HARVEST {id}"),
			Action::Drop(id) => write!(f, "DROP {id}"),
			Action::Plant(id, kind) => write!(f, "PLANT {id} {kind}"),
			Action::Pick(id, kind) => write!(f, "PICK {id} {kind}"),
			Action::Train(ms, cc, hp, cp) => write!(f, "TRAIN {ms} {cc} {hp} {cp}"),
			Action::Chop(id) => write!(f, "CHOP {id}"),
			Action::Mine(id) => write!(f, "MINE {id}"),
		}
	}
}

#[derive(Default)]
struct Grid {
	g: [u64; MAX_H as usize],
	w: Axis,
	h: Axis,
}

impl Grid {
	const GRASS: u64 = 0b00;
	const WATER: u64 = 0b01;
	const IRON: u64 = 0b10;
	const ROCK: u64 = 0b11;

	fn read() -> (
		Self,
		Coord,
		Coord,
		Vec<Coord>,
		Vec<Coord>,
		Vec<Coord>,
		Vec<Coord>,
	) {
		let mut s = String::new();
		io::stdin().read_line(&mut s).unwrap();
		let input = s.split(" ").collect::<Vec<_>>();

		let mut grid = Self {
			g: [u64::MAX; MAX_H as usize],
			w: parse_input!(input[0], Axis),
			h: parse_input!(input[1], Axis),
		};

		let mut my_shack = Coord::default();
		let mut op_shack = Coord::default();

		let mut water_list = Vec::new();
		let mut rock_list = Vec::new();
		let mut iron_list = Vec::new();

		for y in 0..grid.h as usize {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();

			for (x, c) in s.trim_matches('\n').chars().enumerate() {
				let pos = (x as Axis, y as Axis);
				match c {
					'.' => {
						grid.set_cell(pos, Self::GRASS);
					}
					'~' => {
						grid.set_cell(pos, Self::WATER);
						water_list.push(pos);
					}
					'#' => {
						grid.set_cell(pos, Self::ROCK);
						rock_list.push(pos);
					}
					'+' => {
						grid.set_cell(pos, Self::IRON);
						iron_list.push(pos);
					}
					'0' => {
						my_shack = pos;
					}
					'1' => {
						op_shack = pos;
					}
					_ => panic!("invalid grid character '{c}' at ({x}, {y})"),
				}
			}
		}

		water_list.shrink_to_fit();
		rock_list.shrink_to_fit();
		iron_list.shrink_to_fit();

		let mut grass_next_to_water_list = Vec::new();
		for (x, y) in &water_list {
			for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
				let x = *x as i8 + dx;
				if x < 0 || x >= grid.w as i8 {
					continue;
				}
				let y = *y as i8 + dy;
				if y < 0 || y >= grid.h as i8 {
					continue;
				}

				let pos = (x as Axis, y as Axis);
				if grid.is_grass(pos) {
					grass_next_to_water_list.push(pos);
				}
			}
		}
		grass_next_to_water_list.shrink_to_fit();

		(
			grid,
			my_shack,
			op_shack,
			grass_next_to_water_list,
			water_list,
			rock_list,
			iron_list,
		)
	}

	fn set_cell(&mut self, (x, y): Coord, cell: u64) {
		self.g[y as usize] &= !(0b11 << (x * 2));
		self.g[y as usize] |= cell << (x * 2);
	}

	fn is_grass(&self, (x, y): Coord) -> bool {
		(self.g[y as usize] >> (x * 2)) & Self::GRASS != 0
	}

	fn is_water_next_to(&self, (x, y): Coord) -> bool {
		if x > 0 && (self.g[y as usize] >> ((x - 1) as usize * 2)) & Self::WATER != 0 {
			return true;
		}

		let right = x + 1;
		if right < self.w && (self.g[y as usize] >> (right as usize * 2)) & Self::WATER != 0 {
			return true;
		}

		if y > 0 && (self.g[(y - 1) as usize] >> (x as usize * 2)) & Self::WATER != 0 {
			return true;
		}

		let down = y + 1;
		if down < self.h && (self.g[down as usize] >> (x as usize * 2)) & Self::WATER != 0 {
			return true;
		}

		false
	}
}

impl Env {
	fn read() -> Self {
		let (grid, my_shack, op_shack, grass_next_to_water_list, water_list, rock_list, iron_list) =
			Grid::read();

		Self {
			grid,
			my_shack,
			op_shack,
			grass_next_to_water_list,
			water_list,
			rock_list,
			iron_list,
		}
	}
}

impl Tree {
	fn read(env: &Env) -> Self {
		let mut s = String::new();
		io::stdin().read_line(&mut s).unwrap();
		let input = s.split(" ").collect::<Vec<_>>();

		let pos = (parse_input!(input[1], Axis), parse_input!(input[2], Axis));

		Self {
			kind: parse_input!(input[0], ResourceKind),
			pos,
			is_next_to_water: env.grid.is_water_next_to(pos),
			size: parse_input!(input[3], TreeSize),
			health: parse_input!(input[4], TreeHealth),
			fruit: parse_input!(input[5], TreeFruit),
			cooldown: parse_input!(input[6], TreeCooldown),
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

	fn is_empty(&self) -> bool {
		self.plum == 0
			&& self.lemon == 0
			&& self.apple == 0
			&& self.banana == 0
			&& self.iron == 0
			&& self.wood == 0
	}

	fn able_to_plant(&self) -> Option<ResourceKind> {
		if self.banana > 0 {
			Some(ResourceKind::Banana)
		} else if self.plum > 0 {
			Some(ResourceKind::Plum)
		} else if self.lemon > 0 {
			Some(ResourceKind::Lemon)
		} else if self.apple > 0 {
			Some(ResourceKind::Apple)
		} else {
			None
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
			my_troll: parse_input!(input[1], u8) == 0,
			pos: (parse_input!(input[2], Axis), parse_input!(input[3], Axis)),
			move_speed: parse_input!(input[4], MoveSpeed),
			carry_capacity: parse_input!(input[5], CarryCapacity),
			harvest_power: parse_input!(input[6], HarvestPower),
			chop_power: parse_input!(input[7], ChopPower),
			carry: PlayerInventory::parse(&input[8..]),
		}
	}
}

impl TurnState {
	fn read(env: &Env) -> Self {
		let mut s = String::new();

		let my_inventory = PlayerInventory::read();
		let op_inventory = PlayerInventory::read();

		io::stdin().read_line(&mut s).unwrap();
		let tree_count = parse_input!(s, usize);

		let mut tree_list = Vec::with_capacity(tree_count);
		for _ in 0..tree_count {
			tree_list.push(Tree::read(env));
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

fn chop_tree(troll: &Troll, tree: &Tree) -> Action {
	if troll.pos.0 == tree.pos.0 && troll.pos.1 == tree.pos.1 {
		Action::Chop(troll.id)
	} else {
		Action::Move(troll.id, tree.pos)
	}
}

fn is_valid_plant_spot(env: &Env, state: &TurnState, pos: Coord) -> bool {
	env.grid.is_grass(pos) && state.tree_list.iter().find(|t| t.pos == pos).is_none()
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

fn best_stat_train(troll_count: usize, resource_available: Resource) -> Resource {
	let base_cost = troll_count as Resource;

	let mut best: Resource = 0;
	for i in 1..=resource_available {
		let cost = base_cost + (i * i);
		if cost > resource_available {
			break;
		}
		best = i;
	}

	best
}

fn best_close_to_shack_tree<'a>(
	env: &'a Env,
	state: &'a TurnState,
	troll: &'a Troll,
) -> Option<&'a Tree> {
	let mut best_tree = None;
	let mut best_tree_dist_to_troll = u8::MAX;

	for tree in &state.tree_list {
		let dist_to_shack = dist(tree.pos, env.my_shack);
		if dist_to_shack > 3 {
			continue;
		}

		let dist_to_troll = dist(troll.pos, tree.pos);
		if dist_to_troll < best_tree_dist_to_troll {
			best_tree = Some(tree);
			best_tree_dist_to_troll = dist_to_troll;
		}
	}

	best_tree
}

fn find_best_close_to_shack_plant_spot(env: &Env, state: &TurnState) -> Option<Coord> {
	let (cx, cy) = env.my_shack;

	let mut valid_spot_list = Vec::with_capacity(4);

	if cx > 0 {
		let n = (cx - 1, cy);
		if is_valid_plant_spot(env, state, n) {
			valid_spot_list.push(n);
		}
	}
	if cx < env.grid.w - 1 {
		let n = (cx + 1, cy);
		if is_valid_plant_spot(env, state, n) {
			valid_spot_list.push(n);
		}
	}
	if cy > 0 {
		let n = (cx, cy - 1);
		if is_valid_plant_spot(env, state, n) {
			valid_spot_list.push(n);
		}
	}
	if cy < env.grid.h - 1 {
		let n = (cx, cy + 1);
		if is_valid_plant_spot(env, state, n) {
			valid_spot_list.push(n);
		}
	}

	if valid_spot_list.is_empty() {
		return None;
	}

	let mut best_spot = valid_spot_list.pop().expect("valid_spot_list is not empty");
	let mut best_dist_to_op_shack = dist(best_spot, env.op_shack);
	for spot in valid_spot_list {
		let dist_to_op_shack = dist(spot, env.op_shack);
		if dist_to_op_shack > best_dist_to_op_shack {
			best_spot = spot;
			best_dist_to_op_shack = dist_to_op_shack;
		}
	}

	Some(best_spot)
}

fn plant_best_tree_close_to_shack(env: &Env, state: &TurnState, troll: &Troll) -> Action {
	let Some(spot) = find_best_close_to_shack_plant_spot(env, state) else {
		dbg!("no valid spot to plant close to shack");
		return Action::Move(troll.id, env.op_shack);
	};

	if troll.pos.0 != spot.0 || troll.pos.1 != spot.1 {
		dbg!(troll.pos, spot);
		return Action::Move(troll.id, spot);
	}

	if let Some(resource_kind) = troll.carry.able_to_plant() {
		Action::Plant(troll.id, resource_kind)
	} else if let Some(resource_kind) = state.my_inventory.able_to_plant() {
		return Action::Pick(troll.id, resource_kind);
	} else {
		dbg!("no resource to plant");
		Action::Move(troll.id, env.op_shack)
	}
}

fn find_best_tree_to_chop<'a>(
	env: &'a Env,
	state: &'a TurnState,
	troll: &'a Troll,
) -> Option<&'a Tree> {
	let mut best_tree = None;
	let mut best_tree_score = u8::MAX;

	for tree in &state.tree_list {
		let dist_to_troll = dist(troll.pos, tree.pos);
		let dist_to_shack = dist(tree.pos, env.my_shack);
		let dist_to_op_shack = dist(tree.pos, env.op_shack);

		let score = dist_to_troll + dist_to_shack + dist_to_op_shack;
		if score < best_tree_score {
			best_tree = Some(tree);
			best_tree_score = score;
		}
	}

	best_tree
}

fn solve_troll_action(env: &Env, state: &TurnState, troll: &Troll) -> Action {
	if !troll.carry.is_empty() {
		if troll.carry.able_to_plant().is_some() {
			return plant_best_tree_close_to_shack(env, state, troll);
		} else {
			return drop_to_shack(troll, env);
		}
	}

	if state
		.tree_list
		.iter()
		.find(|t| t.pos == troll.pos)
		.is_some()
	{
		return Action::Chop(troll.id);
	}

	if troll.move_speed == 1
		&& troll.carry_capacity == 1
		&& troll.harvest_power == 1
		&& troll.chop_power == 1
	{
		if let Some(tree) = best_close_to_shack_tree(env, state, troll) {
			return chop_tree(troll, tree);
		} else if state.my_inventory.able_to_plant().is_some() {
			return plant_best_tree_close_to_shack(env, state, troll);
		}
	}

	if let Some(tree) = find_best_tree_to_chop(env, state, troll) {
		return chop_tree(troll, tree);
	}

	Action::Move(troll.id, env.op_shack)
}

fn solve(env: &Env, state: &TurnState) -> Vec<Action> {
	// TODO: check if not better to allocate on stack with [Action; MAX_TROLL_COUNT]
	let mut action_list = Vec::with_capacity(state.my_troll_list.len());

	if state.my_troll_list.len() <= 1 {
		let troll_count = state.my_troll_list.len();
		action_list.push(Action::Train(
			best_stat_train(troll_count, state.my_inventory.plum) as MoveSpeed,
			best_stat_train(troll_count, state.my_inventory.lemon) as CarryCapacity,
			// best_stat_train(troll_count, state.my_inventory.apple) as HarvestPower,
			0,
			best_stat_train(troll_count, state.my_inventory.iron) as ChopPower,
		));
	}

	for troll in &state.my_troll_list {
		action_list.push(solve_troll_action(env, state, troll));
	}

	action_list
}

fn main() {
	let env = Env::read();

	loop {
		let state = TurnState::read(&env);

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
