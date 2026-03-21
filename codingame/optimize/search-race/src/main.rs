mod dist;
mod output_repr;
mod parse;
mod referee;
mod segment;
mod solve;
#[cfg(feature = "visualize")]
mod visualize;

use crate::{output_repr::Solution, referee::solution_into_real_output};

fn main() -> Result<(), String> {
	let path = parse::get_path()?;
	let validator_name = get_validator_name(&path);

	let (car_init_state, checkpoint_list) = parse::parse(&path)?;

	#[cfg(feature = "visualize")]
	visualize::clear_output(&validator_name);
	#[cfg(feature = "visualize")]
	let base_doc = visualize::checkpoint_list(&checkpoint_list);

	let solution = solve::solve(
		&validator_name,
		&checkpoint_list,
		&car_init_state,
		#[cfg(feature = "visualize")]
		base_doc,
	)?;

	output_solution(&solution, &validator_name)
}

const OUTPUT_DIR: &str = "output";
const OUTPUT_SOLUTION_REPR_FILE: &str = "solution_repr.ron";
const OUTPUT_SOLUTION_REAL_FILE: &str = "solution.txt";

fn get_validator_name(path: &str) -> String {
	std::path::Path::new(path)
		.file_stem()
		.expect("invalid path")
		.to_string_lossy()
		.to_string()
}

fn output_solution(solution_repr: &Solution, validator_name: &str) -> Result<(), String> {
	let output_dir = std::path::Path::new(OUTPUT_DIR).join(validator_name);
	if !output_dir.exists() {
		std::fs::create_dir_all(&output_dir).unwrap_or_else(|e| {
			panic!("failed to create output directory {output_dir:?}: {e}");
		});
	}

	{
		let output_repr_path = output_dir.join(OUTPUT_SOLUTION_REPR_FILE);
		std::fs::write(
			&output_repr_path,
			ron::to_string(solution_repr)
				.map_err(|e| format!("failed to serialize solution representation to RON: {e}"))?,
		)
		.map_err(|e| {
			format!("failed to write solution representation to file {output_repr_path:?}: {e}")
		})?;
	}

	{
		let solution_output = solution_into_real_output(solution_repr);
		let output_solution_path = output_dir.join(OUTPUT_SOLUTION_REAL_FILE);
		std::fs::write(&output_solution_path, solution_output).map_err(|e| {
			format!("failed to write solution to file {output_solution_path:?}: {e}")
		})?;
	}

	Ok(())
}
