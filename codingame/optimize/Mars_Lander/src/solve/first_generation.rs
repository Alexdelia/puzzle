use rand::Rng;

use crate::{
	output_repr::{Solution, Step, ThrustChange},
	solve::SOLUTION_PER_GENERATION,
};

const INITIAL_SOLUTION_STEP_SIZE: usize = 128;

pub fn init_first_generation() -> [Solution; SOLUTION_PER_GENERATION] {
	let mut generation = Vec::with_capacity(SOLUTION_PER_GENERATION);

	let mut rng = rand::rng();

	for _ in 0..SOLUTION_PER_GENERATION {
		let mut solution = Vec::with_capacity(INITIAL_SOLUTION_STEP_SIZE);
		for _ in 0..INITIAL_SOLUTION_STEP_SIZE {
			solution.push(Step {
				tilt: rng.random_range(-15..=15),
				thrust: match rng.random_range(0..3) {
					0 => ThrustChange::Decrease,
					1 => ThrustChange::Keep,
					_ => ThrustChange::Increase,
				},
			});
		}
		generation.push(solution);
	}

	generation.try_into().expect("generation size mismatch")
}
