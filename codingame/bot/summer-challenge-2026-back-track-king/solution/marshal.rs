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

const EMPTY: u8 = 0;
const MINE: u8 = 1;
const FOE: u8 = 2;
const VIRT: u8 = 3;
const NEUTRAL: u8 = 3;

const CELL_WEIGHT: u32 = 2048;
const REFEREE_MODE: u32 = 9;

const READ_BUFFER: usize = 1 << 16;
const END_OF_INPUT: u8 = 0;

const ECHO_INIT: bool = false;

struct Reader {
	input: io::StdinLock<'static>,
	buffer: Box<[u8]>,
	at: usize,
	filled: usize,
	echo: Vec<u8>,
	echoing: bool,
}

impl Reader {
	fn new() -> Reader {
		Reader {
			input: io::stdin().lock(),
			buffer: vec![0; READ_BUFFER].into_boxed_slice(),
			at: 0,
			filled: 0,
			echo: Vec::new(),
			echoing: ECHO_INIT,
		}
	}

	fn flush_echo(&mut self) {
		if !ECHO_INIT || !self.echoing {
			return;
		}
		self.echoing = false;
		eprint!("{}", String::from_utf8_lossy(&self.echo).trim_end());
		eprintln!();
		self.echo = Vec::new();
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
		if ECHO_INIT && self.echoing {
			self.echo.push(self.buffer[self.at]);
		}
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
	links: Vec<(TownId, TownId)>,
	region_cells: Vec<Vec<Cell>>,
	region_has_town: Vec<bool>,
	requesters: Vec<TownId>,
	salt: u32,
	aligned: Vec<u8>,
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
				desired,
				first_link: 0,
			});
		}

		let mut links = Vec::new();
		let mut requesters = Vec::new();
		for (from, town) in towns.iter_mut().enumerate() {
			town.first_link = links.len();
			for &to in &town.desired {
				links.push((from as TownId, to));
			}
			if !town.desired.is_empty() {
				requesters.push(from as TownId);
			}
		}

		let mut aligned = vec![0u8; cells];
		for &(from, to) in &links {
			let a = towns[from as usize].at as usize;
			let b = towns[to as usize].at as usize;
			let (ax, ay) = (a % w, a / w);
			let (bx, by) = (b % w, b / w);
			if ax == bx {
				for y in ay.min(by)..=ay.max(by) {
					aligned[y * w + ax] += 1;
				}
			} else if ay == by {
				for x in ax.min(bx)..=ax.max(bx) {
					aligned[ay * w + x] += 1;
				}
			}
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
			links,
			region_cells,
			region_has_town,
			requesters,
			salt: (me as u32 + 1).wrapping_mul(0x9E37_79B9),
			aligned,
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
	foe_seen: Vec<u8>,
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
			foe_seen: vec![0; map.region_cells.len()],
		}
	}

	fn read(&mut self, map: &Map, input: &mut Reader) -> bool {
		if input.at_end() {
			return false;
		}
		self.my_score = input.i32();
		self.foe_score = input.i32();
		self.foe_seen.fill(0);
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
			if self.owner[slot] == FOE {
				self.foe_seen[region] = self.foe_seen[region].saturating_add(1);
			}
			let mut count = 0u16;
			input.town_pairs(|_, _| count += 1);
			self.mult[slot] = count;
		}
		true
	}
}

const UNREACHED: u16 = u16::MAX;
const FEATS: usize = 11;
const LIN: [f64; FEATS] = [
	-227.37703, -6.47221, 10.08946, 20.81373, 14.01935, -56.12361, 520.84578, -2.15219, -18.25436,
	31.51967, 30.01429,
];

fn features(map: &Map, state: &State, mult: &[u16], remaining: f64, at: usize) -> [f64; FEATS] {
	let region = map.region[at] as usize;
	let town_region = if map.region_has_town[region] {
		1.0
	} else {
		0.0
	};
	let fmult = mult.get(at).copied().unwrap_or(0) as f64;
	let mut live = 0.0;
	let mut touching = 0.0;
	let mut foes = 0.0;
	for way in 0..4 {
		let next = map.neigh[at][way];
		if next == NO_CELL {
			continue;
		}
		let next = next as usize;
		live += state.mult[next] as f64;
		if map.is_town[next] || state.owner[next] == MINE {
			touching += 1.0;
		}
		if state.owner[next] == FOE || state.owner[next] == NEUTRAL {
			foes += 1.0;
		}
	}
	let horizon = remaining / 100.0;
	[
		1.0,
		fmult,
		live,
		map.cost[at] as f64,
		town_region,
		state.instability[region] as f64,
		horizon,
		touching,
		foes,
		fmult * horizon,
		fmult * town_region,
	]
}

fn learned(map: &Map, state: &State, mult: &[u16], remaining: f64, at: usize) -> f64 {
	let feat = features(map, state, mult, remaining, at);
	let mut worth = 0.0f64;
	for index in 0..FEATS {
		worth += LIN[index] * feat[index];
	}
	worth.max(0.0)
}

#[derive(Clone)]
struct Tune {
	risk_base: u32,
	risk_inst: u32,
	scenarios: usize,
	flip: bool,
	disrupt: u8,
	self_penalty: f64,
	plan_penalty: f64,
	modes: u32,
	fill: bool,
	fill_cap: usize,
	paint_power: f64,
	shelter: u32,
	crude_self: f64,
	crude_floor: f64,
	precharge: bool,
	late_ink: i32,
	sever_bonus: f64,
	sever: bool,
	commit_bonus: f64,
	anti_sever: bool,
	think_ms: u128,
	crude_mix: f64,
	swing_gain: f64,
	avoid_inst: u8,
	crude_base: f64,
	scen_floor: f64,
	scen_gain: f64,
	mirror_weight: f64,
	preempt: f64,
	foe_ms: u128,
	toll: u32,
	life_base: f64,
	life_slope: f64,
	pre: f64,
	pre_paint: u32,
	pre_order: u8,
	pre_safe: f64,
	gate: f64,
	fill_mult: f64,
	pre_risk: u8,
	pre_cap: u32,
	route_mult: u32,
	pre_turns: i32,
	ink_hope: f64,
	fill_shelter: f64,
	fill_gate: f64,
	fill_calm: f64,
	fill_cost: f64,
	foe_toll: u32,
	foe_len: u32,
	neu_toll: u32,
	own_len: u32,
	own_toll: u32,
	step: u32,
	trace_inst: u8,
	weigh_life: f64,
	lin: u8,
	plots: u32,
	exact: bool,
	ink_roll: usize,
	horizon: usize,
	decay: f64,
	tail: usize,
	foe_mode: u8,
	me_mode: u8,
	end_value: f64,
	sim_precharge: bool,
	sim_base: f64,
	sim_self: f64,
	foe_first: bool,
	follow: bool,
	closure: f64,
	pre_skip: i32,
	pre_run: bool,
	early_base: f64,
	early_turns: i32,
	align: f64,
	pre_foe: f64,
	pre_only_foe: bool,
	dedup: bool,
	poison: bool,
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
			risk_base: knob("BTK_RISK_BASE", 1),
			risk_inst: knob("BTK_RISK_INST", 4),
			scenarios: knob("BTK_SCENARIOS", 2),
			flip: knob::<u8>("BTK_FLIP", 0) != 0,
			disrupt: knob("BTK_DISRUPT", 3),
			self_penalty: knob("BTK_SELF_PENALTY", 1.0),
			plan_penalty: knob("BTK_PLAN_PENALTY", 1.0),
			modes: knob("BTK_MODES", 767),
			fill: knob::<u8>("BTK_FILL", 1) != 0,
			fill_cap: knob("BTK_FILL_CAP", 48),
			paint_power: knob("BTK_PAINT_POWER", 1.5),
			shelter: knob("BTK_SHELTER", 0),
			crude_self: knob("BTK_CRUDE_SELF", 2.0),
			crude_floor: knob("BTK_CRUDE_FLOOR", 0.0),
			precharge: knob::<u8>("BTK_PRECHARGE", 1) != 0,
			late_ink: knob("BTK_LATE_INK", 100),
			sever_bonus: knob("BTK_SEVER_BONUS", 4.0),
			sever: knob::<u8>("BTK_SEVER", 1) != 0,
			commit_bonus: knob("BTK_COMMIT_BONUS", 1.0),
			anti_sever: knob::<u8>("BTK_ANTI_SEVER", 0) != 0,
			think_ms: knob("BTK_THINK_MS", 40),
			crude_mix: knob("BTK_CRUDE_MIX", 2.0),
			swing_gain: knob("BTK_SWING", 1.0),
			avoid_inst: knob("BTK_AVOID_INST", 3),
			crude_base: knob("BTK_CRUDE_BASE", 0.25),
			scen_floor: knob("BTK_SCEN_FLOOR", 0.0),
			scen_gain: knob("BTK_SCEN_GAIN", 1.0),
			mirror_weight: knob("BTK_MIRROR_WEIGHT", 0.0),
			preempt: knob("BTK_PREEMPT", 0.0),
			foe_ms: knob("BTK_FOE_MS", 8),
			toll: knob("BTK_TOLL", 0),
			life_base: knob("BTK_LIFE_BASE", 0.0),
			life_slope: knob("BTK_LIFE_SLOPE", 3.0),
			pre: knob("BTK_PRE", 1.0),
			pre_paint: knob("BTK_PRE_PAINT", 12),
			pre_order: knob("BTK_PRE_ORDER", 0),
			pre_safe: knob("BTK_PRE_SAFE", 1.5),
			gate: knob("BTK_GATE", 0.0),
			fill_mult: knob("BTK_FILL_MULT", 0.5),
			pre_risk: knob("BTK_PRE_RISK", 0),
			pre_cap: knob("BTK_PRE_CAP", 0),
			route_mult: knob("BTK_ROUTE_MULT", 0),
			pre_turns: knob("BTK_PRE_TURNS", 0),
			ink_hope: knob("BTK_INK_HOPE", 0.0),
			fill_shelter: knob("BTK_FILL_SHELTER", 0.0),
			fill_gate: knob("BTK_FILL_GATE", 0.0),
			fill_calm: knob("BTK_FILL_CALM", 1.0),
			fill_cost: knob("BTK_FILL_COST", 1.0),
			foe_toll: knob("BTK_FOE_TOLL", 2),
			foe_len: knob("BTK_FOE_LEN", 0),
			neu_toll: knob("BTK_NEU_TOLL", 0),
			own_len: knob("BTK_OWN_LEN", 2),
			own_toll: knob("BTK_OWN_TOLL", 6),
			step: knob("BTK_STEP", 1),
			trace_inst: knob("BTK_TRACE_INST", 3),
			weigh_life: knob("BTK_WEIGH_LIFE", 1.5),
			lin: knob("BTK_LIN", 2),
			plots: knob("BTK_PLOTS", 250),
			exact: knob::<u8>("BTK_EXACT", 0) != 0,
			ink_roll: knob("BTK_INK_ROLL", 0),
			horizon: knob("BTK_HORIZON", 6),
			decay: knob("BTK_DECAY", 0.5),
			tail: knob("BTK_TAIL", 4),
			foe_mode: knob("BTK_FOE_MODE", 3),
			me_mode: knob("BTK_ME_MODE", 3),
			end_value: knob("BTK_END_VALUE", 500.0),
			sim_precharge: knob::<u8>("BTK_SIM_PRECHARGE", 1) != 0,
			sim_base: knob("BTK_SIM_BASE", 0.25),
			sim_self: knob("BTK_SIM_SELF", 2.0),
			foe_first: knob::<u8>("BTK_FOE_FIRST", 1) != 0,
			follow: knob::<u8>("BTK_FOLLOW", 1) != 0,
			closure: knob("BTK_CLOSURE", 0.0),
			pre_skip: knob("BTK_PRE_SKIP", 0),
			pre_run: knob::<u8>("BTK_PRE_RUN", 0) != 0,
			early_base: knob("BTK_EARLY_BASE", 0.25),
			early_turns: knob("BTK_EARLY_TURNS", 0),
			align: knob("BTK_ALIGN", 0.0),
			pre_foe: knob("BTK_PRE_FOE", 0.0),
			pre_only_foe: knob::<u8>("BTK_PRE_ONLY_FOE", 0) != 0,
			dedup: knob::<u8>("BTK_DEDUP", 0) != 0,
			poison: knob::<u8>("BTK_POISON", 0) != 0,
		}
	}
}

const NO_ROUTE: u16 = u16::MAX;
const NO_REGION: usize = usize::MAX;
const PAINT_UNIT: u32 = 1024;
const STEP_UNIT: u32 = 4;

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

	fn spread(&mut self, map: &Map, owner: &[u8], from: Cell) {
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
	}

	fn run(&mut self, map: &Map, owner: &[u8]) {
		self.mult.fill(0);
		self.connected.fill(false);
		self.mine = 0;
		self.foe = 0;
		for &from in &map.requesters {
			let town = &map.towns[from as usize];
			self.spread(map, owner, town.at);
			let epoch = self.epoch;
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

	fn trail(&mut self, map: &Map, owner: &[u8], from: TownId, to: TownId, out: &mut Vec<Cell>) {
		out.clear();
		self.spread(map, owner, map.towns[from as usize].at);
		let goal = map.towns[to as usize].at;
		if self.seen[goal as usize] != self.epoch {
			return;
		}
		let mut walk = goal;
		while walk != NO_CELL {
			out.push(walk);
			walk = self.parent[walk as usize];
		}
		out.reverse();
	}

	fn any_connected(&self) -> bool {
		self.connected.iter().any(|&done| done)
	}
}

struct Engine {
	dist: Vec<u16>,
	queue: Vec<Cell>,
	dests: Vec<u8>,
	slot_of_town: Vec<usize>,
	ranks: Vec<u32>,
	seen: Vec<u32>,
	epoch: u32,
	came: Vec<Cell>,
	heap: BinaryHeap<Reverse<(u32, Cell)>>,
	route: Vec<Cell>,
	trial: Vec<u8>,
	shade: Vec<u8>,
	clear: Vec<u8>,
	salt: u32,
	comp: Vec<u16>,
	stack: Vec<Cell>,
	mult: Vec<u16>,
	ride: bool,
	spent: u32,
	exact: bool,
	flow: Flow,
}

impl Engine {
	fn new(map: &Map) -> Engine {
		let mut dests: Vec<u8> = Vec::new();
		for &(_, to) in &map.links {
			if !dests.contains(&to) {
				dests.push(to);
			}
		}
		let mut slot_of_town = vec![usize::MAX; map.towns.len()];
		for (slot, &town) in dests.iter().enumerate() {
			slot_of_town[town as usize] = slot;
		}
		Engine {
			dist: vec![UNREACHED; dests.len() * map.cells],
			queue: Vec::with_capacity(map.cells),
			dests,
			slot_of_town,
			ranks: vec![0; map.cells],
			seen: vec![0; map.cells],
			epoch: 0,
			came: vec![NO_CELL; map.cells],
			heap: BinaryHeap::with_capacity(map.cells),
			route: Vec::with_capacity(map.cells),
			trial: vec![EMPTY; map.cells],
			shade: vec![EMPTY; map.cells],
			clear: vec![0; map.cells],
			salt: (map.me as u32).wrapping_mul(0x9E37_79B9),
			comp: vec![0; map.cells],
			stack: Vec::with_capacity(map.cells),
			mult: vec![0; map.cells],
			ride: false,
			spent: 0,
			exact: false,
			flow: Flow::new(map),
		}
	}

	fn spread(&mut self, map: &Map, owner: &[u8], slot: usize, source: Cell) {
		let base = slot * map.cells;
		let field = &mut self.dist[base..base + map.cells];
		field.fill(UNREACHED);
		field[source as usize] = 0;
		self.queue.clear();
		self.queue.push(source);
		let mut head = 0;
		while head < self.queue.len() {
			let at = self.queue[head];
			head += 1;
			let step = field[at as usize] + 1;
			for way in 0..4 {
				let next = map.neigh[at as usize][way];
				if next == NO_CELL {
					continue;
				}
				let cell = next as usize;
				if field[cell] != UNREACHED || !(map.is_town[cell] || owner[cell] != EMPTY) {
					continue;
				}
				field[cell] = step;
				self.queue.push(next);
			}
		}
	}

	fn edge(&mut self, map: &Map, owner: &[u8]) -> i32 {
		if self.exact {
			self.flow.run(map, owner);
			return self.flow.mine - self.flow.foe;
		}
		for slot in 0..self.dests.len() {
			let town = self.dests[slot] as usize;
			self.spread(map, owner, slot, map.towns[town].at);
		}
		let mut edge = 0;
		for &(from, to) in &map.links {
			let base = self.slot_of_town[to as usize] * map.cells;
			let goal = map.towns[to as usize].at;
			let mut at = map.towns[from as usize].at;
			let mut left = self.dist[base + at as usize];
			if left == UNREACHED {
				continue;
			}
			while at != goal {
				match owner[at as usize] {
					MINE => edge += 1,
					FOE => edge -= 1,
					_ => {}
				}
				let want = left - 1;
				let mut stepped = false;
				for way in 0..4 {
					let next = map.neigh[at as usize][way];
					if next != NO_CELL && self.dist[base + next as usize] == want {
						at = next;
						left = want;
						stepped = true;
						break;
					}
				}
				if !stepped {
					break;
				}
			}
		}
		edge
	}

	fn tally(&mut self, map: &Map, owner: &[u8], mult: &mut [u16]) {
		if self.exact {
			self.flow.run(map, owner);
			for (slot, count) in self.flow.mult.iter().enumerate() {
				mult[slot] += count;
			}
			return;
		}
		for slot in 0..self.dests.len() {
			let town = self.dests[slot] as usize;
			self.spread(map, owner, slot, map.towns[town].at);
		}
		for &(from, to) in &map.links {
			let base = self.slot_of_town[to as usize] * map.cells;
			let goal = map.towns[to as usize].at;
			let mut at = map.towns[from as usize].at;
			let mut left = self.dist[base + at as usize];
			if left == UNREACHED {
				continue;
			}
			while at != goal {
				mult[at as usize] += 1;
				let want = left - 1;
				let mut stepped = false;
				for way in 0..4 {
					let next = map.neigh[at as usize][way];
					if next != NO_CELL && self.dist[base + next as usize] == want {
						at = next;
						left = want;
						stepped = true;
						break;
					}
				}
				if !stepped {
					break;
				}
			}
		}
	}

	fn trail(&mut self, map: &Map, owner: &[u8], from: TownId, to: TownId, out: &mut Vec<Cell>) {
		if self.exact {
			self.flow.trail(map, owner, from, to, out);
			return;
		}
		out.clear();
		let slot = self.slot_of_town[to as usize];
		if slot == usize::MAX {
			return;
		}
		self.spread(map, owner, slot, map.towns[to as usize].at);
		let base = slot * map.cells;
		let goal = map.towns[to as usize].at;
		let mut at = map.towns[from as usize].at;
		let mut left = self.dist[base + at as usize];
		if left == UNREACHED {
			return;
		}
		loop {
			out.push(at);
			if at == goal {
				return;
			}
			let want = left - 1;
			let mut stepped = false;
			for way in 0..4 {
				let next = map.neigh[at as usize][way];
				if next != NO_CELL && self.dist[base + next as usize] == want {
					at = next;
					left = want;
					stepped = true;
					break;
				}
			}
			if !stepped {
				return;
			}
		}
	}

	fn links_left(&mut self, map: &Map, inked: &[bool], sealed: usize) -> usize {
		const NO_GROUP: u16 = u16::MAX;
		self.comp.fill(NO_GROUP);
		let mut group = 0u16;
		for seed in 0..map.cells {
			let blocked = |cell: usize| {
				let region = map.region[cell] as usize;
				inked[region] || region == sealed
			};
			if self.comp[seed] != NO_GROUP || blocked(seed) {
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
					if self.comp[cell] != NO_GROUP || blocked(cell) {
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

	fn trace_route(
		&mut self,
		map: &Map,
		state: &State,
		tune: &Tune,
		owner: &[u8],
		from: Cell,
		to: Cell,
	) -> Option<u32> {
		self.spent += 1;
		self.epoch += 1;
		let epoch = self.epoch;
		self.queue.clear();
		self.queue.push(from);
		self.seen[from as usize] = epoch;
		self.came[from as usize] = NO_CELL;
		let mut head = 0;
		let mut reached = false;
		while head < self.queue.len() {
			let at = self.queue[head];
			head += 1;
			if at == to {
				reached = true;
				break;
			}
			for way in 0..4 {
				let next = map.neigh[at as usize][way];
				if next == NO_CELL {
					continue;
				}
				let cell = next as usize;
				if self.seen[cell] == epoch {
					continue;
				}
				let region = map.region[cell] as usize;
				if !map.is_town[cell]
					&& owner[cell] == EMPTY
					&& (state.inked[region]
						|| (!map.region_has_town[region]
							&& state.instability[region] >= tune.trace_inst))
				{
					continue;
				}
				self.seen[cell] = epoch;
				self.came[cell] = at;
				self.queue.push(next);
			}
		}
		if !reached {
			return None;
		}
		self.route.clear();
		let mut paint = 0;
		let mut walk = to;
		while walk != NO_CELL {
			let cell = walk as usize;
			if !map.is_town[cell] && owner[cell] == EMPTY {
				self.route.push(walk);
				paint += map.cost[cell] as u32;
			}
			walk = self.came[cell];
		}
		self.route.reverse();
		Some(paint)
	}

	#[allow(clippy::too_many_arguments)]
	fn plot(
		&mut self,
		map: &Map,
		state: &State,
		tune: &Tune,
		owner: &[u8],
		from: Cell,
		to: Cell,
		short_first: bool,
		risk_aware: bool,
		avoid_foe: bool,
		drag: (u32, u32),
		banned: &[u8],
	) -> Option<u32> {
		self.spent += 1;
		self.epoch += 1;
		let epoch = self.epoch;
		self.heap.clear();
		let width = map.width as usize;
		let goal = (to as usize % width, to as usize / width);
		let pace = if short_first { 4 * CELL_WEIGHT } else { 4 };
		let guess = |cell: usize| -> u32 {
			let x = cell % width;
			let y = cell / width;
			(x.abs_diff(goal.0) + y.abs_diff(goal.1)) as u32 * pace
		};
		self.ranks[from as usize] = 0;
		self.seen[from as usize] = epoch;
		self.came[from as usize] = NO_CELL;
		self.heap.push(Reverse((guess(from as usize), from)));

		while let Some(Reverse((reached, at))) = self.heap.pop() {
			let here = at as usize;
			if self.seen[here] != epoch || self.ranks[here] + guess(here) != reached {
				continue;
			}
			let reached = self.ranks[here];
			if at == to {
				self.route.clear();
				let mut paint = 0;
				let mut walk = at;
				while walk != NO_CELL {
					let cell = walk as usize;
					if !map.is_town[cell] && owner[cell] == EMPTY {
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
				if banned[cell] != 0 && next != to {
					continue;
				}
				let region = map.region[cell] as usize;
				let toll = if map.is_town[cell] {
					0
				} else if owner[cell] != EMPTY {
					if avoid_foe && owner[cell] == FOE {
						continue;
					}
					match owner[cell] {
						FOE => drag.0,
						NEUTRAL => tune.neu_toll,
						_ => 0,
					}
				} else if state.inked[region]
					|| (!map.region_has_town[region]
						&& state.instability[region] >= tune.avoid_inst)
				{
					continue;
				} else {
					let mut price = map.cost[cell] as u32;
					if risk_aware && !map.region_has_town[region] {
						price += tune.risk_base
							+ tune.risk_inst * state.instability[region].min(3) as u32;
						price = price.saturating_sub(tune.shelter * state.foe_seen[region] as u32);
					}
					price.max(1)
				};
				let crossing = tune.toll > 0
					&& !map.region_has_town[region]
					&& map.region[here] != map.region[cell]
					&& !map.is_town[cell];
				let toll = toll + if crossing { tune.toll } else { 0 };
				let mut stride = if short_first {
					4 * (CELL_WEIGHT + toll)
				} else {
					4 * (toll * CELL_WEIGHT + tune.step)
				};
				if short_first && !map.is_town[cell] {
					let pull = match owner[cell] {
						FOE => drag.1,
						NEUTRAL => tune.neu_toll.min(drag.1),
						_ => 0,
					};
					stride += 4 * CELL_WEIGHT * pull;
				}
				if self.ride {
					let bonus = tune.route_mult * self.mult[cell] as u32;
					stride -= bonus.min(stride.saturating_sub(1));
				}
				let total =
					reached + stride + (((cell as u32) ^ self.salt).wrapping_mul(2654435761) >> 30);
				if self.seen[cell] == epoch && self.ranks[cell] <= total {
					continue;
				}
				self.seen[cell] = epoch;
				self.ranks[cell] = total;
				self.came[cell] = at;
				self.heap.push(Reverse((total + guess(cell), next)));
			}
		}
		None
	}
}

fn ink_odds(tune: &Tune, instability: u8) -> f64 {
	let raw = match instability {
		0 => 0.10,
		1 => 0.25,
		2 => 0.50,
		_ => 0.90,
	};
	(raw * tune.scen_gain).max(tune.scen_floor).min(1.0)
}

struct Outlook {
	weights: Vec<(Option<usize>, f64)>,
	span: f64,
}

fn foe_ink_value(map: &Map, state: &State, tune: &Tune, region: usize) -> f64 {
	let mut value = 0.0;
	for &at in &map.region_cells[region] {
		let weight = state.mult[at as usize] as f64 + tune.crude_base;
		match state.owner[at as usize] {
			MINE => value += weight,
			FOE => value -= weight * tune.crude_self,
			_ => {}
		}
	}
	value
}

impl Outlook {
	fn build(map: &Map, state: &State, tune: &Tune) -> Outlook {
		let mut threats: Vec<(f64, usize)> = Vec::new();
		if tune.scenarios > 0 {
			for region in 0..map.region_cells.len() {
				if map.region_has_town[region] || state.inked[region] {
					continue;
				}
				let held = map.region_cells[region]
					.iter()
					.filter(|&&at| state.owner[at as usize] == MINE)
					.count();
				if held == 0 {
					continue;
				}
				let steps = INSTABILITY_THRESHOLD
					.saturating_sub(state.instability[region])
					.max(1) as f64;
				let hunted = (foe_ink_value(map, state, tune, region) / steps).max(0.0);
				let odds = ink_odds(tune, state.instability[region]);
				threats.push((odds * held as f64 + tune.mirror_weight * hunted, region));
			}
			threats.sort_by(|a, b| b.0.total_cmp(&a.0));
			threats.truncate(tune.scenarios);
		}
		let mut weights = vec![(None, 1.0)];
		for (rank, (_, region)) in threats.iter().enumerate() {
			let odds = ink_odds(tune, state.instability[*region]);
			weights.push((Some(*region), odds * if rank == 0 { 1.0 } else { 0.6 }));
		}
		let span = weights.iter().map(|(_, weight)| weight).sum();
		Outlook { weights, span }
	}
}

fn appraise(map: &Map, engine: &mut Engine, owner: &[u8], outlook: &Outlook) -> f64 {
	let mut worth = 0.0;
	for &(region, weight) in &outlook.weights {
		let value = match region {
			None => engine.edge(map, owner) as f64,
			Some(region) => {
				engine.shade.copy_from_slice(owner);
				for &at in &map.region_cells[region] {
					engine.shade[at as usize] = EMPTY;
				}
				let shade = std::mem::take(&mut engine.shade);
				let value = engine.edge(map, &shade) as f64;
				engine.shade = shade;
				value
			}
		};
		worth += weight * value;
	}
	worth
}

struct Choice {
	cells: Vec<Cell>,
	score: f64,
}

fn route_shape(tune: &Tune, mode: u32) -> (bool, bool, bool, (u32, u32)) {
	match mode {
		0 => (false, true, false, (tune.foe_toll, tune.foe_len)),
		1 => (true, true, false, (tune.foe_toll, tune.foe_len)),
		2 => (true, false, true, (0, 0)),
		3 => (true, true, false, (tune.foe_toll, tune.foe_len)),
		4 => (true, false, false, (0, tune.own_len)),
		5 => (false, true, false, (tune.own_toll, 0)),
		6 => (true, false, false, (0, 0)),
		7 => (false, false, false, (tune.foe_toll * 2, 0)),
		_ => (true, true, true, (0, 0)),
	}
}

#[allow(clippy::too_many_arguments)]
fn survey(
	map: &Map,
	state: &State,
	tune: &Tune,
	engine: &mut Engine,
	owner: &[u8],
	field: &[u8],
	outlook: &Outlook,
	remaining: f64,
	clock: &Instant,
	committed: &[Cell],
	contested: &[Cell],
	deadline: u128,
) -> Option<Choice> {
	let base = appraise(map, engine, field, outlook);
	let mut seen_routes: Vec<u64> = Vec::new();
	let mut best: Option<Choice> = None;
	let mut banned = vec![0u8; map.cells];
	let mut current: Vec<Cell> = Vec::new();
	let weigh = |engine: &mut Engine, cells: &[Cell], paint: u32, loyalty: f64| {
		engine.trial.copy_from_slice(field);
		for &at in cells {
			engine.trial[at as usize] = MINE;
		}
		let worth = {
			let trial = std::mem::take(&mut engine.trial);
			let worth = appraise(map, engine, &trial, outlook);
			engine.trial = trial;
			worth
		};
		let gain = (worth - base) / outlook.span;
		let mut turns = remaining - paint as f64 / PAINT_PER_TURN as f64;
		if tune.life_base > 0.0 {
			let mut exposure = 0.0;
			let mut counted: Vec<u8> = Vec::new();
			for &at in cells {
				let region = map.region[at as usize];
				if map.region_has_town[region as usize] || counted.contains(&region) {
					continue;
				}
				counted.push(region);
				exposure += 1.0 + state.instability[region as usize] as f64;
			}
			let life = tune.life_base - tune.life_slope * exposure;
			turns = turns.min(life.max(1.0));
		}
		if turns <= 0.0 {
			return None;
		}
		if tune.weigh_life > 0.0 {
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
			if !cells.is_empty() {
				turns *= (live / cells.len() as f64).max(0.05);
			}
		}
		if gain <= 0.0 {
			return None;
		}
		Some(loyalty * gain * turns / (paint as f64).powf(tune.paint_power))
	};

	let any_foe = owner.contains(&FOE);
	let mut shapes: Vec<(bool, bool, bool, u32, u32)> = Vec::new();
	let mut live_modes: Vec<u32> = Vec::new();
	for mode in 0..10u32 {
		if tune.modes & (1 << mode) == 0 {
			continue;
		}
		if mode == REFEREE_MODE {
			live_modes.push(mode);
			continue;
		}
		let (short_first, risk_aware, avoid_foe, drag) = route_shape(tune, mode);
		let drag = if any_foe { drag } else { (0, 0) };
		let avoid_foe = avoid_foe && any_foe;
		let shape = (short_first, risk_aware, avoid_foe, drag.0, drag.1);
		if mode != 3 && shapes.contains(&shape) {
			continue;
		}
		shapes.push(shape);
		live_modes.push(mode);
	}

	for index in 0..map.links.len() {
		if clock.elapsed().as_millis() > deadline || (tune.plots > 0 && engine.spent > tune.plots) {
			break;
		}
		let (from, to) = map.links[index];
		let start = map.towns[from as usize].at;
		let goal = map.towns[to as usize].at;

		engine.trail(map, owner, from, to, &mut current);
		let detour = current.len() > 2;
		if detour {
			for &at in &current[1..current.len() - 1] {
				banned[at as usize] = 1;
			}
		}

		for &mode in &live_modes {
			if (mode == 3 && !detour)
				|| clock.elapsed().as_millis() > deadline
				|| (tune.plots > 0 && engine.spent > tune.plots)
			{
				continue;
			}
			let found = if mode == REFEREE_MODE {
				engine.trace_route(map, state, tune, owner, start, goal)
			} else {
				let (short_first, risk_aware, avoid_foe, drag) = route_shape(tune, mode);
				let blocked = if mode == 3 {
					std::mem::take(&mut banned)
				} else {
					std::mem::take(&mut engine.clear)
				};
				let found = engine.plot(
					map,
					state,
					tune,
					owner,
					start,
					goal,
					short_first,
					risk_aware,
					avoid_foe,
					drag,
					&blocked,
				);
				if mode == 3 {
					banned = blocked;
				} else {
					engine.clear = blocked;
				}
				found
			};
			let Some(paint) = found else { continue };
			if paint == 0 || engine.route.is_empty() {
				continue;
			}
			let cells = engine.route.clone();
			{
				let mut mark = 1469598103934665603u64;
				for &at in &cells {
					mark ^= at as u64;
					mark = mark.wrapping_mul(1099511628211);
				}
				if seen_routes.contains(&mark) {
					continue;
				}
				seen_routes.push(mark);
			}
			let Some(score) = weigh(engine, &cells, paint, 1.0) else {
				continue;
			};
			if best.as_ref().is_none_or(|top| score > top.score) {
				best = Some(Choice { cells, score });
			}
		}

		if detour {
			for &at in &current[1..current.len() - 1] {
				banned[at as usize] = 0;
			}
		}
	}

	for (extra, loyalty) in [(committed, tune.commit_bonus), (contested, tune.preempt)] {
		if loyalty <= 1.0 || extra.is_empty() {
			continue;
		}
		let paint = extra
			.iter()
			.map(|&at| map.cost[at as usize] as u32)
			.sum::<u32>();
		if paint > 0
			&& let Some(score) = weigh(engine, extra, paint, loyalty)
			&& best.as_ref().is_none_or(|top| score > top.score)
		{
			best = Some(Choice {
				cells: extra.to_vec(),
				score,
			});
		}
	}
	best
}

fn foresee(
	map: &Map,
	state: &State,
	tune: &Tune,
	engine: &mut Engine,
	owner: &[u8],
	outlook: &Outlook,
	remaining: f64,
) -> Vec<Cell> {
	let mut theirs: Vec<u8> = owner
		.iter()
		.map(|&who| match who {
			MINE => FOE,
			FOE => MINE,
			other => other,
		})
		.collect();
	let clock = Instant::now();
	let nothing: Vec<Cell> = Vec::new();
	let found = survey(
		map,
		state,
		tune,
		engine,
		&theirs,
		&theirs,
		outlook,
		remaining,
		&clock,
		&nothing,
		&nothing,
		tune.foe_ms,
	);
	theirs.clear();
	found.map(|choice| choice.cells).unwrap_or_default()
}

fn forecast(
	map: &Map,
	state: &State,
	tune: &Tune,
	engine: &mut Engine,
	owner: &[u8],
) -> (Vec<u8>, Vec<u16>) {
	let mut hope = owner.to_vec();
	let mut order: Vec<usize> = (0..map.links.len()).collect();
	if tune.pre_order != 0 {
		order.sort_by_key(|&index| {
			let (from, to) = map.links[index];
			let (ax, ay) = map.point(map.towns[from as usize].at);
			let (bx, by) = map.point(map.towns[to as usize].at);
			let span = (ax.abs_diff(bx) + ay.abs_diff(by)) as i32;
			if tune.pre_order == 2 { -span } else { span }
		});
	}
	for index in order {
		let (from, to) = map.links[index];
		let start = map.towns[from as usize].at;
		let goal = map.towns[to as usize].at;
		let blocked = std::mem::take(&mut engine.clear);
		let found = engine.plot(
			map,
			state,
			tune,
			&hope,
			start,
			goal,
			true,
			tune.pre_risk != 0,
			false,
			(tune.foe_toll, tune.foe_len),
			&blocked,
		);
		engine.clear = blocked;
		if let Some(paint) = found
			&& (tune.pre_cap == 0 || paint <= tune.pre_cap)
		{
			for &at in &engine.route {
				hope[at as usize] = VIRT;
			}
		}
	}
	let mut mult = vec![0u16; map.cells];
	engine.tally(map, &hope, &mut mult);
	(hope, mult)
}

#[allow(clippy::too_many_arguments)]
fn at_gateway(map: &Map, slot: usize) -> bool {
	(0..4).any(|way| {
		let next = map.neigh[slot][way];
		next != NO_CELL && map.is_town[next as usize]
	})
}

fn preclaim(
	map: &Map,
	state: &State,
	tune: &Tune,
	owner: &[u8],
	hope: &[u8],
	mult: &[u16],
	mult_foe: &[u16],
	remaining: f64,
) -> Option<Choice> {
	let mut wanted: Vec<(f64, Cell)> = Vec::new();
	for slot in 0..map.cells {
		if owner[slot] != EMPTY || hope[slot] != VIRT || mult[slot] == 0 {
			continue;
		}
		if tune.pre_only_foe && mult_foe[slot] == 0 {
			continue;
		}
		let worth = if tune.lin == 1 || tune.lin == 2 {
			learned(map, state, mult, remaining, slot) / remaining
		} else {
			let shelter = if map.region_has_town[map.region[slot] as usize] {
				1.0 + tune.pre_safe
			} else {
				1.0
			};
			shelter * mult[slot] as f64
		};
		let worth = if tune.gate > 0.0 && at_gateway(map, slot) {
			worth * (1.0 + tune.gate)
		} else {
			worth
		};
		let worth = worth * (1.0 + tune.align * map.aligned[slot] as f64);
		let worth =
			worth * (1.0 + tune.pre_foe * mult_foe[slot] as f64 / (1.0 + mult[slot] as f64));
		wanted.push((worth / map.cost[slot] as f64, slot as Cell));
	}
	wanted.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
	let mut cells: Vec<Cell> = Vec::new();
	let mut paint = 0u32;
	let mut gain = 0.0;
	if tune.pre_run {
		let anchored = |at: Cell, run: &[Cell]| {
			(0..4).any(|way| {
				let next = map.neigh[at as usize][way];
				next != NO_CELL
					&& (map.is_town[next as usize]
						|| owner[next as usize] == MINE
						|| run.contains(&next))
			})
		};
		let seed = wanted
			.iter()
			.find(|&&(_, at)| anchored(at, &[]))
			.or(wanted.first())
			.map(|&(_, at)| at);
		if let Some(seed) = seed {
			cells.push(seed);
			paint += map.cost[seed as usize] as u32;
			loop {
				let Some(&(_, at)) = wanted.iter().find(|&&(_, at)| {
					!cells.contains(&at)
						&& paint + map.cost[at as usize] as u32 <= tune.pre_paint
						&& anchored(at, &cells)
				}) else {
					break;
				};
				cells.push(at);
				paint += map.cost[at as usize] as u32;
			}
		}
		for &at in &cells {
			gain += if tune.lin == 1 || tune.lin == 3 {
				learned(map, state, mult, remaining, at as usize) / remaining
			} else {
				mult[at as usize] as f64
			};
		}
	}
	for &(_, at) in &wanted {
		if tune.pre_run {
			break;
		}
		let price = map.cost[at as usize] as u32;
		if paint + price > tune.pre_paint {
			continue;
		}
		paint += price;
		gain += if tune.lin == 1 || tune.lin == 3 {
			learned(map, state, mult, remaining, at as usize) / remaining
		} else {
			mult[at as usize] as f64
		};
		cells.push(at);
	}
	let turns = remaining - paint as f64 / PAINT_PER_TURN as f64;
	if cells.is_empty() || paint == 0 || turns <= 0.0 {
		return None;
	}
	Some(Choice {
		cells,
		score: tune.pre * gain * turns / (paint as f64).powf(tune.paint_power),
	})
}

fn topup(
	map: &Map,
	state: &State,
	tune: &Tune,
	engine: &mut Engine,
	owner: &[u8],
	mult: &[u16],
	budget: u32,
) -> Option<Cell> {
	let base = engine.edge(map, owner) as f64;
	let mut seeds: Vec<(f64, Cell)> = Vec::new();
	for slot in 0..map.cells {
		if map.is_town[slot]
			|| owner[slot] != EMPTY
			|| state.inked[map.region[slot] as usize]
			|| map.cost[slot] as u32 > budget
		{
			continue;
		}
		let mut touching = 0;
		for way in 0..4 {
			let next = map.neigh[slot][way];
			if next != NO_CELL && (map.is_town[next as usize] || owner[next as usize] == MINE) {
				touching += 1;
			}
		}
		let worth = tune.fill_mult * mult[slot] as f64;
		if touching == 0 && worth <= 0.0 {
			continue;
		}
		let region = map.region[slot] as usize;
		let shelter = if map.region_has_town[region] {
			tune.fill_shelter
		} else {
			0.0
		};
		let calm = -tune.fill_calm * state.instability[region] as f64;
		let gate = if at_gateway(map, slot) {
			tune.fill_gate
		} else {
			0.0
		};
		seeds.push((
			worth + shelter + calm + gate - tune.fill_cost * map.cost[slot] as f64,
			slot as Cell,
		));
	}
	seeds.sort_by(|a, b| b.0.total_cmp(&a.0));
	seeds.truncate(tune.fill_cap);

	let mut best: Option<(f64, Cell)> = None;
	for (rank, &(hint, at)) in seeds.iter().enumerate() {
		engine.trial.copy_from_slice(owner);
		engine.trial[at as usize] = MINE;
		let after = {
			let trial = std::mem::take(&mut engine.trial);
			let after = engine.edge(map, &trial) as f64;
			engine.trial = trial;
			after
		};
		if after < base {
			continue;
		}
		let score = (after - base) * 100.0 + hint - rank as f64 * 0.001;
		if best.as_ref().is_none_or(|(top, _)| score > *top) {
			best = Some((score, at));
		}
	}
	best.map(|(_, at)| at)
}

#[allow(clippy::too_many_arguments)]
fn pick_disrupt(
	map: &Map,
	state: &State,
	tune: &Tune,
	engine: &mut Engine,
	owner: &[u8],
	mult: &[u16],
	planned: &[Cell],
	turn: i32,
	ranked: &mut Vec<(f64, u8)>,
) -> (Option<u8>, bool) {
	ranked.clear();
	if tune.disrupt == 0 {
		return (None, false);
	}
	let mut best: Option<(f64, u8)> = None;
	let mut idle: Option<(f64, u8)> = None;
	let ahead = state.my_score > state.foe_score;
	let outearned = engine.edge(map, owner) < 0;
	let closing = ahead && outearned;
	let live_links = engine.links_left(map, &state.inked, usize::MAX);
	let base = if tune.disrupt == 2 {
		0.0
	} else {
		engine.edge(map, owner) as f64
	};
	for region in 0..map.region_cells.len() {
		if map.region_has_town[region] || state.inked[region] {
			continue;
		}
		let mut mine = 0;
		let mut foe = 0;
		let mut crude = 0.0;
		let weight_base = if turn <= tune.early_turns {
			tune.early_base
		} else {
			tune.crude_base
		};
		for &at in &map.region_cells[region] {
			let weight = state.mult[at as usize] as f64 + weight_base;
			match owner[at as usize] {
				MINE => {
					mine += 1;
					crude -= weight * tune.crude_self;
				}
				FOE => {
					foe += 1;
					crude += weight;
				}
				_ => crude += tune.ink_hope * mult[at as usize] as f64,
			}
		}
		let mut value = if tune.disrupt == 2 {
			crude
		} else {
			let mut swing = if mine == 0 && foe == 0 {
				0.0
			} else {
				engine.shade.copy_from_slice(owner);
				for &at in &map.region_cells[region] {
					engine.shade[at as usize] = EMPTY;
				}
				let shade = std::mem::take(&mut engine.shade);
				let after = engine.edge(map, &shade) as f64;
				engine.shade = shade;
				after - base
			};
			if swing < 0.0 {
				swing *= tune.self_penalty;
			}
			if tune.disrupt == 3 {
				tune.swing_gain * swing + tune.crude_mix * crude
			} else {
				swing
			}
		};
		for &at in planned {
			if map.region[at as usize] as usize == region {
				value -= tune.plan_penalty;
			}
		}
		let steps = INSTABILITY_THRESHOLD
			.saturating_sub(state.instability[region])
			.max(1) as f64;
		if steps <= 1.0 && tune.sever {
			let survivors = engine.links_left(map, &state.inked, region);
			if survivors == 0 {
				if ahead {
					return (Some(region as u8), true);
				}
				continue;
			}
			let severed = (live_links - survivors) as f64;
			if closing {
				value += tune.sever_bonus * severed;
			} else if !ahead && tune.anti_sever {
				value -= tune.sever_bonus * severed;
			}
		}
		let score = value / steps;
		if value > tune.crude_floor && turn <= tune.late_ink {
			ranked.push((score, region as u8));
			if best.as_ref().is_none_or(|(top, _)| score > *top) {
				best = Some((score, region as u8));
			}
		} else if tune.precharge
			&& steps > 1.0
			&& foe > 0
			&& mine == 0
			&& idle.as_ref().is_none_or(|(top, _)| score > *top)
		{
			idle = Some((score, region as u8));
		}
	}
	ranked.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
	(best.or(idle).map(|(_, region)| region), false)
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
	#[allow(dead_code)]
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
				let weight = self.mult[cell] as f64 + tune.sim_base;
				if self.owner[cell] == opp {
					value += weight;
					theirs += 1;
				} else if self.owner[cell] == who {
					value -= weight * tune.sim_self;
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
			} else if tune.sim_precharge
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

#[allow(clippy::too_many_arguments)]
fn decide(
	map: &Map,
	state: &State,
	tune: &Tune,
	engine: &mut Engine,
	router: &mut Router,
	sim: &mut Sim,
	turn: i32,
	commands: &mut Vec<Command>,
	committed: &mut Vec<Cell>,
) {
	let clock = Instant::now();
	engine.spent = 0;
	commands.clear();
	committed.retain(|&at| {
		state.owner[at as usize] == EMPTY && !state.inked[map.region[at as usize] as usize]
	});

	let remaining = (MAX_TURNS - turn + 1).max(1) as f64;
	let outlook = Outlook::build(map, state, tune);
	let mut owner = state.owner.clone();
	let mut budget = PAINT_PER_TURN;
	let mut built: Vec<Cell> = Vec::new();
	let mut planned: Vec<Cell> = Vec::new();
	engine.ride = false;
	let (hope, mult) = if tune.pre > 0.0 || tune.fill_mult > 0.0 || tune.route_mult > 0 {
		forecast(map, state, tune, engine, &owner)
	} else {
		(Vec::new(), vec![0u16; map.cells])
	};
	if tune.route_mult > 0 {
		engine.mult.copy_from_slice(&mult);
		engine.ride = true;
	}
	let mut hope = hope;
	let mut hope_foe: Vec<u8> = Vec::new();
	let mult_foe = if tune.poison || (tune.pre > 0.0 && (tune.pre_foe > 0.0 || tune.pre_only_foe)) {
		let theirs: Vec<u8> = owner
			.iter()
			.map(|&who| match who {
				MINE => FOE,
				FOE => MINE,
				other => other,
			})
			.collect();
		let salt = engine.salt;
		engine.salt = (1 - map.me as u32).wrapping_mul(0x9E37_79B9);
		let (foe_hope, foe_mult) = forecast(map, state, tune, engine, &theirs);
		engine.salt = salt;
		hope_foe = foe_hope;
		foe_mult
	} else {
		vec![0u16; map.cells]
	};
	let mut pre = if tune.pre > 0.0 && turn > tune.pre_skip {
		preclaim(map, state, tune, &owner, &hope, &mult, &mult_foe, remaining)
	} else {
		None
	};
	let contested = if tune.preempt > 1.0 {
		foresee(map, state, tune, engine, &owner, &outlook, remaining)
	} else {
		Vec::new()
	};

	let mut field = owner.clone();
	for _ in 0..3 {
		if budget == 0 || clock.elapsed().as_millis() > tune.think_ms {
			break;
		}
		let Some(choice) = survey(
			map,
			state,
			tune,
			engine,
			&owner,
			&field,
			&outlook,
			remaining,
			&clock,
			committed,
			&contested,
			tune.think_ms,
		) else {
			break;
		};
		let choice = match pre.take() {
			Some(early) if early.score > choice.score || turn <= tune.pre_turns => early,
			other => {
				pre = other;
				choice
			}
		};
		let mut cells = choice.cells;
		if tune.flip && map.me == 1 {
			cells.reverse();
		}
		let mut whole = true;
		committed.clear();
		for at in cells {
			let price = map.cost[at as usize] as u32;
			if tune.dedup && owner[at as usize] != EMPTY {
				continue;
			}
			if price <= budget {
				budget -= price;
				owner[at as usize] = MINE;
				field[at as usize] = MINE;
				if !hope.is_empty() {
					hope[at as usize] = MINE;
				}
				built.push(at);
			} else {
				whole = false;
				planned.push(at);
				committed.push(at);
			}
		}
		if !whole {
			break;
		}
	}

	while tune.fill && budget > 0 && clock.elapsed().as_millis() <= tune.think_ms {
		let Some(at) = topup(map, state, tune, engine, &owner, &mult, budget) else {
			break;
		};
		budget -= map.cost[at as usize] as u32;
		owner[at as usize] = MINE;
		built.push(at);
	}

	for at in &built {
		let (x, y) = map.point(*at);
		commands.push(Command::Place(x, y));
	}
	let mut ranked: Vec<(f64, u8)> = Vec::new();
	let (mut ink, forced) = pick_disrupt(
		map,
		state,
		tune,
		engine,
		&owner,
		&mult,
		&planned,
		turn,
		&mut ranked,
	);
	if tune.poison && !forced && ranked.is_empty() && !hope_foe.is_empty() {
		let mut best: Option<((u8, usize), usize)> = None;
		for region in 0..map.region_cells.len() {
			if map.region_has_town[region]
				|| state.inked[region]
				|| state.instability[region] + 1 >= INSTABILITY_THRESHOLD
			{
				continue;
			}
			let mut theirs = 0usize;
			let mut ours = false;
			for &at in &map.region_cells[region] {
				let cell = at as usize;
				if owner[cell] == MINE || (!hope.is_empty() && hope[cell] == VIRT) {
					ours = true;
					break;
				}
				if hope_foe[cell] == VIRT {
					theirs += 1;
				}
			}
			if ours || theirs == 0 {
				continue;
			}
			let key = (state.instability[region], theirs);
			if best.is_none_or(|(top, _)| key > top) {
				best = Some((key, region));
			}
		}
		if let Some((_, region)) = best {
			ink = Some(region as u8);
		}
	}
	if tune.ink_roll > 0 && !forced && ranked.len() > 1 {
		let mine = Plan::build(
			map,
			state,
			tune,
			router,
			&mut engine.flow,
			&state.owner,
			MINE,
		);
		let theirs = Plan::build(
			map,
			state,
			tune,
			router,
			&mut engine.flow,
			&state.owner,
			FOE,
		);
		let plans = [&mine, &theirs];
		sim.follow = false;
		let null = sim.rollout(
			map,
			state,
			tune,
			router,
			&plans,
			&built,
			None,
			remaining as i32,
		);
		sim.follow = tune.follow;
		let mut best_score = f64::NEG_INFINITY;
		let mut best_region: Option<u8> = None;
		let mut tried: Vec<u8> = Vec::new();
		let mut options: Vec<u8> = Vec::new();
		if let Some(region) = ink {
			options.push(region);
		}
		if let Some(region) = null.ink {
			options.push(region as u8);
		}
		for &(_, region) in ranked.iter().take(tune.ink_roll) {
			options.push(region);
		}
		for region in options {
			if tried.contains(&region) || clock.elapsed().as_millis() > tune.think_ms {
				continue;
			}
			tried.push(region);
			let outcome = sim.rollout(
				map,
				state,
				tune,
				router,
				&plans,
				&built,
				Some(region as usize),
				remaining as i32,
			);
			if outcome.score > best_score {
				best_score = outcome.score;
				best_region = Some(region);
			}
		}
		if best_region.is_some() {
			ink = best_region;
		}
	}
	if let Some(region) = ink {
		commands.push(Command::Disrupt(region));
	}
}

fn main() {
	let mut input = Reader::new();
	let mut out = io::stdout().lock();
	let tune = Tune::load();
	let map = Map::read(&mut input);
	input.flush_echo();
	let mut state = State::new(&map);
	let mut engine = Engine::new(&map);
	engine.exact = tune.exact;
	let mut router = Router::new(&map);
	let mut sim = Sim::new(&map);
	let mut commands = Vec::with_capacity(5);
	let mut committed: Vec<Cell> = Vec::new();
	let mut turn = 0;

	while state.read(&map, &mut input) {
		turn += 1;
		decide(
			&map,
			&state,
			&tune,
			&mut engine,
			&mut router,
			&mut sim,
			turn,
			&mut commands,
			&mut committed,
		);
		send(&commands, &mut out).unwrap();
	}
}
