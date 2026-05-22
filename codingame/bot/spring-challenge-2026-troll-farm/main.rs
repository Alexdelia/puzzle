use std::{fmt::Display, io, str::FromStr};

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

type Axis = u8;
const MAX_H: Axis = 11;

type Coord = (Axis, Axis);
type TrollId = u8;
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

type Turn = u16;

#[derive(Clone, Copy, PartialEq, Eq)]
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
	my_troll: bool,
	pos: Coord,
	move_speed: MoveSpeed,
	carry_capacity: CarryCapacity,
	harvest_power: HarvestPower,
	chop_power: ChopPower,
	carry: PlayerInventory,
}

#[derive(Default)]
struct Grid {
	g: [u64; MAX_H as usize],
	w: Axis,
	h: Axis,
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
	#[allow(dead_code)]
	op_inventory: PlayerInventory,
	my_troll_list: Vec<Troll>,
	#[allow(dead_code)]
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TrollRole {
	/// Initial troll with stats (1,1,1,1)
	Initial,
	/// Harvester with stats (2,2,2,0)
	Harvester,
	/// Carrier with stats (3,4,1,2)
	Carrier,
	/// Woodcutter with stats (2,4,0,3)
	Woodcutter,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Goal {
	/// train harvester (2,2,2,0)
	/// plant to have 2 lemon, 4 plum
	TrainHarvester,
	/// train carrier (3,4,1,2)
	/// plan to have 4 lemon, 4 plum
	TrainCarrier,
	/// train woodcutter (2,4,0,3)
	TrainWoodcutter,
	/// ghather banana and chop tree
	GatherPoint,
	/// turn >= 280: final push
	Endgame,
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
		(self.g[y as usize] >> (x as usize * 2)) & 0b11 == Self::GRASS
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

	fn get(&self, kind: ResourceKind) -> Resource {
		match kind {
			ResourceKind::Plum => self.plum,
			ResourceKind::Lemon => self.lemon,
			ResourceKind::Apple => self.apple,
			ResourceKind::Banana => self.banana,
			ResourceKind::Iron => self.iron,
			ResourceKind::Wood => self.wood,
		}
	}

	fn total(&self) -> Resource {
		self.plum + self.lemon + self.apple + self.banana + self.iron + self.wood
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

	fn able_to_plant_for_training(&self) -> Option<ResourceKind> {
		if self.plum > 0 {
			Some(ResourceKind::Plum)
		} else if self.lemon > 0 {
			Some(ResourceKind::Lemon)
		} else if self.apple > 0 {
			Some(ResourceKind::Apple)
		} else {
			None
		}
	}

	fn surplus_above(&self, cost: &PlayerInventory) -> PlayerInventory {
		PlayerInventory {
			plum: self.plum.saturating_sub(cost.plum),
			lemon: self.lemon.saturating_sub(cost.lemon),
			apple: self.apple.saturating_sub(cost.apple),
			banana: self.banana,
			iron: 0,
			wood: 0,
		}
	}

	fn free_capacity(&self, carry_capacity: CarryCapacity) -> Resource {
		carry_capacity.saturating_sub(self.total())
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

	fn role(&self) -> TrollRole {
		if self.move_speed >= 3 {
			TrollRole::Carrier
		} else if self.harvest_power >= 2 {
			TrollRole::Harvester
		} else if self.chop_power >= 3 {
			TrollRole::Woodcutter
		} else {
			TrollRole::Initial
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

fn dist(a: Coord, b: Coord) -> u8 {
	(a.0 as i8 - b.0 as i8).abs() as u8 + (a.1 as i8 - b.1 as i8).abs() as u8
}

fn is_adjacent(a: Coord, b: Coord) -> bool {
	dist(a, b) == 1
}

fn is_next_to_shack(troll: &Troll, env: &Env) -> bool {
	dist(troll.pos, env.my_shack) <= 1
}

fn move_or_do(troll: &Troll, target: Coord, action: Action) -> Action {
	if troll.pos == target {
		action
	} else {
		Action::Move(troll.id, target)
	}
}

fn drop_to_shack(troll: &Troll, env: &Env) -> Action {
	if is_next_to_shack(troll, env) {
		Action::Drop(troll.id)
	} else {
		Action::Move(troll.id, env.my_shack)
	}
}

fn is_valid_plant_spot(env: &Env, state: &TurnState, pos: Coord) -> bool {
	env.grid.is_grass(pos) && state.tree_list.iter().all(|t| t.pos != pos)
}

/// Find grass spots within `max_dist` of shack, sorted by distance to shack (closest first),
/// tie-broken by distance to op_shack (farthest first).
fn get_plant_spot_list_near_shack(env: &Env, state: &TurnState, max_dist: u8) -> Vec<Coord> {
	let mut spot_list = Vec::new();
	let (sx, sy) = env.my_shack;

	for dy in -(max_dist as i8)..=(max_dist as i8) {
		let y = sy as i8 + dy;
		if y < 0 || y >= env.grid.h as i8 {
			continue;
		}
		let remaining = max_dist as i8 - dy.abs();
		for dx in -remaining..=remaining {
			let x = sx as i8 + dx;
			if x < 0 || x >= env.grid.w as i8 {
				continue;
			}
			let pos = (x as Axis, y as Axis);
			if pos == env.my_shack {
				continue;
			}
			if is_valid_plant_spot(env, state, pos) {
				spot_list.push(pos);
			}
		}
	}

	spot_list.sort_by(|a, b| {
		let da = dist(*a, env.my_shack);
		let db = dist(*b, env.my_shack);
		da.cmp(&db)
			.then_with(|| {
				let oa = dist(*a, env.op_shack);
				let ob = dist(*b, env.op_shack);
				ob.cmp(&oa)
			})
			.then_with(|| {
				let wa = env.grid.is_water_next_to(*a);
				let wb = env.grid.is_water_next_to(*b);
				wb.cmp(&wa)
			})
	});

	spot_list
}

fn count_tree_near_shack(env: &Env, state: &TurnState, kind: ResourceKind, max_dist: u8) -> u8 {
	state
		.tree_list
		.iter()
		.filter(|t| t.kind == kind && dist(t.pos, env.my_shack) <= max_dist)
		.count() as u8
}

fn find_closest_tree_with_fruit<'a>(state: &'a TurnState, troll: &Troll) -> Option<&'a Tree> {
	state
		.tree_list
		.iter()
		.filter(|t| t.fruit > 0)
		.min_by_key(|t| dist(troll.pos, t.pos))
}

fn find_closest_tree_of_kind<'a>(
	state: &'a TurnState,
	troll: &Troll,
	kind: ResourceKind,
) -> Option<&'a Tree> {
	state
		.tree_list
		.iter()
		.filter(|t| t.kind == kind && t.fruit > 0)
		.min_by_key(|t| dist(troll.pos, t.pos))
}

fn chop_cost_per_wood(tree: &Tree, troll: &Troll, env: &Env) -> u32 {
	let wood = (tree.size as u16).min(troll.carry.free_capacity(troll.carry_capacity));
	if wood == 0 || troll.chop_power == 0 {
		return u32::MAX;
	}
	let chop_turns = (tree.health as u16 + troll.chop_power - 1) / troll.chop_power;
	let travel = dist(troll.pos, tree.pos) as u16 + dist(tree.pos, env.my_shack) as u16;
	(chop_turns + travel + 1) as u32 * 10 / wood as u32
}

fn find_best_tree_to_chop<'a>(
	env: &'a Env,
	state: &'a TurnState,
	troll: &'a Troll,
) -> Option<&'a Tree> {
	if troll.chop_power == 0 {
		return None;
	}
	state
		.tree_list
		.iter()
		.filter(|t| t.size > 0)
		.min_by_key(|t| chop_cost_per_wood(t, troll, env))
}

fn find_best_tree_to_chop_near_shack<'a>(
	env: &'a Env,
	state: &'a TurnState,
	troll: &'a Troll,
) -> Option<&'a Tree> {
	if troll.chop_power == 0 {
		return None;
	}
	state
		.tree_list
		.iter()
		.filter(|t| t.size > 0)
		.min_by_key(|t| {
			let d_shack = dist(t.pos, env.my_shack) as u32;
			let cost = chop_cost_per_wood(t, troll, env);
			(d_shack, cost)
		})
}

fn is_adjacent_to_iron(env: &Env, pos: Coord) -> bool {
	env.iron_list.iter().any(|&iron| dist(pos, iron) == 1)
}

fn find_closest_grass_near_iron(env: &Env, troll: &Troll) -> Option<Coord> {
	let mut best: Option<Coord> = None;
	let mut best_dist = u8::MAX;

	for &iron_pos in &env.iron_list {
		for (dx, dy) in [(0i8, 1i8), (0, -1), (1, 0), (-1, 0)] {
			let x = iron_pos.0 as i8 + dx;
			let y = iron_pos.1 as i8 + dy;
			if x < 0 || x >= env.grid.w as i8 || y < 0 || y >= env.grid.h as i8 {
				continue;
			}
			let pos = (x as Axis, y as Axis);
			if !env.grid.is_grass(pos) {
				continue;
			}
			let d = dist(troll.pos, pos);
			if d < best_dist {
				best = Some(pos);
				best_dist = d;
			}
		}
	}

	best
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

fn determine_goal(state: &TurnState, turn: Turn) -> Goal {
	if turn >= 280 {
		return Goal::Endgame;
	}

	let has_harvester = state
		.my_troll_list
		.iter()
		.any(|t| t.role() == TrollRole::Harvester);
	if !has_harvester {
		return Goal::TrainHarvester;
	}

	let has_carrier = state
		.my_troll_list
		.iter()
		.any(|t| t.role() == TrollRole::Carrier);
	if !has_carrier {
		return Goal::TrainCarrier;
	}

	let has_woodcutter = state
		.my_troll_list
		.iter()
		.any(|t| t.role() == TrollRole::Woodcutter);
	if !has_woodcutter {
		return Goal::TrainWoodcutter;
	}

	Goal::GatherPoint
}

struct TrainTarget {
	move_speed: MoveSpeed,
	carry_capacity: Resource,
	harvest_power: Resource,
	chop_power: Resource,
	plant_plum: u8,
	plant_lemon: u8,
}

impl TrainTarget {
	fn cost(&self, troll_count: usize) -> PlayerInventory {
		let base = troll_count as Resource;
		PlayerInventory {
			plum: base + (self.move_speed as Resource) * (self.move_speed as Resource),
			lemon: base + self.carry_capacity * self.carry_capacity,
			apple: base + self.harvest_power * self.harvest_power,
			banana: 0,
			iron: base + self.chop_power * self.chop_power,
			wood: 0,
		}
	}

	fn can_afford(&self, inventory: &PlayerInventory, troll_count: usize) -> bool {
		let cost = self.cost(troll_count);
		inventory.plum >= cost.plum
			&& inventory.lemon >= cost.lemon
			&& inventory.apple >= cost.apple
			&& inventory.iron >= cost.iron
	}

	fn train_action(&self) -> Action {
		Action::Train(
			self.move_speed,
			self.carry_capacity,
			self.harvest_power,
			self.chop_power,
		)
	}
}

impl TrollRole {
	const fn target(self) -> TrainTarget {
		match self {
			TrollRole::Harvester => TrainTarget {
				move_speed: 2,
				carry_capacity: 2,
				harvest_power: 2,
				chop_power: 0,
				plant_plum: 2,
				plant_lemon: 2,
			},
			TrollRole::Carrier => TrainTarget {
				move_speed: 3,
				carry_capacity: 4,
				harvest_power: 1,
				chop_power: 2,
				plant_plum: 4,
				plant_lemon: 4,
			},
			TrollRole::Woodcutter => TrainTarget {
				move_speed: 2,
				carry_capacity: 4,
				harvest_power: 0,
				chop_power: 3,
				plant_plum: 4,
				plant_lemon: 4,
			},
			TrollRole::Initial => TrainTarget {
				move_speed: 0,
				carry_capacity: 0,
				harvest_power: 0,
				chop_power: 0,
				plant_plum: 0,
				plant_lemon: 0,
			},
		}
	}
}

fn solve_goal_train(env: &Env, state: &TurnState, role: TrollRole) -> Vec<Action> {
	let mut action_list = Vec::new();
	let target = role.target();
	let troll_count = state.my_troll_list.len();

	if target.can_afford(&state.my_inventory, troll_count) {
		action_list.push(target.train_action());
		for troll in &state.my_troll_list {
			action_list.push(solve_troll_gather(env, state, troll));
		}
		return action_list;
	}

	let cost = target.cost(troll_count);
	let need_apple = state.my_inventory.apple < cost.apple;
	let need_iron = state.my_inventory.iron < cost.iron;

	let plum_near = count_tree_near_shack(env, state, ResourceKind::Plum, 3);
	let lemon_near = count_tree_near_shack(env, state, ResourceKind::Lemon, 3);
	let need_plant_plum = plum_near < target.plant_plum;
	let need_plant_lemon = lemon_near < target.plant_lemon;
	let need_planting = need_plant_plum || need_plant_lemon;

	let has_harvester = state
		.my_troll_list
		.iter()
		.any(|t| t.role() == TrollRole::Harvester);
	let has_miner = state
		.my_troll_list
		.iter()
		.any(|t| matches!(t.role(), TrollRole::Carrier | TrollRole::Woodcutter));

	let mut planter_assigned = false;
	let mut iron_assigned = false;
	let mut apple_assigned = false;

	for troll in &state.my_troll_list {
		let action = if need_planting && !planter_assigned {
			planter_assigned = true;
			solve_troll_plant_for_goal(env, state, troll, need_plant_plum, need_plant_lemon, &cost)
		} else {
			match troll.role() {
				TrollRole::Harvester if need_apple && !apple_assigned => {
					apple_assigned = true;
					solve_troll_harvest_resource(env, state, troll, ResourceKind::Apple, &cost)
				}
				TrollRole::Carrier | TrollRole::Woodcutter if need_iron && !iron_assigned => {
					iron_assigned = true;
					solve_troll_mine_iron(env, state, troll, &cost)
				}
				_ if need_iron && !iron_assigned && !has_miner && troll.chop_power > 0 => {
					iron_assigned = true;
					solve_troll_mine_iron(env, state, troll, &cost)
				}
				_ if need_apple && !apple_assigned && !has_harvester => {
					apple_assigned = true;
					solve_troll_harvest_resource(env, state, troll, ResourceKind::Apple, &cost)
				}
				_ => solve_troll_accumulate(env, state, troll, &cost),
			}
		};
		action_list.push(action);
	}

	action_list
}

fn solve_troll_plant_for_goal(
	env: &Env,
	state: &TurnState,
	troll: &Troll,
	need_plum: bool,
	need_lemon: bool,
	cost: &PlayerInventory,
) -> Action {
	if !troll.carry.is_empty() {
		let plant_kind = if need_plum && troll.carry.plum > 0 {
			Some(ResourceKind::Plum)
		} else if need_lemon && troll.carry.lemon > 0 {
			Some(ResourceKind::Lemon)
		} else {
			None
		};

		if let Some(kind) = plant_kind {
			let spot_list = get_plant_spot_list_near_shack(env, state, 3);
			if let Some(&spot) = spot_list.first() {
				return move_or_do(troll, spot, Action::Plant(troll.id, kind));
			}
		}

		return drop_to_shack(troll, env);
	}

	let pick_kind = if need_plum && state.my_inventory.plum > 0 {
		Some(ResourceKind::Plum)
	} else if need_lemon && state.my_inventory.lemon > 0 {
		Some(ResourceKind::Lemon)
	} else {
		None
	};

	if let Some(kind) = pick_kind {
		if is_next_to_shack(troll, env) {
			return Action::Pick(troll.id, kind);
		}
		return Action::Move(troll.id, env.my_shack);
	}

	if need_plum {
		if let Some(tree) = find_closest_tree_of_kind(state, troll, ResourceKind::Plum) {
			return move_or_do(troll, tree.pos, Action::Harvest(troll.id));
		}
	}
	if need_lemon {
		if let Some(tree) = find_closest_tree_of_kind(state, troll, ResourceKind::Lemon) {
			return move_or_do(troll, tree.pos, Action::Harvest(troll.id));
		}
	}

	solve_troll_accumulate(env, state, troll, cost)
}

fn is_needed_resource(kind: ResourceKind, inventory: &PlayerInventory, cost: &PlayerInventory) -> bool {
	match kind {
		ResourceKind::Plum => inventory.plum < cost.plum,
		ResourceKind::Lemon => inventory.lemon < cost.lemon,
		ResourceKind::Apple => inventory.apple < cost.apple,
		_ => false,
	}
}

fn solve_troll_accumulate(
	env: &Env,
	state: &TurnState,
	troll: &Troll,
	cost: &PlayerInventory,
) -> Action {
	if !troll.carry.is_empty() {
		return drop_to_shack(troll, env);
	}

	if state
		.tree_list
		.iter()
		.any(|t| t.pos == troll.pos && t.fruit > 0 && is_needed_resource(t.kind, &state.my_inventory, cost))
	{
		return Action::Harvest(troll.id);
	}

	if let Some(tree) = state
		.tree_list
		.iter()
		.filter(|t| t.fruit > 0 && is_needed_resource(t.kind, &state.my_inventory, cost))
		.min_by_key(|t| dist(troll.pos, t.pos))
	{
		return Action::Move(troll.id, tree.pos);
	}

	Action::Move(troll.id, env.op_shack)
}

fn solve_troll_harvest_resource(
	env: &Env,
	state: &TurnState,
	troll: &Troll,
	kind: ResourceKind,
	cost: &PlayerInventory,
) -> Action {
	if !troll.carry.is_empty() {
		return drop_to_shack(troll, env);
	}

	if let Some(tree) = state.tree_list.iter().find(|t| t.pos == troll.pos) {
		if tree.kind == kind && tree.fruit > 0 {
			return Action::Harvest(troll.id);
		}
	}

	if let Some(tree) = find_closest_tree_of_kind(state, troll, kind) {
		return Action::Move(troll.id, tree.pos);
	}

	solve_troll_accumulate(env, state, troll, cost)
}

fn solve_troll_mine_iron(env: &Env, state: &TurnState, troll: &Troll, cost: &PlayerInventory) -> Action {
	if !troll.carry.is_empty() {
		return drop_to_shack(troll, env);
	}

	if is_adjacent_to_iron(env, troll.pos) {
		return Action::Mine(troll.id);
	}

	if let Some(grass_pos) = find_closest_grass_near_iron(env, troll) {
		return Action::Move(troll.id, grass_pos);
	}

	solve_troll_accumulate(env, state, troll, cost)
}

fn solve_goal_gather_point(env: &Env, state: &TurnState) -> Vec<Action> {
	let mut action_list = Vec::new();

	for troll in &state.my_troll_list {
		let role = troll.role();
		match role {
			TrollRole::Initial | TrollRole::Harvester => {
				action_list.push(solve_troll_banana_planter(env, state, troll));
			}
			_ => {
				action_list.push(solve_troll_chopper(env, state, troll));
			}
		}
	}

	action_list
}

fn solve_troll_banana_planter(env: &Env, state: &TurnState, troll: &Troll) -> Action {
	if !troll.carry.is_empty() {
		if troll.carry.able_to_plant().is_some() {
			let spot_list = get_plant_spot_list_near_shack(env, state, 5);
			if let Some(&spot) = spot_list.first() {
				return move_or_do(
					troll,
					spot,
					Action::Plant(troll.id, troll.carry.able_to_plant().unwrap()),
				);
			}
		}
		return drop_to_shack(troll, env);
	}

	if let Some(_tree) = state
		.tree_list
		.iter()
		.find(|t| t.pos == troll.pos && t.fruit > 0)
	{
		return Action::Harvest(troll.id);
	}

	if let Some(tree) = find_closest_tree_of_kind(state, troll, ResourceKind::Banana) {
		return Action::Move(troll.id, tree.pos);
	}

	if state.my_inventory.banana > 0 && is_next_to_shack(troll, env) {
		return Action::Pick(troll.id, ResourceKind::Banana);
	}
	if state.my_inventory.banana > 0 {
		return Action::Move(troll.id, env.my_shack);
	}

	if let Some(kind) = state.my_inventory.able_to_plant() {
		if is_next_to_shack(troll, env) {
			return Action::Pick(troll.id, kind);
		}
		return Action::Move(troll.id, env.my_shack);
	}

	if let Some(tree) = find_closest_tree_with_fruit(state, troll) {
		return Action::Move(troll.id, tree.pos);
	}

	solve_troll_chopper(env, state, troll)
}

fn solve_troll_chopper(env: &Env, state: &TurnState, troll: &Troll) -> Action {
	if !troll.carry.is_empty() {
		return drop_to_shack(troll, env);
	}

	if troll.chop_power > 0 {
		if state.tree_list.iter().any(|t| t.pos == troll.pos) {
			return Action::Chop(troll.id);
		}

		if let Some(tree) = find_best_tree_to_chop(env, state, troll) {
			return Action::Move(troll.id, tree.pos);
		}
	}

	Action::Move(troll.id, env.op_shack)
}

fn solve_goal_endgame(env: &Env, state: &TurnState) -> Vec<Action> {
	let mut action_list = Vec::new();

	for troll in &state.my_troll_list {
		let role = troll.role();
		match role {
			TrollRole::Harvester => {
				action_list.push(solve_troll_banana_planter(env, state, troll));
			}
			_ => {
				action_list.push(solve_troll_chopper_near_shack(env, state, troll));
			}
		}
	}

	action_list
}

fn solve_troll_chopper_near_shack(env: &Env, state: &TurnState, troll: &Troll) -> Action {
	if !troll.carry.is_empty() {
		return drop_to_shack(troll, env);
	}

	if troll.chop_power > 0 {
		if state.tree_list.iter().any(|t| t.pos == troll.pos) {
			return Action::Chop(troll.id);
		}

		if let Some(tree) = find_best_tree_to_chop_near_shack(env, state, troll) {
			return Action::Move(troll.id, tree.pos);
		}
	}

	Action::Move(troll.id, env.op_shack)
}

fn solve_troll_gather(env: &Env, state: &TurnState, troll: &Troll) -> Action {
	if !troll.carry.is_empty() {
		return drop_to_shack(troll, env);
	}

	if let Some(tree) = find_closest_tree_with_fruit(state, troll) {
		return move_or_do(troll, tree.pos, Action::Harvest(troll.id));
	}

	if troll.chop_power > 0 {
		if let Some(tree) = find_best_tree_to_chop(env, state, troll) {
			return move_or_do(troll, tree.pos, Action::Chop(troll.id));
		}
	}

	Action::Move(troll.id, env.op_shack)
}

fn solve(env: &Env, state: &TurnState, turn: Turn) -> Vec<Action> {
	let goal = determine_goal(state, turn);
	dbg!(&goal);

	match goal {
		Goal::TrainHarvester => solve_goal_train(env, state, TrollRole::Harvester),
		Goal::TrainCarrier => solve_goal_train(env, state, TrollRole::Carrier),
		Goal::TrainWoodcutter => solve_goal_train(env, state, TrollRole::Woodcutter),
		Goal::GatherPoint => solve_goal_gather_point(env, state),
		Goal::Endgame => solve_goal_endgame(env, state),
	}
}

fn main() {
	let env = Env::read();

	let mut turn: Turn = 0;

	loop {
		let state = TurnState::read(&env);
		turn += 1;

		let action_list = solve(&env, &state, turn);
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
