mod dist;
mod output_repr;
mod output_solution;
mod parse;
mod referee;
mod segment;
mod solve;
#[cfg(feature = "visualize")]
mod visualize;

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

	output_solution::output_solution(&solution, &validator_name)
}

fn get_validator_name(path: &str) -> String {
	std::path::Path::new(path)
		.file_stem()
		.expect("invalid path")
		.to_string_lossy()
		.to_string()
}
