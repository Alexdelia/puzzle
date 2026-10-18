use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fmt::Display;
use std::io::{self, Read, Write};
use std::time::Instant;

const MAX_TURNS: i32 = 100;
const PAINT_PER_TURN: u32 = 3;
const INSTABILITY_THRESHOLD: u8 = 4;

const NO_TRACK: i8 = -1;
const NEUTRAL_TRACK: i8 = 2;

type Cell = u16;
type TownId = u8;

const NO_CELL: Cell = Cell::MAX;
const NO_ROUTE: u16 = u16::MAX;
const NO_REGION: usize = usize::MAX;

const EMPTY: u8 = 0;
const MINE: u8 = 1;
const FOE: u8 = 2;
const NEUTRAL: u8 = 3;
const VIRT: u8 = 4;

const PAINT_UNIT: u32 = 1024;
const STEP_UNIT: u32 = 4;

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
	desired: Vec<TownId>,
	first_link: usize,
}

struct Map {
	me: u8,
	width: u8,
	cells: usize,
	neigh: Vec<[Cell; 4]>,
	cost: Vec<u8>,
	is_town: Vec<bool>,
	region: Vec<u8>,
	towns: Vec<Town>,
	requesters: Vec<TownId>,
	links: Vec<(TownId, TownId)>,
	region_cells: Vec<Vec<Cell>>,
	region_has_town: Vec<bool>,
	salt: u32,
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
		let mut links = Vec::new();
		let mut requesters = Vec::new();
		for id in 0..town_count {
			let _given = input.usize();
			let x = input.u8();
			let y = input.u8();
			let mut desired = Vec::new();
			input.town_list(&mut desired);
			let at = y as usize * w + x as usize;
			is_town[at] = true;
			region_has_town[region[at] as usize] = true;
			let first_link = links.len();
			for &to in &desired {
				links.push((id as TownId, to));
			}
			if !desired.is_empty() {
				requesters.push(id as TownId);
			}
			towns.push(Town {
				at: at as Cell,
				desired,
				first_link,
			});
		}

		Map {
			me,
			width,
			cells,
			neigh,
			cost,
			is_town,
			region,
			towns,
			requesters,
			links,
			region_cells,
			region_has_town,
			salt: (me as u32 + 1).wrapping_mul(0x9E37_79B9),
		}
	}

	fn point(&self, at: Cell) -> (u8, u8) {
		let w = self.width as usize;
		((at as usize % w) as u8, (at as usize / w) as u8)
	}

	fn noise(&self, at: usize) -> u32 {
		((at as u32) ^ self.salt).wrapping_mul(2654435761) >> 30
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

#[derive(Clone)]
struct Tune {
	horizon: usize,
	decay: f64,
	foe_mode: u8,
	me_mode: u8,
	cand_cells: usize,
	cand_gates: usize,
	cand_inks: usize,
	think_ms: u128,
	rolls: u32,
	avoid_inst: u8,
	crude_base: f64,
	crude_self: f64,
	precharge: bool,
	foe_first: bool,
	sever: bool,
	end_value: f64,
	foe_toll: u32,
	neu_toll: u32,
	debug: i32,
	prefixes: bool,
	tail: usize,
	follow: bool,
	cand_steals: usize,
	ink_roll: bool,
	plan_penalty: f64,
	foe_plan_bonus: f64,
	commit: f64,
	predict: bool,
	closure: f64,
	survey: bool,
	paint_power: f64,
	weigh_life: f64,
	feats: String,
	policy: bool,
}

fn knob<T: std::str::FromStr>(name: &str, fallback: T) -> T {
	std::env::var(name)
		.ok()
		.and_then(|text| text.parse().ok())
		.unwrap_or(fallback)
}

impl Tune {
	fn load() -> Tune {
		Tune {
			horizon: knob("BTK_HORIZON", 12),
			decay: knob("BTK_DECAY", 0.5),
			foe_mode: knob("BTK_FOE_MODE", 3),
			me_mode: knob("BTK_ME_MODE", 3),
			cand_cells: knob("BTK_CAND_CELLS", 24),
			cand_gates: knob("BTK_CAND_GATES", 4),
			cand_inks: knob("BTK_CAND_INKS", 6),
			think_ms: knob("BTK_THINK_MS", 35),
			rolls: knob("BTK_ROLLS", 0),
			avoid_inst: knob("BTK_AVOID_INST", 3),
			crude_base: knob("BTK_CRUDE_BASE", 0.25),
			crude_self: knob("BTK_CRUDE_SELF", 2.0),
			precharge: knob::<u8>("BTK_PRECHARGE", 1) != 0,
			foe_first: knob::<u8>("BTK_FOE_FIRST", 1) != 0,
			sever: knob::<u8>("BTK_SEVER", 1) != 0,
			end_value: knob("BTK_END_VALUE", 500.0),
			foe_toll: knob("BTK_FOE_TOLL", 2),
			neu_toll: knob("BTK_NEU_TOLL", 1),
			debug: knob("BTK_DEBUG", 0),
			prefixes: knob::<u8>("BTK_PREFIXES", 1) != 0,
			tail: knob("BTK_TAIL", 4),
			follow: knob::<u8>("BTK_FOLLOW", 1) != 0,
			cand_steals: knob("BTK_CAND_STEALS", 6),
			ink_roll: knob::<u8>("BTK_INK_ROLL", 0) != 0,
			plan_penalty: knob("BTK_PLAN_PENALTY", 1.0),
			foe_plan_bonus: knob("BTK_FOE_PLAN_BONUS", 0.5),
			commit: knob("BTK_COMMIT", 0.5),
			predict: knob::<u8>("BTK_PREDICT", 0) != 0,
			closure: knob("BTK_CLOSURE", 0.0),
			survey: knob::<u8>("BTK_SURVEY", 1) != 0,
			paint_power: knob("BTK_PAINT_POWER", 1.5),
			weigh_life: knob("BTK_WEIGH_LIFE", 1.5),
			feats: knob("BTK_FEATS", String::new()),
			policy: knob::<u8>("BTK_POLICY", 0) != 0,
		}
	}
}

struct Flow {
	parent: Vec<Cell>,
	seen: Vec<u32>,
	epoch: u32,
	queue: Vec<Cell>,
	mult: Vec<u16>,
	connected: Vec<bool>,
	mine: i32,
	foe: i32,
}

impl Flow {
	fn new(map: &Map) -> Flow {
		Flow {
			parent: vec![NO_CELL; map.cells],
			seen: vec![0; map.cells],
			epoch: 0,
			queue: Vec::with_capacity(map.cells),
			mult: vec![0; map.cells],
			connected: vec![false; map.links.len()],
			mine: 0,
			foe: 0,
		}
	}

	fn run(&mut self, map: &Map, owner: &[u8]) {
		self.mult.fill(0);
		self.connected.fill(false);
		self.mine = 0;
		self.foe = 0;
		for &from in &map.requesters {
			let town = &map.towns[from as usize];
			self.epoch += 1;
			let epoch = self.epoch;
			self.queue.clear();
			self.queue.push(town.at);
			self.seen[town.at as usize] = epoch;
			self.parent[town.at as usize] = NO_CELL;
			let mut head = 0;
			while head < self.queue.len() {
				let at = self.queue[head];
				head += 1;
				for way in 0..4 {
					let next = map.neigh[at as usize][way];
					if next == NO_CELL {
						continue;
					}
					let cell = next as usize;
					if self.seen[cell] == epoch || !(map.is_town[cell] || owner[cell] != EMPTY) {
						continue;
					}
					self.seen[cell] = epoch;
					self.parent[cell] = at;
					self.queue.push(next);
				}
			}
			for (k, &to) in town.desired.iter().enumerate() {
				let goal = map.towns[to as usize].at;
				if self.seen[goal as usize] != epoch {
					continue;
				}
				self.connected[town.first_link + k] = true;
				let mut walk = goal;
				while walk != NO_CELL {
					let cell = walk as usize;
					self.mult[cell] += 1;
					match owner[cell] {
						MINE => self.mine += 1,
						FOE => self.foe += 1,
						_ => {}
					}
					walk = self.parent[cell];
				}
			}
		}
	}

	fn any_connected(&self) -> bool {
		self.connected.iter().any(|&done| done)
	}
}

struct Route {
	link: usize,
	cells: Vec<Cell>,
}

struct Plan {
	routes: Vec<Route>,
	route_of: Vec<u16>,
	mult: Vec<u16>,
	by_mult: Vec<Cell>,
}

struct Router {
	dist: Vec<u32>,
	seen: Vec<u32>,
	epoch: u32,
	came: Vec<Cell>,
	heap: BinaryHeap<Reverse<(u32, Cell)>>,
	route: Vec<Cell>,
	salt: u32,
}

impl Router {
	fn new(map: &Map) -> Router {
		Router {
			dist: vec![0; map.cells],
			seen: vec![0; map.cells],
			epoch: 0,
			came: vec![NO_CELL; map.cells],
			heap: BinaryHeap::with_capacity(map.cells),
			route: Vec::with_capacity(map.cells),
			salt: 0,
		}
	}

	fn noise(&self, at: usize) -> u32 {
		((at as u32) ^ self.salt).wrapping_mul(2654435761) >> 30
	}

	fn passable(map: &Map, inked: &[bool], inst: &[u8], tune: &Tune, cell: usize) -> bool {
		if map.is_town[cell] {
			return true;
		}
		let region = map.region[cell] as usize;
		!inked[region] && (map.region_has_town[region] || inst[region] < tune.avoid_inst)
	}

	#[allow(clippy::too_many_arguments)]
	fn plot(
		&mut self,
		map: &Map,
		inked: &[bool],
		inst: &[u8],
		tune: &Tune,
		net: &[u8],
		seat: u8,
		from: Cell,
		to: Cell,
	) -> Option<u32> {
		let opp = if seat == MINE { FOE } else { MINE };
		self.epoch += 1;
		let epoch = self.epoch;
		self.heap.clear();
		self.dist[from as usize] = 0;
		self.seen[from as usize] = epoch;
		self.came[from as usize] = NO_CELL;
		self.heap.push(Reverse((0, from)));
		while let Some(Reverse((reached, at))) = self.heap.pop() {
			let here = at as usize;
			if self.dist[here] != reached {
				continue;
			}
			if at == to {
				self.route.clear();
				let mut paint = 0;
				let mut walk = at;
				while walk != NO_CELL {
					let cell = walk as usize;
					if !map.is_town[cell] && net[cell] == EMPTY {
						self.route.push(walk);
						paint += map.cost[cell] as u32;
					}
					walk = self.came[cell];
				}
				self.route.reverse();
				return Some(paint);
			}
			for way in 0..4 {
				let next = map.neigh[here][way];
				if next == NO_CELL {
					continue;
				}
				let cell = next as usize;
				if !Router::passable(map, inked, inst, tune, cell) {
					continue;
				}
				let stride = if map.is_town[cell] {
					STEP_UNIT
				} else if net[cell] == EMPTY {
					map.cost[cell] as u32 * PAINT_UNIT + STEP_UNIT
				} else if net[cell] == opp {
					tune.foe_toll * PAINT_UNIT + STEP_UNIT
				} else if net[cell] == NEUTRAL {
					tune.neu_toll * PAINT_UNIT + STEP_UNIT
				} else {
					STEP_UNIT
				};
				let total = reached + stride + self.noise(cell);
				if self.seen[cell] == epoch && self.dist[cell] <= total {
					continue;
				}
				self.seen[cell] = epoch;
				self.dist[cell] = total;
				self.came[cell] = at;
				self.heap.push(Reverse((total, next)));
			}
		}
		None
	}
}

impl Plan {
	fn build(
		map: &Map,
		state: &State,
		tune: &Tune,
		router: &mut Router,
		flow: &mut Flow,
		owner: &[u8],
		seat: u8,
	) -> Plan {
		router.salt = (seat as u32 + 1).wrapping_mul(0x9E37_79B9) ^ map.salt;
		let mut net = owner.to_vec();
		let mut order: Vec<(u32, usize)> = Vec::with_capacity(map.links.len());
		for (index, &(from, to)) in map.links.iter().enumerate() {
			let start = map.towns[from as usize].at;
			let goal = map.towns[to as usize].at;
			if let Some(paint) = router.plot(
				map,
				&state.inked,
				&state.instability,
				tune,
				&net,
				seat,
				start,
				goal,
			) {
				order.push((paint, index));
			}
		}
		order.sort_unstable();

		let mut routes = Vec::new();
		let mut route_of = vec![NO_ROUTE; map.cells];
		let mut cells = Vec::new();
		for (_, index) in order {
			let (from, to) = map.links[index];
			let start = map.towns[from as usize].at;
			let goal = map.towns[to as usize].at;
			let Some(paint) = router.plot(
				map,
				&state.inked,
				&state.instability,
				tune,
				&net,
				seat,
				start,
				goal,
			) else {
				continue;
			};
			if paint == 0 {
				continue;
			}
			let id = routes.len() as u16;
			for &at in &router.route {
				net[at as usize] = VIRT;
				route_of[at as usize] = id;
				cells.push(at);
			}
			routes.push(Route {
				link: index,
				cells: router.route.clone(),
			});
		}

		flow.run(map, &net);
		let mult = flow.mult.clone();
		let mut by_mult = cells;
		by_mult.sort_by(|&a, &b| {
			let worth = |at: Cell| mult[at as usize] as f64 / map.cost[at as usize] as f64;
			worth(b)
				.total_cmp(&worth(a))
				.then(map.noise(a as usize).cmp(&map.noise(b as usize)))
				.then(a.cmp(&b))
		});
		Plan {
			routes,
			route_of,
			mult,
			by_mult,
		}
	}
}

struct Sim {
	owner: Vec<u8>,
	inst: Vec<u8>,
	inked: Vec<bool>,
	routes: [Vec<Vec<Cell>>; 2],
	links: [Vec<usize>; 2],
	net: Vec<u8>,
	link_done: Vec<bool>,
	mult: Vec<u16>,
	order: Vec<(u32, u16)>,
	flow: Flow,
	comp: Vec<u16>,
	stack: Vec<Cell>,
	rolls: u32,
	schedule: Vec<[Option<usize>; 2]>,
	follow: bool,
	recent: Vec<f64>,
	recent_abs: Vec<(i32, i32)>,
	pending: Vec<u8>,
	close_diff: f64,
}

struct Outcome {
	score: f64,
	fill: Vec<Cell>,
	ink: Option<usize>,
}

impl Sim {
	fn new(map: &Map) -> Sim {
		Sim {
			owner: vec![EMPTY; map.cells],
			inst: vec![0; map.region_cells.len()],
			inked: vec![false; map.region_cells.len()],
			routes: [Vec::new(), Vec::new()],
			links: [Vec::new(), Vec::new()],
			net: vec![EMPTY; map.cells],
			link_done: vec![false; map.links.len()],
			mult: vec![0; map.cells],
			order: Vec::new(),
			flow: Flow::new(map),
			comp: vec![0; map.cells],
			stack: Vec::with_capacity(map.cells),
			rolls: 0,
			schedule: Vec::new(),
			follow: false,
			recent: Vec::new(),
			recent_abs: Vec::new(),
			pending: vec![0; map.cells],
			close_diff: 0.0,
		}
	}

	fn links_left(&mut self, map: &Map, sealed: usize) -> usize {
		const NO_GROUP: u16 = u16::MAX;
		self.comp.fill(NO_GROUP);
		let mut group = 0u16;
		for seed in 0..map.cells {
			let blocked = |cell: usize, inked: &[bool]| {
				let region = map.region[cell] as usize;
				inked[region] || region == sealed
			};
			if self.comp[seed] != NO_GROUP || blocked(seed, &self.inked) {
				continue;
			}
			self.stack.clear();
			self.stack.push(seed as Cell);
			self.comp[seed] = group;
			while let Some(at) = self.stack.pop() {
				for way in 0..4 {
					let next = map.neigh[at as usize][way];
					if next == NO_CELL {
						continue;
					}
					let cell = next as usize;
					if self.comp[cell] != NO_GROUP || blocked(cell, &self.inked) {
						continue;
					}
					self.comp[cell] = group;
					self.stack.push(next);
				}
			}
			group += 1;
		}
		map.links
			.iter()
			.filter(|&&(from, to)| {
				let here = self.comp[map.towns[from as usize].at as usize];
				let there = self.comp[map.towns[to as usize].at as usize];
				here != NO_GROUP && here == there
			})
			.count()
	}

	fn claim(&mut self, at: Cell, who: u8) {
		self.owner[at as usize] = who;
	}

	fn replot(&mut self, map: &Map, tune: &Tune, router: &mut Router, side: usize, index: usize) {
		self.net.copy_from_slice(&self.owner);
		for (other, route) in self.routes[side].iter().enumerate() {
			if other == index {
				continue;
			}
			for &at in route {
				if self.net[at as usize] == EMPTY {
					self.net[at as usize] = VIRT;
				}
			}
		}
		let (from, to) = map.links[self.links[side][index]];
		let seat = if side == 0 { MINE } else { FOE };
		let found = router.plot(
			map,
			&self.inked,
			&self.inst,
			tune,
			&self.net,
			seat,
			map.towns[from as usize].at,
			map.towns[to as usize].at,
		);
		self.routes[side][index].clear();
		if found.is_some() {
			self.routes[side][index].extend_from_slice(&router.route);
		}
	}

	fn replot_through(
		&mut self,
		map: &Map,
		tune: &Tune,
		router: &mut Router,
		side: usize,
		hit: impl Fn(Cell) -> bool,
	) {
		for index in 0..self.routes[side].len() {
			if self.routes[side][index].iter().any(|&at| hit(at)) {
				self.replot(map, tune, router, side, index);
			}
		}
	}

	fn remaining(&self, map: &Map, side: usize, index: usize) -> u32 {
		self.routes[side][index]
			.iter()
			.filter(|&&at| self.owner[at as usize] == EMPTY)
			.map(|&at| map.cost[at as usize] as u32)
			.sum()
	}

	#[allow(clippy::too_many_arguments)]
	fn build(
		&mut self,
		map: &Map,
		plans: &[&Plan; 2],
		who: u8,
		mode: u8,
		mut paint: u32,
		turn: usize,
		record: Option<&mut Vec<Cell>>,
	) {
		let side = if who == MINE { 0 } else { 1 };
		let plan = plans[side];
		let mut record = record;
		let trunk_first = match mode {
			0 => false,
			1 => true,
			_ => turn % 2 == 1,
		};
		if trunk_first {
			for index in 0..plan.by_mult.len() {
				if paint == 0 {
					return;
				}
				let at = plan.by_mult[index];
				let cell = at as usize;
				let price = map.cost[cell] as u32;
				if self.owner[cell] != EMPTY
					|| self.inked[map.region[cell] as usize]
					|| price > paint
				{
					continue;
				}
				let route = plan.route_of[cell] as usize;
				if self.link_done[plan.routes[route].link] {
					continue;
				}
				self.claim(at, who);
				paint -= price;
				if let Some(list) = record.as_deref_mut() {
					list.push(at);
				}
			}
			return;
		}
		self.order.clear();
		for index in 0..self.routes[side].len() {
			if self.link_done[self.links[side][index]] {
				continue;
			}
			let left = self.remaining(map, side, index);
			if left == 0 {
				continue;
			}
			let key = if mode == 3 {
				let carried: u32 = self.routes[side][index]
					.iter()
					.map(|&at| plan.mult[at as usize] as u32 + 1)
					.sum();
				(left * 1024 / carried.max(1)).max(1)
			} else {
				left
			};
			self.order.push((key, index as u16));
		}
		self.order.sort_unstable();
		for slot in 0..self.order.len() {
			let route = self.order[slot].1 as usize;
			for index in 0..self.routes[side][route].len() {
				if paint == 0 {
					return;
				}
				let at = self.routes[side][route][index];
				let cell = at as usize;
				let price = map.cost[cell] as u32;
				if self.owner[cell] != EMPTY || price > paint {
					continue;
				}
				self.claim(at, who);
				paint -= price;
				if let Some(list) = record.as_deref_mut() {
					list.push(at);
				}
			}
		}
	}

	fn aim(&self, map: &Map, tune: &Tune, who: u8) -> Option<usize> {
		let opp = if who == MINE { FOE } else { MINE };
		let mut best: Option<(f64, usize)> = None;
		let mut idle: Option<(i32, usize)> = None;
		for region in 0..map.region_cells.len() {
			if map.region_has_town[region] || self.inked[region] {
				continue;
			}
			let mut value = 0.0;
			let mut own = 0;
			let mut theirs = 0;
			for &at in &map.region_cells[region] {
				let cell = at as usize;
				let weight = self.mult[cell] as f64 + tune.crude_base;
				if self.owner[cell] == opp {
					value += weight;
					theirs += 1;
				} else if self.owner[cell] == who {
					value -= weight * tune.crude_self;
					own += 1;
				}
			}
			let steps = INSTABILITY_THRESHOLD
				.saturating_sub(self.inst[region])
				.max(1) as f64;
			let score = value / steps;
			if value > 0.0 {
				if best.is_none_or(|(top, _)| score > top) {
					best = Some((score, region));
				}
			} else if tune.precharge
				&& own == 0 && theirs > 0
				&& idle.is_none_or(|(top, _)| theirs > top)
			{
				idle = Some((theirs, region));
			}
		}
		best.map(|(_, region)| region)
			.or(idle.map(|(_, region)| region))
	}

	fn close(&mut self, map: &Map) -> (f64, f64) {
		self.pending.fill(0);
		let mut paint = 0u32;
		for side in 0..2 {
			for (index, route) in self.routes[side].iter().enumerate() {
				if self.link_done[self.links[side][index]] {
					continue;
				}
				for &at in route {
					let cell = at as usize;
					if self.owner[cell] == EMPTY {
						if self.pending[cell] == 0 {
							paint += map.cost[cell] as u32;
						}
						self.pending[cell] |= 1 << side;
					}
				}
			}
		}
		self.net.copy_from_slice(&self.owner);
		for cell in 0..map.cells {
			self.net[cell] = match self.pending[cell] {
				1 => MINE,
				2 => FOE,
				3 => NEUTRAL,
				_ => self.net[cell],
			};
		}
		self.flow.run(map, &self.net);
		let diff = (self.flow.mine - self.flow.foe) as f64;
		(diff, (paint as f64 / (2.0 * PAINT_PER_TURN as f64)).ceil())
	}

	fn wash(&mut self, map: &Map, tune: &Tune, router: &mut Router) {
		for region in 0..map.region_cells.len() {
			if self.inked[region]
				|| map.region_has_town[region]
				|| self.inst[region] < INSTABILITY_THRESHOLD
			{
				continue;
			}
			self.inked[region] = true;
			for &at in &map.region_cells[region] {
				self.owner[at as usize] = EMPTY;
			}
			for side in 0..2 {
				self.replot_through(map, tune, router, side, |at| {
					map.region[at as usize] as usize == region
				});
			}
		}
	}

	#[allow(clippy::too_many_arguments)]
	fn rollout(
		&mut self,
		map: &Map,
		state: &State,
		tune: &Tune,
		router: &mut Router,
		plans: &[&Plan; 2],
		first: &[Cell],
		first_ink: Option<usize>,
		remaining: i32,
	) -> Outcome {
		self.rolls += 1;
		self.owner.copy_from_slice(&state.owner);
		self.inst.copy_from_slice(&state.instability);
		self.inked.copy_from_slice(&state.inked);
		self.mult.copy_from_slice(&state.mult);
		for (side, plan) in plans.iter().enumerate() {
			self.routes[side].clear();
			self.links[side].clear();
			for route in &plan.routes {
				self.routes[side].push(route.cells.clone());
				self.links[side].push(route.link);
			}
		}
		self.link_done.fill(false);

		let mut fill = Vec::new();
		let mut ink = None;
		let mut total = 0.0;
		self.recent.clear();
		self.recent_abs.clear();
		if !self.follow {
			self.schedule.clear();
		}
		let horizon = tune.horizon.min(remaining.max(1) as usize);
		for turn in 0..horizon {
			let mut my_paint = PAINT_PER_TURN;
			if turn == 0 {
				for &at in first {
					my_paint = my_paint.saturating_sub(map.cost[at as usize] as u32);
					self.claim(at, MINE);
				}
				if !first.is_empty() {
					self.replot_through(map, tune, router, 1, |at| first.contains(&at));
				}
			}
			let me_first = if turn == 0 {
				!tune.foe_first
			} else {
				turn % 2 == 1
			};
			let record = if turn == 0 { Some(&mut fill) } else { None };
			if me_first {
				self.build(map, plans, MINE, tune.me_mode, my_paint, turn, record);
				self.build(map, plans, FOE, tune.foe_mode, PAINT_PER_TURN, turn, None);
			} else {
				self.build(map, plans, FOE, tune.foe_mode, PAINT_PER_TURN, turn, None);
				self.build(map, plans, MINE, tune.me_mode, my_paint, turn, record);
			}

			let planned = if self.follow && turn < self.schedule.len() {
				self.schedule[turn]
			} else {
				[None, None]
			};
			let mut my_aim = match planned[0] {
				Some(region) if !self.inked[region] => Some(region),
				_ => self.aim(map, tune, MINE),
			};
			if let Some(region) = first_ink
				&& !self.inked[region]
			{
				my_aim = Some(region);
			}
			let foe_aim = match planned[1] {
				Some(region) if !self.inked[region] => Some(region),
				_ => self.aim(map, tune, FOE),
			};
			if !self.follow {
				self.schedule.push([my_aim, foe_aim]);
			}
			if turn == 0 {
				ink = my_aim;
			}
			if let Some(region) = my_aim {
				self.inst[region] += 1;
			}
			if let Some(region) = foe_aim {
				self.inst[region] += 1;
			}
			self.wash(map, tune, router);

			self.flow.run(map, &self.owner);
			self.link_done.copy_from_slice(&self.flow.connected);
			self.mult.copy_from_slice(&self.flow.mult);
			let diff = (self.flow.mine - self.flow.foe) as f64;
			total += diff;
			self.recent.push(diff);
			self.recent_abs.push((self.flow.mine, self.flow.foe));
			if !self.flow.any_connected() && self.links_left(map, NO_REGION) == 0 {
				let lead = (state.my_score - state.foe_score) as f64 + total;
				let verdict = if lead > 0.0 {
					tune.end_value
				} else {
					-tune.end_value
				};
				return Outcome {
					score: total + verdict,
					fill,
					ink,
				};
			}
		}
		let tail = (remaining - horizon as i32).max(0) as f64;
		let window = tune.tail.clamp(1, self.recent.len().max(1));
		let recent = &self.recent[self.recent.len() - window..];
		let steady = recent.iter().sum::<f64>() / window as f64;
		if tune.closure > 0.0 {
			let (close_diff, closing_turns) = self.close(map);
			self.close_diff = close_diff;
			let gap = tail.min(closing_turns);
			total += steady * gap * tune.decay;
			total += close_diff * (tail - gap) * tune.closure;
		} else {
			total += steady * tail * tune.decay;
		}
		Outcome {
			score: total,
			fill,
			ink,
		}
	}
}

const NF: usize = 24;
const POLICY: [f64; NF] = [
	-0.33881, -0.43617, 0.07390, -0.51738, 0.62307, 0.25367, 0.46347, 0.01920, 0.01190, 0.19175,
	0.03929, -0.19703, 3.28858, 0.02738, 0.48743, 1.32305, -6.08377, 1.71372, 0.00000, -0.02019,
	0.19532, 0.43308, -0.10303, -0.07213,
];

struct Featurizer {
	rect: Vec<u8>,
	aligned: Vec<u8>,
	town_degree: Vec<u8>,
	region_size: Vec<u16>,
	dist_town: Vec<u16>,
	dist_own: Vec<u16>,
	queue: Vec<Cell>,
	route_left: [Vec<u32>; 2],
	region_own: Vec<u16>,
	region_foe: Vec<u16>,
	base_diff: i32,
	base_pairs: i32,
	trial: Vec<u8>,
	height: usize,
}

impl Featurizer {
	fn new(map: &Map) -> Featurizer {
		let height = map.cells / map.width as usize;
		let width = map.width as usize;
		let mut rect = vec![0u8; map.cells];
		let mut aligned = vec![0u8; map.cells];
		for &(from, to) in &map.links {
			let (ax, ay) = map.point(map.towns[from as usize].at);
			let (bx, by) = map.point(map.towns[to as usize].at);
			let (x0, x1) = (ax.min(bx) as usize, ax.max(bx) as usize);
			let (y0, y1) = (ay.min(by) as usize, ay.max(by) as usize);
			for y in y0..=y1 {
				for x in x0..=x1 {
					rect[y * width + x] += 1;
					if x0 == x1 || y0 == y1 {
						aligned[y * width + x] += 1;
					}
				}
			}
		}
		let mut town_degree = vec![0u8; map.towns.len()];
		for &(from, to) in &map.links {
			town_degree[from as usize] += 1;
			town_degree[to as usize] += 1;
		}
		let mut region_size = vec![0u16; map.region_cells.len()];
		for (region, cells) in map.region_cells.iter().enumerate() {
			region_size[region] = cells.len() as u16;
		}
		let mut out = Featurizer {
			rect,
			aligned,
			town_degree,
			region_size,
			dist_town: vec![0; map.cells],
			dist_own: vec![0; map.cells],
			queue: Vec::with_capacity(map.cells),
			route_left: [Vec::new(), Vec::new()],
			region_own: vec![0; map.region_cells.len()],
			region_foe: vec![0; map.region_cells.len()],
			base_diff: 0,
			base_pairs: 0,
			trial: vec![EMPTY; map.cells],
			height,
		};
		let seeds: Vec<Cell> = map.towns.iter().map(|town| town.at).collect();
		out.dist_town = out.multi_bfs(map, &seeds, None);
		out
	}

	fn multi_bfs(&mut self, map: &Map, seeds: &[Cell], inked: Option<&[bool]>) -> Vec<u16> {
		let mut dist = vec![u16::MAX; map.cells];
		self.queue.clear();
		for &at in seeds {
			dist[at as usize] = 0;
			self.queue.push(at);
		}
		let mut head = 0;
		while head < self.queue.len() {
			let at = self.queue[head];
			head += 1;
			for way in 0..4 {
				let next = map.neigh[at as usize][way];
				if next == NO_CELL {
					continue;
				}
				let cell = next as usize;
				if dist[cell] != u16::MAX {
					continue;
				}
				if let Some(inked) = inked
					&& inked[map.region[cell] as usize]
				{
					continue;
				}
				dist[cell] = dist[at as usize] + 1;
				self.queue.push(next);
			}
		}
		dist
	}

	fn prepare(&mut self, map: &Map, state: &State, plans: &[&Plan; 2], flow: &mut Flow) {
		let seeds: Vec<Cell> = (0..map.cells)
			.filter(|&cell| state.owner[cell] == MINE)
			.map(|cell| cell as Cell)
			.collect();
		self.dist_own = self.multi_bfs(map, &seeds, Some(&state.inked));
		for (side, plan) in plans.iter().enumerate() {
			self.route_left[side].clear();
			for route in &plan.routes {
				self.route_left[side].push(
					route
						.cells
						.iter()
						.filter(|&&at| state.owner[at as usize] == EMPTY)
						.map(|&at| map.cost[at as usize] as u32)
						.sum(),
				);
			}
		}
		self.region_own.fill(0);
		self.region_foe.fill(0);
		for cell in 0..map.cells {
			let region = map.region[cell] as usize;
			match state.owner[cell] {
				MINE => self.region_own[region] += 1,
				FOE | NEUTRAL => self.region_foe[region] += 1,
				_ => {}
			}
		}
		flow.run(map, &state.owner);
		self.base_diff = flow.mine - flow.foe;
		self.base_pairs = flow.connected.iter().filter(|&&done| done).count() as i32;
		self.trial.copy_from_slice(&state.owner);
	}

	fn candidate(&self, map: &Map, state: &State, tune: &Tune, cell: usize) -> bool {
		!map.is_town[cell]
			&& state.owner[cell] == EMPTY
			&& self.trial[cell] == EMPTY
			&& Router::passable(map, &state.inked, &state.instability, tune, cell)
	}

	#[allow(clippy::too_many_arguments)]
	fn features(
		&mut self,
		map: &Map,
		state: &State,
		plans: &[&Plan; 2],
		flow: &mut Flow,
		turn: i32,
		cell: usize,
	) -> [f64; NF] {
		let region = map.region[cell] as usize;
		let mut own_adj = 0.0;
		let mut foe_adj = 0.0;
		let mut town_adj = 0.0;
		let mut degree_adj = 0.0;
		let mut live_adj = 0.0;
		let mut touching = false;
		for way in 0..4 {
			let next = map.neigh[cell][way];
			if next == NO_CELL {
				continue;
			}
			let other = next as usize;
			if map.is_town[other] {
				town_adj += 1.0;
				touching = true;
				if let Some(id) = map.towns.iter().position(|town| town.at == next) {
					degree_adj += self.town_degree[id] as f64;
				}
			}
			match self.trial[other] {
				MINE => {
					own_adj += 1.0;
					touching = true;
				}
				FOE | NEUTRAL => {
					foe_adj += 1.0;
					touching = true;
				}
				_ => {}
			}
			live_adj += state.mult[other] as f64;
		}
		let (delta, new_pairs) = if touching {
			self.trial[cell] = MINE;
			flow.run(map, &self.trial);
			self.trial[cell] = EMPTY;
			let pairs = flow.connected.iter().filter(|&&done| done).count() as i32;
			(
				(flow.mine - flow.foe - self.base_diff) as f64,
				(pairs - self.base_pairs) as f64,
			)
		} else {
			(0.0, 0.0)
		};
		let route_mine = plans[0].route_of[cell];
		let route_foe = plans[1].route_of[cell];
		let left_mine = if route_mine == NO_ROUTE {
			0.0
		} else {
			self.route_left[0][route_mine as usize] as f64
		};
		let (x, y) = map.point(cell as Cell);
		let edge = (x as usize)
			.min(y as usize)
			.min(map.width as usize - 1 - x as usize)
			.min(self.height - 1 - y as usize)
			.min(3) as f64;
		let dist_own = self.dist_own[cell].min(20) as f64 / 20.0;
		let dist_town = self.dist_town[cell].min(20) as f64 / 20.0;
		[
			1.0,
			map.cost[cell] as f64,
			if map.region_has_town[region] {
				1.0
			} else {
				0.0
			},
			state.instability[region] as f64,
			own_adj,
			foe_adj,
			town_adj,
			degree_adj,
			delta,
			new_pairs,
			self.rect[cell] as f64,
			self.aligned[cell] as f64,
			if route_mine == NO_ROUTE { 0.0 } else { 1.0 },
			plans[0].mult[cell] as f64,
			if route_foe == NO_ROUTE { 0.0 } else { 1.0 },
			1.0 / (1.0 + left_mine),
			dist_own,
			dist_town,
			turn as f64 / 100.0,
			live_adj,
			edge,
			self.region_size[region] as f64 / 20.0,
			self.region_foe[region] as f64,
			self.region_own[region] as f64,
		]
	}
}

fn score_policy(feat: &[f64; NF]) -> f64 {
	let mut total = 0.0;
	for index in 0..NF {
		total += POLICY[index] * feat[index];
	}
	total
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

struct Brain {
	router: Router,
	flow: Flow,
	sim: Sim,
	featurizer: Featurizer,
	dump: Option<std::fs::File>,
}

impl Brain {
	fn new(map: &Map) -> Brain {
		Brain {
			router: Router::new(map),
			flow: Flow::new(map),
			sim: Sim::new(map),
			featurizer: Featurizer::new(map),
			dump: None,
		}
	}

	fn out_of_time(&self, tune: &Tune, clock: &Instant) -> bool {
		clock.elapsed().as_millis() > tune.think_ms
			|| (tune.rolls > 0 && self.sim.rolls >= tune.rolls)
	}

	fn dump_features(
		&mut self,
		map: &Map,
		state: &State,
		tune: &Tune,
		plans: &[&Plan; 2],
		turn: i32,
	) {
		use std::io::Write as _;
		if self.dump.is_none() {
			self.dump = std::fs::OpenOptions::new()
				.create(true)
				.append(true)
				.open(&tune.feats)
				.ok();
		}
		let Some(file) = self.dump.as_mut() else {
			return;
		};
		self.featurizer.prepare(map, state, plans, &mut self.flow);
		let mut text = String::new();
		for cell in 0..map.cells {
			if !self.featurizer.candidate(map, state, tune, cell) {
				continue;
			}
			let feat = self
				.featurizer
				.features(map, state, plans, &mut self.flow, turn, cell);
			let (x, y) = map.point(cell as Cell);
			text.push_str(&format!("{turn}\t{x}\t{y}"));
			for value in feat {
				text.push_str(&format!("\t{value}"));
			}
			text.push('\n');
		}
		let _ = file.write_all(text.as_bytes());
	}

	fn policy(
		&mut self,
		map: &Map,
		state: &State,
		tune: &Tune,
		plans: &[&Plan; 2],
		turn: i32,
	) -> Vec<Cell> {
		self.featurizer.prepare(map, state, plans, &mut self.flow);
		let mut chosen: Vec<Cell> = Vec::new();
		let mut budget = PAINT_PER_TURN;
		let mut scored: Vec<(f64, Cell)> = Vec::new();
		for cell in 0..map.cells {
			if !self.featurizer.candidate(map, state, tune, cell) {
				continue;
			}
			let feat = self
				.featurizer
				.features(map, state, plans, &mut self.flow, turn, cell);
			scored.push((score_policy(&feat), cell as Cell));
		}
		while budget > 0 && !scored.is_empty() {
			scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
			let Some(&(_, at)) = scored
				.iter()
				.find(|&&(_, at)| map.cost[at as usize] as u32 <= budget)
			else {
				break;
			};
			budget -= map.cost[at as usize] as u32;
			chosen.push(at);
			self.featurizer.trial[at as usize] = MINE;
			self.flow.run(map, &self.featurizer.trial);
			self.featurizer.base_diff = self.flow.mine - self.flow.foe;
			self.featurizer.base_pairs =
				self.flow.connected.iter().filter(|&&done| done).count() as i32;
			scored.retain(|&(_, other)| other != at);
			let refresh = scored.len().min(40);
			for entry in scored.iter_mut().take(refresh) {
				let cell = entry.1 as usize;
				let feat = self
					.featurizer
					.features(map, state, plans, &mut self.flow, turn, cell);
				entry.0 = score_policy(&feat);
			}
		}
		chosen
	}

	fn survey(
		&mut self,
		map: &Map,
		state: &State,
		tune: &Tune,
		plan: &Plan,
		remaining: i32,
		debug: bool,
	) -> Vec<Cell> {
		let mut owner = state.owner.clone();
		let mut chosen: Vec<Cell> = Vec::new();
		let mut budget = PAINT_PER_TURN;
		let life = |cells: &[Cell]| -> f64 {
			let mut live = 0.0;
			for &at in cells {
				let region = map.region[at as usize] as usize;
				live += if map.region_has_town[region] {
					1.0
				} else {
					let left =
						INSTABILITY_THRESHOLD.saturating_sub(state.instability[region]) as f64;
					1.0 - tune.weigh_life * (1.0 - left / INSTABILITY_THRESHOLD as f64)
				};
			}
			(live / cells.len().max(1) as f64).max(0.05)
		};
		while budget > 0 {
			self.flow.run(map, &owner);
			let base = (self.flow.mine - self.flow.foe) as f64;
			let mut best: Option<(f64, Vec<Cell>)> = None;
			let mut consider = |flow: &mut Flow, cells: &[Cell], owner: &mut Vec<u8>| {
				let open: Vec<Cell> = cells
					.iter()
					.copied()
					.filter(|&at| owner[at as usize] == EMPTY)
					.collect();
				if open.is_empty() {
					return;
				}
				let paint: u32 = open.iter().map(|&at| map.cost[at as usize] as u32).sum();
				for &at in &open {
					owner[at as usize] = MINE;
				}
				flow.run(map, owner);
				for &at in &open {
					owner[at as usize] = EMPTY;
				}
				let gain = (flow.mine - flow.foe) as f64 - base;
				let turns = (remaining as f64 - paint as f64 / PAINT_PER_TURN as f64) * life(&open);
				if gain <= 0.0 || turns <= 0.0 {
					return;
				}
				let score = gain * turns / (paint as f64).powf(tune.paint_power);
				if best.as_ref().is_none_or(|(top, _)| score > *top) {
					best = Some((score, open));
				}
			};
			for route in &plan.routes {
				consider(&mut self.flow, &route.cells, &mut owner);
			}
			let singles: Vec<Cell> = (0..map.cells)
				.filter(|&cell| {
					!map.is_town[cell]
						&& owner[cell] == EMPTY
						&& Router::passable(map, &state.inked, &state.instability, tune, cell)
						&& (0..4).any(|way| {
							let next = map.neigh[cell][way];
							next != NO_CELL
								&& (map.is_town[next as usize] || owner[next as usize] != EMPTY)
						})
				})
				.map(|cell| cell as Cell)
				.collect();
			for at in singles {
				consider(&mut self.flow, &[at], &mut owner);
			}
			let Some((score, cells)) = best else { break };
			if debug {
				eprintln!("survey {score:.1} {} cells", cells.len());
			}
			let mut placed = false;
			for at in cells {
				let price = map.cost[at as usize] as u32;
				if price <= budget {
					budget -= price;
					owner[at as usize] = MINE;
					chosen.push(at);
					placed = true;
				} else {
					break;
				}
			}
			if !placed {
				break;
			}
		}
		chosen
	}

	fn steals(&mut self, map: &Map, state: &State, tune: &Tune) -> Vec<Cell> {
		if tune.cand_steals == 0 {
			return Vec::new();
		}
		self.sim.owner.copy_from_slice(&state.owner);
		self.flow.run(map, &state.owner);
		let base = self.flow.mine - self.flow.foe;
		let mut ranked: Vec<(f64, Cell)> = Vec::new();
		for cell in 0..map.cells {
			if map.is_town[cell]
				|| state.owner[cell] != EMPTY
				|| !Router::passable(map, &state.inked, &state.instability, tune, cell)
			{
				continue;
			}
			let beside_path = (0..4).any(|way| {
				let next = map.neigh[cell][way];
				next != NO_CELL && (map.is_town[next as usize] || state.mult[next as usize] > 0)
			});
			if !beside_path {
				continue;
			}
			self.sim.owner[cell] = MINE;
			self.flow.run(map, &self.sim.owner);
			self.sim.owner[cell] = EMPTY;
			let delta = (self.flow.mine - self.flow.foe - base) as f64;
			if delta > 0.0 {
				ranked.push((delta / map.cost[cell] as f64, cell as Cell));
			}
		}
		ranked.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
		ranked
			.iter()
			.take(tune.cand_steals)
			.map(|&(_, at)| at)
			.collect()
	}

	fn candidates(&mut self, map: &Map, state: &State, tune: &Tune, plan: &Plan) -> Vec<Vec<Cell>> {
		let mut found: Vec<Vec<Cell>> = vec![Vec::new()];
		for at in self.steals(map, state, tune) {
			found.push(vec![at]);
		}
		for &at in plan.by_mult.iter().take(tune.cand_cells) {
			found.push(vec![at]);
		}
		for route in &plan.routes {
			let paint: u32 = route
				.cells
				.iter()
				.map(|&at| map.cost[at as usize] as u32)
				.sum();
			if paint <= PAINT_PER_TURN {
				if route.cells.len() > 1 {
					found.push(route.cells.clone());
				}
				continue;
			}
			if !tune.prefixes {
				continue;
			}
			let mut prefix = Vec::new();
			let mut spent = 0;
			for &at in &route.cells {
				let price = map.cost[at as usize] as u32;
				if spent + price > PAINT_PER_TURN {
					break;
				}
				spent += price;
				prefix.push(at);
			}
			if prefix.len() > 1 {
				found.push(prefix);
			}
		}
		let mut gates: Vec<(f64, Cell)> = Vec::new();
		for town in &map.towns {
			let degree = town.desired.len()
				+ map
					.links
					.iter()
					.filter(|&&(_, to)| map.towns[to as usize].at == town.at)
					.count();
			if degree == 0 {
				continue;
			}
			for way in 0..4 {
				let next = map.neigh[town.at as usize][way];
				if next == NO_CELL {
					continue;
				}
				let cell = next as usize;
				if map.is_town[cell]
					|| state.owner[cell] != EMPTY
					|| plan.route_of[cell] != NO_ROUTE
					|| !Router::passable(map, &state.inked, &state.instability, tune, cell)
				{
					continue;
				}
				gates.push((degree as f64 / map.cost[cell] as f64, next));
			}
		}
		gates.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
		for &(_, at) in gates.iter().take(tune.cand_gates) {
			found.push(vec![at]);
		}
		found
	}

	fn decide(
		&mut self,
		map: &Map,
		state: &State,
		tune: &Tune,
		turn: i32,
		commands: &mut Vec<Command>,
	) {
		let clock = Instant::now();
		self.sim.rolls = 0;
		commands.clear();
		let remaining = (MAX_TURNS - turn + 1).max(1);
		let mine = Plan::build(
			map,
			state,
			tune,
			&mut self.router,
			&mut self.flow,
			&state.owner,
			MINE,
		);
		let theirs = Plan::build(
			map,
			state,
			tune,
			&mut self.router,
			&mut self.flow,
			&state.owner,
			FOE,
		);
		let plans = [&mine, &theirs];
		let debug = tune.debug == turn;
		if debug {
			for (side, plan) in plans.iter().enumerate() {
				eprintln!("plan side{side}: {} routes", plan.routes.len());
				for route in &plan.routes {
					let (from, to) = map.links[route.link];
					let cells: Vec<String> = route
						.cells
						.iter()
						.map(|&at| {
							let (x, y) = map.point(at);
							format!("({x},{y})m{}", plan.mult[at as usize])
						})
						.collect();
					eprintln!("  {}->{} {}", from, to, cells.join(" "));
				}
			}
		}

		if !tune.feats.is_empty() {
			self.dump_features(map, state, tune, &plans, turn);
		}
		let mut chosen: Vec<Cell> = Vec::new();
		let mut best: Option<Outcome> = None;
		self.sim.follow = false;
		if tune.policy {
			chosen = self.policy(map, state, tune, &plans, turn);
			let outcome = self.sim.rollout(
				map,
				state,
				tune,
				&mut self.router,
				&plans,
				&chosen,
				None,
				remaining,
			);
			self.sim.follow = tune.follow;
			best = Some(outcome);
		} else if tune.survey {
			chosen = self.survey(map, state, tune, &mine, remaining, debug);
			let outcome = self.sim.rollout(
				map,
				state,
				tune,
				&mut self.router,
				&plans,
				&chosen,
				None,
				remaining,
			);
			self.sim.follow = tune.follow;
			best = Some(outcome);
		}
		for candidate in self.candidates(map, state, tune, &mine) {
			if tune.survey {
				break;
			}
			if self.out_of_time(tune, &clock) && best.is_some() {
				break;
			}
			let outcome = self.sim.rollout(
				map,
				state,
				tune,
				&mut self.router,
				&plans,
				&candidate,
				None,
				remaining,
			);
			self.sim.follow = tune.follow;
			if debug {
				let cells: Vec<String> = candidate
					.iter()
					.chain(outcome.fill.iter())
					.map(|&at| {
						let (x, y) = map.point(at);
						format!("({x},{y})")
					})
					.collect();
				eprintln!(
					"cand {:>8.1} ink {:?} {} | {}",
					outcome.score,
					outcome.ink,
					candidate.len(),
					cells.join(" ")
				);
			}
			if best.as_ref().is_none_or(|top| outcome.score > top.score) {
				chosen = candidate;
				best = Some(outcome);
			}
		}
		let Some(mut best) = best else {
			return;
		};
		chosen.append(&mut best.fill);
		if tune.predict {
			let outcome = self.sim.rollout(
				map,
				state,
				tune,
				&mut self.router,
				&plans,
				&chosen,
				None,
				remaining,
			);
			let steps: Vec<String> = self
				.sim
				.recent
				.iter()
				.zip(&self.sim.recent_abs)
				.map(|(d, (m, f))| format!("{m}/{f}={d:.0}"))
				.collect();
			eprintln!(
				"predict turn {turn} lead {} score {:.0} close {:.0} diffs {}",
				state.my_score - state.foe_score,
				outcome.score,
				self.sim.close_diff,
				steps.join(" ")
			);
		}

		let fatal = self.fatal_regions(map, state, tune);
		let ahead = state.my_score > state.foe_score;
		let ranked = self.ink_candidates(map, state, tune, &plans, &chosen);
		if debug {
			for &(score, region) in ranked.iter().take(8) {
				eprintln!("proxy {region:>3} {score:>8.2}");
			}
		}
		let mut ink = ranked.first().map(|&(_, region)| region);
		if tune.ink_roll && ranked.len() > 1 {
			let mut ink_score = f64::NEG_INFINITY;
			for &(_, region) in ranked.iter().take(tune.cand_inks) {
				if self.out_of_time(tune, &clock) {
					break;
				}
				let outcome = self.sim.rollout(
					map,
					state,
					tune,
					&mut self.router,
					&plans,
					&chosen,
					Some(region),
					remaining,
				);
				if debug {
					eprintln!("ink {region:>3} {:>8.1}", outcome.score);
				}
				if outcome.score > ink_score {
					ink_score = outcome.score;
					ink = Some(region);
				}
			}
		}
		if ahead && !fatal.is_empty() {
			ink = Some(fatal[0]);
		} else if ink.is_some_and(|region| fatal.contains(&region)) {
			ink = None;
		}

		for &at in &chosen {
			let (x, y) = map.point(at);
			commands.push(Command::Place(x, y));
		}
		if let Some(region) = ink {
			commands.push(Command::Disrupt(region as u8));
		}
	}

	fn ink_candidates(
		&mut self,
		map: &Map,
		state: &State,
		tune: &Tune,
		plans: &[&Plan; 2],
		chosen: &[Cell],
	) -> Vec<(f64, usize)> {
		let mut ranked: Vec<(f64, usize)> = Vec::new();
		let mut idle: Vec<(f64, usize)> = Vec::new();
		for region in 0..map.region_cells.len() {
			if map.region_has_town[region] || state.inked[region] {
				continue;
			}
			let mut value = 0.0;
			let mut own = 0.0;
			let mut theirs = 0.0;
			let mut planned_me = 0.0;
			let mut planned_foe = 0.0;
			for &at in &map.region_cells[region] {
				let cell = at as usize;
				let weight = state.mult[cell] as f64 + tune.crude_base;
				if state.owner[cell] == MINE || chosen.contains(&at) {
					value -= weight * tune.crude_self;
					own += 1.0;
				} else if state.owner[cell] == FOE {
					value += weight;
					theirs += 1.0;
				} else if state.owner[cell] == EMPTY {
					if plans[0].route_of[cell] != NO_ROUTE {
						planned_me += 1.0;
					}
					if plans[1].route_of[cell] != NO_ROUTE {
						planned_foe += 1.0;
					}
				}
			}
			value -= tune.plan_penalty * planned_me;
			value += tune.foe_plan_bonus * planned_foe;
			let charged = state.instability[region] as f64;
			let steps = INSTABILITY_THRESHOLD
				.saturating_sub(state.instability[region])
				.max(1) as f64;
			let score = value / steps + tune.commit * charged;
			if value > 0.0 {
				ranked.push((score, region));
			} else if own == 0.0 && planned_me == 0.0 && theirs + planned_foe > 0.0 {
				idle.push((theirs + planned_foe + tune.commit * charged, region));
			}
		}
		ranked.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
		idle.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
		if ranked.is_empty() { idle } else { ranked }
	}

	fn fatal_regions(&mut self, map: &Map, state: &State, tune: &Tune) -> Vec<usize> {
		let mut fatal = Vec::new();
		if !tune.sever {
			return fatal;
		}
		self.sim.inked.copy_from_slice(&state.inked);
		for region in 0..map.region_cells.len() {
			if map.region_has_town[region]
				|| state.inked[region]
				|| state.instability[region] + 1 < INSTABILITY_THRESHOLD
			{
				continue;
			}
			if self.sim.links_left(map, region) == 0 {
				fatal.push(region);
			}
		}
		fatal
	}
}

fn main() {
	let mut input = Reader::new();
	let mut out = io::stdout().lock();
	let tune = Tune::load();
	let map = Map::read(&mut input);
	let mut state = State::new(&map);
	let mut brain = Brain::new(&map);
	let mut commands = Vec::with_capacity(5);
	let mut turn = 0;

	while state.read(&map, &mut input) {
		turn += 1;
		brain.decide(&map, &state, &tune, turn, &mut commands);
		send(&commands, &mut out).unwrap();
	}
}
