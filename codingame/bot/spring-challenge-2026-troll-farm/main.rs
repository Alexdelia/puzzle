use std::{collections::VecDeque, fmt::Display, io, str::FromStr};

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

type Axis = u8;
const MAX_H: Axis = 11;
#[allow(dead_code)]
const MAX_W: Axis = MAX_H * 2;

type Turn = u16;
const MAX_TURN: Turn = 300;
const ENDGAME_TURN: Turn = MAX_TURN - 20;
#[allow(dead_code)]
const FIRST_TURN_MS_LIMIT: u64 = 1000;
#[allow(dead_code)]
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

impl ResourceKind {
	const fn cooldown(self, water: bool) -> u16 {
		match self {
			Self::Plum | Self::Lemon => {
				if water {
					3
				} else {
					8
				}
			}
			Self::Apple => {
				if water {
					2
				} else {
					9
				}
			}
			Self::Banana => {
				if water {
					4
				} else {
					6
				}
			}
			_ => 8,
		}
	}
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
	#[allow(dead_code)]
	is_next_to_water: bool,
	size: TreeSize,
	health: TreeHealth,
	fruit: TreeFruit,
	#[allow(dead_code)]
	cooldown: TreeCooldown,
}

impl Tree {
	const MAX_SIZE: TreeSize = 4;

	fn is_max_size(&self) -> bool {
		self.size >= Self::MAX_SIZE
	}

	fn max_health(&self) -> TreeHealth {
		match self.kind {
			ResourceKind::Plum | ResourceKind::Lemon => self.size as TreeHealth * 2 + 4,
			ResourceKind::Apple => self.size as TreeHealth * 3 + 8,
			ResourceKind::Banana => self.size as TreeHealth + 2,
			_ => 0,
		}
	}

	fn is_damaged(&self) -> bool {
		self.health < self.max_health()
	}
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
	turn: Turn,
	my_inventory: PlayerInventory,
	#[allow(dead_code)]
	op_inventory: PlayerInventory,
	my_troll_list: Vec<Troll>,
	#[allow(dead_code)]
	op_troll_list: Vec<Troll>,
	tree_list: Vec<Tree>,
	reserved: Vec<Coord>,
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

impl Action {
	fn target_pos(&self, troll: &Troll) -> Coord {
		match self {
			Action::Move(_, pos) => *pos,
			_ => troll.pos,
		}
	}
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
		if x > 0 && (self.g[y as usize] >> ((x - 1) as usize * 2)) & 0b11 == Self::WATER {
			return true;
		}
		let right = x + 1;
		if right < self.w && (self.g[y as usize] >> (right as usize * 2)) & 0b11 == Self::WATER {
			return true;
		}
		if y > 0 && (self.g[(y - 1) as usize] >> (x as usize * 2)) & 0b11 == Self::WATER {
			return true;
		}
		let down = y + 1;
		if down < self.h && (self.g[down as usize] >> (x as usize * 2)) & 0b11 == Self::WATER {
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

		// debug: shack distance map
		eprintln!("=== dist to my shack ===");
		for y in 0..grid.h {
			let mut row = String::new();
			for x in 0..grid.w {
				let pos = (x, y);
				let d = my_shack_dist[y as usize * grid.w as usize + x as usize];
				if pos == my_shack {
					row.push_str(" 0 ");
				} else if pos == op_shack {
					row.push_str(" 1 ");
				} else if d == u8::MAX {
					row.push_str(" # ");
				} else {
					row.push_str(&format!("{:2} ", d));
				}
			}
			eprintln!("{row}");
		}

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
		let d = self.bfs_dist[ai * self.bfs_n + bi];
		if d == u8::MAX && a == self.my_shack {
			return self.dist_to_my_shack(b).saturating_add(1);
		}
		d
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

	#[allow(dead_code)]
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
	fn read(env: &Env, turn: Turn) -> Self {
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

		my_troll_list.sort_by(|a, b| b.id.cmp(&a.id));

		let reserved = my_troll_list.iter().map(|t| t.pos).collect();

		Self {
			turn,
			my_inventory,
			op_inventory,
			my_troll_list,
			op_troll_list,
			tree_list,
			reserved,
		}
	}
}

fn manhattan_dist(a: Coord, b: Coord) -> u8 {
	(a.0 as i8 - b.0 as i8).unsigned_abs() + (a.1 as i8 - b.1 as i8).unsigned_abs()
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

fn drop_to_shack(env: &Env, state: &TurnState, troll: &Troll) -> Action {
	if is_next_to_shack(troll, env) {
		return Action::Drop(troll.id);
	}

	let shack = env.my_shack;
	let mut best = shack;
	let mut best_dist = u8::MAX;

	for (dx, dy) in [(0i8, 1i8), (0, -1), (1, 0), (-1, 0)] {
		let x = shack.0 as i8 + dx;
		let y = shack.1 as i8 + dy;
		if x < 0 || x >= env.grid.w as i8 || y < 0 || y >= env.grid.h as i8 {
			continue;
		}
		let pos = (x as Axis, y as Axis);
		if !env.grid.is_grass(pos) || state.reserved.contains(&pos) {
			continue;
		}
		let d = env.dist(troll.pos, pos);
		if d < best_dist {
			best = pos;
			best_dist = d;
		}
	}

	Action::Move(troll.id, best)
}

fn is_valid_plant_spot(env: &Env, state: &TurnState, pos: Coord) -> bool {
	env.grid.is_grass(pos) && state.tree_list.iter().all(|t| t.pos != pos)
}

fn plant_spot_score(env: &Env, pos: Coord, troll: &Troll, turn: Turn) -> u16 {
	let remaining = MAX_TURN.saturating_sub(turn);
	if remaining == 0 {
		return 0;
	}
	let cooldown = if env.grid.is_water_next_to(pos) {
		3u16
	} else {
		8u16
	};
	let growth = 5 * cooldown;
	let dist = env.dist_to_my_shack(pos) as u16;
	let harvest_cycle = 2 * dist / troll.move_speed.max(1) as u16 + 2;
	let bottleneck = cooldown.max(harvest_cycle);
	let producing = remaining.saturating_sub(growth);
	producing / bottleneck.max(1)
}

fn find_best_plant_spot(
	env: &Env,
	state: &TurnState,
	troll: &Troll,
	max_dist: u8,
) -> Option<Coord> {
	let mut best: Option<Coord> = None;
	let mut best_score = 0u16;

	for y in 0..env.grid.h {
		for x in 0..env.grid.w {
			let pos = (x, y);
			if pos == env.my_shack {
				continue;
			}
			let d = env.dist_to_my_shack(pos);
			if d > max_dist || !is_valid_plant_spot(env, state, pos) {
				continue;
			}
			let score = plant_spot_score(env, pos, troll, state.turn);
			if score > best_score
				|| (score == best_score
					&& best.is_some_and(|b| env.dist(troll.pos, pos) < env.dist(troll.pos, b)))
			{
				best = Some(pos);
				best_score = score;
			}
		}
	}

	best
}

fn find_closest_plant_spot(
	env: &Env,
	state: &TurnState,
	troll: &Troll,
	max_dist: u8,
) -> Option<Coord> {
	let mut best: Option<Coord> = None;
	let mut best_score = i16::MAX;

	for y in 0..env.grid.h {
		for x in 0..env.grid.w {
			let pos = (x, y);
			if pos == env.my_shack {
				continue;
			}
			let d = env.dist_to_my_shack(pos);
			if d > max_dist || !is_valid_plant_spot(env, state, pos) {
				continue;
			}
			let td = env.dist(troll.pos, pos).div_ceil(troll.move_speed.max(1)) as i16;
			let op = env.dist_to_op_shack(pos) as i16;
			let water_bonus = if env.grid.is_water_next_to(pos) { 4 } else { 0 };
			let score = d as i16 * 3 + td - op - water_bonus;
			if score < best_score {
				best = Some(pos);
				best_score = score;
			}
		}
	}

	best
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

fn chop_cost_per_wood(tree: &Tree, troll: &Troll, env: &Env) -> u32 {
	let wood = (tree.size as u16).min(troll.carry.free_capacity(troll.carry_capacity));
	if wood == 0 || troll.chop_power == 0 {
		return u32::MAX;
	}
	let speed = troll.move_speed.max(1) as u16;
	let chop_turns = (tree.health as u16).div_ceil(troll.chop_power);
	let dist_to = env.dist(troll.pos, tree.pos) as u16;
	let dist_back = env.dist_to_my_shack(tree.pos) as u16;
	let travel = (dist_to + dist_back).div_ceil(speed);
	(chop_turns + travel + 1) as u32 * 10 / wood as u32
}

fn chop_banana_penalty(env: &Env, state: &TurnState, tree: &Tree) -> u32 {
	if tree.kind != ResourceKind::Banana || env.dist_to_my_shack(tree.pos) > 5 {
		return 0;
	}
	let mature_banana_near = state
		.tree_list
		.iter()
		.filter(|t| {
			t.kind == ResourceKind::Banana && t.is_max_size() && env.dist_to_my_shack(t.pos) <= 5
		})
		.count();
	if mature_banana_near >= 4 { 0 } else { 1000 }
}

fn chop_score(env: &Env, state: &TurnState, tree: &Tree, troll: &Troll) -> u32 {
	chop_cost_per_wood(tree, troll, env) + chop_banana_penalty(env, state, tree)
}

fn is_op_chopping(state: &TurnState, tree: &Tree) -> bool {
	tree.is_damaged()
		&& state
			.op_troll_list
			.iter()
			.any(|t| t.pos == tree.pos && t.chop_power > 0)
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
		.filter(|t| t.size > 0 && !state.reserved.contains(&t.pos))
		.min_by_key(|t| chop_score(env, state, t, troll))
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
		.filter(|t| t.size > 0 && !state.reserved.contains(&t.pos))
		.min_by_key(|t| {
			let d_shack = env.dist_to_my_shack(t.pos) as u32;
			(d_shack, chop_score(env, state, t, troll))
		})
}

fn find_best_tree_to_chop_near_op_shack<'a>(
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
		.filter(|t| t.size > 0 && !state.reserved.contains(&t.pos))
		.min_by_key(|t| {
			let cost = chop_score(env, state, t, troll);
			let d_op = env.dist_to_op_shack(t.pos) as u32;
			let d_my = env.dist_to_my_shack(t.pos) as u32;
			let grief_bonus = if state.my_troll_list.len() > state.op_troll_list.len() {
				0
			} else {
				d_my.saturating_sub(d_op)
			};
			cost.saturating_sub(grief_bonus * 3)
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

fn determine_goal(state: &TurnState, train_duration: Turn) -> Goal {
	if state.turn >= ENDGAME_TURN {
		return Goal::Endgame;
	}

	let has_harvester = state
		.my_troll_list
		.iter()
		.any(|t| t.role() == TrollRole::Harvester);
	if !has_harvester {
		return Goal::TrainHarvester;
	}

	if train_duration > 0 {
		let remaining = (ENDGAME_TURN).saturating_sub(state.turn);
		if remaining < train_duration {
			return Goal::GatherPoint;
		}
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
			},
			TrollRole::Carrier => TrainTarget {
				move_speed: 3,
				carry_capacity: 4,
				harvest_power: 1,
				chop_power: 2,
			},
			TrollRole::Woodcutter => TrainTarget {
				move_speed: 2,
				carry_capacity: 4,
				harvest_power: 0,
				chop_power: 3,
			},
			TrollRole::Initial => TrainTarget {
				move_speed: 0,
				carry_capacity: 0,
				harvest_power: 0,
				chop_power: 0,
			},
		}
	}
}

fn solve_goal_train(env: &Env, state: &mut TurnState, role: TrollRole) -> Vec<Action> {
	let mut action_list = Vec::new();
	let target = role.target();
	let troll_count = state.my_troll_list.len();

	if target.can_afford(&state.my_inventory, troll_count) {
		action_list.push(target.train_action());

		let cost = target.cost(troll_count);
		state.my_inventory.plum = state.my_inventory.plum.saturating_sub(cost.plum);
		state.my_inventory.lemon = state.my_inventory.lemon.saturating_sub(cost.lemon);
		state.my_inventory.apple = state.my_inventory.apple.saturating_sub(cost.apple);
		state.my_inventory.iron = state.my_inventory.iron.saturating_sub(cost.iron);

		let next_role = match role {
			TrollRole::Harvester => Some(TrollRole::Carrier),
			TrollRole::Carrier => Some(TrollRole::Woodcutter),
			_ => None,
		};

		if let Some(next) = next_role {
			action_list.extend(solve_goal_train(env, state, next));
		} else {
			for troll in &state.my_troll_list {
				action_list.push(solve_troll_gather(env, state, troll));
			}
		}
		return action_list;
	}

	let cost = target.cost(troll_count);
	let need_apple = state.my_inventory.apple < cost.apple;
	let need_iron = state.my_inventory.iron < cost.iron;

	let plum_remaining = cost.plum.saturating_sub(state.my_inventory.plum);
	let lemon_remaining = cost.lemon.saturating_sub(state.my_inventory.lemon);
	let total_carry: Resource = state.my_troll_list.iter().map(|t| t.carry_capacity).sum();
	let target_plum = if plum_remaining > total_carry {
		(plum_remaining / 5).max(2)
	} else {
		0
	} as u8;
	let target_lemon = if lemon_remaining > total_carry {
		(lemon_remaining / 5).max(2)
	} else {
		0
	} as u8;

	let plum_near = count_tree_near_shack(env, state, ResourceKind::Plum, 3);
	let lemon_near = count_tree_near_shack(env, state, ResourceKind::Lemon, 3);
	let op_near_shack = state
		.op_troll_list
		.iter()
		.any(|t| env.dist_to_my_shack(t.pos) <= 2);

	let op_has_chopper = state.op_troll_list.iter().any(|t| t.chop_power >= 2);
	let plum_growing = op_has_chopper
		&& state.tree_list.iter().any(|t| {
			t.kind == ResourceKind::Plum && !t.is_max_size() && env.dist_to_my_shack(t.pos) <= 3
		});
	let lemon_growing = op_has_chopper
		&& state.tree_list.iter().any(|t| {
			t.kind == ResourceKind::Lemon && !t.is_max_size() && env.dist_to_my_shack(t.pos) <= 3
		});

	let need_plant_plum = plum_near < target_plum && !plum_growing;
	let need_plant_lemon = lemon_near < target_lemon && !lemon_growing;
	let need_planting = !op_near_shack && (need_plant_plum || need_plant_lemon);

	let threatened_fruit = state.op_troll_list.iter().any(|t| t.chop_power >= 2)
		&& state.tree_list.iter().any(|t| {
			t.fruit > 0
				&& env.dist_to_my_shack(t.pos) <= 2
				&& is_needed_resource(t.kind, &state.my_inventory, &cost)
		});

	let miner_id = if need_iron && !threatened_fruit {
		state
			.my_troll_list
			.iter()
			.filter(|t| t.chop_power > 0)
			.max_by_key(|t| (t.chop_power, t.move_speed))
			.map(|t| t.id)
	} else {
		None
	};

	let mut planter_assigned = false;
	// let mut apple_assigned = false;

	eprintln!(
		"=== solve_goal_train: need_planting={} need_apple={} need_iron={} miner_id={:?} threatened={}",
		need_planting, need_apple, need_iron, miner_id, threatened_fruit
	);
	eprintln!(
		"  plum_near={} target_plum={} lemon_near={} target_lemon={}",
		plum_near, target_plum, lemon_near, target_lemon
	);
	/*
	let has_harvester = state
		.my_troll_list
		.iter()
		.any(|t| t.role() == TrollRole::Harvester);
	*/
	for i in 0..state.my_troll_list.len() {
		let troll_pos = state.my_troll_list[i].pos;
		state.reserved.retain(|&p| p != troll_pos);
		let (path, action) = if miner_id == Some(state.my_troll_list[i].id) {
			(
				"mine_iron",
				solve_troll_mine_iron(env, state, &state.my_troll_list[i], &cost),
			)
		} else if need_planting && !planter_assigned {
			planter_assigned = true;
			(
				"plant_for_goal",
				solve_troll_plant_for_goal(
					env,
					state,
					&state.my_troll_list[i],
					need_plant_plum,
					need_plant_lemon,
					&cost,
				),
			)
		} else {
			(
				"accumulate",
				solve_troll_accumulate(env, state, &state.my_troll_list[i], &cost),
			)
		};
		eprintln!(
			"  troll {} at ({},{}) -> {} -> {}",
			state.my_troll_list[i].id,
			state.my_troll_list[i].pos.0,
			state.my_troll_list[i].pos.1,
			path,
			action
		);

		let troll_pos = state.my_troll_list[i].pos;
		let hp = state.my_troll_list[i].harvest_power;
		let free = state.my_troll_list[i]
			.carry
			.free_capacity(state.my_troll_list[i].carry_capacity);
		match &action {
			Action::Harvest(_) => {
				if let Some(tree) = state.tree_list.iter_mut().find(|t| t.pos == troll_pos) {
					let taken = (tree.fruit as u16).min(hp).min(free);
					tree.fruit = tree.fruit.saturating_sub(taken as u8);
				}
			}
			Action::Chop(_) => {
				if let Some(tree) = state.tree_list.iter_mut().find(|t| t.pos == troll_pos) {
					tree.fruit = 0;
				}
			}
			_ => {}
		}

		state
			.reserved
			.push(action.target_pos(&state.my_troll_list[i]));
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

		if let Some(kind) = plant_kind
			&& let Some(spot) = find_best_plant_spot(env, state, troll, 3)
		{
			return move_or_do(troll, spot, Action::Plant(troll.id, kind));
		}

		return drop_to_shack(env, state, troll);
	}

	let can_pick_to_plant = |kind: ResourceKind| {
		state.my_troll_list.len() < 2
			|| !state
				.tree_list
				.iter()
				.any(|t| t.kind == kind && t.fruit > 0)
	};
	let pick_kind = if need_plum
		&& state.my_inventory.plum > 0
		&& can_pick_to_plant(ResourceKind::Plum)
	{
		Some(ResourceKind::Plum)
	} else if need_lemon && state.my_inventory.lemon > 0 && can_pick_to_plant(ResourceKind::Lemon) {
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

	eprintln!(
		"plant_for_goal troll {} at ({},{}) need_plum={} need_lemon={}",
		troll.id, troll.pos.0, troll.pos.1, need_plum, need_lemon
	);
	for t in state.tree_list.iter().filter(|t| {
		t.fruit > 0
			&& ((need_plum && t.kind == ResourceKind::Plum)
				|| (need_lemon && t.kind == ResourceKind::Lemon))
	}) {
		eprintln!(
			"  candidate: {} at ({},{}) fruit={} cd={} dist={}",
			t.kind,
			t.pos.0,
			t.pos.1,
			t.fruit,
			t.cooldown,
			env.dist(troll.pos, t.pos)
		);
	}
	if let Some(tree) = state
		.tree_list
		.iter()
		.filter(|t| {
			t.fruit > 0
				&& ((need_plum && t.kind == ResourceKind::Plum)
					|| (need_lemon && t.kind == ResourceKind::Lemon))
		})
		.min_by_key(|t| env.dist(troll.pos, t.pos))
	{
		eprintln!(
			"  -> picked: {} at ({},{}) dist={}",
			tree.kind,
			tree.pos.0,
			tree.pos.1,
			env.dist(troll.pos, tree.pos)
		);
		return move_or_do(troll, tree.pos, Action::Harvest(troll.id));
	}

	solve_troll_accumulate(env, state, troll, cost)
}

fn harvest_or_wait_score(env: &Env, state: &TurnState, tree: &Tree, troll: &Troll) -> u32 {
	if tree.fruit > 0 {
		return harvest_trip_score(env, tree, troll);
	}
	let fruit = 1u16
		.min(troll.harvest_power)
		.min(troll.carry.free_capacity(troll.carry_capacity));
	if fruit == 0 {
		return u32::MAX;
	}
	let water = env.grid.is_water_next_to(tree.pos);
	let cd = tree.kind.cooldown(water);
	let remaining_growth = (Tree::MAX_SIZE as u16).saturating_sub(tree.size as u16);
	let turns_to_fruit = tree.cooldown as u16 + remaining_growth * cd;
	if is_op_chopping(state, tree) {
		let op_chop = state
			.op_troll_list
			.iter()
			.find(|t| t.pos == tree.pos && t.chop_power > 0)
			.unwrap()
			.chop_power;
		let turns_to_death = (tree.health as u16).div_ceil(op_chop);
		if turns_to_fruit >= turns_to_death {
			return u32::MAX;
		}
	}
	let speed = troll.move_speed.max(1) as u16;
	let dist_to = env.dist(troll.pos, tree.pos) as u16;
	let travel_to = dist_to.div_ceil(speed);
	let wait = turns_to_fruit.saturating_sub(travel_to);
	let dist_back = env.dist_to_my_shack(tree.pos) as u16;
	let trip = travel_to + wait + dist_back.div_ceil(speed) + 2;
	trip as u32 * 10 / fruit as u32
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

fn harvest_trip_score(env: &Env, tree: &Tree, troll: &Troll) -> u32 {
	let fruit = (tree.fruit as u16)
		.min(troll.harvest_power)
		.min(troll.carry.free_capacity(troll.carry_capacity));
	if fruit == 0 {
		return u32::MAX;
	}
	let speed = troll.move_speed.max(1) as u16;
	let dist_to = env.dist(troll.pos, tree.pos) as u16;
	let dist_back = env.dist_to_my_shack(tree.pos) as u16;
	let trip_turns = (dist_to + dist_back).div_ceil(speed) + 2;
	let score = trip_turns as u32 * 10 / fruit as u32;
	eprintln!(
		"  trip_score: {} at ({},{}) dist_to={} dist_back={} trip={} fruit={} score={}",
		tree.kind, tree.pos.0, tree.pos.1, dist_to, dist_back, trip_turns, fruit, score
	);
	score
}

fn solve_troll_accumulate(
	env: &Env,
	state: &TurnState,
	troll: &Troll,
	cost: &PlayerInventory,
) -> Action {
	if !troll.carry.is_empty() {
		if let Some(action) = try_continue_harvesting(env, state, troll) {
			return action;
		}
		return drop_to_shack(env, state, troll);
	}

	let on_needed_tree = state.tree_list.iter().any(|t| {
		t.pos == troll.pos && t.fruit > 0 && is_needed_resource(t.kind, &state.my_inventory, cost)
	});

	if on_needed_tree {
		return Action::Harvest(troll.id);
	}

	eprintln!(
		"accumulate troll {} at ({},{}) - needed trees:",
		troll.id, troll.pos.0, troll.pos.1
	);
	if let Some(tree) = state
		.tree_list
		.iter()
		.filter(|t| {
			(t.fruit > 0 || t.size > 0)
				&& is_needed_resource(t.kind, &state.my_inventory, cost)
				&& !state.reserved.contains(&t.pos)
		})
		.min_by_key(|t| harvest_or_wait_score(env, state, t, troll))
	{
		let score = harvest_or_wait_score(env, state, tree, troll);
		if score < u32::MAX {
			eprintln!(
				"  -> best needed: {} at ({},{}) fruit={} cd={} sz={} score={}",
				tree.kind, tree.pos.0, tree.pos.1, tree.fruit, tree.cooldown, tree.size, score
			);
			return Action::Move(troll.id, tree.pos);
		}
	}

	if state
		.tree_list
		.iter()
		.any(|t| t.pos == troll.pos && t.fruit > 0)
	{
		return Action::Harvest(troll.id);
	}

	let training = cost.total() > 0;
	eprintln!(
		"accumulate troll {} - any trees (training={}):",
		troll.id, training
	);
	if let Some(tree) = state
		.tree_list
		.iter()
		.filter(|t| t.fruit > 0 && (!training || t.kind != ResourceKind::Banana))
		.min_by_key(|t| harvest_trip_score(env, t, troll))
	{
		eprintln!(
			"  -> best any: {} at ({},{})",
			tree.kind, tree.pos.0, tree.pos.1
		);
		return Action::Move(troll.id, tree.pos);
	}

	Action::Move(troll.id, env.op_shack)
}

fn solve_troll_mine_iron(
	env: &Env,
	state: &TurnState,
	troll: &Troll,
	cost: &PlayerInventory,
) -> Action {
	if !troll.carry.is_empty() {
		if let Some(action) = try_continue_mining(env, troll) {
			return action;
		}
		return drop_to_shack(env, state, troll);
	}

	if is_adjacent_to_iron(env, troll.pos) {
		return Action::Mine(troll.id);
	}

	if let Some(grass_pos) = find_closest_grass_near_iron(env, troll) {
		return Action::Move(troll.id, grass_pos);
	}

	solve_troll_accumulate(env, state, troll, cost)
}

fn solve_troll_harvest_and_store(env: &Env, state: &TurnState, troll: &Troll) -> Action {
	if !troll.carry.is_empty() {
		if let Some(action) = try_continue_harvesting(env, state, troll) {
			return action;
		}
		return drop_to_shack(env, state, troll);
	}

	if state
		.tree_list
		.iter()
		.any(|t| t.pos == troll.pos && t.fruit > 0)
	{
		return Action::Harvest(troll.id);
	}

	if let Some(tree) = state
		.tree_list
		.iter()
		.filter(|t| t.fruit > 0 && !state.reserved.contains(&t.pos))
		.min_by_key(|t| harvest_trip_score(env, t, troll))
	{
		return Action::Move(troll.id, tree.pos);
	}

	Action::Move(troll.id, env.op_shack)
}

fn solve_goal_gather_point(env: &Env, state: &mut TurnState) -> Vec<Action> {
	let mut action_list = Vec::new();

	let banana_near = state
		.tree_list
		.iter()
		.filter(|t| t.kind == ResourceKind::Banana && env.dist_to_my_shack(t.pos) <= 5)
		.count();
	let enough_banana_to_stop_planting = banana_near >= 8;

	for i in 0..state.my_troll_list.len() {
		let role = state.my_troll_list[i].role();
		let chop = state.my_troll_list[i].chop_power;
		let troll_pos = state.my_troll_list[i].pos;
		state.reserved.retain(|&p| p != troll_pos);
		let action = match role {
			TrollRole::Harvester => solve_troll_banana_planter(env, state, &state.my_troll_list[i]),
			TrollRole::Initial if !enough_banana_to_stop_planting => {
				solve_troll_banana_planter(env, state, &state.my_troll_list[i])
			}
			_ if chop > 0 && role == TrollRole::Carrier => {
				solve_troll_chopper_near_op_shack(env, state, &state.my_troll_list[i])
			}
			_ if chop > 0 => solve_troll_chopper(env, state, &state.my_troll_list[i]),
			_ => solve_troll_harvest_and_store(env, state, &state.my_troll_list[i]),
		};
		state
			.reserved
			.push(action.target_pos(&state.my_troll_list[i]));
		action_list.push(action);
	}

	action_list
}

fn solve_troll_chopper_near_op_shack(env: &Env, state: &TurnState, troll: &Troll) -> Action {
	if !troll.carry.is_empty() {
		if let Some(action) = try_continue_chopping(env, state, troll) {
			return action;
		}
		return drop_to_shack(env, state, troll);
	}

	if troll.chop_power > 0
		&& let Some(best) = find_best_tree_to_chop_near_op_shack(env, state, troll)
	{
		if best.pos == troll.pos {
			return Action::Chop(troll.id);
		}
		return Action::Move(troll.id, best.pos);
	}

	Action::Move(troll.id, env.op_shack)
}

fn solve_troll_banana_planter(env: &Env, state: &TurnState, troll: &Troll) -> Action {
	if !troll.carry.is_empty() {
		if troll.carry.banana > 0
			&& let Some(spot) = find_closest_plant_spot(env, state, troll, 5)
		{
			return move_or_do(troll, spot, Action::Plant(troll.id, ResourceKind::Banana));
		}
		return drop_to_shack(env, state, troll);
	}

	if state
		.tree_list
		.iter()
		.any(|t| t.pos == troll.pos && t.kind == ResourceKind::Banana && t.fruit > 0)
	{
		return Action::Harvest(troll.id);
	}

	if let Some(tree) = state
		.tree_list
		.iter()
		.filter(|t| {
			t.kind == ResourceKind::Banana && t.fruit > 0 && !state.reserved.contains(&t.pos)
		})
		.min_by_key(|t| env.dist(troll.pos, t.pos))
	{
		return Action::Move(troll.id, tree.pos);
	}

	if state.my_inventory.banana > 0 && is_next_to_shack(troll, env) {
		return Action::Pick(troll.id, ResourceKind::Banana);
	}
	if state.my_inventory.banana > 0 {
		return Action::Move(troll.id, env.my_shack);
	}

	if let Some(tree) = state
		.tree_list
		.iter()
		.filter(|t| t.fruit > 0 && !state.reserved.contains(&t.pos))
		.min_by_key(|t| env.dist(troll.pos, t.pos))
	{
		return Action::Move(troll.id, tree.pos);
	}

	solve_troll_chopper(env, state, troll)
}

fn try_continue_harvesting(env: &Env, state: &TurnState, troll: &Troll) -> Option<Action> {
	let free = troll.carry.free_capacity(troll.carry_capacity);
	if free == 0 || troll.harvest_power == 0 {
		return None;
	}
	if state
		.tree_list
		.iter()
		.any(|t| t.pos == troll.pos && t.fruit > 0)
	{
		return Some(Action::Harvest(troll.id));
	}
	let dist_shack = env.dist_to_my_shack(troll.pos);
	state
		.tree_list
		.iter()
		.filter(|t| t.fruit > 0 && env.dist(troll.pos, t.pos) < dist_shack)
		.min_by_key(|t| env.dist(troll.pos, t.pos))
		.map(|t| Action::Move(troll.id, t.pos))
}

fn try_continue_mining(env: &Env, troll: &Troll) -> Option<Action> {
	let free = troll.carry.free_capacity(troll.carry_capacity);
	if free == 0 || troll.chop_power == 0 {
		return None;
	}
	if is_adjacent_to_iron(env, troll.pos) {
		return Some(Action::Mine(troll.id));
	}
	let dist_shack = env.dist_to_my_shack(troll.pos);
	find_closest_grass_near_iron(env, troll)
		.filter(|&pos| env.dist(troll.pos, pos) < dist_shack)
		.map(|pos| Action::Move(troll.id, pos))
}

fn try_continue_chopping(env: &Env, state: &TurnState, troll: &Troll) -> Option<Action> {
	let free = troll.carry.free_capacity(troll.carry_capacity);
	if free == 0 || troll.chop_power == 0 {
		return None;
	}
	let best = find_best_tree_to_chop(env, state, troll)?;
	let dist_tree = env.dist(troll.pos, best.pos);
	let dist_shack = env.dist_to_my_shack(troll.pos);
	if dist_tree >= dist_shack {
		return None;
	}
	if best.pos == troll.pos {
		Some(Action::Chop(troll.id))
	} else {
		Some(Action::Move(troll.id, best.pos))
	}
}

fn solve_troll_chopper(env: &Env, state: &TurnState, troll: &Troll) -> Action {
	if !troll.carry.is_empty() {
		if let Some(action) = try_continue_chopping(env, state, troll) {
			return action;
		}
		return drop_to_shack(env, state, troll);
	}

	if troll.chop_power > 0
		&& let Some(best) = find_best_tree_to_chop(env, state, troll)
	{
		if best.pos == troll.pos {
			return Action::Chop(troll.id);
		}
		return Action::Move(troll.id, best.pos);
	}

	Action::Move(troll.id, env.op_shack)
}

fn solve_goal_endgame(env: &Env, state: &mut TurnState) -> Vec<Action> {
	let mut action_list = Vec::new();

	for i in 0..state.my_troll_list.len() {
		let troll_pos = state.my_troll_list[i].pos;
		let chop = state.my_troll_list[i].chop_power;
		let remaining = MAX_TURN.saturating_sub(state.turn);
		state.reserved.retain(|&p| p != troll_pos);
		let action = if chop > 0 {
			solve_troll_chopper_near_shack(env, state, &state.my_troll_list[i])
		} else if remaining >= 10 {
			solve_troll_banana_planter(env, state, &state.my_troll_list[i])
		} else {
			solve_troll_harvest_and_store(env, state, &state.my_troll_list[i])
		};
		state
			.reserved
			.push(action.target_pos(&state.my_troll_list[i]));
		action_list.push(action);
	}

	action_list
}

fn solve_troll_chopper_near_shack(env: &Env, state: &TurnState, troll: &Troll) -> Action {
	if !troll.carry.is_empty() {
		if let Some(action) = try_continue_chopping(env, state, troll) {
			return action;
		}
		return drop_to_shack(env, state, troll);
	}

	if troll.chop_power > 0
		&& let Some(best) = find_best_tree_to_chop_near_shack(env, state, troll)
	{
		if best.pos == troll.pos {
			return Action::Chop(troll.id);
		}
		return Action::Move(troll.id, best.pos);
	}

	Action::Move(troll.id, env.op_shack)
}

fn solve_troll_gather(env: &Env, state: &TurnState, troll: &Troll) -> Action {
	if !troll.carry.is_empty() {
		return drop_to_shack(env, state, troll);
	}

	if let Some(tree) = find_closest_tree_with_fruit(env, state, troll) {
		return move_or_do(troll, tree.pos, Action::Harvest(troll.id));
	}

	if troll.chop_power > 0
		&& let Some(tree) = find_best_tree_to_chop(env, state, troll)
	{
		return move_or_do(troll, tree.pos, Action::Chop(troll.id));
	}

	Action::Move(troll.id, env.op_shack)
}

fn solve(env: &Env, state: &mut TurnState, train_duration: Turn) -> Vec<Action> {
	let goal = determine_goal(state, train_duration);
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
	let mut last_train_turn: Turn = 0;
	let mut last_troll_count: usize = 1;
	let mut train_duration: Turn = 0;

	loop {
		turn += 1;
		let mut state = TurnState::read(&env, turn);

		if state.my_troll_list.len() > last_troll_count {
			train_duration = turn.saturating_sub(last_train_turn);
			last_train_turn = turn;
			last_troll_count = state.my_troll_list.len();
			eprintln!("trained! duration={train_duration} trolls={last_troll_count}");
		}

		let t0 = std::time::Instant::now();
		let action_list = solve(&env, &mut state, train_duration);
		println!(
			"{};MSG {:?}",
			action_list
				.into_iter()
				.map(|a| a.to_string())
				.collect::<Vec<_>>()
				.join(";"),
			t0.elapsed()
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
