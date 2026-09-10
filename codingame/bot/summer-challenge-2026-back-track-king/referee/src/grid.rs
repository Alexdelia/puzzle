use crate::jhash::JavaHash;
use std::collections::BTreeMap;

pub const TYPE_GRASS: u8 = 0;
pub const TYPE_WATER: u8 = 1;
pub const TYPE_MOUNTAIN: u8 = 2;
pub const TYPE_POI: u8 = 3;

pub const TRACK_NONE: i8 = -1;
pub const TRACK_NEUTRAL: i8 = 2;
pub const TOWN_NONE: i16 = -1;
pub const ZONE_NONE: i16 = -1;

pub const TERRAIN_CHAR: [u8; 4] = [b'.', b'~', b'^', b'!'];

pub const BASE_RAIL_COST: i32 = 1;
pub const GRASS_COST_MULTIPLIER: i32 = 1;
pub const RIVER_COST_MULTIPLIER: i32 = 2;
pub const MOUNTAIN_COST_MULTIPLIER: i32 = 3;
pub const POI_COST_MULTIPLIER: i32 = 3;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Coord {
	pub x: i32,
	pub y: i32,
}

pub const fn coord(x: i32, y: i32) -> Coord {
	Coord { x, y }
}

impl Coord {
	pub fn sub(self, other: Coord) -> Coord {
		coord(self.x - other.x, self.y - other.y)
	}

	pub fn manhattan_to(self, other: Coord) -> i32 {
		(other.x - self.x).abs() + (other.y - self.y).abs()
	}
}

impl JavaHash for Coord {
	fn java_hash(&self) -> i32 {
		31i32
			.wrapping_mul(31i32.wrapping_add(self.x))
			.wrapping_add(self.y)
	}
}

impl std::fmt::Display for Coord {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "({}, {})", self.x, self.y)
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
	North,
	East,
	South,
	West,
	Unset,
}

impl Direction {
	pub fn coord(self) -> Coord {
		match self {
			Direction::North => coord(0, -1),
			Direction::East => coord(1, 0),
			Direction::South => coord(0, 1),
			Direction::West => coord(-1, 0),
			Direction::Unset => coord(0, 0),
		}
	}

	pub fn ordinal(self) -> i32 {
		match self {
			Direction::North => 0,
			Direction::East => 1,
			Direction::South => 2,
			Direction::West => 3,
			Direction::Unset => 4,
		}
	}

	pub fn opposite(self) -> Direction {
		match self {
			Direction::North => Direction::South,
			Direction::East => Direction::West,
			Direction::South => Direction::North,
			Direction::West => Direction::East,
			Direction::Unset => Direction::Unset,
		}
	}

	pub fn from_coord(delta: Coord) -> Direction {
		match (delta.x, delta.y) {
			(0, -1) => Direction::North,
			(1, 0) => Direction::East,
			(0, 1) => Direction::South,
			(-1, 0) => Direction::West,
			_ => Direction::Unset,
		}
	}
}

pub const ADJACENCY: [Coord; 4] = [coord(0, -1), coord(1, 0), coord(0, 1), coord(-1, 0)];

pub const ADJACENCY_8: [Coord; 8] = [
	coord(0, -1),
	coord(1, 0),
	coord(0, 1),
	coord(-1, 0),
	coord(-1, -1),
	coord(1, 1),
	coord(1, -1),
	coord(-1, 1),
];

#[derive(Clone, Debug)]
pub struct Tile {
	pub kind: u8,
	pub zone: i16,
	pub track: i8,
	pub town: i16,
	pub connections: Vec<(u8, u8)>,
}

impl Default for Tile {
	fn default() -> Self {
		Tile {
			kind: TYPE_GRASS,
			zone: ZONE_NONE,
			track: TRACK_NONE,
			town: TOWN_NONE,
			connections: Vec::new(),
		}
	}
}

impl Tile {
	pub fn is_water(&self) -> bool {
		self.kind == TYPE_WATER
	}

	pub fn is_mountain(&self) -> bool {
		self.kind == TYPE_MOUNTAIN
	}

	pub fn is_plains(&self) -> bool {
		self.kind == TYPE_GRASS
	}

	pub fn is_town(&self) -> bool {
		self.town != TOWN_NONE
	}

	pub fn is_track(&self) -> bool {
		self.track != TRACK_NONE
	}

	pub fn is_track_or_town(&self) -> bool {
		self.is_town() || self.is_track()
	}

	pub fn rail_cost(&self) -> i32 {
		BASE_RAIL_COST
			* match self.kind {
				TYPE_MOUNTAIN => MOUNTAIN_COST_MULTIPLIER,
				TYPE_WATER => RIVER_COST_MULTIPLIER,
				TYPE_POI => POI_COST_MULTIPLIER,
				_ => GRASS_COST_MULTIPLIER,
			}
	}
}

#[derive(Clone, Debug)]
pub struct Zone {
	pub id: usize,
	pub coords: Vec<Coord>,
	pub neighbours: Vec<usize>,
	pub towns: Vec<usize>,
	pub instability: i32,
	pub inked: bool,
}

impl Zone {
	pub fn new(id: usize, coords: Vec<Coord>) -> Self {
		Zone {
			id,
			coords,
			neighbours: Vec::new(),
			towns: Vec::new(),
			instability: 0,
			inked: false,
		}
	}
}

#[derive(Clone, Debug)]
pub struct Town {
	pub id: usize,
	pub coord: Coord,
	pub desired: Vec<usize>,
	pub active: Vec<usize>,
	pub paths: BTreeMap<usize, Vec<Coord>>,
}

impl Town {
	pub fn new(id: usize, coord: Coord) -> Self {
		Town {
			id,
			coord,
			desired: Vec::new(),
			active: Vec::new(),
			paths: BTreeMap::new(),
		}
	}
}

#[derive(Clone, Debug)]
pub struct Grid {
	pub width: i32,
	pub height: i32,
	pub tiles: Vec<Tile>,
	pub towns: Vec<Town>,
	pub zones: Vec<Zone>,
}

impl Grid {
	pub fn new(width: i32, height: i32) -> Self {
		Grid {
			tiles: vec![Tile::default(); (width * height) as usize],
			towns: Vec::new(),
			zones: Vec::new(),
			width,
			height,
		}
	}

	pub fn cells(&self) -> usize {
		self.tiles.len()
	}

	pub fn holds(&self, at: Coord) -> bool {
		at.x >= 0 && at.y >= 0 && at.x < self.width && at.y < self.height
	}

	pub fn index(&self, at: Coord) -> usize {
		(at.y * self.width + at.x) as usize
	}

	pub fn coord_of(&self, cell: usize) -> Coord {
		coord(cell as i32 % self.width, cell as i32 / self.width)
	}

	pub fn get(&self, at: Coord) -> Option<&Tile> {
		self.holds(at).then(|| &self.tiles[self.index(at)])
	}

	pub fn tile(&self, at: Coord) -> &Tile {
		&self.tiles[self.index(at)]
	}

	pub fn tile_mut(&mut self, at: Coord) -> &mut Tile {
		let cell = self.index(at);
		&mut self.tiles[cell]
	}

	pub fn zone_of(&self, at: Coord) -> &Zone {
		&self.zones[self.tile(at).zone as usize]
	}

	pub fn neighbours(&self, at: Coord) -> Neighbours {
		Neighbours {
			at,
			deltas: &ADJACENCY,
			next: 0,
			width: self.width,
			height: self.height,
		}
	}

	pub fn neighbours_8(&self, at: Coord) -> Neighbours {
		Neighbours {
			at,
			deltas: &ADJACENCY_8,
			next: 0,
			width: self.width,
			height: self.height,
		}
	}

	pub fn can_train_pass(&self, at: Coord) -> bool {
		self.get(at)
			.is_some_and(|tile| tile.is_town() || tile.track != TRACK_NONE)
	}
}

pub struct Neighbours {
	at: Coord,
	deltas: &'static [Coord],
	next: usize,
	width: i32,
	height: i32,
}

impl Iterator for Neighbours {
	type Item = Coord;

	fn next(&mut self) -> Option<Coord> {
		while self.next < self.deltas.len() {
			let delta = self.deltas[self.next];
			self.next += 1;
			let at = coord(self.at.x + delta.x, self.at.y + delta.y);
			if at.x >= 0 && at.y >= 0 && at.x < self.width && at.y < self.height {
				return Some(at);
			}
		}
		None
	}
}

pub fn desired_text(desired: &[usize]) -> String {
	if desired.is_empty() {
		return "x".into();
	}
	desired
		.iter()
		.map(|id| id.to_string())
		.collect::<Vec<_>>()
		.join(",")
}

pub fn parse(text: &str) -> Result<Grid, String> {
	let mut lines = text
		.lines()
		.map(str::trim)
		.filter(|line| !line.is_empty() && !line.starts_with("seed "));
	let header: Vec<&str> = lines
		.next()
		.ok_or("empty map")?
		.split_whitespace()
		.collect();
	if header.len() != 3 || header[0] != "size" {
		return Err("expected `size W H`".into());
	}
	let width: i32 = header[1].parse().map_err(|_| "bad width")?;
	let height: i32 = header[2].parse().map_err(|_| "bad height")?;
	if width <= 0 || height <= 0 || width > 1024 || height > 1024 {
		return Err(format!("size {width}x{height} is out of range"));
	}
	let mut grid = Grid::new(width, height);

	if lines.next() != Some("terrain") {
		return Err("expected `terrain`".into());
	}
	for y in 0..height {
		let row = lines.next().ok_or("terrain truncated")?;
		if row.len() != width as usize {
			return Err(format!("terrain row width {} != {width}", row.len()));
		}
		for (x, byte) in row.bytes().enumerate() {
			let kind = TERRAIN_CHAR
				.iter()
				.position(|&c| c == byte)
				.ok_or_else(|| format!("bad terrain char {}", byte as char))?;
			grid.tile_mut(coord(x as i32, y)).kind = kind as u8;
		}
	}

	if lines.next() != Some("regions") {
		return Err("expected `regions`".into());
	}
	let mut zone_count = 0usize;
	for y in 0..height {
		let row = lines.next().ok_or("regions truncated")?;
		let ids: Vec<&str> = row.split_whitespace().collect();
		if ids.len() != width as usize {
			return Err(format!("region row width {} != {width}", ids.len()));
		}
		for (x, id) in ids.iter().enumerate() {
			let zone: i16 = id.parse().map_err(|_| "bad region id")?;
			if zone < 0 {
				return Err("region ids start at 0".into());
			}
			zone_count = zone_count.max(zone as usize + 1);
			grid.tile_mut(coord(x as i32, y)).zone = zone;
		}
	}
	grid.zones = (0..zone_count)
		.map(|id| Zone::new(id, Vec::new()))
		.collect();
	for cell in 0..grid.cells() {
		let at = grid.coord_of(cell);
		let zone = grid.tiles[cell].zone as usize;
		grid.zones[zone].coords.push(at);
	}
	if let Some(empty) = grid.zones.iter().find(|zone| zone.coords.is_empty()) {
		return Err(format!("region {} holds no cell", empty.id));
	}
	for id in 0..grid.zones.len() {
		let mut neighbours: Vec<usize> = Vec::new();
		for at in grid.zones[id].coords.clone() {
			for n in grid.neighbours(at) {
				let other = grid.tile(n).zone as usize;
				if other != id && !neighbours.contains(&other) {
					neighbours.push(other);
				}
			}
		}
		neighbours.sort_unstable();
		grid.zones[id].neighbours = neighbours;
	}

	let head: Vec<&str> = loop {
		let line = lines.next().ok_or("expected `towns N`")?;
		if line.starts_with("regioncount") || line.starts_with("region ") {
			continue;
		}
		break line.split_whitespace().collect();
	};
	if head.len() != 2 || head[0] != "towns" {
		return Err("expected `towns N`".into());
	}
	let town_count: usize = head[1].parse().map_err(|_| "bad town count")?;
	for id in 0..town_count {
		let parts: Vec<&str> = lines
			.next()
			.ok_or("towns truncated")?
			.split_whitespace()
			.collect();
		if parts.len() != 5 || parts[0] != "town" {
			return Err("expected `town ID X Y DESIRED`".into());
		}
		if parts[1].parse::<usize>() != Ok(id) {
			return Err(format!("expected town {id}, got `{}`", parts[1]));
		}
		let at = coord(
			parts[2].parse().map_err(|_| "bad town x")?,
			parts[3].parse().map_err(|_| "bad town y")?,
		);
		if !grid.holds(at) {
			return Err(format!("town {id} at {at} is off the map"));
		}
		let desired = if parts[4] == "x" {
			Vec::new()
		} else {
			parts[4]
				.split(',')
				.map(|id| {
					id.parse::<usize>()
						.map_err(|_| "bad desired id".to_string())
				})
				.collect::<Result<Vec<_>, _>>()?
		};
		let mut town = Town::new(id, at);
		town.desired = desired;
		grid.towns.push(town);
	}
	for id in 0..grid.towns.len() {
		let at = grid.towns[id].coord;
		if grid.tile(at).is_town() {
			return Err(format!(
				"towns {} and {id} share a cell",
				grid.tile(at).town
			));
		}
		for &other in &grid.towns[id].desired {
			if other >= town_count {
				return Err(format!("town {id} desires unknown town {other}"));
			}
		}
		let zone = grid.tile(at).zone as usize;
		grid.zones[zone].towns.push(id);
		let tile = grid.tile_mut(at);
		tile.town = id as i16;
		tile.kind = TYPE_GRASS;
	}
	Ok(grid)
}

pub fn describe(grid: &Grid) -> String {
	use std::fmt::Write;
	let mut out = String::new();
	let _ = writeln!(out, "size {} {}", grid.width, grid.height);
	out.push_str("terrain\n");
	for y in 0..grid.height {
		let row: Vec<u8> = (0..grid.width)
			.map(|x| TERRAIN_CHAR[grid.tile(coord(x, y)).kind as usize])
			.collect();
		let _ = writeln!(out, "{}", String::from_utf8(row).unwrap());
	}
	out.push_str("regions\n");
	for y in 0..grid.height {
		let row: Vec<String> = (0..grid.width)
			.map(|x| grid.tile(coord(x, y)).zone.to_string())
			.collect();
		let _ = writeln!(out, "{}", row.join(" "));
	}
	let _ = writeln!(out, "regioncount {}", grid.zones.len());
	for zone in &grid.zones {
		let _ = write!(out, "region {} cells", zone.id);
		for at in &zone.coords {
			let _ = write!(out, " {},{}", at.x, at.y);
		}
		out.push_str(" borders");
		for id in &zone.neighbours {
			let _ = write!(out, " {id}");
		}
		out.push_str(" towns");
		for id in &zone.towns {
			let _ = write!(out, " {id}");
		}
		out.push('\n');
	}
	let _ = writeln!(out, "towns {}", grid.towns.len());
	for town in &grid.towns {
		let _ = writeln!(
			out,
			"town {} {} {} {}",
			town.id,
			town.coord.x,
			town.coord.y,
			desired_text(&town.desired)
		);
	}
	let _ = writeln!(
		out,
		"tracks {}",
		grid.tiles.iter().filter(|tile| tile.is_track()).count()
	);
	out
}
