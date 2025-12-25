mod output_repr;
mod parse;
mod referee;
mod segment;
mod solve;
#[cfg(feature = "visualize")]
mod visualize;

fn main() -> Result<(), String> {
	let path = parse::get_path()?;
	#[cfg(feature = "visualize")]
	let validator_name = visualize::get_validator_name(&path);

	let (lander_init_state, landscape) = parse::parse(&path)?;

	#[cfg(feature = "visualize")]
	let base_doc = visualize::landscape(&landscape);

	solve::solve(
		&landscape,
		lander_init_state,
		#[cfg(feature = "visualize")]
		base_doc,
		#[cfg(feature = "visualize")]
		&validator_name,
	)
}
