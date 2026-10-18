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
const NEUTRAL: u8 = 3;

const CELL_WEIGHT: u32 = 2048;

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
			risk_base: knob("BTK_RISK_BASE", 0),
			risk_inst: knob("BTK_RISK_INST", 4),
			scenarios: knob("BTK_SCENARIOS", 2),
			flip: knob::<u8>("BTK_FLIP", 0) != 0,
			disrupt: knob("BTK_DISRUPT", 3),
			self_penalty: knob("BTK_SELF_PENALTY", 1.0),
			plan_penalty: knob("BTK_PLAN_PENALTY", 1.0),
			modes: knob("BTK_MODES", 15),
			fill: knob::<u8>("BTK_FILL", 1) != 0,
			fill_cap: knob("BTK_FILL_CAP", 48),
			paint_power: knob("BTK_PAINT_POWER", 2.0),
			shelter: knob("BTK_SHELTER", 0),
			crude_self: knob("BTK_CRUDE_SELF", 2.0),
			crude_floor: knob("BTK_CRUDE_FLOOR", 0.0),
			precharge: knob::<u8>("BTK_PRECHARGE", 1) != 0,
			late_ink: knob("BTK_LATE_INK", 100),
			sever_bonus: knob("BTK_SEVER_BONUS", 4.0),
			sever: knob::<u8>("BTK_SEVER", 1) != 0,
			commit_bonus: knob("BTK_COMMIT_BONUS", 1.0),
			anti_sever: knob::<u8>("BTK_ANTI_SEVER", 0) != 0,
			think_ms: knob("BTK_THINK_MS", 20),
			crude_mix: knob("BTK_CRUDE_MIX", 2.0),
			swing_gain: knob("BTK_SWING", 1.0),
			avoid_inst: knob("BTK_AVOID_INST", 3),
			crude_base: knob("BTK_CRUDE_BASE", 0.25),
			scen_floor: knob("BTK_SCEN_FLOOR", 0.0),
			scen_gain: knob("BTK_SCEN_GAIN", 1.0),
			mirror_weight: knob("BTK_MIRROR_WEIGHT", 0.0),
		}
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

	fn trail(&mut self, map: &Map, owner: &[u8], from: TownId, to: TownId, out: &mut Vec<Cell>) {
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
		banned: &[u8],
	) -> Option<u32> {
		self.epoch += 1;
		let epoch = self.epoch;
		self.heap.clear();
		self.ranks[from as usize] = 0;
		self.seen[from as usize] = epoch;
		self.came[from as usize] = NO_CELL;
		self.heap.push(Reverse((0, from)));

		while let Some(Reverse((reached, at))) = self.heap.pop() {
			let here = at as usize;
			if self.seen[here] != epoch || self.ranks[here] != reached {
				continue;
			}
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
					0
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
				let stride = if short_first {
					4 * (CELL_WEIGHT + toll)
				} else {
					4 * (toll * CELL_WEIGHT + 1)
				};
				let total =
					reached + stride + (((cell as u32) ^ self.salt).wrapping_mul(2654435761) >> 30);
				if self.seen[cell] == epoch && self.ranks[cell] <= total {
					continue;
				}
				self.seen[cell] = epoch;
				self.ranks[cell] = total;
				self.came[cell] = at;
				self.heap.push(Reverse((total, next)));
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

#[allow(clippy::too_many_arguments)]
fn survey(
	map: &Map,
	state: &State,
	tune: &Tune,
	engine: &mut Engine,
	owner: &[u8],
	outlook: &Outlook,
	remaining: f64,
	clock: &Instant,
	committed: &[Cell],
) -> Option<Choice> {
	let base = appraise(map, engine, owner, outlook);
	let mut best: Option<Choice> = None;
	let mut banned = vec![0u8; map.cells];
	let mut current: Vec<Cell> = Vec::new();
	let weigh = |engine: &mut Engine, cells: &[Cell], paint: u32, loyalty: f64| {
		engine.trial.copy_from_slice(owner);
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
		let turns = remaining - paint as f64 / PAINT_PER_TURN as f64;
		if gain <= 0.0 || turns <= 0.0 {
			return None;
		}
		Some(loyalty * gain * turns / (paint as f64).powf(tune.paint_power))
	};

	for index in 0..map.links.len() {
		if clock.elapsed().as_millis() > tune.think_ms {
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

		for mode in 0..4u32 {
			if tune.modes & (1 << mode) == 0
				|| (mode == 3 && !detour)
				|| clock.elapsed().as_millis() > tune.think_ms
			{
				continue;
			}
			let (short_first, risk_aware, avoid_foe) = match mode {
				0 => (false, true, false),
				1 => (true, true, false),
				2 => (true, false, true),
				_ => (true, true, false),
			};
			let found = {
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

	if tune.commit_bonus > 1.0 && !committed.is_empty() {
		let paint = committed
			.iter()
			.map(|&at| map.cost[at as usize] as u32)
			.sum::<u32>();
		if paint > 0
			&& let Some(score) = weigh(engine, committed, paint, tune.commit_bonus)
			&& best.as_ref().is_none_or(|top| score > top.score)
		{
			best = Some(Choice {
				cells: committed.to_vec(),
				score,
			});
		}
	}
	best
}

fn topup(
	map: &Map,
	state: &State,
	tune: &Tune,
	engine: &mut Engine,
	owner: &[u8],
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
		if touching == 0 {
			continue;
		}
		let region = map.region[slot] as usize;
		let shelter = if map.region_has_town[region] {
			2.0
		} else {
			0.0
		};
		let calm = -(state.instability[region] as f64);
		seeds.push((shelter + calm - map.cost[slot] as f64, slot as Cell));
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

fn pick_disrupt(
	map: &Map,
	state: &State,
	tune: &Tune,
	engine: &mut Engine,
	owner: &[u8],
	planned: &[Cell],
	turn: i32,
) -> Option<u8> {
	if tune.disrupt == 0 {
		return None;
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
		for &at in &map.region_cells[region] {
			let weight = state.mult[at as usize] as f64 + tune.crude_base;
			match owner[at as usize] {
				MINE => {
					mine += 1;
					crude -= weight * tune.crude_self;
				}
				FOE => {
					foe += 1;
					crude += weight;
				}
				_ => {}
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
					return Some(region as u8);
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
	best.or(idle).map(|(_, region)| region)
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

fn decide(
	map: &Map,
	state: &State,
	tune: &Tune,
	engine: &mut Engine,
	turn: i32,
	commands: &mut Vec<Command>,
	committed: &mut Vec<Cell>,
) {
	let clock = Instant::now();
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

	for _ in 0..3 {
		if budget == 0 || clock.elapsed().as_millis() > tune.think_ms {
			break;
		}
		let Some(choice) = survey(
			map, state, tune, engine, &owner, &outlook, remaining, &clock, committed,
		) else {
			break;
		};
		let mut cells = choice.cells;
		if tune.flip && map.me == 1 {
			cells.reverse();
		}
		let mut whole = true;
		committed.clear();
		for at in cells {
			let price = map.cost[at as usize] as u32;
			if price <= budget {
				budget -= price;
				owner[at as usize] = MINE;
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
		let Some(at) = topup(map, state, tune, engine, &owner, budget) else {
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
	if let Some(region) = pick_disrupt(map, state, tune, engine, &owner, &planned, turn) {
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
			turn,
			&mut commands,
			&mut committed,
		);
		send(&commands, &mut out).unwrap();
	}
}
