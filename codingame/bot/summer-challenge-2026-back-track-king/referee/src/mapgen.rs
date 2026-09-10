use crate::map::{MOUNTAIN, Map, PLAIN, RIVER, Town};
use crate::rng::Rng;

pub const MIN_WIDTH: usize = 21;
pub const MAX_WIDTH: usize = 30;
pub const MIN_HEIGHT: usize = 14;
pub const MAX_HEIGHT: usize = 20;
pub const MIN_TOWNS: usize = 4;
pub const MAX_TOWNS: usize = 12;

const UNCLAIMED: u16 = u16::MAX;

#[derive(Clone, Copy, Debug, Default)]
pub struct GenParams {
	pub width: Option<usize>,
	pub height: Option<usize>,
	pub towns: Option<usize>,
}

pub fn generate(seed: u64, params: &GenParams) -> Map {
	let mut rng = Rng::new(seed);
	let width = params
		.width
		.unwrap_or_else(|| rng.between(MIN_WIDTH, MAX_WIDTH));
	let height = params
		.height
		.unwrap_or_else(|| rng.between(MIN_HEIGHT, MAX_HEIGHT));
	let cells = width * height;
	let town_count = params
		.towns
		.unwrap_or_else(|| rng.between(MIN_TOWNS, (cells / 45).clamp(MIN_TOWNS, MAX_TOWNS)));
	let neighbors = neighbor_table(width, height);

	let mut region_count = (town_count * 4).clamp(16, cells / 8);
	let mut attempt = 0;
	let (region, town_regions) = loop {
		let mut region = partition(&mut rng, cells, region_count, &neighbors);
		let total = absorb_lone_cells(&mut region, &neighbors, region_count);
		let adjacency = region_adjacency(&region, &neighbors, total);
		if let Some(picked) = independent_set(&mut rng, &adjacency, total, town_count) {
			break (region, picked);
		}
		attempt += 1;
		assert!(
			attempt < 64,
			"cannot fit {town_count} towns into {cells} cells"
		);
		region_count = (region_count * 3 / 2).min(cells / 2);
	};

	let mut terrain = vec![PLAIN; cells];
	for _ in 0..rng.between(1, 3) {
		carve_river(&mut rng, width, height, &mut terrain);
	}
	for _ in 0..rng.between(3, 6) {
		raise_mountain(&mut rng, cells, &neighbors, &mut terrain);
	}

	let mut town_cells: Vec<usize> = town_regions
		.iter()
		.map(|&r| {
			let members: Vec<usize> = (0..cells).filter(|&c| region[c] as usize == r).collect();
			*rng.pick(&members)
		})
		.collect();
	rng.shuffle(&mut town_cells);
	for &cell in &town_cells {
		terrain[cell] = PLAIN;
	}

	let desired = desired_connections(&mut rng, town_count);
	let towns = town_cells
		.iter()
		.enumerate()
		.map(|(id, &cell)| Town {
			x: cell % width,
			y: cell / width,
			desired: desired[id].clone(),
		})
		.collect();

	Map::new(width, height, terrain, region, towns)
}

fn neighbor_table(width: usize, height: usize) -> Vec<[i32; 4]> {
	let mut out = vec![[-1i32; 4]; width * height];
	for y in 0..height {
		for x in 0..width {
			let cell = y * width + x;
			let w = width as i32;
			out[cell] = [
				if y > 0 { cell as i32 - w } else { -1 },
				if x + 1 < width { cell as i32 + 1 } else { -1 },
				if y + 1 < height { cell as i32 + w } else { -1 },
				if x > 0 { cell as i32 - 1 } else { -1 },
			];
		}
	}
	out
}

fn partition(rng: &mut Rng, cells: usize, count: usize, neighbors: &[[i32; 4]]) -> Vec<u16> {
	let mut region = vec![UNCLAIMED; cells];
	let mut order: Vec<usize> = (0..cells).collect();
	rng.shuffle(&mut order);

	let mut frontier: Vec<Vec<u32>> = vec![Vec::new(); count];
	let mut growing: Vec<u16> = Vec::with_capacity(count);
	for (id, &seed) in order.iter().take(count).enumerate() {
		region[seed] = id as u16;
		frontier[id].extend(open_neighbors(neighbors, &region, seed));
		growing.push(id as u16);
	}

	while !growing.is_empty() {
		let slot = rng.below(growing.len());
		let id = growing[slot] as usize;
		loop {
			let Some(pick) = pop_random(rng, &mut frontier[id]) else {
				growing.swap_remove(slot);
				break;
			};
			if region[pick] != UNCLAIMED {
				continue;
			}
			region[pick] = id as u16;
			let open: Vec<u32> = open_neighbors(neighbors, &region, pick).collect();
			frontier[id].extend(open);
			break;
		}
	}
	region
}

fn absorb_lone_cells(region: &mut [u16], neighbors: &[[i32; 4]], count: usize) -> usize {
	let mut size = vec![0usize; count];
	for &id in region.iter() {
		size[id as usize] += 1;
	}
	loop {
		let Some(lone) = (0..count).find(|&id| size[id] == 1) else {
			break;
		};
		let cell = region.iter().position(|&id| id as usize == lone).unwrap();
		let host = neighbors[cell]
			.iter()
			.filter(|&&n| n >= 0)
			.map(|&n| region[n as usize] as usize)
			.filter(|&id| id != lone)
			.max_by_key(|&id| size[id]);
		let Some(host) = host else { break };
		region[cell] = host as u16;
		size[host] += 1;
		size[lone] = 0;
	}

	let mut renumber = vec![u16::MAX; count];
	let mut total = 0u16;
	for id in region.iter() {
		if renumber[*id as usize] == u16::MAX {
			renumber[*id as usize] = total;
			total += 1;
		}
	}
	for id in region.iter_mut() {
		*id = renumber[*id as usize];
	}
	total as usize
}

fn open_neighbors<'a>(
	neighbors: &'a [[i32; 4]],
	region: &'a [u16],
	cell: usize,
) -> impl Iterator<Item = u32> + 'a {
	neighbors[cell]
		.iter()
		.copied()
		.filter(move |&n| n >= 0 && region[n as usize] == UNCLAIMED)
		.map(|n| n as u32)
}

fn pop_random(rng: &mut Rng, list: &mut Vec<u32>) -> Option<usize> {
	(!list.is_empty()).then(|| {
		let i = rng.below(list.len());
		list.swap_remove(i) as usize
	})
}

fn region_adjacency(region: &[u16], neighbors: &[[i32; 4]], count: usize) -> Vec<Vec<u16>> {
	let mut adjacency = vec![Vec::new(); count];
	for cell in 0..region.len() {
		let a = region[cell] as usize;
		for &n in &neighbors[cell] {
			if n < 0 {
				continue;
			}
			let b = region[n as usize];
			if b as usize != a && !adjacency[a].contains(&b) {
				adjacency[a].push(b);
			}
		}
	}
	adjacency
}

fn independent_set(
	rng: &mut Rng,
	adjacency: &[Vec<u16>],
	count: usize,
	want: usize,
) -> Option<Vec<usize>> {
	for _ in 0..64 {
		let mut order: Vec<usize> = (0..count).collect();
		rng.shuffle(&mut order);
		let mut taken = vec![false; count];
		let mut picked = Vec::with_capacity(want);
		for id in order {
			if taken[id] || adjacency[id].iter().any(|&n| taken[n as usize]) {
				continue;
			}
			taken[id] = true;
			picked.push(id);
			if picked.len() == want {
				return Some(picked);
			}
		}
	}
	None
}

fn carve_river(rng: &mut Rng, width: usize, height: usize, terrain: &mut [u8]) {
	let vertical = rng.chance(1, 2);
	let (mut x, mut y) = if vertical {
		(rng.below(width), 0)
	} else {
		(0, rng.below(height))
	};
	let (span, cross) = if vertical {
		(height, width)
	} else {
		(width, height)
	};
	for _ in 0..4 * span {
		terrain[y * width + x] = RIVER;
		let (along, across) = if vertical {
			(&mut y, &mut x)
		} else {
			(&mut x, &mut y)
		};
		let left = *across > 0;
		let right = *across + 1 < cross;
		if rng.chance(3, 5) || !(left || right) {
			*along += 1;
			if *along == span {
				return;
			}
		} else if right && (!left || rng.chance(1, 2)) {
			*across += 1;
		} else {
			*across -= 1;
		}
	}
}

fn raise_mountain(rng: &mut Rng, cells: usize, neighbors: &[[i32; 4]], terrain: &mut [u8]) {
	let size = rng.between(5, 25);
	let mut frontier = vec![rng.below(cells) as u32];
	let mut grown = 0;
	while grown < size {
		let Some(cell) = pop_random(rng, &mut frontier) else {
			return;
		};
		if terrain[cell] == MOUNTAIN {
			continue;
		}
		terrain[cell] = MOUNTAIN;
		grown += 1;
		for &n in &neighbors[cell] {
			if n >= 0 && terrain[n as usize] != MOUNTAIN {
				frontier.push(n as u32);
			}
		}
	}
}

fn desired_connections(rng: &mut Rng, town_count: usize) -> Vec<Vec<u8>> {
	let mut cycle: Vec<usize> = (0..town_count).collect();
	rng.shuffle(&mut cycle);
	let mut out = vec![Vec::new(); town_count];
	let mut in_degree = vec![0usize; town_count];
	for i in 0..town_count {
		let (from, to) = (cycle[i], cycle[(i + 1) % town_count]);
		out[from].push(to);
		in_degree[to] += 1;
	}

	for _ in 0..rng.below(town_count + 1) {
		let from = rng.below(town_count);
		let to = rng.below(town_count);
		if from == to || out[from].contains(&to) || out[to].contains(&from) {
			continue;
		}
		out[from].push(to);
		in_degree[to] += 1;
	}

	for _ in 0..rng.below(3) {
		let town = rng.below(town_count);
		if out[town].iter().any(|&to| in_degree[to] < 2) {
			continue;
		}
		for &to in &out[town] {
			in_degree[to] -= 1;
		}
		out[town].clear();
	}

	out.into_iter()
		.map(|mut list| {
			list.sort_unstable();
			list.iter().map(|&t| t as u8).collect()
		})
		.collect()
}
