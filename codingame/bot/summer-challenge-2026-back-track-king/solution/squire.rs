use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fmt::Display;
use std::io::{self, Read, Write};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, IndexMut, Sub, SubAssign};

const MAX_WIDTH: u8 = 30;
const MAX_HEIGHT: u8 = 20;
const ROWS: usize = MAX_HEIGHT as usize;

const PAINT_PER_TURN: u8 = 3;
const MAX_TURNS: i32 = 100;

const NO_TRACK: i8 = -1;
const NEUTRAL_TRACK: i8 = 2;

type Point = (u8, u8);
type TownId = u8;
type LinkId = u8;
type RegionId = u8;

const NO_LINK: LinkId = LinkId::MAX;
const RING_LINK: LinkId = 200;
const BYPASS_LINK: LinkId = 190;
const CELL_LINK: LinkId = 180;

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
struct Mask([u64; ROWS]);

impl Mask {
	const EMPTY: Mask = Mask([0; ROWS]);

	fn contains(&self, (x, y): Point) -> bool {
		(self.0[y as usize] >> x) & 1 != 0
	}

	fn insert(&mut self, (x, y): Point) {
		self.0[y as usize] |= 1u64 << x;
	}

	fn is_empty(&self) -> bool {
		self.0.iter().all(|row| *row == 0)
	}

	fn iter(&self) -> MaskIter {
		MaskIter { rows: self.0, y: 0 }
	}
}

struct MaskIter {
	rows: [u64; ROWS],
	y: usize,
}

impl Iterator for MaskIter {
	type Item = Point;

	fn next(&mut self) -> Option<Point> {
		while self.y < ROWS {
			let row = self.rows[self.y];
			if row != 0 {
				self.rows[self.y] = row & (row - 1);
				return Some((row.trailing_zeros() as u8, self.y as u8));
			}
			self.y += 1;
		}
		None
	}
}

impl BitAndAssign for Mask {
	fn bitand_assign(&mut self, other: Mask) {
		for (row, mask) in self.0.iter_mut().zip(other.0) {
			*row &= mask;
		}
	}
}

impl BitOrAssign for Mask {
	fn bitor_assign(&mut self, other: Mask) {
		for (row, mask) in self.0.iter_mut().zip(other.0) {
			*row |= mask;
		}
	}
}

impl SubAssign for Mask {
	fn sub_assign(&mut self, other: Mask) {
		for (row, mask) in self.0.iter_mut().zip(other.0) {
			*row &= !mask;
		}
	}
}

impl BitAnd for Mask {
	type Output = Mask;

	fn bitand(mut self, other: Mask) -> Mask {
		self &= other;
		self
	}
}

impl BitOr for Mask {
	type Output = Mask;

	fn bitor(mut self, other: Mask) -> Mask {
		self |= other;
		self
	}
}

impl Sub for Mask {
	type Output = Mask;

	fn sub(mut self, other: Mask) -> Mask {
		self -= other;
		self
	}
}

struct Field<T> {
	values: Vec<T>,
	width: u8,
}

impl<T: Clone> Field<T> {
	fn new(width: u8, height: u8, value: T) -> Field<T> {
		Field {
			values: vec![value; width as usize * height as usize],
			width,
		}
	}
}

impl<T> Index<Point> for Field<T> {
	type Output = T;

	fn index(&self, (x, y): Point) -> &T {
		debug_assert!(x < self.width);
		&self.values[y as usize * self.width as usize + x as usize]
	}
}

impl<T> IndexMut<Point> for Field<T> {
	fn index_mut(&mut self, (x, y): Point) -> &mut T {
		debug_assert!(x < self.width);
		&mut self.values[y as usize * self.width as usize + x as usize]
	}
}

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Terrain {
	Plains,
	River,
	Mountain,
	Poi,
}

impl Terrain {
	fn read(kind: u8) -> Terrain {
		match kind {
			0 => Terrain::Plains,
			1 => Terrain::River,
			2 => Terrain::Mountain,
			_ => Terrain::Poi,
		}
	}

	fn paint_cost(self) -> u8 {
		match self {
			Terrain::Plains => 1,
			Terrain::River => 2,
			Terrain::Mountain | Terrain::Poi => 3,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Track {
	Mine,
	Foe,
	Neutral,
}

impl Track {
	fn read(owner: i8, me: u8) -> Option<Track> {
		match owner {
			NO_TRACK => None,
			NEUTRAL_TRACK => Some(Track::Neutral),
			owner if owner == me as i8 => Some(Track::Mine),
			_ => Some(Track::Foe),
		}
	}
}

struct Town {
	at: Point,
	desired: Vec<TownId>,
}

#[derive(Clone, Copy)]
struct Link {
	from: TownId,
	to: TownId,
}

#[derive(Clone, Default)]
struct Region {
	cells: Mask,
	has_town: bool,
}

struct Map {
	me: u8,
	width: u8,
	height: u8,
	terrain: Field<Terrain>,
	region: Field<RegionId>,
	regions: Vec<Region>,
	towns: Vec<Town>,
	town_cells: Mask,
	links: Vec<Link>,
	link_index: Vec<LinkId>,
}

impl Map {
	fn read(input: &mut Reader) -> Map {
		let me = input.u8();
		let width = input.u8();
		let height = input.u8();
		assert!(width <= MAX_WIDTH && height <= MAX_HEIGHT);

		let mut terrain = Field::new(width, height, Terrain::Plains);
		let mut region = Field::new(width, height, 0);
		let mut region_count = 0;
		for y in 0..height {
			for x in 0..width {
				let id = input.u8();
				region[(x, y)] = id;
				terrain[(x, y)] = Terrain::read(input.u8());
				region_count = region_count.max(id as usize + 1);
			}
		}

		let mut regions = vec![Region::default(); region_count];
		for y in 0..height {
			for x in 0..width {
				regions[region[(x, y)] as usize].cells.insert((x, y));
			}
		}

		let town_count = input.usize();
		let mut towns = Vec::with_capacity(town_count);
		let mut town_cells = Mask::EMPTY;
		for id in 0..town_count {
			let given = input.usize();
			debug_assert_eq!(given, id);
			let at = (input.u8(), input.u8());
			let mut desired = Vec::new();
			input.town_list(&mut desired);

			town_cells.insert(at);
			regions[region[at] as usize].has_town = true;
			towns.push(Town { at, desired });
		}

		let mut links = Vec::new();
		let mut link_index = vec![NO_LINK; town_count * town_count];
		for (from, town) in towns.iter().enumerate() {
			for &to in &town.desired {
				link_index[from * town_count + to as usize] = links.len() as LinkId;
				links.push(Link {
					from: from as TownId,
					to,
				});
			}
		}

		Map {
			me,
			width,
			height,
			terrain,
			region,
			regions,
			towns,
			town_cells,
			links,
			link_index,
		}
	}

	fn points(&self) -> impl Iterator<Item = Point> {
		let width = self.width;
		(0..self.height).flat_map(move |y| (0..width).map(move |x| (x, y)))
	}

	fn neighbours(&self, (x, y): Point) -> impl Iterator<Item = Point> {
		let (width, height) = (self.width, self.height);
		[
			(y > 0).then(|| (x, y - 1)),
			(x + 1 < width).then_some((x + 1, y)),
			(y + 1 < height).then_some((x, y + 1)),
			(x > 0).then(|| (x - 1, y)),
		]
		.into_iter()
		.flatten()
	}

	fn paint_cost(&self, at: Point) -> u8 {
		self.terrain[at].paint_cost()
	}

	fn region_of(&self, at: Point) -> RegionId {
		self.region[at]
	}

	fn link_of(&self, from: TownId, to: TownId) -> Option<LinkId> {
		let found = self.link_index[from as usize * self.towns.len() + to as usize];
		(found != NO_LINK).then_some(found)
	}

	fn endpoints_of(&self, link: LinkId) -> (Point, Point) {
		let Link { from, to } = self.links[link as usize];
		(self.towns[from as usize].at, self.towns[to as usize].at)
	}

	fn tour(&self) -> Vec<(Point, Point)> {
		let n = self.towns.len();
		if n < 3 {
			return Vec::new();
		}
		let d = |a: usize, b: usize| walk(self.towns[a].at, self.towns[b].at) as i32;
		let mut order: Vec<usize> = vec![0];
		let mut left: Vec<usize> = (1..n).collect();
		while !left.is_empty() {
			let last = *order.last().unwrap();
			let (index, _) = left
				.iter()
				.enumerate()
				.min_by_key(|(_, t)| d(last, **t))
				.unwrap();
			order.push(left.swap_remove(index));
		}
		let mut improved = true;
		while improved {
			improved = false;
			for i in 0..n - 1 {
				for j in i + 2..n {
					let a = order[i];
					let b = order[i + 1];
					let c = order[j];
					let e = order[(j + 1) % n];
					if d(a, c) + d(b, e) < d(a, b) + d(c, e) {
						order[i + 1..=j].reverse();
						improved = true;
					}
				}
			}
		}
		(0..n)
			.map(|i| (self.towns[order[i]].at, self.towns[order[(i + 1) % n]].at))
			.collect()
	}
}

#[derive(Clone, Copy, Default)]
struct RegionState {
	instability: u8,
	inked: bool,
}

struct Turn {
	my_score: i32,
	foe_score: i32,
	tracks: Mask,
	my_tracks: Mask,
	foe_tracks: Mask,
	passable: Mask,
	buildable: Mask,
	regions: Vec<RegionState>,
	active: Vec<LinkId>,
	paths: Vec<Mask>,
}

impl Turn {
	fn new(map: &Map) -> Turn {
		Turn {
			my_score: 0,
			foe_score: 0,
			tracks: Mask::EMPTY,
			my_tracks: Mask::EMPTY,
			foe_tracks: Mask::EMPTY,
			passable: Mask::EMPTY,
			buildable: Mask::EMPTY,
			regions: vec![RegionState::default(); map.regions.len()],
			active: Vec::with_capacity(map.links.len()),
			paths: vec![Mask::EMPTY; map.links.len()],
		}
	}

	fn read(&mut self, map: &Map, input: &mut Reader) -> bool {
		if input.at_end() {
			return false;
		}
		self.my_score = input.i32();
		self.foe_score = input.i32();
		self.tracks = Mask::EMPTY;
		self.my_tracks = Mask::EMPTY;
		self.foe_tracks = Mask::EMPTY;
		self.passable = Mask::EMPTY;
		self.buildable = Mask::EMPTY;
		self.active.clear();
		self.paths.fill(Mask::EMPTY);

		for at in map.points() {
			let track = Track::read(input.i8(), map.me);
			let state = RegionState {
				instability: input.u8(),
				inked: input.bool(),
			};
			self.regions[map.region_of(at) as usize] = state;

			match track {
				Some(owner) => {
					self.tracks.insert(at);
					self.passable.insert(at);
					match owner {
						Track::Mine => self.my_tracks.insert(at),
						Track::Foe => self.foe_tracks.insert(at),
						Track::Neutral => {}
					}
				}
				None if map.town_cells.contains(at) => self.passable.insert(at),
				None if !state.inked => self.buildable.insert(at),
				None => {}
			}

			input.town_pairs(|from, to| {
				let Some(link) = map.link_of(from, to) else {
					return;
				};
				if self.paths[link as usize].is_empty() {
					self.active.push(link);
				}
				self.paths[link as usize].insert(at);
			});
		}
		true
	}

	fn can_disrupt(&self, map: &Map, region: RegionId) -> bool {
		!map.regions[region as usize].has_town && !self.regions[region as usize].inked
	}
}

enum Command {
	Place(Point),
	Disrupt(RegionId),
	Wait,
}

impl Display for Command {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			Command::Place((x, y)) => write!(f, "PLACE_TRACKS {x} {y}"),
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

fn walk((ax, ay): Point, (bx, by): Point) -> u16 {
	(ax.abs_diff(bx) + ay.abs_diff(by)) as u16
}

const SCALE: u32 = 100;
const BLOCKED: u32 = u32::MAX;
const STRIDE: u32 = 1 << 14;
const NO_POINT: Point = (u8::MAX, u8::MAX);

struct Tune {
	foe_toll: f64,
	neutral_toll: f64,
	avoid_inst: u8,
	risk: f64,
	power: f64,
	share: f64,
	horizon: f64,
	order: u8,
	spill: u8,
	min_rate: f64,
	length_mode: bool,
	ink: bool,
	ink_swing: f64,
	ink_foe: f64,
	ink_foe_path: f64,
	ink_self: f64,
	ink_self_path: f64,
	ink_plan: f64,
	ink_foe_plan: f64,
	ink_commit: f64,
	ink_floor: f64,
	ink_my_loss: f64,
	ink_alt: f64,
	ink_stick: f64,
	net: bool,
	sched: u8,
	anchor: bool,
	fill: bool,
	ink_plan_all: bool,
	crowd: f64,
	safe: f64,
	foe_region: f64,
	ring: u8,
	insure: f64,
	insure_min: u32,
	load: f64,
	own_load: f64,
	cells: u8,
	step: f64,
	cell_power: f64,
	cell_budget: usize,
	avoid_share: f64,
	gate: f64,
	gate_turns: i32,
	gate_min: u16,
	foe_gate_toll: f64,
	sever: bool,
	shortlist: usize,
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
			foe_toll: knob("BTK_FOE_TOLL", 1.0),
			neutral_toll: knob("BTK_NEU_TOLL", 0.5),
			avoid_inst: knob("BTK_AVOID_INST", 3),
			risk: knob("BTK_RISK", 0.0),
			power: knob("BTK_POWER", 1.0),
			share: knob("BTK_SHARE", 0.0),
			horizon: knob("BTK_HORIZON", 100.0),
			order: knob("BTK_ORDER", 5),
			spill: knob("BTK_SPILL", 3),
			min_rate: knob("BTK_MIN_RATE", 0.0),
			length_mode: knob::<u8>("BTK_LENGTH", 1) != 0,
			ink: knob::<u8>("BTK_INK", 1) != 0,
			ink_swing: knob("BTK_INK_SWING", 1.0),
			ink_foe: knob("BTK_INK_FOE", 1.0),
			ink_foe_path: knob("BTK_INK_FOE_PATH", 1.0),
			ink_self: knob("BTK_INK_SELF", 2.0),
			ink_self_path: knob("BTK_INK_SELF_PATH", 1.0),
			ink_plan: knob("BTK_INK_PLAN", 1.0),
			ink_foe_plan: knob("BTK_INK_FOE_PLAN", 0.5),
			ink_commit: knob("BTK_INK_COMMIT", 0.5),
			ink_floor: knob("BTK_INK_FLOOR", 0.0),
			ink_my_loss: knob("BTK_INK_MY_LOSS", 1.0),
			ink_alt: knob("BTK_INK_ALT", 0.0),
			ink_stick: knob("BTK_INK_STICK", 0.0),
			net: knob::<u8>("BTK_NET", 1) != 0,
			sched: knob("BTK_SCHED", 0),
			anchor: knob::<u8>("BTK_ANCHOR", 0) != 0,
			fill: knob::<u8>("BTK_FILL", 1) != 0,
			ink_plan_all: knob::<u8>("BTK_INK_PLAN_ALL", 0) != 0,
			crowd: knob("BTK_CROWD", 0.0),
			safe: knob("BTK_SAFE", 0.0),
			foe_region: knob("BTK_FOE_REGION", 0.0),
			ring: knob("BTK_RING", 0),
			insure: knob("BTK_INSURE", 0.0),
			insure_min: knob("BTK_INSURE_MIN", 6),
			load: knob("BTK_LOAD", 0.0),
			own_load: knob("BTK_OWN_LOAD", 0.0),
			cells: knob("BTK_CELLS", 2),
			step: knob("BTK_STEP", 0.3),
			cell_power: knob("BTK_CELL_POWER", 1.0),
			cell_budget: knob("BTK_CELL_BUDGET", 400),
			avoid_share: knob("BTK_AVOID_SHARE", 0.0),
			gate: knob("BTK_GATE", 0.0),
			gate_turns: knob("BTK_GATE_TURNS", 100),
			gate_min: knob("BTK_GATE_MIN", 3),
			foe_gate_toll: knob("BTK_FOE_GATE_TOLL", 0.0),
			sever: knob::<u8>("BTK_SEVER", 1) != 0,
			shortlist: knob("BTK_SHORTLIST", 14),
		}
	}
}

struct Trains {
	came: Field<Point>,
	queue: Vec<Point>,
	dist: Vec<Field<u16>>,
	base_links: Vec<(u32, u32, u16)>,
}

const UNREACHABLE: u16 = u16::MAX;

struct Income {
	mine: u32,
	foe: u32,
}

impl Trains {
	fn new(map: &Map) -> Trains {
		Trains {
			came: Field::new(map.width, map.height, NO_POINT),
			queue: Vec::with_capacity(map.width as usize * map.height as usize),
			dist: (0..map.towns.len())
				.map(|_| Field::new(map.width, map.height, UNREACHABLE))
				.collect(),
			base_links: vec![(0, 0, UNREACHABLE); map.links.len()],
		}
	}

	fn prepare(&mut self, map: &Map, board: &Board) {
		for (town, field) in map.towns.iter().zip(self.dist.iter_mut()) {
			field.values.fill(UNREACHABLE);
			field[town.at] = 0;
			self.queue.clear();
			self.queue.push(town.at);
			let mut head = 0;
			while head < self.queue.len() {
				let at = self.queue[head];
				head += 1;
				let next_dist = field[at] + 1;
				for next in map.neighbours(at) {
					if board.passable.contains(next) && field[next] == UNREACHABLE {
						field[next] = next_dist;
						self.queue.push(next);
					}
				}
			}
		}
		for link in 0..map.links.len() {
			let (_, goal) = map.endpoints_of(link as LinkId);
			let length = self.dist[map.links[link].from as usize][goal];
			let (mine, foe) = if length == UNREACHABLE {
				(0, 0)
			} else {
				self.paid_on(map, board.passable, board.mine, board.foe, link as LinkId)
			};
			self.base_links[link] = (mine, foe, length);
		}
	}

	fn cell_gain(&mut self, map: &Map, board: &Board, at: Point) -> (i64, i64) {
		let mut passable = board.passable;
		let mut mine = board.mine;
		passable.insert(at);
		mine.insert(at);
		let mut delta = (0i64, 0i64);
		for link in 0..map.links.len() {
			let Link { from, to } = map.links[link];
			let (base_mine, base_foe, length) = self.base_links[link];
			let near = |field: &Field<u16>| -> u16 {
				let mut best = UNREACHABLE;
				for next in map.neighbours(at) {
					if board.passable.contains(next) {
						best = best.min(field[next]);
					}
				}
				best
			};
			let da = near(&self.dist[from as usize]);
			let db = near(&self.dist[to as usize]);
			if da == UNREACHABLE || db == UNREACHABLE {
				continue;
			}
			let via = da as u32 + db as u32 + 2;
			if length != UNREACHABLE && via > length as u32 {
				continue;
			}
			let (new_mine, new_foe) = self.paid_on(map, passable, mine, board.foe, link as LinkId);
			delta.0 += new_mine as i64 - base_mine as i64;
			delta.1 += new_foe as i64 - base_foe as i64;
		}
		delta
	}

	fn income(&mut self, map: &Map, passable: Mask, mine: Mask, foe: Mask) -> Income {
		let mut total = Income { mine: 0, foe: 0 };
		for link in 0..map.links.len() as LinkId {
			let (a, b) = self.paid_on(map, passable, mine, foe, link);
			total.mine += a;
			total.foe += b;
		}
		total
	}

	fn paid_on(
		&mut self,
		map: &Map,
		passable: Mask,
		mine: Mask,
		foe: Mask,
		link: LinkId,
	) -> (u32, u32) {
		let (start, goal) = map.endpoints_of(link);
		let mut seen = Mask::EMPTY;
		self.queue.clear();
		self.queue.push(start);
		seen.insert(start);
		self.came[start] = NO_POINT;

		let mut head = 0;
		while head < self.queue.len() {
			let at = self.queue[head];
			head += 1;
			if at == goal {
				let mut paid = (0, 0);
				let mut walk = goal;
				while walk != NO_POINT {
					paid.0 += u32::from(mine.contains(walk));
					paid.1 += u32::from(foe.contains(walk));
					walk = self.came[walk];
				}
				return paid;
			}
			for next in map.neighbours(at) {
				if !passable.contains(next) || seen.contains(next) {
					continue;
				}
				seen.insert(next);
				self.came[next] = at;
				self.queue.push(next);
			}
		}
		(0, 0)
	}
}

struct Board {
	passable: Mask,
	mine: Mask,
	foe: Mask,
	tracks: Mask,
}

impl Board {
	fn of(turn: &Turn) -> Board {
		Board {
			passable: turn.passable,
			mine: turn.my_tracks,
			foe: turn.foe_tracks,
			tracks: turn.tracks,
		}
	}

	fn mirrored(&self) -> Board {
		Board {
			passable: self.passable,
			mine: self.foe,
			foe: self.mine,
			tracks: self.tracks,
		}
	}

	fn claim(&mut self, at: Point) {
		self.passable.insert(at);
		self.mine.insert(at);
		self.tracks.insert(at);
	}
}

struct Route {
	cells: Vec<Point>,
	paint: u32,
	length: u32,
}

struct Router {
	load: Vec<u32>,
	cell_load: Field<u8>,
	step: u32,
	cost: Field<u32>,
	dist: Field<u32>,
	came: Field<Point>,
	heap: BinaryHeap<Reverse<(u32, Point)>>,
}

impl Router {
	fn new(map: &Map) -> Router {
		Router {
			load: vec![0; map.regions.len()],
			cell_load: Field::new(map.width, map.height, 0),
			step: 0,
			cost: Field::new(map.width, map.height, BLOCKED),
			dist: Field::new(map.width, map.height, BLOCKED),
			came: Field::new(map.width, map.height, NO_POINT),
			heap: BinaryHeap::new(),
		}
	}

	fn price(
		&mut self,
		map: &Map,
		turn: &Turn,
		tune: &Tune,
		board: &Board,
		share: Option<&Field<u16>>,
	) {
		self.step = (tune.step * SCALE as f64) as u32;
		let mut crowd = vec![0u8; map.regions.len()];
		if tune.crowd != 0.0 {
			for at in board.mine.iter() {
				crowd[map.region_of(at) as usize] += 1;
			}
		}
		let mut foe_crowd = vec![0u8; map.regions.len()];
		if tune.foe_region != 0.0 {
			for at in board.foe.iter() {
				foe_crowd[map.region_of(at) as usize] += 1;
			}
		}
		for at in map.points() {
			let region = map.region_of(at) as usize;
			let state = turn.regions[region];
			let townless = !map.regions[region].has_town;
			self.cost[at] = if map.town_cells.contains(at) {
				0
			} else if board.mine.contains(at) {
				let mult = if townless {
					self.cell_load[at] as f64
				} else {
					0.0
				};
				(tune.own_load * mult * SCALE as f64 / 10.0) as u32
			} else if board.foe.contains(at) {
				let gate = map.neighbours(at).any(|next| map.town_cells.contains(next));
				let toll = if gate {
					tune.foe_toll + tune.foe_gate_toll
				} else {
					tune.foe_toll
				};
				(toll * SCALE as f64) as u32
			} else if board.tracks.contains(at) {
				(tune.neutral_toll * SCALE as f64) as u32
			} else if state.inked
				|| !turn.buildable.contains(at)
				|| (townless && state.instability >= tune.avoid_inst)
			{
				BLOCKED
			} else {
				let mut paint = map.paint_cost(at) as f64;
				if townless {
					paint *= 1.0 + tune.risk * state.instability as f64;
					paint *= 1.0 + tune.crowd * crowd[region] as f64;
					paint *= 1.0 + tune.foe_region * foe_crowd[region] as f64;
					paint *= 1.0 + tune.load * self.load[region] as f64 / 10.0;
				} else {
					paint *= 1.0 - tune.safe;
				}
				if let Some(share) = share {
					paint /= 1.0 + tune.share * share[at] as f64;
					paint *= 1.0 + tune.avoid_share * share[at] as f64;
				}
				(paint * SCALE as f64) as u32
			};
		}
	}

	fn flood(&mut self, map: &Map, start: Point, goal: Point, stride: u32) {
		self.dist.values.fill(BLOCKED);
		self.heap.clear();
		self.dist[start] = 0;
		self.heap.push(Reverse((0, start)));
		while let Some(Reverse((spent, at))) = self.heap.pop() {
			if at == goal {
				return;
			}
			if spent > self.dist[at] {
				continue;
			}
			for next in map.neighbours(at) {
				let toll = self.cost[next];
				if toll == BLOCKED {
					continue;
				}
				let total = spent
					+ stride + if stride == 0 {
					toll + 1 + self.step
				} else {
					toll / SCALE
				};
				if total < self.dist[next] {
					self.dist[next] = total;
					self.came[next] = at;
					self.heap.push(Reverse((total, next)));
				}
			}
		}
	}

	fn referee_route(
		&mut self,
		map: &Map,
		board: &Board,
		start: Point,
		goal: Point,
	) -> Option<Route> {
		self.dist.values.fill(BLOCKED);
		let mut queue: Vec<Point> = Vec::with_capacity(map.width as usize * map.height as usize);
		self.dist[start] = 0;
		queue.push(start);
		let mut head = 0;
		let mut found = false;
		while head < queue.len() {
			let at = queue[head];
			head += 1;
			if at == goal {
				found = true;
				break;
			}
			for next in map.neighbours(at) {
				if self.dist[next] != BLOCKED {
					continue;
				}
				let allowed = board.passable.contains(next) || self.cost[next] != BLOCKED;
				if !allowed {
					continue;
				}
				self.dist[next] = self.dist[at] + 1;
				self.came[next] = at;
				queue.push(next);
			}
		}
		if !found {
			return None;
		}
		let mut cells = Vec::new();
		let mut paint = 0;
		let mut length = 0;
		let mut at = goal;
		while at != start {
			if !board.passable.contains(at) {
				paint += map.paint_cost(at) as u32;
				cells.push(at);
			}
			length += 1;
			at = self.came[at];
		}
		cells.reverse();
		Some(Route {
			cells,
			paint,
			length,
		})
	}

	fn route(
		&mut self,
		map: &Map,
		board: &Board,
		start: Point,
		goal: Point,
		stride: u32,
	) -> Option<Route> {
		if stride > 0 {
			return self.referee_route(map, board, start, goal);
		}
		self.flood(map, start, goal, stride);
		if self.dist[goal] == BLOCKED {
			return None;
		}
		let mut cells = Vec::new();
		let mut paint = 0;
		let mut length = 0;
		let mut at = goal;
		while at != start {
			if !board.passable.contains(at) {
				paint += map.paint_cost(at) as u32;
				cells.push(at);
			}
			length += 1;
			at = self.came[at];
		}
		cells.reverse();
		Some(Route {
			cells,
			paint,
			length,
		})
	}
}

struct Candidate {
	link: LinkId,
	route: Route,
	gain: f64,
	score: f64,
}

struct PlannedLink {
	link: LinkId,
	cells: Vec<Point>,
	paint: u32,
	owned: u32,
}

struct Planner {
	router: Router,
	trains: Trains,
	share: Field<u16>,
	planned: Mask,
	committed: Mask,
	foe_planned: Mask,
	plan: Vec<PlannedLink>,
	tour: Vec<(Point, Point)>,
	alt: Field<u16>,
	mult: Field<u8>,
	flood_seen: Mask,
	flood_queue: Vec<Point>,
}

impl Planner {
	fn new(map: &Map) -> Planner {
		Planner {
			router: Router::new(map),
			trains: Trains::new(map),
			share: Field::new(map.width, map.height, 0),
			planned: Mask::EMPTY,
			committed: Mask::EMPTY,
			foe_planned: Mask::EMPTY,
			plan: Vec::new(),
			tour: map.tour(),
			alt: Field::new(map.width, map.height, 0),
			mult: Field::new(map.width, map.height, 0),
			flood_seen: Mask::EMPTY,
			flood_queue: Vec::new(),
		}
	}

	fn forecast(
		&mut self,
		map: &Map,
		turn: &Turn,
		tune: &Tune,
		board: &Board,
	) -> (Field<u16>, Mask, Vec<PlannedLink>) {
		let mut share = Field::new(map.width, map.height, 0u16);
		let mut planned = Mask::EMPTY;
		let mut links = Vec::new();
		self.router.price(map, turn, tune, board, None);
		let mut order: Vec<(u32, LinkId)> = Vec::with_capacity(map.links.len());
		for link in 0..map.links.len() as LinkId {
			let (start, goal) = map.endpoints_of(link);
			if let Some(route) = self.router.route(map, board, start, goal, 0) {
				order.push((route.paint, link));
			}
		}
		order.sort_unstable();
		let mut grown = Board {
			passable: board.passable,
			mine: board.mine,
			foe: board.foe,
			tracks: board.tracks,
		};
		for (_, link) in order {
			let (start, goal) = map.endpoints_of(link);
			self.router.price(map, turn, tune, &grown, None);
			let Some(route) = self.router.route(map, &grown, start, goal, 0) else {
				continue;
			};
			let mut at = goal;
			let mut owned = 0;
			while at != start {
				share[at] += 1;
				owned += u32::from(grown.mine.contains(at) || !grown.passable.contains(at));
				at = self.router.came[at];
			}
			share[start] += 1;
			for &at in &route.cells {
				grown.claim(at);
				planned.insert(at);
			}
			links.push(PlannedLink {
				link,
				cells: route.cells,
				paint: route.paint,
				owned,
			});
		}
		(share, planned, links)
	}

	fn alternatives(
		&mut self,
		map: &Map,
		turn: &Turn,
		tune: &Tune,
		board: &Board,
		links: &[PlannedLink],
	) -> Field<u16> {
		let mut alt = Field::new(map.width, map.height, 0u16);
		let blocked = Board {
			passable: board.passable,
			mine: board.mine,
			foe: board.foe,
			tracks: board.tracks,
		};
		self.router.price(map, turn, tune, &blocked, None);
		for at in (self.planned | board.mine).iter() {
			self.router.cost[at] = BLOCKED;
		}
		for planned in links {
			let (start, goal) = map.endpoints_of(planned.link);
			if self.router.route(map, &blocked, start, goal, 0).is_none() {
				continue;
			}
			let mut at = goal;
			while at != start {
				alt[at] += 1;
				at = self.router.came[at];
			}
		}
		alt
	}

	fn game_goes_on(&mut self, map: &Map, turn: &Turn, inked: RegionId) -> bool {
		let mut open = Mask::EMPTY;
		for at in map.points() {
			let region = map.region_of(at);
			if region != inked && !turn.regions[region as usize].inked {
				open.insert(at);
			}
		}
		for at in map.town_cells.iter() {
			open.insert(at);
		}
		for link in 0..map.links.len() as LinkId {
			let (start, goal) = map.endpoints_of(link);
			self.flood_seen = Mask::EMPTY;
			self.flood_queue.clear();
			self.flood_queue.push(start);
			self.flood_seen.insert(start);
			let mut head = 0;
			while head < self.flood_queue.len() {
				let at = self.flood_queue[head];
				head += 1;
				if at == goal {
					return true;
				}
				for next in map.neighbours(at) {
					if open.contains(next) && !self.flood_seen.contains(next) {
						self.flood_seen.insert(next);
						self.flood_queue.push(next);
					}
				}
			}
		}
		false
	}

	#[allow(clippy::too_many_arguments)]
	fn candidates(
		&mut self,
		map: &Map,
		turn: &Turn,
		tune: &Tune,
		board: &Board,
		base: &Income,
		left: f64,
		spent: u32,
		count: i32,
		found: &mut Vec<Candidate>,
	) {
		found.clear();
		if tune.gate > 0.0 && count <= tune.gate_turns {
			let earning = left.min(tune.horizon) - 1.0;
			for town in &map.towns {
				for at in map.neighbours(town.at) {
					if !turn.buildable.contains(at)
						|| board.tracks.contains(at)
						|| self.share[at] < tune.gate_min
					{
						continue;
					}
					let paint = map.paint_cost(at) as u32;
					let gain = tune.gate * self.share[at] as f64;
					found.push(Candidate {
						link: NO_LINK,
						route: Route {
							cells: vec![at],
							paint,
							length: 1,
						},
						gain,
						score: gain * earning / (paint as f64).powf(tune.power),
					});
				}
			}
		}
		let share = std::mem::replace(&mut self.share, Field::new(1, 1, 0));
		self.router.price(map, turn, tune, board, Some(&share));
		self.share = share;
		let modes: &[u32] = if tune.length_mode { &[0, STRIDE] } else { &[0] };
		let mut routes: Vec<(LinkId, Route)> = Vec::with_capacity(map.links.len() * modes.len());
		if tune.ring > 0 {
			for (index, &(start, goal)) in self.tour.iter().enumerate() {
				let Some(route) = self.router.route(map, board, start, goal, 0) else {
					continue;
				};
				if route.paint == 0 {
					continue;
				}
				routes.push((RING_LINK + index as LinkId, route));
			}
		}
		for link in 0..map.links.len() as LinkId {
			if tune.ring == 1 {
				break;
			}
			let (start, goal) = map.endpoints_of(link);
			for &stride in modes {
				let Some(route) = self.router.route(map, board, start, goal, stride) else {
					continue;
				};
				if route.paint == 0 {
					continue;
				}
				if routes
					.iter()
					.any(|(other, known)| *other == link && known.cells == route.cells)
				{
					continue;
				}
				routes.push((link, route));
			}
		}
		routes.sort_by_key(|(link, route)| (route.paint, *link));
		routes.truncate(tune.shortlist);
		let mut insured: Option<(f64, Route)> = None;
		if tune.insure > 0.0
			&& let Some((at_risk, route)) = self.bypass(map, turn, tune, board)
		{
			insured = Some((at_risk, route));
		}
		if let Some((at_risk, route)) = insured {
			let build_turns = ((route.paint + spent) as f64 / PAINT_PER_TURN as f64).ceil();
			let earning = left.min(tune.horizon) - build_turns;
			if earning > 0.0 {
				let gain = tune.insure * at_risk;
				found.push(Candidate {
					link: BYPASS_LINK,
					score: gain * earning / (route.paint as f64).powf(tune.power),
					route,
					gain,
				});
			}
		}
		for (link, route) in routes {
			let mut trial = Board {
				passable: board.passable,
				mine: board.mine,
				foe: board.foe,
				tracks: board.tracks,
			};
			for &at in &route.cells {
				trial.claim(at);
			}
			let after = self
				.trains
				.income(map, trial.passable, trial.mine, trial.foe);
			let mut gain = after.mine as f64 - base.mine as f64;
			if tune.net {
				gain -= after.foe as f64 - base.foe as f64;
			}
			if gain <= 0.0 {
				continue;
			}
			let build_turns = ((route.paint + spent) as f64 / PAINT_PER_TURN as f64).ceil();
			let earning = left.min(tune.horizon) - build_turns;
			if earning <= 0.0 {
				continue;
			}
			let score = gain * earning / (route.paint as f64).powf(tune.power);
			found.push(Candidate {
				link,
				route,
				gain,
				score,
			});
		}
		if tune.cells > 0 {
			let earning = left.min(tune.horizon) - 1.0;
			if earning > 0.0 {
				let mut pool: Vec<(u16, Point)> = Vec::new();
				for at in turn.buildable.iter() {
					if board.tracks.contains(at) {
						continue;
					}
					let region = map.region_of(at) as usize;
					if !map.regions[region].has_town
						&& turn.regions[region].instability >= tune.avoid_inst
					{
						continue;
					}
					let mut near_mine = false;
					let mut near_path = 0u16;
					let mut touching = false;
					for next in map.neighbours(at) {
						if board.passable.contains(next) {
							touching = true;
						}
						if board.mine.contains(next) || map.town_cells.contains(next) {
							near_mine = true;
						}
						near_path += self.mult[next] as u16;
					}
					if !touching || (tune.cells == 1 && !near_mine) {
						continue;
					}
					pool.push((near_path + self.share[at], at));
				}
				pool.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
				pool.truncate(tune.cell_budget / (1 + spent as usize));
				self.trains.prepare(map, board);
				for (_, at) in pool {
					let (mine_delta, foe_delta) = self.trains.cell_gain(map, board, at);
					let mut gain = mine_delta as f64;
					if tune.net {
						gain -= foe_delta as f64;
					}
					if gain <= 0.0 {
						continue;
					}
					let paint = map.paint_cost(at) as u32;
					found.push(Candidate {
						link: CELL_LINK,
						route: Route {
							cells: vec![at],
							paint,
							length: 1,
						},
						gain,
						score: gain * earning / (paint as f64).powf(tune.cell_power),
					});
				}
			}
		}
		found.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.link.cmp(&b.link)));
	}

	fn ordered(&mut self, map: &Map, tune: &Tune, board: &Board, route: &Route) -> Vec<Point> {
		let mut cells = route.cells.clone();
		match tune.order {
			1 => cells.reverse(),
			2 => {
				let anchored = |at: Point| {
					map.neighbours(at)
						.any(|next| board.mine.contains(next) || map.town_cells.contains(next))
				};
				let head = cells.first().is_some_and(|&at| anchored(at));
				let tail = cells.last().is_some_and(|&at| anchored(at));
				if tail && !head {
					cells.reverse();
				}
			}
			3 => cells.sort_by_key(|&at| Reverse(self.share[at])),
			4 => cells.sort_by_key(|&at| {
				Reverse(
					map.neighbours(at)
						.filter(|&next| board.foe.contains(next))
						.count(),
				)
			}),
			5 => return self.by_segment(map, board, route),
			_ => {}
		}
		cells
	}

	fn by_segment(&mut self, map: &Map, board: &Board, route: &Route) -> Vec<Point> {
		let mut segments: Vec<Vec<Point>> = vec![Vec::new()];
		let mut previous: Option<Point> = None;
		for &at in &route.cells {
			if let Some(last) = previous
				&& !map.neighbours(last).any(|next| next == at)
			{
				segments.push(Vec::new());
			}
			segments.last_mut().unwrap().push(at);
			previous = Some(at);
		}
		if segments.len() == 1 {
			return route.cells.clone();
		}
		let base = self
			.trains
			.income(map, board.passable, board.mine, board.foe);
		let mut rated: Vec<(f64, f64, usize)> = Vec::with_capacity(segments.len());
		for (index, segment) in segments.iter().enumerate() {
			let mut trial = Board {
				passable: board.passable,
				mine: board.mine,
				foe: board.foe,
				tracks: board.tracks,
			};
			let mut paint = 0.0;
			for &at in segment {
				trial.claim(at);
				paint += map.paint_cost(at) as f64;
			}
			let after = self
				.trains
				.income(map, trial.passable, trial.mine, trial.foe);
			let gain = after.mine as f64 - base.mine as f64 - (after.foe as f64 - base.foe as f64);
			rated.push((gain / paint, paint, index));
		}
		rated.sort_by(|a, b| {
			b.0.total_cmp(&a.0)
				.then(a.1.total_cmp(&b.1))
				.then(a.2.cmp(&b.2))
		});
		rated
			.into_iter()
			.flat_map(|(_, _, index)| segments[index].iter().copied())
			.collect()
	}

	fn bypass(
		&mut self,
		map: &Map,
		turn: &Turn,
		tune: &Tune,
		board: &Board,
	) -> Option<(f64, Route)> {
		let mut load = vec![0u32; map.regions.len()];
		for at in board.mine.iter() {
			let region = map.region_of(at) as usize;
			if !map.regions[region].has_town && !turn.regions[region].inked {
				load[region] += self.mult[at] as u32;
			}
		}
		let (region, &at_risk) = load.iter().enumerate().max_by_key(|(_, l)| **l)?;
		if at_risk < tune.insure_min {
			return None;
		}
		let cells = map.regions[region].cells;
		let mut anchors: Vec<(u8, Point)> = Vec::new();
		for at in cells.iter() {
			if !board.mine.contains(at) || self.mult[at] == 0 {
				continue;
			}
			for next in map.neighbours(at) {
				if !cells.contains(next)
					&& board.passable.contains(next)
					&& !board.foe.contains(next)
				{
					anchors.push((self.mult[at], next));
				}
			}
		}
		anchors.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
		anchors.dedup_by_key(|a| a.1);
		if anchors.len() < 2 {
			return None;
		}
		let start = anchors[0].1;
		let mut best: Option<Route> = None;
		for &(_, goal) in anchors.iter().skip(1).take(3) {
			if map.region_of(goal) == map.region_of(start) && cells.contains(goal) {
				continue;
			}
			self.router.price(map, turn, tune, board, None);
			for at in cells.iter() {
				self.router.cost[at] = BLOCKED;
			}
			let Some(route) = self.router.route(map, board, start, goal, 0) else {
				continue;
			};
			if route.paint == 0 || route.paint > 12 {
				continue;
			}
			if best.as_ref().is_none_or(|top| route.paint < top.paint) {
				best = Some(route);
			}
		}
		best.map(|route| (at_risk as f64, route))
	}

	fn hub_cells(&self, map: &Map, tune: &Tune, board: &Board, left: f64) -> Vec<(f64, Point)> {
		let mut score = Field::new(map.width, map.height, 0.0f64);
		for planned in &self.plan {
			if planned.paint == 0 {
				continue;
			}
			let build_turns = (planned.paint as f64 / PAINT_PER_TURN as f64).ceil();
			let earning = left.min(tune.horizon) - build_turns;
			if earning <= 0.0 {
				continue;
			}
			let rate = planned.owned as f64 * earning / (planned.paint as f64).powf(tune.power);
			for &at in &planned.cells {
				score[at] += rate;
			}
		}
		let mut cells: Vec<(f64, Point)> = self
			.planned
			.iter()
			.filter(|&at| !board.passable.contains(at))
			.filter(|&at| {
				!tune.anchor
					|| map
						.neighbours(at)
						.any(|next| board.mine.contains(next) || map.town_cells.contains(next))
			})
			.map(|at| (score[at] / map.paint_cost(at) as f64, at))
			.collect();
		cells.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
		cells
	}

	fn pick_ink(
		&mut self,
		map: &Map,
		turn: &Turn,
		tune: &Tune,
		board: &Board,
		base: &Income,
		log: &mut Option<std::fs::File>,
	) -> Option<RegionId> {
		let mut best: Option<(f64, RegionId)> = None;
		let mut fallback: Option<(f64, RegionId)> = None;
		for (id, region) in map.regions.iter().enumerate() {
			let id = id as RegionId;
			if !turn.can_disrupt(map, id) {
				continue;
			}
			let mut foe = 0.0;
			let mut mine = 0.0;
			let mut plan = 0.0;
			let mut foe_plan = 0.0;
			let mut has_track = false;
			for at in region.cells.iter() {
				let paid = self.mult[at] as f64;
				if board.foe.contains(at) {
					foe += tune.ink_foe + tune.ink_foe_path * paid;
					has_track |= paid > 0.0;
				} else if board.mine.contains(at) {
					mine += tune.ink_self + tune.ink_self_path * paid;
					has_track |= paid > 0.0;
				} else if board.tracks.contains(at) {
					has_track |= paid > 0.0;
				}
				if (tune.ink_plan_all && self.planned.contains(at)) || self.committed.contains(at) {
					plan += tune.ink_plan;
				}
				if self.foe_planned.contains(at) {
					foe_plan += tune.ink_foe_plan;
				}
				foe_plan += tune.ink_alt * self.alt[at] as f64;
			}
			let swing = if has_track {
				let passable = board.passable - region.cells;
				let after = self.trains.income(map, passable, board.mine, board.foe);
				let foe_loss = base.foe as f64 - after.foe as f64;
				let my_loss = base.mine as f64 - after.mine as f64;
				foe_loss - tune.ink_my_loss * my_loss
			} else {
				0.0
			};
			let mut value = tune.ink_swing * swing + foe - mine - plan + foe_plan;
			let charged = turn.regions[id as usize].instability as f64;
			if tune.sever && charged >= 3.0 && !self.game_goes_on(map, turn, id) {
				if turn.my_score > turn.foe_score {
					return Some(id);
				}
				value = f64::MIN / 4.0;
			}
			let score = value * (1.0 + tune.ink_commit * charged) + tune.ink_stick * charged;
			if let Some(log) = log {
				let _ = writeln!(
					log,
					"  ink r{id} inst {charged} swing {swing:.0} foe {foe:.1} mine {mine:.1} plan {plan:.1} foeplan {foe_plan:.1} -> {score:.1}"
				);
			}
			if value > tune.ink_floor && best.is_none_or(|(top, _)| score > top) {
				best = Some((score, id));
			}
			if fallback.is_none_or(|(top, _)| score > top) {
				fallback = Some((score, id));
			}
		}
		best.or(fallback).map(|(_, id)| id)
	}
}

#[allow(clippy::too_many_arguments)]
fn decide(
	map: &Map,
	turn: &Turn,
	tune: &Tune,
	planner: &mut Planner,
	count: i32,
	commands: &mut Vec<Command>,
	found: &mut Vec<Candidate>,
	log: &mut Option<std::fs::File>,
) {
	commands.clear();
	let clock = std::time::Instant::now();
	if let Some(log) = log {
		let _ = writeln!(
			log,
			"turn {count} score {} {}",
			turn.my_score, turn.foe_score
		);
		let mut split = [0u32; 6];
		for &link in &turn.active {
			for at in turn.paths[link as usize].iter() {
				let gate = map.neighbours(at).any(|next| map.town_cells.contains(next));
				let near_mine = map.neighbours(at).any(|next| turn.my_tracks.contains(next));
				let near_foe = map
					.neighbours(at)
					.any(|next| turn.foe_tracks.contains(next));
				if turn.foe_tracks.contains(at) {
					split[if gate {
						0
					} else if near_mine {
						1
					} else {
						2
					}] += 1;
				} else if turn.my_tracks.contains(at) {
					split[if gate {
						3
					} else if near_foe {
						4
					} else {
						5
					}] += 1;
				}
			}
		}
		let _ = writeln!(
			log,
			"  income foe gate {} nearmine {} other {} | mine gate {} nearfoe {} other {}",
			split[0], split[1], split[2], split[3], split[4], split[5]
		);
	}
	let left = (MAX_TURNS - count + 1).max(1) as f64;
	let mut board = Board::of(turn);

	for at in map.points() {
		planner.mult[at] = 0;
	}
	for &link in &turn.active {
		for at in turn.paths[link as usize].iter() {
			planner.mult[at] = planner.mult[at].saturating_add(1);
		}
	}

	planner.router.load.fill(0);
	for at in map.points() {
		planner.router.cell_load[at] = planner.mult[at];
	}
	if tune.load != 0.0 {
		for at in turn.my_tracks.iter() {
			planner.router.load[map.region_of(at) as usize] += planner.mult[at] as u32;
		}
	}
	let (share, planned, links) = planner.forecast(map, turn, tune, &board);
	planner.share = share;
	planner.planned = planned;
	planner.plan = links;
	if tune.ink_foe_plan != 0.0 {
		let (_, foe_planned, _) = planner.forecast(map, turn, tune, &board.mirrored());
		planner.foe_planned = foe_planned;
	}
	if tune.ink_alt != 0.0 {
		let plan = std::mem::take(&mut planner.plan);
		planner.alt = planner.alternatives(map, turn, tune, &board, &plan);
		planner.plan = plan;
	}

	if let Some(log) = log {
		let _ = writeln!(
			log,
			"  stage forecast {:.1}ms",
			clock.elapsed().as_secs_f64() * 1000.0
		);
	}
	let mut base = planner
		.trains
		.income(map, board.passable, board.mine, board.foe);
	let ink_base = Income {
		mine: base.mine,
		foe: base.foe,
	};

	let mut budget = PAINT_PER_TURN;
	let mut spent = 0;
	let mut chosen = 0;
	planner.committed = Mask::EMPTY;
	if tune.sched == 1 {
		let cells = planner.hub_cells(map, tune, &board, left);
		if let Some(log) = log {
			for (score, at) in cells.iter().take(6) {
				let _ = writeln!(log, "  hub {at:?} {score:.1}");
			}
		}
		for (_, at) in cells {
			let cost = map.paint_cost(at);
			if cost > budget {
				continue;
			}
			budget -= cost;
			board.claim(at);
			commands.push(Command::Place(at));
			if budget == 0 {
				break;
			}
		}
	}
	while budget > 0 && chosen < tune.spill {
		planner.candidates(map, turn, tune, &board, &base, left, spent, count, found);
		let Some(best) = found.first() else {
			break;
		};
		if best.score <= tune.min_rate {
			break;
		}
		if let Some(log) = log {
			for cand in found.iter().take(6) {
				let Link { from, to } = if cand.link >= RING_LINK {
					Link {
						from: 99,
						to: cand.link - RING_LINK,
					}
				} else if cand.link == BYPASS_LINK {
					Link { from: 88, to: 88 }
				} else if cand.link == CELL_LINK {
					Link { from: 77, to: 77 }
				} else {
					map.links[cand.link as usize]
				};
				let _ = writeln!(
					log,
					"  link {}->{} gain {:.0} paint {} len {} score {:.2} cells {:?}",
					from,
					to,
					cand.gain,
					cand.route.paint,
					cand.route.length,
					cand.score,
					cand.route.cells
				);
			}
		}
		let cells = planner.ordered(map, tune, &board, &best.route);
		for &at in &cells {
			planner.committed.insert(at);
		}
		let mut placed = 0;
		for at in cells {
			let cost = map.paint_cost(at);
			if cost > budget {
				continue;
			}
			budget -= cost;
			spent += cost as u32;
			board.claim(at);
			commands.push(Command::Place(at));
			placed += 1;
		}
		chosen += 1;
		if placed < best.route.cells.len() {
			break;
		}
		base = planner
			.trains
			.income(map, board.passable, board.mine, board.foe);
	}

	if let Some(log) = log {
		let _ = writeln!(
			log,
			"  stage build {:.1}ms",
			clock.elapsed().as_secs_f64() * 1000.0
		);
	}
	if tune.fill && budget > 0 {
		let cells = planner.hub_cells(map, tune, &board, left);
		for (_, at) in cells {
			let cost = map.paint_cost(at);
			if cost > budget {
				continue;
			}
			budget -= cost;
			board.claim(at);
			planner.committed.insert(at);
			commands.push(Command::Place(at));
			if budget == 0 {
				break;
			}
		}
	}

	if let Some(log) = log {
		let _ = writeln!(
			log,
			"  stage fill {:.1}ms",
			clock.elapsed().as_secs_f64() * 1000.0
		);
	}
	if tune.ink
		&& let Some(region) = planner.pick_ink(map, turn, tune, &Board::of(turn), &ink_base, log)
	{
		commands.push(Command::Disrupt(region));
	}
}

fn main() {
	let mut input = Reader::new();
	let mut out = io::stdout().lock();
	let map = Map::read(&mut input);
	input.flush_echo();
	let tune = Tune::load();
	let mut planner = Planner::new(&map);
	let mut turn = Turn::new(&map);
	let mut commands = Vec::with_capacity(4);
	let mut found = Vec::with_capacity(map.links.len() * 2);
	let mut count = 0;
	let mut log = std::env::var("BTK_LOG")
		.ok()
		.and_then(|path| std::fs::File::create(path).ok());

	while turn.read(&map, &mut input) {
		count += 1;
		let started = std::time::Instant::now();
		decide(
			&map,
			&turn,
			&tune,
			&mut planner,
			count,
			&mut commands,
			&mut found,
			&mut log,
		);
		send(&commands, &mut out).unwrap();
		if let Some(log) = &mut log {
			let _ = writeln!(
				log,
				"  elapsed {:.1}ms",
				started.elapsed().as_secs_f64() * 1000.0
			);
		}
	}
}
