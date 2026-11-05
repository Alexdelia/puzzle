use std::collections::HashMap;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

pub const MAX_NODES: usize = u64::BITS as usize;
pub const MAX_PATH_LENGTH: i32 = 400;
pub const MAX_TURNS: i32 = 200;
pub const DOMINATION_SHARE: f64 = 0.8;
const PAIR_WORDS: usize = 5;
const TRIANGLE_WORDS: usize = 8;
const NO_PAIR: u16 = u16::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Bits<const W: usize>([u64; W]);

impl<const W: usize> Default for Bits<W> {
	fn default() -> Self {
		Bits([0; W])
	}
}

impl<const W: usize> Bits<W> {
	pub const CAPACITY: usize = W * u64::BITS as usize;

	pub fn has(&self, index: usize) -> bool {
		self.0[index / 64] >> (index % 64) & 1 != 0
	}

	pub fn insert(&mut self, index: usize) {
		self.0[index / 64] |= 1 << (index % 64);
	}

	pub fn remove(&mut self, index: usize) {
		self.0[index / 64] &= !(1 << (index % 64));
	}

	pub fn assign(&mut self, index: usize, present: bool) {
		if present {
			self.insert(index);
		} else {
			self.remove(index);
		}
	}

	pub fn is_empty(&self) -> bool {
		self.0.iter().all(|&word| word == 0)
	}

	pub fn len(&self) -> usize {
		self.0.iter().map(|word| word.count_ones() as usize).sum()
	}

	pub fn iter(self) -> impl Iterator<Item = usize> {
		self.0
			.into_iter()
			.enumerate()
			.flat_map(|(w, word)| nodes_of(word).map(move |bit| w * u64::BITS as usize + bit))
	}
}

impl<const W: usize> BitAnd for Bits<W> {
	type Output = Self;

	fn bitand(self, other: Self) -> Self {
		Bits(std::array::from_fn(|w| self.0[w] & other.0[w]))
	}
}

impl<const W: usize> BitOr for Bits<W> {
	type Output = Self;

	fn bitor(self, other: Self) -> Self {
		Bits(std::array::from_fn(|w| self.0[w] | other.0[w]))
	}
}

impl<const W: usize> Not for Bits<W> {
	type Output = Self;

	fn not(self) -> Self {
		Bits(self.0.map(|word| !word))
	}
}

impl<const W: usize> BitAndAssign for Bits<W> {
	fn bitand_assign(&mut self, other: Self) {
		*self = *self & other;
	}
}

impl<const W: usize> BitOrAssign for Bits<W> {
	fn bitor_assign(&mut self, other: Self) {
		*self = *self | other;
	}
}

pub type PairSet = Bits<PAIR_WORDS>;
pub type TriangleSet = Bits<TRIANGLE_WORDS>;

pub fn bit(node: usize) -> u64 {
	1 << node
}

pub fn nodes_of(mut set: u64) -> impl Iterator<Item = usize> {
	std::iter::from_fn(move || {
		(set != 0).then(|| {
			let node = set.trailing_zeros() as usize;
			set &= set - 1;
			node
		})
	})
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
	pub x: i32,
	pub y: i32,
}

impl Point {
	pub fn dist2(self, other: Point) -> i32 {
		let (dx, dy) = (self.x - other.x, self.y - other.y);
		dx * dx + dy * dy
	}
}

fn ccw(a: Point, b: Point, c: Point) -> bool {
	(c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x)
}

fn is_crossing(points: &[Point], [a, b]: [usize; 2], [c, d]: [usize; 2]) -> bool {
	if a == b || a == c || b == c || a == d || b == d {
		return false;
	}
	let [a, b, c, d] = [a, b, c, d].map(|n| points[n]);
	ccw(a, c, d) != ccw(b, c, d) && ccw(a, b, c) != ccw(a, b, d)
}

fn is_inside(points: &[Point], node: usize, corners: [usize; 3]) -> bool {
	if corners.contains(&node) {
		return false;
	}
	let p = points[node];
	let [a, b, c] = corners.map(|n| points[n]);
	let turns = [ccw(p, a, b), ccw(p, b, c), ccw(p, c, a)];
	!(turns.contains(&false) && turns.contains(&true))
}

fn has_node_inside(points: &[Point], corners: [usize; 3]) -> bool {
	let [a, b, c] = corners.map(|n| points[n]);
	let box_bounds_inside = ccw(a, b, c) || ccw(a, c, b);
	let [left, right] = [a.x.min(b.x).min(c.x), a.x.max(b.x).max(c.x)];
	let [top, bottom] = [a.y.min(b.y).min(c.y), a.y.max(b.y).max(c.y)];
	(0..points.len()).any(|n| {
		let p = points[n];
		let in_box = (left..=right).contains(&p.x) && (top..=bottom).contains(&p.y);
		(in_box || !box_bounds_inside) && is_inside(points, n, corners)
	})
}

pub struct Map {
	pub points: Vec<Point>,
	pub pairs: Vec<[usize; 2]>,
	pair_ids: Vec<u16>,
	crossings: Vec<PairSet>,
	pub triangles: Vec<[usize; 3]>,
	triangle_pairs: Vec<[usize; 3]>,
	pair_triangles: Vec<Vec<usize>>,
	corner_sets: Vec<u64>,
	triangle_ids: HashMap<[usize; 3], usize>,
	pub all_triangles: TriangleSet,
}

impl Map {
	pub fn new(points: Vec<Point>) -> Self {
		let count = points.len();
		assert!(count <= MAX_NODES, "at most {MAX_NODES} nodes");
		let reach = MAX_PATH_LENGTH * MAX_PATH_LENGTH;
		let mut pairs = Vec::new();
		let mut pair_ids = vec![NO_PAIR; count * count];
		for a in 0..count {
			for b in a + 1..count {
				if points[a].dist2(points[b]) <= reach {
					pair_ids[a * count + b] = pairs.len() as u16;
					pair_ids[b * count + a] = pairs.len() as u16;
					pairs.push([a, b]);
				}
			}
		}
		assert!(pairs.len() <= PairSet::CAPACITY, "too many pairs");
		let crossings = pairs
			.iter()
			.map(|&pair| {
				let mut crossing = PairSet::default();
				for (other, &candidate) in pairs.iter().enumerate() {
					if is_crossing(&points, pair, candidate) {
						crossing.insert(other);
					}
				}
				crossing
			})
			.collect();
		let pair_of = |a: usize, b: usize| pair_ids[a * count + b];
		let mut triangles = Vec::new();
		let mut triangle_pairs = Vec::new();
		let mut pair_triangles = vec![Vec::new(); pairs.len()];
		for &[a, b] in &pairs {
			for c in b + 1..count {
				let sides = [pair_of(a, b), pair_of(a, c), pair_of(b, c)];
				if sides.contains(&NO_PAIR) || has_node_inside(&points, [a, b, c]) {
					continue;
				}
				let sides = sides.map(usize::from);
				for side in sides {
					pair_triangles[side].push(triangles.len());
				}
				triangles.push([a, b, c]);
				triangle_pairs.push(sides);
			}
		}
		assert!(
			triangles.len() <= TriangleSet::CAPACITY,
			"too many triangles"
		);
		let corner_sets = triangles
			.iter()
			.map(|corners| corners.iter().fold(0, |set, &c| set | bit(c)))
			.collect();
		let triangle_ids = triangles
			.iter()
			.enumerate()
			.map(|(t, &corners)| (corners, t))
			.collect();
		let mut all_triangles = TriangleSet::default();
		for t in 0..triangles.len() {
			all_triangles.insert(t);
		}
		Map {
			points,
			pairs,
			pair_ids,
			crossings,
			triangles,
			triangle_pairs,
			pair_triangles,
			corner_sets,
			triangle_ids,
			all_triangles,
		}
	}

	pub fn len(&self) -> usize {
		self.points.len()
	}

	pub fn is_empty(&self) -> bool {
		self.points.is_empty()
	}

	pub fn pair(&self, a: usize, b: usize) -> Option<usize> {
		let id = self.pair_ids[a * self.len() + b];
		(id != NO_PAIR).then_some(usize::from(id))
	}

	pub fn triangle(&self, mut corners: [usize; 3]) -> Option<usize> {
		corners.sort_unstable();
		self.triangle_ids.get(&corners).copied()
	}

	pub fn corners(&self, triangle: usize) -> u64 {
		self.corner_sets[triangle]
	}

	pub fn can_connect(&self, edges: &PairSet, from: usize, to: usize) -> bool {
		self.pair(from, to)
			.is_some_and(|pair| (self.crossings[pair] & *edges).is_empty())
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Command {
	Move {
		from: usize,
		to: usize,
		amount: i32,
	},
	Spawn {
		triangle: usize,
		node: usize,
	},
	Attack {
		triangle: usize,
		target: usize,
	},
	AddPath {
		triangle: usize,
		from: usize,
		to: usize,
	},
	RemovePath {
		triangle: usize,
		from: usize,
		to: usize,
	},
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
	pub units: [[i32; 2]; MAX_NODES],
	pub adjacency: [u64; MAX_NODES],
	pub edges: PairSet,
	pub live: TriangleSet,
	pub owner: [TriangleSet; 2],
	pub initial_owner: [TriangleSet; 2],
	pub can_capture: [TriangleSet; 2],
	pub score: [i32; 2],
}

impl State {
	pub fn new(map: &Map, units: &[[i32; 2]], edges: &[(usize, usize)]) -> Self {
		let mut state = State {
			units: [[0; 2]; MAX_NODES],
			adjacency: [0; MAX_NODES],
			edges: PairSet::default(),
			live: TriangleSet::default(),
			owner: [TriangleSet::default(); 2],
			initial_owner: [TriangleSet::default(); 2],
			can_capture: [map.all_triangles; 2],
			score: [0; 2],
		};
		state.units[..units.len()].copy_from_slice(units);
		for &(a, b) in edges {
			state.connect(map, a, b);
		}
		state
	}

	pub fn set_triangle(&mut self, triangle: usize, owner: Option<usize>, can_capture: [bool; 2]) {
		for (player, &capturable) in can_capture.iter().enumerate() {
			self.owner[player].assign(triangle, owner == Some(player));
			self.initial_owner[player].assign(triangle, owner == Some(player));
			self.can_capture[player].assign(triangle, capturable);
		}
	}

	pub fn connected(&self, a: usize, b: usize) -> bool {
		self.adjacency[a] & bit(b) != 0
	}

	fn connect(&mut self, map: &Map, a: usize, b: usize) {
		let pair = map
			.pair(a, b)
			.expect("paths are never longer than the limit");
		self.adjacency[a] |= bit(b);
		self.adjacency[b] |= bit(a);
		self.edges.insert(pair);
		for &t in &map.pair_triangles[pair] {
			if map.triangle_pairs[t]
				.iter()
				.all(|&side| self.edges.has(side))
			{
				self.live.insert(t);
			}
		}
	}

	fn disconnect(&mut self, map: &Map, a: usize, b: usize) {
		let pair = map.pair(a, b).expect("only existing paths are removed");
		self.adjacency[a] &= !bit(b);
		self.adjacency[b] &= !bit(a);
		self.edges.remove(pair);
		for &t in &map.pair_triangles[pair] {
			self.live.remove(t);
		}
	}

	fn adjacent_to(&self, nodes: u64) -> u64 {
		nodes_of(nodes).fold(0, |around, n| around | self.adjacency[n])
	}

	pub fn first_step(&self, from: usize, to: usize) -> Option<usize> {
		let mut seen = bit(to);
		let mut frontier = seen;
		while frontier != 0 {
			let next = self.adjacent_to(frontier) & !seen;
			if next & bit(from) != 0 {
				let step = self.adjacency[from] & frontier;
				return (step != 0).then(|| step.trailing_zeros() as usize);
			}
			seen |= next;
			frontier = next;
		}
		None
	}

	pub fn units_of(&self, player: usize) -> i32 {
		self.units.iter().map(|u| u[player]).sum()
	}

	pub fn triangles_of(&self, player: usize) -> usize {
		(self.owner[player] & self.live).len()
	}

	pub fn occupied_by(&self, player: usize) -> u64 {
		self.units
			.iter()
			.enumerate()
			.fold(0, |set, (n, u)| set | (u64::from(u[player] > 0) << n))
	}

	fn advantage(&self, player: usize) -> u64 {
		self.units.iter().enumerate().fold(0, |set, (n, u)| {
			set | (u64::from(u[player] > u[1 - player]) << n)
		})
	}

	fn dominated_by(&self, advantage: u64) -> u64 {
		self.adjacency
			.iter()
			.enumerate()
			.fold(0, |set, (n, &around)| {
				set | (u64::from(around != 0 && around & !advantage == 0) << n)
			})
	}

	pub fn node_owners(&self) -> [u64; 2] {
		let advantage = [0, 1].map(|p| self.advantage(p));
		let dominated = advantage.map(|a| self.dominated_by(a));
		[
			dominated[0] | (!dominated[1] & advantage[0]),
			dominated[1] | (!dominated[0] & advantage[1]),
		]
	}

	fn has_neighbor(&self, map: &Map, triangle: usize, node: usize) -> bool {
		nodes_of(map.corners(triangle)).any(|corner| self.connected(corner, node))
	}

	fn can_use(
		&self,
		map: &Map,
		remaining: &Remaining,
		player: usize,
		triangle: usize,
		cost: i32,
	) -> bool {
		self.owner[player].has(triangle)
			&& self.initial_owner[player].has(triangle)
			&& nodes_of(map.corners(triangle)).all(|c| remaining[c][player] >= cost)
	}

	fn use_triangle(
		&mut self,
		map: &Map,
		remaining: &mut Remaining,
		player: usize,
		triangle: usize,
		cost: i32,
	) {
		for corner in nodes_of(map.corners(triangle)) {
			let taken = cost.min(self.units[corner][player]);
			self.units[corner][player] -= taken;
			remaining[corner][player] -= taken;
		}
		self.can_capture[player].remove(triangle);
		self.owner[player].remove(triangle);
	}

	fn is_well_formed(&self, map: &Map, command: Command) -> bool {
		let node = |n: usize| n < map.len();
		let live = |t: usize| t < map.triangles.len() && self.live.has(t);
		let corner = |t: usize, n: usize| node(n) && map.corners(t) & bit(n) != 0;
		match command {
			Command::Move { from, to, amount } => node(from) && node(to) && amount > 0,
			Command::Spawn { triangle, node } => live(triangle) && corner(triangle, node),
			Command::Attack { triangle, target } => live(triangle) && node(target),
			Command::AddPath { triangle, from, to } => {
				live(triangle) && corner(triangle, from) && node(to) && from != to
			}
			Command::RemovePath { triangle, from, to } => {
				live(triangle) && corner(triangle, from) && node(to)
			}
		}
	}

	fn can_apply(&self, map: &Map, remaining: &Remaining, player: usize, command: Command) -> bool {
		match command {
			Command::Move { from, to, .. } => from != to && remaining[from][player] > 0,
			Command::Spawn { triangle, .. } => self.can_use(map, remaining, player, triangle, 0),
			Command::Attack { triangle, target } => {
				self.can_use(map, remaining, player, triangle, 1)
					&& self.has_neighbor(map, triangle, target)
			}
			Command::AddPath { triangle, from, to } => {
				self.can_use(map, remaining, player, triangle, 1)
					&& !self.connected(from, to)
					&& map.can_connect(&self.edges, from, to)
			}
			Command::RemovePath { triangle, from, to } => {
				self.can_use(map, remaining, player, triangle, 1)
					&& self.connected(from, to)
					&& map.corners(triangle) & bit(to) == 0
			}
		}
	}

	fn apply(&mut self, map: &Map, remaining: &mut Remaining, player: usize, command: Command) {
		match command {
			Command::Move { from, to, amount } => {
				let amount = amount.min(remaining[from][player]);
				if amount <= 0 {
					return;
				}
				if let Some(next) = self.first_step(from, to) {
					self.units[from][player] -= amount;
					remaining[from][player] -= amount;
					self.units[next][player] += amount;
				}
			}
			Command::Spawn { triangle, node } => {
				self.use_triangle(map, remaining, player, triangle, 0);
				self.units[node][player] += 1;
			}
			Command::Attack { triangle, target } => {
				self.use_triangle(map, remaining, player, triangle, 1);
				let opponent = 1 - player;
				self.units[target][opponent] = 0;
				remaining[target][opponent] = 0;
			}
			Command::AddPath { triangle, from, to } => {
				self.use_triangle(map, remaining, player, triangle, 1);
				self.connect(map, from, to);
			}
			Command::RemovePath { triangle, from, to } => {
				self.use_triangle(map, remaining, player, triangle, 1);
				self.disconnect(map, from, to);
			}
		}
	}

	fn resolve_all_at_once(&mut self, map: &Map, remaining: &mut Remaining, queues: Queues) {
		for (player, queue) in queues.iter().enumerate() {
			for &command in *queue {
				if self.can_apply(map, remaining, player, command) {
					self.apply(map, remaining, player, command);
				}
			}
		}
	}

	fn resolve_one_per_frame(&mut self, map: &Map, remaining: &mut Remaining, queues: Queues) {
		let mut cursors = [0; 2];
		loop {
			let picked: [Option<Command>; 2] = std::array::from_fn(|player| {
				while let Some(&command) = queues[player].get(cursors[player]) {
					cursors[player] += 1;
					if self.can_apply(map, remaining, player, command) {
						return Some(command);
					}
				}
				None
			});
			if picked == [None, None] {
				return;
			}
			let second_goes_first = match picked {
				[Some(first), Some(second)] => cost(map, second) < cost(map, first),
				_ => false,
			};
			let order = if second_goes_first { [1, 0] } else { [0, 1] };
			for player in order {
				let Some(command) = picked[player] else {
					continue;
				};
				let still_valid = match command {
					Command::AddPath { from, to, .. } => map.can_connect(&self.edges, from, to),
					_ => true,
				};
				if still_valid {
					self.apply(map, remaining, player, command);
				}
			}
		}
	}

	pub fn step(&mut self, map: &Map, commands: [&[Command]; 2]) {
		let mut remaining: Remaining = self.units;
		let mut accepted: [Vec<Command>; 2] = std::array::from_fn(|player| {
			commands[player]
				.iter()
				.copied()
				.filter(|&c| {
					self.is_well_formed(map, c) && self.can_apply(map, &remaining, player, c)
				})
				.collect()
		});
		for queue in &mut accepted {
			queue.sort_by_key(|&c| (Phase::of(c) as u8, cost(map, c)));
		}
		let mut starts = [0; 2];
		for phase in Phase::ORDER {
			let queues: Queues = std::array::from_fn(|player| {
				let queue = &accepted[player][starts[player]..];
				let length = queue.iter().take_while(|&&c| Phase::of(c) == phase).count();
				starts[player] += length;
				&queue[..length]
			});
			if phase.one_per_frame() {
				self.resolve_one_per_frame(map, &mut remaining, queues);
			} else {
				self.resolve_all_at_once(map, &mut remaining, queues);
			}
		}
		self.finalize(map);
	}

	fn refresh_capture_rights(&mut self, map: &Map) {
		let occupied = [0, 1].map(|p| self.occupied_by(p));
		self.refresh_capture_rights_with(map, occupied);
	}

	fn refresh_capture_rights_with(&mut self, map: &Map, occupied: [u64; 2]) {
		for t in self.live.iter() {
			let owner = (0..2).find(|&p| self.owner[p].has(t));
			for (player, &occupied) in occupied.iter().enumerate() {
				let allowed = match owner {
					Some(owner) => owner != player,
					None => self.can_capture[player].has(t) || map.corners(t) & occupied == 0,
				};
				self.can_capture[player].assign(t, allowed);
			}
		}
	}

	fn kill_surrounded(&mut self) -> bool {
		let advantage = [0, 1].map(|p| self.advantage(p));
		let mut killed = false;
		for player in 0..2 {
			let doomed = self.dominated_by(advantage[1 - player]) & self.occupied_by(player);
			for node in nodes_of(doomed) {
				self.units[node][player] = 0;
				killed = true;
			}
		}
		killed
	}

	fn capture_triangles(&mut self, map: &Map, owned: [u64; 2]) -> bool {
		let mut changed = false;
		for t in self.live.iter() {
			for (player, &owned) in owned.iter().enumerate() {
				if self.can_capture[player].has(t)
					&& map.corners(t) & !owned == 0
					&& !self.owner[player].has(t)
				{
					self.owner[player].insert(t);
					self.owner[1 - player].remove(t);
					changed = true;
				}
			}
		}
		changed
	}

	fn finalize(&mut self, map: &Map) {
		self.refresh_capture_rights(map);
		if self.kill_surrounded() {
			self.refresh_capture_rights(map);
		}
		let occupied = [0, 1].map(|p| self.occupied_by(p));
		let owned = self.node_owners();
		while self.capture_triangles(map, owned) {
			self.refresh_capture_rights_with(map, occupied);
		}
		let dead = map.all_triangles & !self.live;
		for player in 0..2 {
			self.owner[player] &= self.live;
			self.score[player] += self.owner[player].len() as i32;
			self.initial_owner[player] = self.owner[player];
			self.can_capture[player] |= dead;
		}
	}

	pub fn eliminated(&self) -> [bool; 2] {
		let mut score = self.score;
		let mut out = [false; 2];
		let triangle_count = self.live.len() as f64;
		for player in 0..2 {
			let opponent = 1 - player;
			let units = self.units_of(player);
			let triangles = self.triangles_of(player);
			let stranded = units == 0 && triangles == 0;
			let hopeless = units == 1 && triangles == 0 && score[player] < score[opponent];
			let dominated = self.triangles_of(opponent) as f64 >= triangle_count * DOMINATION_SHARE;
			if stranded || hopeless || dominated {
				out[player] = true;
				score[player] = -1;
			}
		}
		out
	}
}

type Remaining = [[i32; 2]; MAX_NODES];
type Queues<'a> = [&'a [Command]; 2];

fn cost(map: &Map, command: Command) -> i32 {
	match command {
		Command::AddPath { from, to, .. } => map.points[from].dist2(map.points[to]),
		_ => 0,
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
	Attack,
	Move,
	Spawn,
	AddPath,
	RemovePath,
}

impl Phase {
	const ORDER: [Phase; 5] = [
		Phase::Attack,
		Phase::Move,
		Phase::Spawn,
		Phase::AddPath,
		Phase::RemovePath,
	];

	fn of(command: Command) -> Phase {
		match command {
			Command::Attack { .. } => Phase::Attack,
			Command::Move { .. } => Phase::Move,
			Command::Spawn { .. } => Phase::Spawn,
			Command::AddPath { .. } => Phase::AddPath,
			Command::RemovePath { .. } => Phase::RemovePath,
		}
	}

	fn one_per_frame(self) -> bool {
		matches!(self, Phase::Attack | Phase::AddPath | Phase::RemovePath)
	}
}
