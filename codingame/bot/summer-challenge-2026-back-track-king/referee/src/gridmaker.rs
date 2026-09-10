use crate::grid::{
	Coord, Direction, Grid, TYPE_GRASS, TYPE_MOUNTAIN, TYPE_WATER, Tile, Town, Zone, coord,
};
use crate::jhash::JHashSet;
use crate::jrand::JavaRandom;
use std::collections::VecDeque;

pub const MIN_GRID_HEIGHT: i32 = 14;
pub const MAX_GRID_HEIGHT: i32 = 20;
pub const ASPECT_RATIO: f32 = 1.5;
pub const MIN_TOWN_DISTANCE: i32 = 4;
pub const AVERAGE_TILES_PER_ZONE_COEFF_TO_GRID_HEIGHT: i32 = 2;
pub const AVERAGE_TILES_PER_TOWN: i32 = 50;
pub const RIVER_SPLIT_PROBA: f32 = 0.055;
pub const RIVER_TO_LAND_MIN_RATIO: f32 = 0.07;
pub const MIN_MOUNTAINS: i32 = 2;
pub const MOUNTAIN_TO_CELL_RATIO: f32 = 0.04;
pub const MIN_RIVER_LENGTH: usize = 3;

pub fn make(seed: i64) -> Grid {
	GridMaker::new(seed).make()
}

struct River {
	current: Coord,
	history: Vec<Coord>,
	preferred: Direction,
}

impl River {
	fn new(at: Coord, history: &[Coord], preferred: Direction) -> Self {
		let mut own = history.to_vec();
		own.push(at);
		River {
			current: at,
			history: own,
			preferred,
		}
	}

	fn is_start(&self) -> bool {
		self.history.len() == 1
	}
}

struct GridMaker {
	rng: JavaRandom,
	w: i32,
	h: i32,
	free_borders: Vec<Coord>,
	free_coords: Vec<Coord>,
	grid: Grid,
}

impl GridMaker {
	fn new(seed: i64) -> Self {
		let mut rng = JavaRandom::new(seed);
		let h = rng.next_int_range(MIN_GRID_HEIGHT, MAX_GRID_HEIGHT + 1);
		let w = java_round(h as f32 * ASPECT_RATIO);
		GridMaker {
			rng,
			w,
			h,
			free_borders: Vec::new(),
			free_coords: Vec::new(),
			grid: Grid::new(w, h),
		}
	}

	fn make(mut self) -> Grid {
		self.initialize_grid();
		self.make_mountains();
		self.make_rivers();

		let average_tiles_per_zone = self.h / AVERAGE_TILES_PER_ZONE_COEFF_TO_GRID_HEIGHT;
		let zone_count = 1.max(self.h * self.w / average_tiles_per_zone);
		self.make_zones(zone_count as usize);

		let town_count = 4.max(self.h * self.w / AVERAGE_TILES_PER_TOWN);
		self.make_towns(town_count, average_tiles_per_zone);
		self.make_town_connections();

		self.grid
	}

	fn is_corner(&self, at: Coord) -> bool {
		(at.x == 0 || at.x == self.w - 1) && (at.y == 0 || at.y == self.h - 1)
	}

	fn is_edge(&self, at: Coord) -> bool {
		at.x == 0 || at.y == 0 || at.x == self.w - 1 || at.y == self.h - 1
	}

	fn initialize_grid(&mut self) {
		for y in 0..self.h {
			for x in 0..self.w {
				let at = coord(x, y);
				self.grid.tile_mut(at).kind = TYPE_GRASS;
				if self.is_edge(at) && !self.is_corner(at) {
					self.free_borders.push(at);
				}
			}
		}
		self.free_coords = (0..self.grid.cells())
			.map(|cell| self.grid.coord_of(cell))
			.collect();

		let mut borders = std::mem::take(&mut self.free_borders);
		self.rng.shuffle(&mut borders);
		self.free_borders = borders;
		let mut coords = std::mem::take(&mut self.free_coords);
		self.rng.shuffle(&mut coords);
		self.free_coords = coords;
	}

	fn available_neighbours(&self, coords: &[Coord], accept: impl Fn(&Tile) -> bool) -> Vec<Coord> {
		let mut availables = JHashSet::new();
		for &at in coords {
			for n in self.grid.neighbours(at) {
				if accept(self.grid.tile(n)) {
					availables.add(n);
				}
			}
		}
		availables.into_list()
	}

	fn make_mountains(&mut self) {
		let ratio = (self.w * self.h) as f32 * MOUNTAIN_TO_CELL_RATIO;
		let mut count = MIN_MOUNTAINS.max(java_round(self.rng.next_float_below(ratio)));
		if self.w * self.h < 10 {
			count = 0;
		}

		for _ in 0..count {
			let size = self.rng.next_int_range(2, 8);
			if self.free_coords.is_empty() {
				return;
			}
			let base = self.free_coords.remove(0);
			let mut mountain = vec![base];
			self.grid.tile_mut(base).kind = TYPE_MOUNTAIN;
			take(&mut self.free_borders, base);

			for _ in 0..size {
				let neighs = self.available_neighbours(&mountain, is_accessible);
				if neighs.is_empty() {
					break;
				}
				let next = neighs[self.rng.next_int_below(neighs.len() as i32) as usize];
				self.grid.tile_mut(next).kind = TYPE_MOUNTAIN;
				mountain.push(next);
				take(&mut self.free_borders, next);
				take(&mut self.free_coords, next);
			}
		}
	}

	fn make_rivers(&mut self) {
		let mut left = java_round((self.w * self.h) as f32 * RIVER_TO_LAND_MIN_RATIO);
		let mut sources: VecDeque<Coord> = self.free_borders.iter().copied().collect();
		let middle = if self.rng.next_boolean() {
			coord(self.w / 2, self.h / 2)
		} else {
			coord(self.w / 2, self.h / 2 + 1)
		};
		sources.push_front(middle);

		let mut initial_river = true;
		let mut generated: Vec<River> = Vec::new();

		while left > 0 && !sources.is_empty() {
			let start = sources.pop_front().unwrap();
			take(&mut self.free_borders, start);
			let direction = self.direction_from_river_start(start);
			if self.has_water_nearby(start, &[]) {
				continue;
			}

			let mut to_expand: VecDeque<River> = VecDeque::new();
			self.create_water(&mut to_expand, River::new(start, &[], direction));
			left -= 1;

			while let Some(river) = to_expand.pop_front() {
				let current = river.current;
				if self.is_edge(current) && !river.is_start() {
					generated.push(river);
					continue;
				}

				let neighs = self.river_flow_neighbours(&river);
				if neighs.is_empty() {
					generated.push(river);
					continue;
				}

				let mut splitting =
					left > 0 && neighs.len() >= 2 && self.rng.next_float() <= RIVER_SPLIT_PROBA;
				splitting |= initial_river;
				initial_river = false;

				let weights: Vec<(Coord, f32)> = neighs
					.iter()
					.map(|&n| (n, weight(n, current, river.preferred)))
					.collect();
				let next_coord = self.pick_weighted(&weights);

				let next = River::new(
					next_coord,
					&river.history,
					if splitting {
						Direction::from_coord(next_coord - current)
					} else {
						river.preferred
					},
				);
				self.create_water(&mut to_expand, next);
				left -= 1;

				if !splitting {
					continue;
				}

				let mut remaining: Vec<Coord> = neighs
					.iter()
					.copied()
					.filter(|&n| n != next_coord)
					.collect();
				self.rng.shuffle(&mut remaining);
				let Some(&split) = remaining.first() else {
					continue;
				};
				let branch = River::new(
					split,
					&river.history,
					Direction::from_coord(split - current),
				);
				self.create_water(&mut to_expand, branch);
				left -= 1;
			}
		}

		self.delete_short_rivers(&generated);
	}

	fn create_water(&mut self, to_expand: &mut VecDeque<River>, river: River) {
		self.grid.tile_mut(river.current).kind = TYPE_WATER;
		take(&mut self.free_coords, river.current);
		to_expand.push_back(river);
	}

	fn delete_short_rivers(&mut self, generated: &[River]) {
		for river in generated {
			if river.history.len() >= MIN_RIVER_LENGTH {
				continue;
			}
			for &at in &river.history {
				self.grid.tile_mut(at).kind = TYPE_GRASS;
				if !self.free_coords.contains(&at) {
					self.free_coords.push(at);
				}
			}
		}
	}

	fn direction_from_river_start(&self, start: Coord) -> Direction {
		if start.x == 0 {
			Direction::East
		} else if start.y == 0 {
			Direction::South
		} else if start.x == self.w - 1 {
			Direction::West
		} else if start.y == self.h - 1 {
			Direction::North
		} else {
			Direction::Unset
		}
	}

	fn has_water_nearby(&self, at: Coord, history: &[Coord]) -> bool {
		let ignored = &history[history.len().saturating_sub(2)..];
		self.grid
			.neighbours_8(at)
			.filter(|n| !ignored.contains(n))
			.any(|n| self.grid.tile(n).is_water())
	}

	fn river_flow_neighbours(&self, river: &River) -> Vec<Coord> {
		self.grid
			.neighbours(river.current)
			.filter(|&n| {
				!self.has_water_nearby(n, &river.history)
					&& !self.grid.tile(n).is_water()
					&& (!river.is_start() || !self.is_edge(n))
					&& is_accessible(self.grid.tile(n))
			})
			.collect()
	}

	fn pick_weighted(&mut self, weights: &[(Coord, f32)]) -> Coord {
		let mut total = 0f32;
		for &(_, weight) in weights {
			total += weight;
		}
		let draw = self.rng.next_float() * total;
		let mut cumulative = 0f32;
		for &(at, weight) in weights {
			cumulative += weight;
			if draw < cumulative {
				return at;
			}
		}
		weights[0].0
	}

	fn make_zones(&mut self, zone_count: usize) {
		let cols = (zone_count as f64).sqrt().ceil() as usize;
		let rows = (zone_count as f64 / cols as f64).ceil() as usize;
		let cell_h = self.h as f64 / rows as f64;

		let mut zones: Vec<Zone> = Vec::with_capacity(zone_count);
		for id in 0..zone_count {
			let row = id / cols;
			let col = id % cols;
			let cell_w = if row == rows - 1 && !zone_count.is_multiple_of(cols) {
				self.w as f64 / (zone_count % cols) as f64
			} else {
				self.w as f64 / cols as f64
			};
			let center = coord(
				((col as f64 + 0.5) * cell_w) as i32,
				((row as f64 + 0.5) * cell_h) as i32,
			);
			self.grid.tile_mut(center).zone = id as i16;
			zones.push(Zone::new(id, vec![center]));
		}

		let mut blocked = vec![false; zone_count];
		let mut blocked_count = 0;
		let mut id = 0;
		loop {
			if !blocked[id] {
				let neighs = self.available_neighbours(&zones[id].coords, |tile| {
					tile.zone == crate::grid::ZONE_NONE
				});
				if neighs.is_empty() {
					blocked[id] = true;
					blocked_count += 1;
					if blocked_count == zone_count {
						break;
					}
				} else {
					let next = neighs[self.rng.next_int_below(neighs.len() as i32) as usize];
					self.grid.tile_mut(next).zone = id as i16;
					zones[id].coords.push(next);
				}
			}
			id = (id + 1) % zone_count;
		}

		for (id, zone) in zones.iter_mut().enumerate() {
			let mut neighbours: Vec<usize> = Vec::new();
			for at in &zone.coords {
				for n in self.grid.neighbours(*at) {
					let other = self.grid.tile(n).zone as usize;
					if other != id && !neighbours.contains(&other) {
						neighbours.push(other);
					}
				}
			}
			neighbours.sort_unstable();
			zone.neighbours = neighbours;
		}

		self.grid.zones = zones;
	}

	fn make_towns(&mut self, town_count: i32, average_tiles_per_zone: i32) {
		let mut available: Vec<usize> = (0..self.grid.zones.len()).collect();
		let mut blacklist: Vec<usize> = Vec::new();
		let mut towns: Vec<Town> = Vec::new();

		let step = 1.max(AVERAGE_TILES_PER_TOWN / average_tiles_per_zone);
		let mut retries = 100;
		if available.is_empty() {
			return;
		}

		let mut i = 0;
		while i < town_count {
			let index = self.rng.next_int_range(i * step, (i + 1) * step);
			let zone = available[index.rem_euclid(available.len() as i32) as usize];
			let mut found = false;

			if !blacklist.contains(&zone) {
				let mut town_coords = self.grid.zones[zone].coords.clone();
				self.rng.shuffle(&mut town_coords);

				for at in town_coords {
					if is_accessible(self.grid.tile(at))
						&& !self.is_edge(at)
						&& towns
							.iter()
							.all(|town| at.manhattan_to(town.coord) >= MIN_TOWN_DISTANCE)
					{
						towns.push(Town::new(i as usize, at));
						found = true;
						blacklist.push(zone);
						blacklist.extend(self.grid.zones[zone].neighbours.iter().copied());
						break;
					}
				}
			}

			if !found && retries > 0 {
				i -= 1;
				retries -= 1;
			}
			i += 1;
		}

		let left_to_place = town_count as usize - towns.len();
		available.retain(|zone| !blacklist.contains(zone));
		self.rng.shuffle(&mut available);

		for i in 0..left_to_place {
			if available.is_empty() {
				break;
			}
			let zone = available.remove(0);
			let mut town_coords = self.grid.zones[zone].coords.clone();
			self.rng.shuffle(&mut town_coords);

			for at in town_coords {
				if towns
					.iter()
					.all(|town| at.manhattan_to(town.coord) >= MIN_TOWN_DISTANCE)
				{
					towns.push(Town::new(i, at));
					break;
				}
			}
		}

		for (id, town) in towns.iter_mut().enumerate() {
			town.id = id;
		}

		for town in &towns {
			let zone = self.grid.tile(town.coord).zone as usize;
			self.grid.zones[zone].towns.push(town.id);
			let tile = self.grid.tile_mut(town.coord);
			tile.town = town.id as i16;
			tile.kind = TYPE_GRASS;
		}

		self.grid.towns = towns;
	}

	fn make_town_connections(&mut self) {
		let count = self.grid.towns.len();
		for id in 0..count {
			let mut others: Vec<usize> = (0..count).filter(|&other| other != id).collect();
			self.rng.shuffle(&mut others);
			let at_least = 3.min(others.len() as i32);
			let at_most = at_least.max(others.len() as i32 - 4);
			let wanted = self.rng.next_int_range(at_least, at_most + 1) as usize;
			let mut desired = others[..wanted].to_vec();
			desired.sort_unstable();
			self.grid.towns[id].desired = desired;
		}

		for id in 0..count {
			let kept: Vec<usize> = self.grid.towns[id]
				.desired
				.iter()
				.copied()
				.filter(|&other| !self.grid.towns[other].desired.contains(&id))
				.collect();
			self.grid.towns[id].desired = kept;
		}
	}
}

fn is_accessible(tile: &Tile) -> bool {
	!tile.is_water() && tile.kind != TYPE_MOUNTAIN
}

fn weight(neighbour: Coord, current: Coord, preferred: Direction) -> f32 {
	let direction = neighbour - current;
	if direction == preferred.coord() {
		1.75
	} else if direction == preferred.opposite().coord() {
		0.25
	} else {
		1.0
	}
}

/// `LinkedList.remove(Object)`: drops the first match, if any.
fn take(list: &mut Vec<Coord>, at: Coord) {
	if let Some(found) = list.iter().position(|&other| other == at) {
		list.remove(found);
	}
}

/// `Math.round(float)`.
fn java_round(value: f32) -> i32 {
	(value as f64 + 0.5).floor() as i32
}
