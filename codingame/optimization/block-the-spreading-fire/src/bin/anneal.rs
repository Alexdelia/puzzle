use block_the_spreading_fire::{Action, Cell, Game, Map, NO_FIRE, Rng, Schedule, improve_plan};

const HOT: f64 = 40.0;

fn play_lenient(map: &Map, plan: &[usize]) -> (u32, Vec<usize>) {
	let mut game = Game::new(map);
	let mut kept = Vec::with_capacity(plan.len());
	for &i in plan {
		if game.is_over() {
			break;
		}
		while game.cooldown() > 0 && game.progress()[i] == NO_FIRE {
			game.play(Action::Wait).unwrap();
		}
		if game.progress()[i] != NO_FIRE {
			continue;
		}
		game.play(Action::Cut(i)).unwrap();
		kept.push(i);
	}
	game.burn_out();
	(game.remaining_value(), kept)
}

fn nearby_cell(map: &Map, rng: &mut Rng, i: usize) -> Option<usize> {
	let (x, y) = map.coords(i);
	let nx = (x as i64 + rng.below(5) as i64 - 2).clamp(0, map.width as i64 - 1);
	let ny = (y as i64 + rng.below(5) as i64 - 2).clamp(0, map.height as i64 - 1);
	let n = map.index(nx as usize, ny as usize);
	(map.cells[n] != Cell::Safe).then_some(n)
}

fn mutate(map: &Map, rng: &mut Rng, plan: &[usize], open: &[usize]) -> Vec<usize> {
	let mut next = plan.to_vec();
	match rng.below(5) {
		0 => {
			let cell = if !next.is_empty() && rng.below(4) != 0 {
				let anchor = next[rng.below(next.len())];
				nearby_cell(map, rng, anchor)
			} else {
				Some(open[rng.below(open.len())])
			};
			if let Some(cell) = cell {
				let at = rng.below(next.len() + 1);
				next.insert(at, cell);
			}
		}
		1 if !next.is_empty() => {
			next.remove(rng.below(next.len()));
		}
		2 if !next.is_empty() => {
			let at = rng.below(next.len());
			if let Some(cell) = nearby_cell(map, rng, next[at]) {
				next[at] = cell;
			}
		}
		3 if next.len() >= 2 => {
			let cell = next.remove(rng.below(next.len()));
			let to = rng.below(next.len() + 1);
			next.insert(to, cell);
		}
		_ if next.len() >= 2 => {
			let at = rng.below(next.len() - 1);
			next.swap(at, at + 1);
		}
		_ => {}
	}
	next
}

fn anneal(map: &Map, start: &[usize], rng: &mut Rng, schedule: &Schedule) -> Vec<usize> {
	let open: Vec<usize> = (0..map.cells.len())
		.filter(|&i| map.cells[i] != Cell::Safe)
		.collect();
	let (mut score, mut plan) = play_lenient(map, start);
	let mut best = (score, plan.clone());
	while let Some(temperature) = schedule.temperature() {
		for _ in 0..64 {
			let (candidate_score, kept) = play_lenient(map, &mutate(map, rng, &plan, &open));
			let delta = candidate_score as f64 - score as f64;
			if delta >= 0.0 || rng.unit() < (delta / temperature).exp() {
				score = candidate_score;
				plan = kept;
				if score > best.0 {
					best = (score, plan.clone());
				}
			}
		}
	}
	best.1
}

fn main() {
	improve_plan("anneal", HOT, anneal);
}
