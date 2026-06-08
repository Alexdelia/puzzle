use std::{fs, path::Path};

use crate::{
	output_repr::{Solution, Step},
	output_solution::{OUTPUT_DIR, OUTPUT_SOLUTION_REPR_FILE},
	referee::env::MAX_STEP,
	solve::SOLUTION_PER_GENERATION,
};

const INITIAL_SOLUTION_STEP_SIZE: usize = MAX_STEP;

pub fn init_first_generation(
	validator_name: &str,
	fresh: bool,
) -> Result<(Box<[Solution]>, bool), String> {
	let mut generation: Vec<Solution> = Vec::with_capacity(SOLUTION_PER_GENERATION);

	let mut rng = rand::rng();

	let stored_solution = if fresh {
		None
	} else {
		read_stored_solution(validator_name)?
	};
	let loaded = stored_solution.is_some();
	if let Some(solution) = stored_solution {
		generation.push(solution);
	}

	let i = generation.len();
	for _ in i..SOLUTION_PER_GENERATION {
		let mut solution = Solution::new();
		for _ in 0..INITIAL_SOLUTION_STEP_SIZE {
			solution.push(Step::random(&mut rng));
		}
		generation.push(solution);
	}

	assert_eq!(generation.len(), SOLUTION_PER_GENERATION);
	Ok((generation.into_boxed_slice(), loaded))
}

fn read_stored_solution(validator_name: &str) -> Result<Option<Solution>, String> {
	let path = Path::new(OUTPUT_DIR)
		.join(validator_name)
		.join(OUTPUT_SOLUTION_REPR_FILE);
	if !path.exists() {
		return Ok(None);
	}

	let stored_content =
		fs::read_to_string(path).map_err(|e| format!("failed to read stored solution: {e}"))?;
	let stored_solution: Solution = ron::from_str(&stored_content)
		.map_err(|e| format!("failed to parse stored solution RON: {e}"))?;

	Ok(Some(stored_solution))
}
