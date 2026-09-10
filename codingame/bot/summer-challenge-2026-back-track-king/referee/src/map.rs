pub const PLAIN: u8 = 0;
pub const RIVER: u8 = 1;
pub const MOUNTAIN: u8 = 2;

pub const TERRAIN_COST: [u32; 3] = [1, 2, 3];
pub const TERRAIN_CHAR: [u8; 3] = [b'.', b'~', b'^'];

pub const NO_TOWN: i16 = -1;

#[derive(Clone, Debug)]
pub struct Town {
	pub x: usize,
	pub y: usize,
	pub desired: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Map {
	pub width: usize,
	pub height: usize,
	pub terrain: Vec<u8>,
	pub region: Vec<u16>,
	pub region_count: usize,
	pub towns: Vec<Town>,
	pub town_at: Vec<i16>,
	pub region_has_town: Vec<bool>,
	pub neighbors: Vec<[i32; 4]>,
}

impl Map {
	pub fn new(
		width: usize,
		height: usize,
		terrain: Vec<u8>,
		region: Vec<u16>,
		towns: Vec<Town>,
	) -> Self {
		let cells = width * height;
		assert_eq!(terrain.len(), cells);
		assert_eq!(region.len(), cells);
		let region_count = region.iter().copied().max().map_or(0, |m| m as usize + 1);
		let mut town_at = vec![NO_TOWN; cells];
		let mut region_has_town = vec![false; region_count];
		for (id, town) in towns.iter().enumerate() {
			let cell = town.y * width + town.x;
			town_at[cell] = id as i16;
			region_has_town[region[cell] as usize] = true;
		}
		Map {
			neighbors: build_neighbors(width, height),
			width,
			height,
			terrain,
			region,
			region_count,
			towns,
			town_at,
			region_has_town,
		}
	}

	pub fn cells(&self) -> usize {
		self.width * self.height
	}

	pub fn idx(&self, x: usize, y: usize) -> usize {
		y * self.width + x
	}

	pub fn xy(&self, cell: usize) -> (usize, usize) {
		(cell % self.width, cell / self.width)
	}

	pub fn cost(&self, cell: usize) -> u32 {
		TERRAIN_COST[self.terrain[cell] as usize]
	}

	pub fn region_of(&self, cell: usize) -> usize {
		self.region[cell] as usize
	}
}

fn build_neighbors(width: usize, height: usize) -> Vec<[i32; 4]> {
	let mut out = vec![[-1i32; 4]; width * height];
	for y in 0..height {
		for x in 0..width {
			let cell = y * width + x;
			let w = width as i32;
			let n = &mut out[cell];
			n[0] = if y > 0 { cell as i32 - w } else { -1 };
			n[1] = if x + 1 < width { cell as i32 + 1 } else { -1 };
			n[2] = if y + 1 < height { cell as i32 + w } else { -1 };
			n[3] = if x > 0 { cell as i32 - 1 } else { -1 };
		}
	}
	out
}

pub fn format_map(map: &Map) -> String {
	use std::fmt::Write;
	let mut out = String::new();
	let _ = writeln!(out, "size {} {}", map.width, map.height);
	let _ = writeln!(out, "terrain");
	for y in 0..map.height {
		let row: Vec<u8> = (0..map.width)
			.map(|x| TERRAIN_CHAR[map.terrain[map.idx(x, y)] as usize])
			.collect();
		let _ = writeln!(out, "{}", String::from_utf8(row).unwrap());
	}
	let _ = writeln!(out, "regions");
	for y in 0..map.height {
		let row: Vec<String> = (0..map.width)
			.map(|x| map.region[map.idx(x, y)].to_string())
			.collect();
		let _ = writeln!(out, "{}", row.join(" "));
	}
	let _ = writeln!(out, "towns {}", map.towns.len());
	for town in &map.towns {
		let desired = if town.desired.is_empty() {
			"x".to_string()
		} else {
			town.desired
				.iter()
				.map(|d| d.to_string())
				.collect::<Vec<_>>()
				.join(",")
		};
		let _ = writeln!(out, "{} {} {}", town.x, town.y, desired);
	}
	out
}

pub fn parse_map(text: &str) -> Result<Map, String> {
	let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
	let header: Vec<&str> = lines
		.next()
		.ok_or("empty map")?
		.split_whitespace()
		.collect();
	if header.len() != 3 || header[0] != "size" {
		return Err("expected `size W H`".into());
	}
	let width: usize = header[1].parse().map_err(|_| "bad width")?;
	let height: usize = header[2].parse().map_err(|_| "bad height")?;
	if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE {
		return Err(format!(
			"size {width}x{height} is not between 1x1 and {MAX_SIDE}x{MAX_SIDE}"
		));
	}

	if lines.next() != Some("terrain") {
		return Err("expected `terrain`".into());
	}
	let mut terrain = Vec::new();
	for _ in 0..height {
		let row = lines.next().ok_or("terrain truncated")?;
		if row.len() != width {
			return Err(format!("terrain row width {} != {width}", row.len()));
		}
		for b in row.bytes() {
			let t = TERRAIN_CHAR
				.iter()
				.position(|&c| c == b)
				.ok_or_else(|| format!("bad terrain char {}", b as char))?;
			terrain.push(t as u8);
		}
	}

	if lines.next() != Some("regions") {
		return Err("expected `regions`".into());
	}
	let mut region = Vec::new();
	for _ in 0..height {
		let row = lines.next().ok_or("regions truncated")?;
		let ids: Vec<&str> = row.split_whitespace().collect();
		if ids.len() != width {
			return Err(format!("region row width {} != {width}", ids.len()));
		}
		for id in ids {
			region.push(id.parse::<u16>().map_err(|_| "bad region id")?);
		}
	}

	let head: Vec<&str> = lines
		.next()
		.ok_or("expected `towns N`")?
		.split_whitespace()
		.collect();
	if head.len() != 2 || head[0] != "towns" {
		return Err("expected `towns N`".into());
	}
	let town_count: usize = head[1].parse().map_err(|_| "bad town count")?;
	let mut towns = Vec::with_capacity(town_count);
	for _ in 0..town_count {
		let parts: Vec<&str> = lines
			.next()
			.ok_or("towns truncated")?
			.split_whitespace()
			.collect();
		if parts.len() != 3 {
			return Err("expected `X Y DESIRED`".into());
		}
		let desired = if parts[2] == "x" {
			Vec::new()
		} else {
			parts[2]
				.split(',')
				.map(|d| d.parse::<u8>().map_err(|_| "bad desired id".to_string()))
				.collect::<Result<Vec<_>, _>>()?
		};
		let town = Town {
			x: parts[0].parse().map_err(|_| "bad town x")?,
			y: parts[1].parse().map_err(|_| "bad town y")?,
			desired,
		};
		if town.x >= width || town.y >= height {
			return Err(format!("town at ({}, {}) is off the map", town.x, town.y));
		}
		towns.push(town);
	}

	let map = Map::new(width, height, terrain, region, towns);
	validate(&map, false)?;
	Ok(map)
}

pub const MAX_SIDE: usize = 1024;

pub const CG_WIDTH: std::ops::RangeInclusive<usize> = 21..=30;
pub const CG_HEIGHT: std::ops::RangeInclusive<usize> = 14..=20;
pub const CG_TOWNS: std::ops::RangeInclusive<usize> = 4..=12;

pub fn validate(map: &Map, cg_ranges: bool) -> Result<(), String> {
	if cg_ranges {
		if !CG_WIDTH.contains(&map.width) {
			return Err(format!("width {} outside {CG_WIDTH:?}", map.width));
		}
		if !CG_HEIGHT.contains(&map.height) {
			return Err(format!("height {} outside {CG_HEIGHT:?}", map.height));
		}
		if !CG_TOWNS.contains(&map.towns.len()) {
			return Err(format!(
				"town count {} outside {CG_TOWNS:?}",
				map.towns.len()
			));
		}
	}
	for (id, town) in map.towns.iter().enumerate() {
		if town.x >= map.width || town.y >= map.height {
			return Err(format!(
				"town {id} at ({}, {}) is off the map",
				town.x, town.y
			));
		}
		let cell = map.idx(town.x, town.y);
		if map.terrain[cell] != PLAIN {
			return Err(format!("town {id} is not on a plain cell"));
		}
		if map.town_at[cell] != id as i16 {
			return Err(format!(
				"town {id} shares a cell with town {}",
				map.town_at[cell]
			));
		}
		let mut seen = town.desired.clone();
		seen.sort_unstable();
		seen.dedup();
		if seen.len() != town.desired.len() {
			return Err(format!("town {id} lists a desired connection twice"));
		}
		for &dst in &town.desired {
			if dst as usize >= map.towns.len() {
				return Err(format!("town {id} desires unknown town {dst}"));
			}
			if dst as usize == id {
				return Err(format!("town {id} desires itself"));
			}
			if map.towns[dst as usize].desired.contains(&(id as u8)) {
				return Err(format!("towns {id} and {dst} desire each other"));
			}
		}
	}
	for id in 0..map.towns.len() {
		if !map.towns.iter().any(|t| t.desired.contains(&(id as u8))) {
			return Err(format!("no town desires a connection to town {id}"));
		}
	}

	let mut town_of_region = vec![NO_TOWN; map.region_count];
	for (id, town) in map.towns.iter().enumerate() {
		let region = map.region_of(map.idx(town.x, town.y));
		if town_of_region[region] != NO_TOWN {
			return Err(format!(
				"region {region} holds towns {} and {id}",
				town_of_region[region]
			));
		}
		town_of_region[region] = id as i16;
	}
	for cell in 0..map.cells() {
		let a = map.region_of(cell);
		for &n in &map.neighbors[cell] {
			if n < 0 {
				continue;
			}
			let b = map.region_of(n as usize);
			if a != b && town_of_region[a] != NO_TOWN && town_of_region[b] != NO_TOWN {
				return Err(format!("bordering regions {a} and {b} both hold a town"));
			}
		}
	}

	let mut seen = vec![false; map.cells()];
	let mut reached = vec![0usize; map.region_count];
	let mut size = vec![0usize; map.region_count];
	for cell in 0..map.cells() {
		size[map.region_of(cell)] += 1;
	}
	let mut queue = std::collections::VecDeque::new();
	for region in 0..map.region_count {
		if size[region] < 2 {
			return Err(format!("region {region} has fewer than two cells"));
		}
		let start = (0..map.cells())
			.find(|&c| map.region_of(c) == region)
			.unwrap();
		queue.clear();
		queue.push_back(start);
		seen[start] = true;
		while let Some(cell) = queue.pop_front() {
			reached[region] += 1;
			for &n in &map.neighbors[cell] {
				if n < 0 {
					continue;
				}
				let n = n as usize;
				if !seen[n] && map.region_of(n) == region {
					seen[n] = true;
					queue.push_back(n);
				}
			}
		}
		if reached[region] != size[region] {
			return Err(format!("region {region} is not contiguous"));
		}
	}
	Ok(())
}
