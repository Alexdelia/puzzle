use rand::RngExt;

use super::{FrozenPrefix, SOLUTION_PER_GENERATION, Score};
use crate::{
	output_repr::{MAX_THRUST, MAX_TILT_CHANGE, MIN_THRUST, MIN_TILT_CHANGE, Solution, Step},
	referee::{car::Car, env::MAX_STEP},
};

const KEEP_RATE: f32 = 0.1;
const RANDOM_RATE: f32 = 0.1;

const BURST_RATE: f32 = 0.2;
const BURST_CHUNK_SIZE: usize = 16;

type MutationRate = f64;

const MUTATION_RATE_FACTOR: MutationRate = 0.1;

const fn gen_mutation_rate() -> [MutationRate; SOLUTION_PER_GENERATION] {
	let mut list = [0.0; SOLUTION_PER_GENERATION];
	let mut i = 0;
	while i < SOLUTION_PER_GENERATION {
		list[i] =
			i as MutationRate / SOLUTION_PER_GENERATION as MutationRate * MUTATION_RATE_FACTOR;
		i += 1;
	}
	list
}

const MUTATION_RATE: [MutationRate; SOLUTION_PER_GENERATION] = gen_mutation_rate();

fn computed_solution_size(
	frozen: &FrozenPrefix,
	best_finished_step_count: Option<usize>,
	step_to_checkpoint_limit: usize,
) -> usize {
	match best_finished_step_count {
		Some(finished_step_count) => finished_step_count,
		None => (frozen.resume_from_step + frozen.reentry_step_count + step_to_checkpoint_limit)
			.min(MAX_STEP),
	}
}

#[allow(clippy::too_many_arguments)]
pub fn breed_generation(
	mut solution_list: Box<[Solution]>,
	score_list: &[Score],
	step_count_list: &[usize],
	frozen_list: Box<[FrozenPrefix]>,
	car_init_state: &Car,
	best_finished_step_count: Option<usize>,
	step_to_checkpoint_limit: usize,
	burst: bool,
) -> (Box<[Solution]>, Box<[FrozenPrefix]>) {
	for i in 0..SOLUTION_PER_GENERATION {
		solution_list[i].truncate(step_count_list[i]);
	}

	let (mut ordered_solution_list, mut ordered_frozen_list) =
		sort(solution_list, score_list, frozen_list);

	let keep_count = (SOLUTION_PER_GENERATION as f32 * KEEP_RATE).ceil() as usize;
	let random_count = (SOLUTION_PER_GENERATION as f32 * RANDOM_RATE).ceil() as usize;

	let mut rng = rand::rng();

	let mut parent_a_index = 0;
	let mut parent_b_index = 1;
	for i in keep_count..(SOLUTION_PER_GENERATION - random_count) {
		let freeze_until = ordered_frozen_list[parent_a_index].resume_from_step;
		let solution_size = computed_solution_size(
			&ordered_frozen_list[parent_a_index],
			best_finished_step_count,
			step_to_checkpoint_limit,
		);
		ordered_solution_list[i] = breed(
			&mut rng,
			MUTATION_RATE[i],
			&ordered_solution_list[parent_a_index],
			&ordered_solution_list[parent_b_index],
			solution_size,
			freeze_until,
		);
		ordered_frozen_list[i] = ordered_frozen_list[parent_a_index];

		parent_b_index += 1;
		if parent_b_index >= keep_count {
			parent_a_index += 1;
			parent_b_index = parent_a_index + 1;
		}
	}

	if burst {
		let burst_count = (SOLUTION_PER_GENERATION as f32 * BURST_RATE).ceil() as usize;
		let bred_section_end = SOLUTION_PER_GENERATION - random_count;
		let burst_start = bred_section_end.saturating_sub(burst_count);
		for i in burst_start..bred_section_end {
			let freeze_until = ordered_frozen_list[i].resume_from_step;
			apply_burst(&mut rng, &mut ordered_solution_list[i], freeze_until);
		}
	}

	for i in 1..keep_count {
		let frozen = ordered_frozen_list[i];
		let solution_size =
			computed_solution_size(&frozen, best_finished_step_count, step_to_checkpoint_limit);
		let solution = &mut ordered_solution_list[i];
		resize_with_random(&mut rng, solution, solution_size);
		for step in solution.iter_mut().skip(frozen.resume_from_step) {
			mutate(&mut rng, MUTATION_RATE[i - 1], step);
		}
	}

	let from_scratch = FrozenPrefix::from_scratch(car_init_state);
	let random_size = computed_solution_size(
		&from_scratch,
		best_finished_step_count,
		step_to_checkpoint_limit,
	);
	for i in (SOLUTION_PER_GENERATION - random_count)..SOLUTION_PER_GENERATION {
		let solution = &mut ordered_solution_list[i];
		solution.clear();
		for _ in 0..random_size {
			solution.push(Step::random(&mut rng));
		}
		ordered_frozen_list[i] = from_scratch;
	}

	(ordered_solution_list, ordered_frozen_list)
}

fn resize_with_random(rng: &mut impl rand::Rng, solution: &mut Solution, solution_size: usize) {
	for _ in solution.len()..solution_size {
		solution.push(Step::random(rng));
	}
	solution.truncate(solution_size);
}

fn apply_burst(rng: &mut impl rand::Rng, solution: &mut Solution, freeze_until: usize) {
	if solution.len() <= freeze_until {
		return;
	}
	let mutable_len = solution.len() - freeze_until;
	let chunk_size = BURST_CHUNK_SIZE.min(mutable_len);
	let max_offset = mutable_len - chunk_size;
	let start = freeze_until + rng.random_range(0..=max_offset);
	for step in solution.iter_mut().skip(start).take(chunk_size) {
		*step = Step::random(rng);
	}
}

fn sort(
	solution_list: Box<[Solution]>,
	score_list: &[Score],
	frozen_list: Box<[FrozenPrefix]>,
) -> (Box<[Solution]>, Box<[FrozenPrefix]>) {
	let mut indices: Vec<usize> = (0..SOLUTION_PER_GENERATION).collect();
	indices.sort_by(|&a, &b| {
		score_list[a]
			.partial_cmp(&score_list[b])
			.unwrap_or(std::cmp::Ordering::Equal)
	});

	let mut sol_in = solution_list.into_vec();
	let frz_in = frozen_list.into_vec();

	let mut ordered_solution_list: Vec<Solution> = Vec::with_capacity(SOLUTION_PER_GENERATION);
	let mut ordered_frozen_list: Vec<FrozenPrefix> = Vec::with_capacity(SOLUTION_PER_GENERATION);

	for &i in &indices {
		ordered_solution_list.push(std::mem::take(&mut sol_in[i]));
		ordered_frozen_list.push(frz_in[i]);
	}

	(
		ordered_solution_list.into_boxed_slice(),
		ordered_frozen_list.into_boxed_slice(),
	)
}

fn breed(
	rng: &mut impl rand::Rng,
	mutation_rate: f64,
	parent_a: &Solution,
	parent_b: &Solution,
	solution_size: usize,
	freeze_until: usize,
) -> Solution {
	let mut child = Solution::new();

	for i in 0..solution_size {
		if i < freeze_until {
			if let Some(&step) = parent_a.get(i) {
				child.push(step);
			} else {
				child.push(Step::random(rng));
			}
		} else if let (Some(step_a), Some(step_b)) = (parent_a.get(i), parent_b.get(i)) {
			let mut step = Step {
				tilt: (step_a.tilt + step_b.tilt) / 2,
				thrust: ((step_a.thrust as u16 + step_b.thrust as u16) / 2) as u8,
			};
			mutate(rng, mutation_rate, &mut step);
			child.push(step);
		} else {
			child.push(Step::random(rng));
		};
	}

	child
}

pub fn mutate(rng: &mut impl rand::Rng, mutation_rate: f64, step: &mut Step) {
	if rng.random_bool(mutation_rate) {
		let bias = step.thrust as f32 / MAX_THRUST as f32;
		let r: f32 = rng.random();
		let p_max = bias * bias;
		let p_min = (1.0 - bias) * (1.0 - bias);
		if r < p_max {
			step.thrust = MAX_THRUST;
		} else if r < p_max + p_min {
			step.thrust = MIN_THRUST;
		} else {
			step.thrust = Step::random_thrust(rng);
		}
	}
	if rng.random_bool(mutation_rate) {
		let bias =
			(step.tilt - MIN_TILT_CHANGE) as f32 / (MAX_TILT_CHANGE - MIN_TILT_CHANGE) as f32;
		let r: f32 = rng.random();
		let p_max = bias * bias;
		let p_min = (1.0 - bias) * (1.0 - bias);
		if r < p_max {
			step.tilt = MAX_TILT_CHANGE;
		} else if r < p_max + p_min {
			step.tilt = MIN_TILT_CHANGE;
		} else {
			step.tilt = Step::random_titl(rng);
		}
	}
}
