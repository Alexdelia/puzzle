#![allow(dead_code)]

use std::fmt::Display;
use std::io::{self, Read, Write};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, IndexMut, Sub, SubAssign};

const MAX_WIDTH: u8 = 30;
const MAX_HEIGHT: u8 = 20;
const ROWS: usize = MAX_HEIGHT as usize;

const PAINT_PER_TURN: u8 = 3;
const INSTABILITY_THRESHOLD: u8 = 4;

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

	fn remove(&mut self, (x, y): Point) {
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

	fn grown(&self, mask: Mask) -> Mask {
		let inside = (1u64 << self.width) - 1;
		let mut out = Mask::EMPTY;
		for y in 0..self.height as usize {
			let row = mask.0[y];
			let mut spread = ((row << 1) | (row >> 1)) & inside;
			if y > 0 {
				spread |= mask.0[y - 1];
			}
			if y + 1 < self.height as usize {
				spread |= mask.0[y + 1];
			}
			out.0[y] = spread;
		}
		out
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

	fn owner_of(&self, at: Point) -> Option<Track> {
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

	fn income_of(&self, link: LinkId) -> (u32, u32) {
		let path = self.paths[link as usize];
		(
			(path & self.my_tracks).len(),
			(path & self.foe_tracks).len(),
		)
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
	Autoplace(Point, Point),
	Disrupt(RegionId),
	Wait,
}

impl Display for Command {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			Command::Place((x, y)) => write!(f, "PLACE_TRACKS {x} {y}"),
			Command::Autoplace((x, y), (to_x, to_y)) => {
				write!(f, "AUTOPLACE {x} {y} {to_x} {to_y}")
			}
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

fn decide(map: &Map, turn: &Turn, goal: Option<LinkId>, commands: &mut Vec<Command>) {
	commands.clear();
	let harassed = turn
		.foe_tracks
		.iter()
		.map(|at| map.region_of(at))
		.find(|&region| turn.can_disrupt(map, region));
	if let Some(region) = harassed {
		commands.push(Command::Disrupt(region));
	}
	if let Some(link) = goal {
		let (from, to) = map.endpoints_of(link);
		commands.push(Command::Autoplace(from, to));
	}
}

fn main() {
	let mut input = Reader::new();
	let mut out = io::stdout().lock();
	let map = Map::read(&mut input);
	input.flush_echo();
	let mut turn = Turn::new(&map);
	let goal: Option<LinkId> = (!map.links.is_empty()).then_some(0);
	let mut commands = Vec::with_capacity(2);

	while turn.read(&map, &mut input) {
		decide(&map, &turn, goal, &mut commands);
		send(&commands, &mut out).unwrap();
	}
}
