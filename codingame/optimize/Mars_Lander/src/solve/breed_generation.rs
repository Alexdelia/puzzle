use super::{SOLUTION_PER_GENERATION, Score};
use crate::output_repr::{Solution, Step};

const KEEP_RATE: f32 = 0.1;
const RANDOM_RATE: f32 = 0.1;

const MUTATION_RATE: f64 = 0.01;

pub fn breed_generation(
	solution_list: [Solution; SOLUTION_PER_GENERATION],
	score_list: [Score; SOLUTION_PER_GENERATION],
) -> [Solution; SOLUTION_PER_GENERATION] {
	let mut ordered_solution_list = sort(solution_list, score_list);

	let keep_count = (SOLUTION_PER_GENERATION as f32 * KEEP_RATE).ceil() as usize;
	let random_count = (SOLUTION_PER_GENERATION as f32 * RANDOM_RATE).ceil() as usize;

	let max_solution_size = ordered_solution_list[0..keep_count]
		.iter()
		.map(|s| s.len())
		.max()
		.expect("no solutions to breed");

	let mut rng = rand::rng();

	let mut parent_a_index = 0;
	let mut parent_b_index = 1;
	for i in keep_count..(SOLUTION_PER_GENERATION - random_count) {
		let parent_a = &ordered_solution_list[parent_a_index];
		let parent_b = &ordered_solution_list[parent_b_index];
		ordered_solution_list[i] = breed(&mut rng, parent_a, parent_b, max_solution_size);

		parent_b_index += 1;
		if parent_b_index >= keep_count {
			parent_a_index += 1;
			parent_b_index = parent_a_index + 1;
		}
	}

	for solution in ordered_solution_list.iter_mut().take(keep_count).skip(1) {
		for step in solution.iter_mut() {
			mutate(&mut rng, step);
		}
	}

	for solution in ordered_solution_list
		.iter_mut()
		.take(SOLUTION_PER_GENERATION)
		.skip(SOLUTION_PER_GENERATION - random_count)
	{
		for i in 0..max_solution_size {
			solution[i] = Step::random(&mut rng);
		}
	}

	ordered_solution_list
}

fn sort(
	solution_list: [Solution; SOLUTION_PER_GENERATION],
	score_list: [Score; SOLUTION_PER_GENERATION],
) -> [Solution; SOLUTION_PER_GENERATION] {
	let mut paired_list: [(Solution, Score); SOLUTION_PER_GENERATION] = solution_list
		.into_iter()
		.zip(score_list)
		.collect::<Vec<_>>()
		.try_into()
		.expect("paired list size mismatch");

	paired_list.sort_by_key(|&(_, score)| score);

	let (ordered_solution_list, _) = paired_list
		.into_iter()
		.unzip::<Solution, Score, Vec<_>, Vec<_>>();

	ordered_solution_list
		.try_into()
		.expect("ordered solution list size mismatch")
}

fn breed(
	rng: &mut impl rand::Rng,
	parent_a: &Solution,
	parent_b: &Solution,
	solution_size: usize,
) -> Solution {
	let mut child = Vec::with_capacity(solution_size);

	for i in 0..solution_size {
		if let (Some(step_a), Some(step_b)) = (parent_a.get(i), parent_b.get(i)) {
			let mut step = Step {
				tilt: (step_a.tilt + step_b.tilt) / 2,
				thrust: if rng.random_bool(0.5) {
					step_a.thrust
				} else {
					step_b.thrust
				},
			};
			assert!(
				step.tilt >= -15 && step.tilt <= 15,
				"Tilt out of bounds after breeding: {tilt}",
				tilt = step.tilt
			);

			mutate(rng, &mut step);
			child.push(step);
		} else {
			child.push(Step::random(rng));
		};
	}

	child
}

pub fn mutate(rng: &mut impl rand::Rng, step: &mut Step) {
	if rng.random_bool(MUTATION_RATE) {
		step.thrust = Step::random_thrust(rng);
	}
	if rng.random_bool(MUTATION_RATE) {
		step.tilt = Step::random_titl(rng);
	}
}
