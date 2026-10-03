use block_the_spreading_fire::{Cell, Map, Rng, Schedule, contain, improve_plan};

const LATENESS_PENALTY: f64 = 1000.0;
const DIRECTIONS: [(i64, i64); 8] = [
	(1, 0),
	(1, -1),
	(0, -1),
	(-1, -1),
	(-1, 0),
	(-1, 1),
	(0, 1),
	(1, 1),
];

#[derive(Clone, Copy)]
enum Edit {
	Turn(usize, u8),
	Insert(usize, u8),
	Remove(usize),
}

#[derive(Clone)]
struct Line {
	anchor: usize,
	steps: Vec<u8>,
}

impl Line {
	fn walk(&self, map: &Map) -> Vec<usize> {
		let mut cells = vec![self.anchor];
		let (mut x, mut y) = map.coords(self.anchor);
		for &step in &self.steps {
			let (dx, dy) = DIRECTIONS[step as usize];
			let (nx, ny) = (x as i64 + dx, y as i64 + dy);
			if nx < 0 || ny < 0 || nx as usize >= map.width || ny as usize >= map.height {
				break;
			}
			(x, y) = (nx as usize, ny as usize);
			let i = map.index(x, y);
			if map.cells[i] == Cell::Safe {
				break;
			}
			cells.push(i);
		}
		cells
	}

	fn edited(&self, edits: &[Edit]) -> Line {
		let mut next = self.clone();
		for &edit in edits {
			match edit {
				Edit::Turn(at, direction) => next.steps[at] = direction,
				Edit::Insert(at, direction) => {
					next.steps.insert(at, direction);
					next.steps.pop();
				}
				Edit::Remove(at) => {
					let last = *next.steps.last().unwrap();
					next.steps.remove(at);
					next.steps.push(last);
				}
			}
		}
		next
	}

	fn energy(&self, map: &Map) -> (f64, Option<(u32, Vec<usize>)>) {
		let mut inside = vec![true; map.cells.len()];
		for i in self.walk(map) {
			inside[i] = false;
		}
		let containment = contain(map, &inside);
		let feasible = containment
			.is_feasible()
			.then(|| (containment.loss, containment.cuts.clone()));
		(containment.energy(LATENESS_PENALTY), feasible)
	}
}

fn anchors(map: &Map) -> Vec<usize> {
	(0..map.cells.len())
		.filter(|&i| {
			i != map.start
				&& map.cells[i] != Cell::Safe
				&& map
					.neighbours(i)
					.iter()
					.any(|&n| map.cells[n] == Cell::Safe)
		})
		.collect()
}

fn shuffled<T>(rng: &mut Rng, mut items: Vec<T>) -> Vec<T> {
	for k in (1..items.len()).rev() {
		items.swap(k, rng.below(k + 1));
	}
	items
}

fn edits(rng: &mut Rng, length: usize) -> Vec<Vec<Edit>> {
	let directions = 0..DIRECTIONS.len() as u8;
	let mut singles = Vec::new();
	for a in 0..length {
		singles.push(vec![Edit::Remove(a)]);
		for d in directions.clone() {
			singles.push(vec![Edit::Turn(a, d)]);
			singles.push(vec![Edit::Insert(a, d)]);
		}
	}
	let mut pairs = Vec::new();
	for a in 0..length {
		for b in a + 1..length {
			for da in directions.clone() {
				for db in directions.clone() {
					pairs.push(vec![Edit::Turn(a, da), Edit::Turn(b, db)]);
				}
			}
		}
	}
	let mut edits = shuffled(rng, singles);
	edits.extend(shuffled(rng, pairs));
	edits
}

fn descend(map: &Map, rng: &mut Rng, schedule: &Schedule, mut line: Line) -> (f64, Line) {
	let (mut energy, _) = line.energy(map);
	let mut improved = true;
	while improved && schedule.has_time() {
		improved = false;
		let length = (line.walk(map).len() + 1).min(line.steps.len());
		for edit in edits(rng, length) {
			let next = line.edited(&edit);
			let (next_energy, _) = next.energy(map);
			if next_energy < energy {
				(line, energy, improved) = (next, next_energy, true);
				break;
			}
		}
	}
	(energy, line)
}

fn straight_lines(map: &Map) -> Vec<Line> {
	let length = map.width + map.height;
	let mut lines: Vec<(f64, Line)> = anchors(map)
		.into_iter()
		.flat_map(|anchor| {
			(0..DIRECTIONS.len() as u8).map(move |direction| Line {
				anchor,
				steps: vec![direction; length],
			})
		})
		.map(|line| (line.energy(map).0, line))
		.collect();
	lines.sort_by(|a, b| a.0.total_cmp(&b.0));
	lines.into_iter().map(|(_, line)| line).collect()
}

fn search(map: &Map, start: &[usize], rng: &mut Rng, schedule: &Schedule) -> Vec<usize> {
	let lines = straight_lines(map);
	let first = rng.below(4);
	let mut best = (map.total_value(), start.to_vec());
	for line in lines.into_iter().skip(first).step_by(4) {
		if !schedule.has_time() {
			break;
		}
		let (_, line) = descend(map, rng, schedule, line);
		if let (_, Some((loss, cuts))) = line.energy(map)
			&& loss < best.0
		{
			best = (loss, cuts);
		}
	}
	best.1
}

fn main() {
	improve_plan("line", 1.0, search);
}
