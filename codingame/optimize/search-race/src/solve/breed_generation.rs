use rand::RngExt;

use super::{SOLUTION_PER_GENERATION, Score};
use crate::output_repr::{Solution, Step};

const KEEP_RATE: f32 = 0.1;
const RANDOM_RATE: f32 = 0.1;

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

pub fn breed_generation(
	mut solution_list: [Solution; SOLUTION_PER_GENERATION],
	score_list: [Score; SOLUTION_PER_GENERATION],
	step_count_list: [usize; SOLUTION_PER_GENERATION],
	pre_last_checkpoint_step_list: [usize; SOLUTION_PER_GENERATION],
) -> [Solution; SOLUTION_PER_GENERATION] {
	for i in 0..SOLUTION_PER_GENERATION {
		solution_list[i].truncate(step_count_list[i]);
	}

	let (mut ordered_solution_list, ordered_freeze_list) =
		sort(solution_list, score_list, pre_last_checkpoint_step_list);

	let keep_count = (SOLUTION_PER_GENERATION as f32 * KEEP_RATE).ceil() as usize;
	let random_count = (SOLUTION_PER_GENERATION as f32 * RANDOM_RATE).ceil() as usize;

	let max_solution_size = ordered_solution_list[0..keep_count]
		.iter()
		.map(|s| s.len())
		.max()
		.expect("no solutions to breed");
	let max_solution_size = max_solution_size.min(300);

	let mut rng = rand::rng();

	let mut parent_a_index = 0;
	let mut parent_b_index = 1;
	for i in keep_count..(SOLUTION_PER_GENERATION - random_count) {
		let parent_a = &ordered_solution_list[parent_a_index];
		let parent_b = &ordered_solution_list[parent_b_index];
		let freeze_until =
			ordered_freeze_list[parent_a_index].min(ordered_freeze_list[parent_b_index]);
		ordered_solution_list[i] = breed(
			&mut rng,
			MUTATION_RATE[i],
			parent_a,
			parent_b,
			max_solution_size,
			freeze_until,
		);

		parent_b_index += 1;
		if parent_b_index >= keep_count {
			parent_a_index += 1;
			parent_b_index = parent_a_index + 1;
		}
	}

	for (i, solution) in ordered_solution_list
		.iter_mut()
		.take(keep_count)
		.skip(1)
		.enumerate()
	{
		let freeze_until = ordered_freeze_list[i + 1];
		for step in solution.iter_mut().skip(freeze_until) {
			mutate(&mut rng, MUTATION_RATE[i], step);
		}
	}

	for solution in ordered_solution_list
		.iter_mut()
		.take(SOLUTION_PER_GENERATION)
		.skip(SOLUTION_PER_GENERATION - random_count)
	{
		for step in solution.iter_mut().take(max_solution_size) {
			*step = Step::random(&mut rng);
		}
	}

	ordered_solution_list
}

fn sort(
	solution_list: [Solution; SOLUTION_PER_GENERATION],
	score_list: [Score; SOLUTION_PER_GENERATION],
	pre_last_checkpoint_step_list: [usize; SOLUTION_PER_GENERATION],
) -> (
	[Solution; SOLUTION_PER_GENERATION],
	[usize; SOLUTION_PER_GENERATION],
) {
	let mut paired_list: [(Solution, Score, usize); SOLUTION_PER_GENERATION] = solution_list
		.into_iter()
		.zip(score_list)
		.zip(pre_last_checkpoint_step_list)
		.map(|((sol, score), freeze)| (sol, score, freeze))
		.collect::<Vec<_>>()
		.try_into()
		.expect("paired list size mismatch");

	paired_list.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

	let mut ordered_solution_list: Vec<Solution> = Vec::with_capacity(SOLUTION_PER_GENERATION);
	let mut ordered_freeze_list: Vec<usize> = Vec::with_capacity(SOLUTION_PER_GENERATION);
	for (sol, _, freeze) in paired_list {
		ordered_solution_list.push(sol);
		ordered_freeze_list.push(freeze);
	}

	(
		ordered_solution_list
			.try_into()
			.expect("ordered solution list size mismatch"),
		ordered_freeze_list
			.try_into()
			.expect("ordered freeze list size mismatch"),
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
	let mut child = Vec::with_capacity(solution_size);

	for i in 0..solution_size {
		if let (Some(step_a), Some(step_b)) = (parent_a.get(i), parent_b.get(i)) {
			let mut step = Step {
				tilt: (step_a.tilt + step_b.tilt) / 2,
				thrust: ((step_a.thrust as u16 + step_b.thrust as u16) / 2) as u8,
			};

			if i >= freeze_until {
				mutate(rng, mutation_rate, &mut step);
			}
			child.push(step);
		} else {
			child.push(Step::random(rng));
		};
	}

	child
}

pub fn mutate(rng: &mut impl rand::Rng, mutation_rate: f64, step: &mut Step) {
	if rng.random_bool(mutation_rate) {
		step.thrust = Step::random_thrust(rng);
	}
	if rng.random_bool(mutation_rate) {
		step.tilt = Step::random_titl(rng);
	}
}
