use std::collections::BinaryHeap;
use std::fmt::Display;
use std::io::{self, Read, Write};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, IndexMut, Sub, SubAssign};

const MAX_WIDTH: u8 = 30;
const MAX_HEIGHT: u8 = 20;
const ROWS: usize = MAX_HEIGHT as usize;

const PAINT_PER_TURN: u8 = 3;
const INSTABILITY_THRESHOLD: u8 = 4;
const MAX_TURNS: i32 = 100;

const NO_TRACK: i8 = -1;
const NEUTRAL_TRACK: i8 = 2;

type Point = (u8, u8);
type TownId = u8;
type LinkId = u8;
type RegionId = u8;

const NO_LINK: LinkId = LinkId::MAX;

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

	fn _remove(&mut self, (x, y): Point) {
		self.0[y as usize] &= !(1u64 << x);
	}

	fn is_empty(&self) -> bool {
		self.0.iter().all(|row| *row == 0)
	}

	fn len(&self) -> u32 {
		self.0.iter().map(|row| row.count_ones()).sum()
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

	fn _owner_of(&self, at: Point) -> Option<Track> {
		if self.my_tracks.contains(at) {
			Some(Track::Mine)
		} else if self.foe_tracks.contains(at) {
			Some(Track::Foe)
		} else if self.tracks.contains(at) {
			Some(Track::Neutral)
		} else {
			None
		}
	}

	fn can_disrupt(&self, map: &Map, region: RegionId) -> bool {
		!map.regions[region as usize].has_town && !self.regions[region as usize].inked
	}

	fn disruptions_to_ink(&self, region: RegionId) -> u8 {
		INSTABILITY_THRESHOLD.saturating_sub(self.regions[region as usize].instability)
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

const SCALE: u32 = 100;
const BLOCKED: u32 = u32::MAX;
const STRIDE: u32 = 2048;
const NO_POINT: Point = (u8::MAX, u8::MAX);

struct Tune {
	demand: f64,
	power: f64,
	foe_toll: f64,
	avoid_inst: u8,
	hold: f64,
	reach: f64,
	shortlist: usize,
	ink_self: f64,
	ink_track: f64,
	ink_commit: f64,
	ink_plan: f64,
	fill: bool,
	debug: bool,
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
			demand: knob("BTK_DEMAND", 1.0),
			power: knob("BTK_POWER", 0.6),
			foe_toll: knob("BTK_FOE_TOLL", 1.5),
			avoid_inst: knob("BTK_AVOID_INST", 3),
			hold: knob("BTK_HOLD", 1.2),
			reach: knob("BTK_REACH", 0.0),
			shortlist: knob("BTK_SHORTLIST", 10),
			ink_self: knob("BTK_INK_SELF", 2.0),
			ink_track: knob("BTK_INK_TRACK", 0.25),
			ink_commit: knob("BTK_INK_COMMIT", 0.5),
			ink_plan: knob("BTK_INK_PLAN", 4.0),
			fill: knob::<u8>("BTK_FILL", 1) != 0,
			debug: knob::<u8>("BTK_DEBUG", 0) != 0,
		}
	}
}

struct Trains {
	came: Field<Point>,
	queue: Vec<Point>,
}

impl Trains {
	fn new(map: &Map) -> Trains {
		Trains {
			came: Field::new(map.width, map.height, NO_POINT),
			queue: Vec::with_capacity(map.width as usize * map.height as usize),
		}
	}

	fn income(&mut self, map: &Map, passable: Mask, mine: Mask) -> u32 {
		let mut paid = 0;
		for link in 0..map.links.len() as LinkId {
			paid += self.paid_on(map, passable, mine, link);
		}
		paid
	}

	fn paid_on(&mut self, map: &Map, passable: Mask, mine: Mask, link: LinkId) -> u32 {
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
				let mut paid = 0;
				let mut walk = goal;
				while walk != NO_POINT {
					paid += u32::from(mine.contains(walk));
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
		0
	}
}

fn walk((ax, ay): Point, (bx, by): Point) -> u16 {
	(ax.abs_diff(bx) + ay.abs_diff(by)) as u16
}

struct Weighing<'a> {
	link: LinkId,
	route: &'a Route,
	base: u32,
	left: f64,
}

struct Route {
	cells: Vec<Point>,
	paint: u32,
	demand: f64,
}

struct Planner {
	cost: Field<u32>,
	seen: Field<u32>,
	came: Field<Point>,
	queue: BinaryHeap<std::cmp::Reverse<(u32, Point)>>,
	demand: Field<u16>,
	trains: Trains,
}

impl Planner {
	fn new(map: &Map) -> Planner {
		let mut planner = Planner {
			cost: Field::new(map.width, map.height, BLOCKED),
			seen: Field::new(map.width, map.height, BLOCKED),
			came: Field::new(map.width, map.height, NO_POINT),
			queue: BinaryHeap::new(),
			demand: Field::new(map.width, map.height, 0),
			trains: Trains::new(map),
		};
		planner.survey(map);
		planner
	}

	fn survey(&mut self, map: &Map) {
		for link in 0..map.links.len() as LinkId {
			let (start, goal) = map.endpoints_of(link);
			let span = walk(start, goal);
			for at in map.points() {
				if walk(start, at) + walk(at, goal) == span {
					self.demand[at] += 1;
				}
			}
		}
	}

	fn flood(&mut self, map: &Map, start: Point, goal: Point, stride: u32) {
		self.seen.values.fill(BLOCKED);
		self.queue.clear();
		self.seen[start] = 0;
		self.queue.push(std::cmp::Reverse((0, start)));
		while let Some(std::cmp::Reverse((spent, at))) = self.queue.pop() {
			if at == goal {
				return;
			}
			if spent > self.seen[at] {
				continue;
			}
			for next in map.neighbours(at) {
				let toll = self.cost[next];
				if toll == BLOCKED {
					continue;
				}
				let total = spent + stride + if stride == 0 { toll } else { toll / SCALE };
				if total < self.seen[next] {
					self.seen[next] = total;
					self.came[next] = at;
					self.queue.push(std::cmp::Reverse((total, next)));
				}
			}
		}
	}

	fn price(&mut self, map: &Map, turn: &Turn, tune: &Tune) {
		let toll = (tune.foe_toll * SCALE as f64) as u32;
		for at in map.points() {
			let region = map.region_of(at) as usize;
			let state = turn.regions[region];
			self.cost[at] = if map.town_cells.contains(at) || turn.my_tracks.contains(at) {
				0
			} else if turn.tracks.contains(at) {
				toll
			} else if state.inked
				|| !turn.buildable.contains(at)
				|| (!map.regions[region].has_town && state.instability >= tune.avoid_inst)
			{
				BLOCKED
			} else {
				map.paint_cost(at) as u32 * SCALE
			};
		}
	}

	fn dig(&mut self, map: &Map, turn: &Turn, link: LinkId, stride: u32) -> Option<Route> {
		let (start, goal) = map.endpoints_of(link);
		self.flood(map, start, goal, stride);
		if self.seen[goal] == BLOCKED {
			return None;
		}

		let mut cells = Vec::new();
		let mut paint = 0;
		let mut demand = 0.0;
		let mut length = 1;
		let mut at = goal;
		while at != start {
			if turn.buildable.contains(at) && !turn.tracks.contains(at) {
				paint += map.paint_cost(at) as u32;
				demand += 1.0 + self.demand[at] as f64;
				cells.push(at);
			}
			length += 1;
			at = self.came[at];
		}
		if paint == 0 {
			return None;
		}
		if stride > 0
			&& length >= turn.paths[link as usize].len()
			&& !turn.paths[link as usize].is_empty()
		{
			return None;
		}
		cells.reverse();
		Some(Route {
			cells,
			paint,
			demand,
		})
	}

	fn candidates(&mut self, map: &Map, turn: &Turn) -> Vec<(LinkId, Route)> {
		let mut found = Vec::new();
		for link in 0..map.links.len() as LinkId {
			for stride in [0, STRIDE] {
				if let Some(route) = self.dig(map, turn, link, stride)
					&& !found
						.iter()
						.any(|(_, other): &(LinkId, Route)| other.cells == route.cells)
				{
					found.push((link, route));
				}
			}
		}
		found
	}

	fn weigh(&mut self, map: &Map, turn: &Turn, tune: &Tune, plan: Weighing<'_>) -> f64 {
		let Weighing {
			link,
			route,
			base,
			left,
		} = plan;
		let (start, goal) = map.endpoints_of(link);
		let span = walk(start, goal) as f64;
		let mut passable = turn.passable;
		let mut mine = turn.my_tracks;
		for &at in &route.cells {
			passable.insert(at);
			mine.insert(at);
		}
		let after = self.trains.income(map, passable, mine) as f64;
		let gain = after - base as f64;
		let earning = left - route.paint as f64 / PAINT_PER_TURN as f64;
		if earning <= 0.0 {
			return f64::MIN;
		}
		(gain + tune.demand * route.demand) * earning * span.powf(tune.reach)
			/ (route.paint as f64).powf(tune.power)
	}
}

struct Plan {
	link: LinkId,
	cells: Vec<Point>,
	worth: f64,
}

fn multiplicity(map: &Map, turn: &Turn) -> Field<u8> {
	let mut count = Field::new(map.width, map.height, 0u8);
	for &link in &turn.active {
		for at in turn.paths[link as usize].iter() {
			count[at] = count[at].saturating_add(1);
		}
	}
	count
}

fn pick_ink(map: &Map, turn: &Turn, tune: &Tune, plan: Option<&Plan>) -> Option<RegionId> {
	let paid = multiplicity(map, turn);
	let mut best: Option<(f64, RegionId)> = None;
	for (id, region) in map.regions.iter().enumerate() {
		let id = id as RegionId;
		if !turn.can_disrupt(map, id) {
			continue;
		}
		let mut worth = 0.0;
		for at in region.cells.iter() {
			let seats = paid[at] as f64;
			if turn.foe_tracks.contains(at) {
				worth += seats + tune.ink_track;
			} else if turn.my_tracks.contains(at) {
				worth -= tune.ink_self * (seats + tune.ink_track);
			}
			if plan.is_some_and(|plan| plan.cells.contains(&at)) {
				worth -= tune.ink_plan;
			}
		}
		let charged = INSTABILITY_THRESHOLD - turn.disruptions_to_ink(id);
		let worth = worth * (1.0 + tune.ink_commit * charged as f64) + charged as f64 * 0.001;
		if best.is_none_or(|(top, _)| worth > top) {
			best = Some((worth, id));
		}
	}
	best.map(|(_, id)| id)
}

fn fill(map: &Map, turn: &Turn, planner: &mut Planner, base: u32, budget: u8) -> Option<Point> {
	let mut best: Option<(f64, Point)> = None;
	for at in turn.buildable.iter() {
		let cost = map.paint_cost(at);
		if cost > budget || !map.neighbours(at).any(|next| turn.passable.contains(next)) {
			continue;
		}
		let mut passable = turn.passable;
		let mut mine = turn.my_tracks;
		passable.insert(at);
		mine.insert(at);
		let after = planner.trains.income(map, passable, mine);
		if after <= base {
			continue;
		}
		let worth = (after - base) as f64 / cost as f64;
		if best.is_none_or(|(top, _)| worth > top) {
			best = Some((worth, at));
		}
	}
	best.map(|(_, at)| at)
}

fn decide(
	map: &Map,
	turn: &Turn,
	tune: &Tune,
	planner: &mut Planner,
	held: &mut Option<Plan>,
	count: i32,
	commands: &mut Vec<Command>,
) {
	commands.clear();
	let left = (MAX_TURNS - count + 1).max(1) as f64;
	planner.price(map, turn, tune);
	let base = planner.trains.income(map, turn.passable, turn.my_tracks);

	let mut ranked = planner.candidates(map, turn);
	ranked.sort_by(|(_, a), (_, b)| {
		(b.demand / b.paint as f64).total_cmp(&(a.demand / a.paint as f64))
	});
	ranked.truncate(tune.shortlist);

	let mut best: Option<Plan> = None;
	for (link, route) in &ranked {
		let worth = planner.weigh(
			map,
			turn,
			tune,
			Weighing {
				link: *link,
				route,
				base,
				left,
			},
		);
		if worth > 0.0 && best.as_ref().is_none_or(|top| worth > top.worth) {
			best = Some(Plan {
				link: *link,
				cells: route.cells.clone(),
				worth,
			});
		}
	}

	if let Some(plan) = held.as_ref() {
		let alive = plan
			.cells
			.iter()
			.all(|&at| turn.buildable.contains(at) || turn.my_tracks.contains(at));
		let again = alive.then(|| Plan {
			link: plan.link,
			cells: plan
				.cells
				.iter()
				.copied()
				.filter(|&at| !turn.tracks.contains(at))
				.collect(),
			worth: plan.worth,
		});
		*held = match again {
			Some(plan) if !plan.cells.is_empty() => Some(plan),
			_ => None,
		};
	}
	match (&held, &best) {
		(Some(plan), Some(fresh)) if fresh.worth > plan.worth * tune.hold => *held = best,
		(None, Some(_)) => *held = best,
		_ => {}
	}

	if let Some(region) = pick_ink(map, turn, tune, held.as_ref()) {
		commands.push(Command::Disrupt(region));
	}

	let mut budget = PAINT_PER_TURN;
	if let Some(plan) = held.as_ref() {
		for &at in &plan.cells {
			let cost = map.paint_cost(at);
			if cost > budget {
				continue;
			}
			budget -= cost;
			commands.push(Command::Place(at));
			if budget == 0 {
				break;
			}
		}
	}
	if tune.fill
		&& budget > 0
		&& let Some(at) = fill(map, turn, planner, base, budget)
	{
		budget -= map.paint_cost(at);
		commands.push(Command::Place(at));
	}
	if tune.debug {
		let live = turn.active.len();
		let (link, cells, worth) = held.as_ref().map_or((NO_LINK, 0, 0.0), |plan| {
			(plan.link, plan.cells.len(), plan.worth)
		});
		eprintln!(
			"t{count} pay={base} live={live}/{} cands={} plan={link} cells={cells} worth={worth:.1} left={budget}",
			map.links.len(),
			ranked.len()
		);
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
	let mut held: Option<Plan> = None;
	let mut commands = Vec::with_capacity(4);
	let mut count = 0;

	while turn.read(&map, &mut input) {
		count += 1;
		decide(
			&map,
			&turn,
			&tune,
			&mut planner,
			&mut held,
			count,
			&mut commands,
		);
		send(&commands, &mut out).unwrap();
	}
}
