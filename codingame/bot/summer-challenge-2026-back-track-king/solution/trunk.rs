#![allow(dead_code)]

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fmt::Display;
use std::io::{self, Read, Write};
use std::time::Instant;

const MAX_TURNS: i32 = 100;
const PAINT_PER_TURN: u32 = 3;
const INSTABILITY_THRESHOLD: u8 = 4;
const THINK_BUDGET_MS: u128 = 35;

const NO_TRACK: i8 = -1;
const NEUTRAL_TRACK: i8 = 2;

type Cell = u16;
type TownId = u8;

const NO_CELL: Cell = Cell::MAX;

const EMPTY: u8 = 0;
const MINE: u8 = 1;
const FOE: u8 = 2;
const NEUTRAL: u8 = 3;

const CELL_WEIGHT: u32 = 2048;

const READ_BUFFER: usize = 1 << 16;
const END_OF_INPUT: u8 = 0;

struct Reader {
	input: io::StdinLock<'static>,
	buffer: Box<[u8]>,
	at: usize,
	filled: usize,
}

impl Reader {
	fn new() -> Reader {
		Reader {
			input: io::stdin().lock(),
			buffer: vec![0; READ_BUFFER].into_boxed_slice(),
			at: 0,
			filled: 0,
		}
	}

	fn peek(&mut self) -> u8 {
		while self.at == self.filled {
			match self.input.read(&mut self.buffer) {
				Ok(0) => return END_OF_INPUT,
				Ok(count) => {
					self.at = 0;
					self.filled = count;
				}
				Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
				Err(_) => return END_OF_INPUT,
			}
		}
		self.buffer[self.at]
	}

	fn bump(&mut self) {
		self.at += 1;
	}

	fn expect(&mut self, byte: u8) {
		debug_assert_eq!(self.peek(), byte);
		self.bump();
	}

	fn skip_blanks(&mut self) {
		while matches!(self.peek(), b' ' | b'\t' | b'\r' | b'\n') {
			self.bump();
		}
	}

	fn at_end(&mut self) -> bool {
		self.skip_blanks();
		self.peek() == END_OF_INPUT
	}

	fn digits(&mut self) -> u32 {
		let mut value = 0;
		while self.peek().is_ascii_digit() {
			value = value * 10 + (self.peek() - b'0') as u32;
			self.bump();
		}
		value
	}

	fn u8(&mut self) -> u8 {
		self.skip_blanks();
		self.digits() as u8
	}

	fn usize(&mut self) -> usize {
		self.skip_blanks();
		self.digits() as usize
	}

	fn bool(&mut self) -> bool {
		self.u8() != 0
	}

	fn i32(&mut self) -> i32 {
		self.skip_blanks();
		let negative = self.peek() == b'-';
		if negative {
			self.bump();
		}
		let value = self.digits() as i32;
		if negative { -value } else { value }
	}

	fn i8(&mut self) -> i8 {
		self.i32() as i8
	}

	fn town_list(&mut self, towns: &mut Vec<TownId>) {
		self.skip_blanks();
		if self.peek() == b'x' {
			self.bump();
			return;
		}
		loop {
			towns.push(self.digits() as TownId);
			if self.peek() != b',' {
				return;
			}
			self.bump();
		}
	}

	fn town_pairs(&mut self, mut pair: impl FnMut(TownId, TownId)) {
		self.skip_blanks();
		if self.peek() == b'x' {
			self.bump();
			return;
		}
		loop {
			let from = self.digits() as TownId;
			self.expect(b'-');
			pair(from, self.digits() as TownId);
			if self.peek() != b',' {
				return;
			}
			self.bump();
		}
	}
}

fn paint_cost(kind: u8) -> u8 {
	match kind {
		0 => 1,
		1 => 2,
		_ => 3,
	}
}

struct Town {
	at: Cell,
	x: u8,
	y: u8,
	desired: Vec<TownId>,
}

struct Map {
	me: u8,
	width: u8,
	height: u8,
	cells: usize,
	neigh: Vec<[Cell; 4]>,
	cost: Vec<u8>,
	is_town: Vec<bool>,
	region: Vec<u8>,
	towns: Vec<Town>,
	links: Vec<(TownId, TownId)>,
	region_cells: Vec<Vec<Cell>>,
	region_has_town: Vec<bool>,
}

impl Map {
	fn read(input: &mut Reader) -> Map {
		let me = input.u8();
		let width = input.u8();
		let height = input.u8();
		let w = width as usize;
		let cells = w * height as usize;

		let mut cost = vec![0u8; cells];
		let mut region = vec![0u8; cells];
		let mut region_count = 0usize;
		for slot in 0..cells {
			let id = input.u8();
			region[slot] = id;
			cost[slot] = paint_cost(input.u8());
			region_count = region_count.max(id as usize + 1);
		}

		let mut neigh = vec![[NO_CELL; 4]; cells];
		for y in 0..height as usize {
			for x in 0..w {
				let slot = y * w + x;
				neigh[slot] = [
					if y > 0 { (slot - w) as Cell } else { NO_CELL },
					if x + 1 < w {
						(slot + 1) as Cell
					} else {
						NO_CELL
					},
					if y + 1 < height as usize {
						(slot + w) as Cell
					} else {
						NO_CELL
					},
					if x > 0 { (slot - 1) as Cell } else { NO_CELL },
				];
			}
		}

		let mut region_cells = vec![Vec::new(); region_count];
		for slot in 0..cells {
			region_cells[region[slot] as usize].push(slot as Cell);
		}

		let town_count = input.usize();
		let mut towns = Vec::with_capacity(town_count);
		let mut is_town = vec![false; cells];
		let mut region_has_town = vec![false; region_count];
		for _ in 0..town_count {
			let _given = input.usize();
			let x = input.u8();
			let y = input.u8();
			let mut desired = Vec::new();
			input.town_list(&mut desired);
			let at = y as usize * w + x as usize;
			is_town[at] = true;
			region_has_town[region[at] as usize] = true;
			towns.push(Town {
				at: at as Cell,
				x,
				y,
				desired,
			});
		}

		let mut links = Vec::new();
		for (from, town) in towns.iter().enumerate() {
			for &to in &town.desired {
				links.push((from as TownId, to));
			}
		}

		Map {
			me,
			width,
			height,
			cells,
			neigh,
			cost,
			is_town,
			region,
			towns,
			links,
			region_cells,
			region_has_town,
		}
	}

	fn point(&self, at: Cell) -> (u8, u8) {
		let w = self.width as usize;
		((at as usize % w) as u8, (at as usize / w) as u8)
	}
}

struct State {
	my_score: i32,
	foe_score: i32,
	owner: Vec<u8>,
	mult: Vec<u16>,
	instability: Vec<u8>,
	inked: Vec<bool>,
}

impl State {
	fn new(map: &Map) -> State {
		State {
			my_score: 0,
			foe_score: 0,
			owner: vec![EMPTY; map.cells],
			mult: vec![0; map.cells],
			instability: vec![0; map.region_cells.len()],
			inked: vec![false; map.region_cells.len()],
		}
	}

	fn read(&mut self, map: &Map, input: &mut Reader) -> bool {
		if input.at_end() {
			return false;
		}
		self.my_score = input.i32();
		self.foe_score = input.i32();
		for slot in 0..map.cells {
			let track = input.i8();
			self.owner[slot] = match track {
				NO_TRACK => EMPTY,
				NEUTRAL_TRACK => NEUTRAL,
				owner if owner == map.me as i8 => MINE,
				_ => FOE,
			};
			let region = map.region[slot] as usize;
			self.instability[region] = input.u8();
			self.inked[region] = input.bool();
			let mut count = 0u16;
			input.town_pairs(|_, _| count += 1);
			self.mult[slot] = count;
		}
		true
	}
}

struct Engine {
	parent: Vec<Cell>,
	seen: Vec<u32>,
	epoch: u32,
	queue: Vec<Cell>,
	dist: Vec<u32>,
	dseen: Vec<u32>,
	depoch: u32,
	came: Vec<Cell>,
	heap: BinaryHeap<Reverse<(u32, Cell)>>,
	path: Vec<Cell>,
	route: Vec<Cell>,
	scratch: Vec<u8>,
}

impl Engine {
	fn new(map: &Map) -> Engine {
		Engine {
			parent: vec![NO_CELL; map.cells],
			seen: vec![0; map.cells],
			epoch: 0,
			queue: Vec::with_capacity(map.cells),
			dist: vec![0; map.cells],
			dseen: vec![0; map.cells],
			depoch: 0,
			came: vec![NO_CELL; map.cells],
			heap: BinaryHeap::with_capacity(map.cells),
			path: Vec::with_capacity(map.cells),
			route: Vec::with_capacity(map.cells),
			scratch: vec![EMPTY; map.cells],
		}
	}

	fn shortest(&mut self, map: &Map, owner: &[u8], from: Cell, to: Cell) -> bool {
		self.epoch += 1;
		let epoch = self.epoch;
		self.queue.clear();
		self.queue.push(from);
		self.seen[from as usize] = epoch;
		self.parent[from as usize] = NO_CELL;
		let mut head = 0;
		while head < self.queue.len() {
			let at = self.queue[head];
			head += 1;
			if at == to {
				self.path.clear();
				let mut walk = at;
				while walk != NO_CELL {
					self.path.push(walk);
					walk = self.parent[walk as usize];
				}
				return true;
			}
			for step in 0..4 {
				let next = map.neigh[at as usize][step];
				if next == NO_CELL {
					continue;
				}
				let slot = next as usize;
				if self.seen[slot] == epoch || !(map.is_town[slot] || owner[slot] != EMPTY) {
					continue;
				}
				self.seen[slot] = epoch;
				self.parent[slot] = at;
				self.queue.push(next);
			}
		}
		false
	}

	fn rates(&mut self, map: &Map, owner: &[u8]) -> i32 {
		let mut edge = 0;
		for index in 0..map.links.len() {
			let (from, to) = map.links[index];
			let start = map.towns[from as usize].at;
			let goal = map.towns[to as usize].at;
			if !self.shortest(map, owner, start, goal) {
				continue;
			}
			for &at in &self.path {
				match owner[at as usize] {
					MINE => edge += 1,
					FOE => edge -= 1,
					_ => {}
				}
			}
		}
		edge
	}

	fn weight(
		&self,
		map: &Map,
		owner: &[u8],
		inked: &[bool],
		slot: usize,
		avoid_foe: bool,
	) -> Option<u32> {
		if map.is_town[slot] {
			return Some(0);
		}
		match owner[slot] {
			EMPTY => {
				if inked[map.region[slot] as usize] {
					None
				} else {
					Some(map.cost[slot] as u32)
				}
			}
			FOE if avoid_foe => None,
			_ => Some(0),
		}
	}

	#[allow(clippy::too_many_arguments)]
	fn plot(
		&mut self,
		map: &Map,
		owner: &[u8],
		inked: &[bool],
		from: Cell,
		to: Cell,
		short_first: bool,
		avoid_foe: bool,
	) -> Option<u32> {
		self.depoch += 1;
		let epoch = self.depoch;
		self.heap.clear();
		self.dist[from as usize] = 0;
		self.dseen[from as usize] = epoch;
		self.came[from as usize] = NO_CELL;
		self.heap.push(Reverse((0, from)));

		while let Some(Reverse((reached, at))) = self.heap.pop() {
			let here = at as usize;
			if self.dseen[here] != epoch || self.dist[here] != reached {
				continue;
			}
			if at == to {
				self.route.clear();
				let mut paint = 0;
				let mut walk = at;
				while walk != NO_CELL {
					let slot = walk as usize;
					if !map.is_town[slot] && owner[slot] == EMPTY {
						self.route.push(walk);
						paint += map.cost[slot] as u32;
					}
					walk = self.came[slot];
				}
				self.route.reverse();
				return Some(paint);
			}
			for step in 0..4 {
				let next = map.neigh[here][step];
				if next == NO_CELL {
					continue;
				}
				let slot = next as usize;
				let Some(cost) = self.weight(map, owner, inked, slot, avoid_foe) else {
					continue;
				};
				let stride = if short_first {
					CELL_WEIGHT + cost
				} else {
					cost * CELL_WEIGHT + 1
				};
				let total = reached + stride;
				if self.dseen[slot] == epoch && self.dist[slot] <= total {
					continue;
				}
				self.dseen[slot] = epoch;
				self.dist[slot] = total;
				self.came[slot] = at;
				self.heap.push(Reverse((total, next)));
			}
		}
		None
	}
}

struct Choice {
	cells: Vec<Cell>,
	paint: u32,
}

fn best_move(
	map: &Map,
	inked: &[bool],
	engine: &mut Engine,
	owner: &[u8],
	remaining: f64,
	clock: &Instant,
) -> Option<Choice> {
	let base = engine.rates(map, owner);
	let mut best: Option<(f64, Vec<Cell>, u32)> = None;

	for index in 0..map.links.len() {
		if clock.elapsed().as_millis() > THINK_BUDGET_MS {
			break;
		}
		let (from, to) = map.links[index];
		let start = map.towns[from as usize].at;
		let goal = map.towns[to as usize].at;
		for mode in 0..3 {
			let (short_first, avoid_foe) = match mode {
				0 => (false, false),
				1 => (true, false),
				_ => (true, true),
			};
			let Some(paint) = engine.plot(map, owner, inked, start, goal, short_first, avoid_foe)
			else {
				continue;
			};
			if paint == 0 || engine.route.is_empty() {
				continue;
			}
			let cells = engine.route.clone();
			engine.scratch.copy_from_slice(owner);
			for &at in &cells {
				engine.scratch[at as usize] = MINE;
			}
			let after = {
				let scratch = std::mem::take(&mut engine.scratch);
				let value = engine.rates(map, &scratch);
				engine.scratch = scratch;
				value
			};
			let gain = (after - base) as f64;
			if gain <= 0.0 {
				continue;
			}
			let turns = remaining - paint as f64 / PAINT_PER_TURN as f64;
			if turns <= 0.0 {
				continue;
			}
			let score = gain * turns / paint as f64;
			if best.as_ref().is_none_or(|(top, _, _)| score > *top) {
				best = Some((score, cells, paint));
			}
		}
	}

	best.map(|(_, cells, paint)| Choice { cells, paint })
}

fn pick_disrupt(map: &Map, state: &State, planned: &[Cell]) -> Option<u8> {
	let mut best: Option<(f64, u8)> = None;
	for region in 0..map.region_cells.len() {
		if map.region_has_town[region] || state.inked[region] {
			continue;
		}
		let mut value = 0.0;
		for &at in &map.region_cells[region] {
			let weight = state.mult[at as usize] as f64 + 0.25;
			match state.owner[at as usize] {
				MINE => value -= weight,
				FOE => value += weight,
				_ => {}
			}
		}
		for &at in planned {
			if map.region[at as usize] as usize == region {
				value -= 1.0;
			}
		}
		if value <= 0.0 {
			continue;
		}
		let steps = INSTABILITY_THRESHOLD
			.saturating_sub(state.instability[region])
			.max(1) as f64;
		let score = value / steps;
		if best.as_ref().is_none_or(|(top, _)| score > *top) {
			best = Some((score, region as u8));
		}
	}
	best.map(|(_, region)| region)
}

enum Command {
	Place(u8, u8),
	Disrupt(u8),
	Wait,
}

impl Display for Command {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			Command::Place(x, y) => write!(f, "PLACE_TRACKS {x} {y}"),
			Command::Disrupt(region) => write!(f, "DISRUPT {region}"),
			Command::Wait => write!(f, "WAIT"),
		}
	}
}

fn send(commands: &[Command], out: &mut impl Write) -> io::Result<()> {
	let commands = if commands.is_empty() {
		&[Command::Wait][..]
	} else {
		commands
	};
	for (index, command) in commands.iter().enumerate() {
		if index > 0 {
			write!(out, ";")?;
		}
		write!(out, "{command}")?;
	}
	writeln!(out)?;
	out.flush()
}

fn decide(map: &Map, state: &State, engine: &mut Engine, turn: i32, commands: &mut Vec<Command>) {
	let clock = Instant::now();
	commands.clear();

	let remaining = (MAX_TURNS - turn + 1).max(1) as f64;
	let mut owner = state.owner.clone();
	let mut budget = PAINT_PER_TURN;
	let mut built: Vec<Cell> = Vec::new();
	let mut planned: Vec<Cell> = Vec::new();

	for _ in 0..4 {
		if budget == 0 || clock.elapsed().as_millis() > THINK_BUDGET_MS {
			break;
		}
		let Some(choice) = best_move(map, &state.inked, engine, &owner, remaining, &clock) else {
			break;
		};
		let mut whole = true;
		for at in choice.cells {
			let price = map.cost[at as usize] as u32;
			if price <= budget {
				budget -= price;
				owner[at as usize] = MINE;
				built.push(at);
			} else {
				whole = false;
				planned.push(at);
			}
		}
		if !whole {
			break;
		}
	}

	for at in &built {
		let (x, y) = map.point(*at);
		commands.push(Command::Place(x, y));
	}
	if let Some(region) = pick_disrupt(map, state, &planned) {
		commands.push(Command::Disrupt(region));
	}
}

fn main() {
	let mut input = Reader::new();
	let mut out = io::stdout().lock();
	let map = Map::read(&mut input);
	let mut state = State::new(&map);
	let mut engine = Engine::new(&map);
	let mut commands = Vec::with_capacity(5);
	let mut turn = 0;

	while state.read(&map, &mut input) {
		turn += 1;
		decide(&map, &state, &mut engine, turn, &mut commands);
		send(&commands, &mut out).unwrap();
	}
}
