use std::cmp::Ordering;

use rand::{RngExt, SeedableRng, rngs::SmallRng};

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
	current_solution_list: &mut [Solution],
	next_solution_list: &mut [Solution],
	score_list: &[Score],
	step_count_list: &[usize],
	current_frozen_list: &[FrozenPrefix],
	next_frozen_list: &mut [FrozenPrefix],
	car_init_state: &Car,
	best_finished_step_count: Option<usize>,
	step_to_checkpoint_limit: usize,
	burst: bool,
) {
	for i in 0..SOLUTION_PER_GENERATION {
		current_solution_list[i].truncate(step_count_list[i]);
	}

	let mut sort_order: Vec<usize> = (0..SOLUTION_PER_GENERATION).collect();
	sort_order.sort_by(|&a, &b| {
		score_list[a]
			.partial_cmp(&score_list[b])
			.unwrap_or(Ordering::Equal)
	});

	let keep_count = (SOLUTION_PER_GENERATION as f32 * KEEP_RATE).ceil() as usize;
	let random_count = (SOLUTION_PER_GENERATION as f32 * RANDOM_RATE).ceil() as usize;

	let mut rng = SmallRng::from_rng(&mut rand::rng());

	for i in 0..keep_count {
		let src = sort_order[i];
		next_solution_list[i] = current_solution_list[src];
		next_frozen_list[i] = current_frozen_list[src];
	}

	let mut parent_a_pos = 0;
	let mut parent_b_pos = 1;
	for i in keep_count..(SOLUTION_PER_GENERATION - random_count) {
		let parent_a_idx = sort_order[parent_a_pos];
		let parent_b_idx = sort_order[parent_b_pos];
		let parent_a_frozen = current_frozen_list[parent_a_idx];
		let freeze_until = parent_a_frozen.resume_from_step;
		let solution_size = computed_solution_size(
			&parent_a_frozen,
			best_finished_step_count,
			step_to_checkpoint_limit,
		);
		breed_into(
			&mut next_solution_list[i],
			&mut rng,
			MUTATION_RATE[i],
			&current_solution_list[parent_a_idx],
			&current_solution_list[parent_b_idx],
			solution_size,
			freeze_until,
		);
		next_frozen_list[i] = parent_a_frozen;

		parent_b_pos += 1;
		if parent_b_pos >= keep_count {
			parent_a_pos += 1;
			parent_b_pos = parent_a_pos + 1;
		}
	}

	if burst {
		let burst_count = (SOLUTION_PER_GENERATION as f32 * BURST_RATE).ceil() as usize;
		let bred_section_end = SOLUTION_PER_GENERATION - random_count;
		let burst_start = bred_section_end.saturating_sub(burst_count);
		for i in burst_start..bred_section_end {
			let freeze_until = next_frozen_list[i].resume_from_step;
			apply_burst(&mut rng, &mut next_solution_list[i], freeze_until);
		}
	}

	for i in 1..keep_count {
		let frozen = next_frozen_list[i];
		let solution_size =
			computed_solution_size(&frozen, best_finished_step_count, step_to_checkpoint_limit);
		let solution = &mut next_solution_list[i];
		resize_with_random(&mut rng, solution, solution_size);
		let len = solution.len();
		let start = frozen.resume_from_step.min(len);
		for step in &mut solution.steps[start..len] {
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
		random_into(&mut next_solution_list[i], &mut rng, random_size);
		next_frozen_list[i] = from_scratch;
	}
}

fn breed_into(
	dst: &mut Solution,
	rng: &mut impl rand::Rng,
	mutation_rate: f64,
	parent_a: &Solution,
	parent_b: &Solution,
	solution_size: usize,
	freeze_until: usize,
) {
	let pa_len = parent_a.len();
	let pb_len = parent_b.len();
	let pa = &parent_a.steps;
	let pb = &parent_b.steps;

	for k in 0..solution_size {
		let step = if k < freeze_until {
			if k < pa_len { pa[k] } else { Step::random(rng) }
		} else if k < pa_len && k < pb_len {
			let sa = pa[k];
			let sb = pb[k];
			let mut s = Step {
				tilt: (sa.tilt + sb.tilt) / 2,
				thrust: ((sa.thrust as u16 + sb.thrust as u16) / 2) as u8,
			};
			mutate(rng, mutation_rate, &mut s);
			s
		} else {
			Step::random(rng)
		};
		dst.steps[k] = step;
	}
	dst.len = solution_size as u16;
}

fn random_into(dst: &mut Solution, rng: &mut impl rand::Rng, solution_size: usize) {
	for k in 0..solution_size {
		dst.steps[k] = Step::random(rng);
	}
	dst.len = solution_size as u16;
}

fn resize_with_random(rng: &mut impl rand::Rng, solution: &mut Solution, solution_size: usize) {
	let old_len = solution.len();
	for k in old_len..solution_size {
		solution.steps[k] = Step::random(rng);
	}
	solution.len = solution_size as u16;
}

fn apply_burst(rng: &mut impl rand::Rng, solution: &mut Solution, freeze_until: usize) {
	let len = solution.len();
	if len <= freeze_until {
		return;
	}
	let mutable_len = len - freeze_until;
	let chunk_size = BURST_CHUNK_SIZE.min(mutable_len);
	let max_offset = mutable_len - chunk_size;
	let start = freeze_until + rng.random_range(0..=max_offset);
	for step in &mut solution.steps[start..start + chunk_size] {
		*step = Step::random(rng);
	}
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
