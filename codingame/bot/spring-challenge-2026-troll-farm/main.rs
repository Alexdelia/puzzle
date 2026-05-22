use std::{collections::VecDeque, fmt::Display, io, str::FromStr};

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

type Axis = u8;
const MAX_H: Axis = 11;
const MAX_W: Axis = MAX_H * 2;

type Turn = u16;
const MAX_TURN: Turn = 300;
const FIRST_TURN_MS_LIMIT: u64 = 1000;
const TURN_MS_LIMIT: u64 = 50;

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
	iron_list: Vec<Coord>,
	bfs_dist: Vec<u8>,
	bfs_n: usize,
	my_shack_dist: Vec<u8>,
	op_shack_dist: Vec<u8>,
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
	Initial,
	Harvester,
	Carrier,
	Woodcutter,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Goal {
	TrainHarvester,
	TrainCarrier,
	TrainWoodcutter,
	GatherPoint,
	Endgame,
}

impl Grid {
	const GRASS: u64 = 0b00;
	const WATER: u64 = 0b01;
	const IRON: u64 = 0b10;
	const ROCK: u64 = 0b11;

	fn read() -> (Self, Coord, Coord, Vec<Coord>) {
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
		let mut iron_list = Vec::new();

		for y in 0..grid.h as usize {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();

			for (x, c) in s.trim_matches('\n').chars().enumerate() {
				let pos = (x as Axis, y as Axis);
				match c {
					'.' => grid.set_cell(pos, Self::GRASS),
					'~' => grid.set_cell(pos, Self::WATER),
					'#' => grid.set_cell(pos, Self::ROCK),
					'+' => {
						grid.set_cell(pos, Self::IRON);
						iron_list.push(pos);
					}
					'0' => my_shack = pos,
					'1' => op_shack = pos,
					_ => panic!("invalid grid character '{c}' at ({x}, {y})"),
				}
			}
		}

		iron_list.shrink_to_fit();
		(grid, my_shack, op_shack, iron_list)
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

fn bfs_from(grid: &Grid, start: Coord, out: &mut [u8]) {
	let w = grid.w as usize;
	let si = start.1 as usize * w + start.0 as usize;
	out[si] = 0;
	let mut queue = VecDeque::new();
	queue.push_back(start);

	while let Some((x, y)) = queue.pop_front() {
		let d = out[y as usize * w + x as usize];
		for (dx, dy) in [(0i8, 1i8), (0, -1), (1, 0), (-1, 0)] {
			let nx = x as i8 + dx;
			let ny = y as i8 + dy;
			if nx < 0 || nx >= grid.w as i8 || ny < 0 || ny >= grid.h as i8 {
				continue;
			}
			let npos = (nx as Axis, ny as Axis);
			if !grid.is_grass(npos) {
				continue;
			}
			let ni = ny as usize * w + nx as usize;
			if out[ni] != u8::MAX {
				continue;
			}
			out[ni] = d + 1;
			queue.push_back(npos);
		}
	}
}

fn compute_all_bfs(grid: &Grid) -> (Vec<u8>, usize) {
	let w = grid.w as usize;
	let h = grid.h as usize;
	let n = w * h;
	let mut dist_map = vec![u8::MAX; n * n];

	for y in 0..h {
		for x in 0..w {
			let pos = (x as Axis, y as Axis);
			if !grid.is_grass(pos) {
				continue;
			}
			let offset = (y * w + x) * n;
			bfs_from(grid, pos, &mut dist_map[offset..offset + n]);
		}
	}

	(dist_map, n)
}

fn compute_shack_dist(grid: &Grid, shack: Coord) -> Vec<u8> {
	let w = grid.w as usize;
	let h = grid.h as usize;
	let n = w * h;
	let mut dist_arr = vec![u8::MAX; n];
	let mut queue = VecDeque::new();

	for (dx, dy) in [(0i8, 1i8), (0, -1), (1, 0), (-1, 0)] {
		let x = shack.0 as i8 + dx;
		let y = shack.1 as i8 + dy;
		if x < 0 || x >= grid.w as i8 || y < 0 || y >= grid.h as i8 {
			continue;
		}
		let pos = (x as Axis, y as Axis);
		if !grid.is_grass(pos) {
			continue;
		}
		let i = y as usize * w + x as usize;
		if dist_arr[i] == u8::MAX {
			dist_arr[i] = 0;
			queue.push_back(pos);
		}
	}

	while let Some((x, y)) = queue.pop_front() {
		let d = dist_arr[y as usize * w + x as usize];
		for (dx, dy) in [(0i8, 1i8), (0, -1), (1, 0), (-1, 0)] {
			let nx = x as i8 + dx;
			let ny = y as i8 + dy;
			if nx < 0 || nx >= grid.w as i8 || ny < 0 || ny >= grid.h as i8 {
				continue;
			}
			let npos = (nx as Axis, ny as Axis);
			if !grid.is_grass(npos) {
				continue;
			}
			let ni = ny as usize * w + nx as usize;
			if dist_arr[ni] != u8::MAX {
				continue;
			}
			dist_arr[ni] = d + 1;
			queue.push_back(npos);
		}
	}

	dist_arr
}

impl Env {
	fn read() -> Self {
		let (grid, my_shack, op_shack, iron_list) = Grid::read();
		let t0 = std::time::Instant::now();
		let (bfs_dist, bfs_n) = compute_all_bfs(&grid);
		let my_shack_dist = compute_shack_dist(&grid, my_shack);
		let op_shack_dist = compute_shack_dist(&grid, op_shack);
		dbg!(t0.elapsed());

		Self {
			grid,
			my_shack,
			op_shack,
			iron_list,
			bfs_dist,
			bfs_n,
			my_shack_dist,
			op_shack_dist,
		}
	}

	fn dist(&self, a: Coord, b: Coord) -> u8 {
		let w = self.grid.w as usize;
		let ai = a.1 as usize * w + a.0 as usize;
		let bi = b.1 as usize * w + b.0 as usize;
		self.bfs_dist[ai * self.bfs_n + bi]
	}

	fn dist_to_my_shack(&self, pos: Coord) -> u8 {
		let w = self.grid.w as usize;
		self.my_shack_dist[pos.1 as usize * w + pos.0 as usize]
	}

	fn dist_to_op_shack(&self, pos: Coord) -> u8 {
		let w = self.grid.w as usize;
		self.op_shack_dist[pos.1 as usize * w + pos.0 as usize]
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

fn manhattan_dist(a: Coord, b: Coord) -> u8 {
	(a.0 as i8 - b.0 as i8).abs() as u8 + (a.1 as i8 - b.1 as i8).abs() as u8
}

fn is_next_to_shack(troll: &Troll, env: &Env) -> bool {
	manhattan_dist(troll.pos, env.my_shack) <= 1
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

fn get_plant_spot_list_near_shack(
	env: &Env,
	state: &TurnState,
	troll: &Troll,
	max_dist: u8,
) -> Vec<Coord> {
	let mut spot_list = Vec::new();

	for y in 0..env.grid.h {
		for x in 0..env.grid.w {
			let pos = (x, y);
			if pos == env.my_shack {
				continue;
			}
			let d = env.dist_to_my_shack(pos);
			if d <= max_dist && is_valid_plant_spot(env, state, pos) {
				spot_list.push(pos);
			}
		}
	}

	spot_list.sort_by(|a, b| {
		env.dist_to_my_shack(*a)
			.cmp(&env.dist_to_my_shack(*b))
			.then_with(|| env.dist(troll.pos, *a).cmp(&env.dist(troll.pos, *b)))
			.then_with(|| env.dist_to_op_shack(*b).cmp(&env.dist_to_op_shack(*a)))
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
		.filter(|t| t.kind == kind && env.dist_to_my_shack(t.pos) <= max_dist)
		.count() as u8
}

fn find_closest_tree_with_fruit<'a>(
	env: &Env,
	state: &'a TurnState,
	troll: &Troll,
) -> Option<&'a Tree> {
	state
		.tree_list
		.iter()
		.filter(|t| t.fruit > 0)
		.min_by_key(|t| env.dist(troll.pos, t.pos))
}

fn find_closest_tree_of_kind<'a>(
	env: &Env,
	state: &'a TurnState,
	troll: &Troll,
	kind: ResourceKind,
) -> Option<&'a Tree> {
	state
		.tree_list
		.iter()
		.filter(|t| t.kind == kind && t.fruit > 0)
		.min_by_key(|t| env.dist(troll.pos, t.pos))
}

fn chop_cost_per_wood(tree: &Tree, troll: &Troll, env: &Env) -> u32 {
	let wood = (tree.size as u16).min(troll.carry.free_capacity(troll.carry_capacity));
	if wood == 0 || troll.chop_power == 0 {
		return u32::MAX;
	}
	let chop_turns = (tree.health as u16 + troll.chop_power - 1) / troll.chop_power;
	let travel = env.dist(troll.pos, tree.pos) as u16 + env.dist_to_my_shack(tree.pos) as u16;
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
			let d_shack = env.dist_to_my_shack(t.pos) as u32;
			let cost = chop_cost_per_wood(t, troll, env);
			(d_shack, cost)
		})
}

fn is_adjacent_to_iron(env: &Env, pos: Coord) -> bool {
	env.iron_list
		.iter()
		.any(|&iron| manhattan_dist(pos, iron) == 1)
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
			let d = env.dist(troll.pos, pos);
			if d < best_dist {
				best = Some(pos);
				best_dist = d;
			}
		}
	}

	best
}

fn determine_goal(state: &TurnState, turn: Turn) -> Goal {
	if turn >= MAX_TURN - 20 {
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
			let spot_list = get_plant_spot_list_near_shack(env, state, troll, 3);
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
		if let Some(tree) = find_closest_tree_of_kind(env, state, troll, ResourceKind::Plum) {
			return move_or_do(troll, tree.pos, Action::Harvest(troll.id));
		}
	}
	if need_lemon {
		if let Some(tree) = find_closest_tree_of_kind(env, state, troll, ResourceKind::Lemon) {
			return move_or_do(troll, tree.pos, Action::Harvest(troll.id));
		}
	}

	solve_troll_accumulate(env, state, troll, cost)
}

fn is_needed_resource(
	kind: ResourceKind,
	inventory: &PlayerInventory,
	cost: &PlayerInventory,
) -> bool {
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

	if state.tree_list.iter().any(|t| {
		t.pos == troll.pos && t.fruit > 0 && is_needed_resource(t.kind, &state.my_inventory, cost)
	}) {
		return Action::Harvest(troll.id);
	}

	if let Some(tree) = state
		.tree_list
		.iter()
		.filter(|t| t.fruit > 0 && is_needed_resource(t.kind, &state.my_inventory, cost))
		.min_by_key(|t| env.dist(troll.pos, t.pos))
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

	if let Some(tree) = find_closest_tree_of_kind(env, state, troll, kind) {
		return Action::Move(troll.id, tree.pos);
	}

	solve_troll_accumulate(env, state, troll, cost)
}

fn solve_troll_mine_iron(
	env: &Env,
	state: &TurnState,
	troll: &Troll,
	cost: &PlayerInventory,
) -> Action {
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
		match troll.role() {
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
			let spot_list = get_plant_spot_list_near_shack(env, state, troll, 5);
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

	if state
		.tree_list
		.iter()
		.any(|t| t.pos == troll.pos && t.fruit > 0)
	{
		return Action::Harvest(troll.id);
	}

	if let Some(tree) = find_closest_tree_of_kind(env, state, troll, ResourceKind::Banana) {
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

	if let Some(tree) = find_closest_tree_with_fruit(env, state, troll) {
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
		match troll.role() {
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

	if let Some(tree) = find_closest_tree_with_fruit(env, state, troll) {
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

#[cfg(test)]
mod tests {
	use super::*;
	use std::time::Instant;

	fn make_max_grid() -> Grid {
		let w: Axis = MAX_W;
		let h: Axis = MAX_H;
		let mut grid = Grid {
			g: [u64::MAX; MAX_H as usize],
			w,
			h,
		};
		for y in 0..h {
			for x in 0..w {
				let cell = match (x as usize * 7 + y as usize * 13) % 30 {
					0 => Grid::ROCK,
					1 => Grid::WATER,
					2 => Grid::IRON,
					_ => Grid::GRASS,
				};
				grid.set_cell((x, y), cell);
			}
		}
		grid
	}

	#[test]
	fn bfs_all_cells_timing() {
		let grid = make_max_grid();

		let grass_count = (0..grid.h)
			.flat_map(|y| (0..grid.w).map(move |x| (x, y)))
			.filter(|&pos| grid.is_grass(pos))
			.count();

		let start = Instant::now();
		let (_dist_map, n) = compute_all_bfs(&grid);
		let elapsed = start.elapsed();

		assert_eq!(n, MAX_W as usize * MAX_H as usize);
		assert!(grass_count < n);

		eprintln!(
			"grid: {}x{}, cells: {n}, grass: {grass_count} ({:.0}%)",
			grid.w,
			grid.h,
			grass_count as f64 / n as f64 * 100.0
		);
		eprintln!("bfs time: {elapsed:?}");
		assert!(elapsed.as_millis() < FIRST_TURN_MS_LIMIT as u128);
	}
}
