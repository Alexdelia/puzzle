use block_the_spreading_fire::{
	Containment, Map, Rng, Schedule, burnt_region, contain, improve_plan,
};

const HOT: f64 = 30.0;
const LATENESS_PENALTY: f64 = 20.0;

fn diamond(map: &Map, rng: &mut Rng, centre: usize) -> Vec<usize> {
	let radius = 1 + rng.below(4) as i64;
	let (x, y) = map.coords(centre);
	let mut cells = Vec::new();
	for dy in -radius..=radius {
		for dx in -radius..=radius {
			let (nx, ny) = (x as i64 + dx, y as i64 + dy);
			let on_map =
				nx >= 0 && ny >= 0 && (nx as usize) < map.width && (ny as usize) < map.height;
			if dx.abs() + dy.abs() <= radius && on_map {
				cells.push(map.index(nx as usize, ny as usize));
			}
		}
	}
	cells
}

fn reshape(map: &Map, rng: &mut Rng, inside: &[bool], current: &Containment) -> Vec<bool> {
	let mut candidate = inside.to_vec();
	let random_cut = |rng: &mut Rng| current.cuts[rng.below(current.cuts.len())];
	let random_burnt = |rng: &mut Rng| current.region[rng.below(current.region.len())];
	match rng.below(10) {
		0..=3 if !current.cuts.is_empty() => candidate[random_cut(rng)] = true,
		4..=7 => candidate[random_burnt(rng)] = false,
		8 if !current.cuts.is_empty() => {
			let centre = random_cut(rng);
			for i in diamond(map, rng, centre) {
				candidate[i] = true;
			}
		}
		_ => {
			let centre = random_burnt(rng);
			for i in diamond(map, rng, centre) {
				candidate[i] = false;
			}
		}
	}
	candidate
}

fn anneal(map: &Map, start: &[usize], rng: &mut Rng, schedule: &Schedule) -> Vec<usize> {
	let mut inside = burnt_region(map, start);
	let mut current = contain(map, &inside);
	let mut best = start.to_vec();
	let mut best_loss = map.total_value();
	while let Some(temperature) = schedule.temperature() {
		for _ in 0..256 {
			let candidate = contain(map, &reshape(map, rng, &inside, &current));
			let delta = candidate.energy(LATENESS_PENALTY) - current.energy(LATENESS_PENALTY);
			if delta <= 0.0 || rng.unit() < (-delta / temperature).exp() {
				inside.fill(false);
				for &i in &candidate.region {
					inside[i] = true;
				}
				if candidate.is_feasible() && candidate.loss < best_loss {
					best_loss = candidate.loss;
					best = candidate.cuts.clone();
				}
				current = candidate;
			}
		}
	}
	best
}

fn main() {
	improve_plan("region", HOT, anneal);
}
