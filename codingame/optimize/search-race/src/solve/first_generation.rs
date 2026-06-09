use std::{fs, path::Path};

use rand::{SeedableRng, rngs::SmallRng};

use crate::{
	output_repr::{Solution, Step},
	output_solution::{OUTPUT_DIR, OUTPUT_SOLUTION_REPR_FILE},
	referee::env::MAX_STEP,
};

const INITIAL_SOLUTION_STEP_SIZE: usize = MAX_STEP;

pub fn init_first_generation(
	validator_name: &str,
	fresh: bool,
	solution_list: &mut [Solution],
) -> Result<bool, String> {
	let mut rng = SmallRng::from_rng(&mut rand::rng());

	let stored_solution = if fresh {
		None
	} else {
		read_stored_solution(validator_name)?
	};
	let loaded = stored_solution.is_some();

	let mut start = 0;
	if let Some(solution) = stored_solution {
		solution_list[0] = solution;
		start = 1;
	}

	for i in start..solution_list.len() {
		let dst = &mut solution_list[i];
		for k in 0..INITIAL_SOLUTION_STEP_SIZE {
			dst.steps[k] = Step::random(&mut rng);
		}
		dst.len = INITIAL_SOLUTION_STEP_SIZE as u16;
	}

	Ok(loaded)
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
