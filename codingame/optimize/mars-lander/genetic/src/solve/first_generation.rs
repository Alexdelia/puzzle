use std::{fs, path::Path};

use crate::{
	OUTPUT_DIR, OUTPUT_SOLUTION_REPR_FILE,
	output_repr::{Solution, Step},
	solve::SOLUTION_PER_GENERATION,
};

const INITIAL_SOLUTION_STEP_SIZE: usize = 84;

pub fn init_first_generation(
	validator_name: &str,
) -> Result<[Solution; SOLUTION_PER_GENERATION], String> {
	let mut generation = Vec::with_capacity(SOLUTION_PER_GENERATION);

	let mut rng = rand::rng();

	let stored_solution = read_stored_solution(validator_name)?;
	if let Some(solution) = stored_solution {
		generation.push(solution);
	}

	let i = generation.len();
	for _ in i..SOLUTION_PER_GENERATION {
		let mut solution = Vec::with_capacity(INITIAL_SOLUTION_STEP_SIZE);
		for _ in 0..INITIAL_SOLUTION_STEP_SIZE {
			solution.push(Step::random(&mut rng));
		}
		generation.push(solution);
	}

	Ok(generation.try_into().expect("generation size mismatch"))
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
