mod output_repr;
mod parse;
mod referee;
mod segment;
#[cfg(feature = "visualize")]
mod visualize;

fn main() -> Result<(), String> {
	let path = parse::get_path()?;
	#[cfg(feature = "visualize")]
	let validator_name = visualize::get_validator_name(&path);

	let landscape = parse::parse(&path)?;

	#[cfg(feature = "visualize")]
	let base_doc = visualize::landscape(&landscape);

	#[cfg(feature = "visualize")]
	{
		let filename = format!("{}_landscape.svg", validator_name);
		svg::save(&filename, &base_doc).map_err(|e| e.to_string())?;
		println!("Landscape SVG saved to {}", filename);
	}

	Ok(())
}
