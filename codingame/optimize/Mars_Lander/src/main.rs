mod output_repr;
mod parse;
mod referee;
mod segment;
mod solve;
#[cfg(feature = "visualize")]
mod visualize;

use crate::solve::VALID_LANDING_INDEX;

fn main() -> Result<(), String> {
	let path = parse::get_path()?;
	#[cfg(feature = "visualize")]
	let validator_name = visualize::get_validator_name(&path);

	let (lander_init_state, mut landscape) = parse::parse(&path)?;

	#[cfg(feature = "visualize")]
	let base_doc = visualize::landscape(&landscape);

	let flat_segment_index = landscape
		.iter()
		.position(|s| s.a.y == s.b.y)
		.ok_or("no flat segment found in landscape")?;
	if flat_segment_index != VALID_LANDING_INDEX {
		landscape.swap(VALID_LANDING_INDEX, flat_segment_index);
	}

	let solution = solve::solve(
		&landscape,
		&lander_init_state,
		#[cfg(feature = "visualize")]
		base_doc,
		#[cfg(feature = "visualize")]
		&validator_name,
	)?;

	dbg!(solution);

	Ok(())
}
