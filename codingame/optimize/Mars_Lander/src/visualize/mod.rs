mod landscape;
pub use landscape::landscape;
mod solution;
pub use solution::solution;

use crate::referee::env::{Axis, MAX_HEIGHT, MAX_WIDTH};

const OUTPUT_DIR: &str = "output";

const SVG_WIDTH: i32 = 1000;
const SVG_HEIGHT: i32 = (SVG_WIDTH as f64 * MAX_HEIGHT / MAX_WIDTH) as i32;

const CONVERSION_WIDTH: f64 = SVG_WIDTH as f64 / MAX_WIDTH;
const CONVERSION_HEIGHT: f64 = SVG_HEIGHT as f64 / MAX_HEIGHT;

pub fn get_validator_name(path: &str) -> String {
	std::path::Path::new(path)
		.file_stem()
		.expect("invalid path")
		.to_string_lossy()
		.to_string()
}

pub fn write_doc(validator_name: &str, doc: &svg::Document, iteration: usize) {
	let output_dir = std::path::Path::new(OUTPUT_DIR).join(validator_name);
	if !output_dir.exists() {
		std::fs::create_dir_all(&output_dir).unwrap_or_else(|e| {
			panic!("failed to create output directory {output_dir:?}: {e}");
		});
	}

	let output_path = output_dir.join(format!("{iteration:04}_landscape.svg"));

	svg::save(&output_path, doc).unwrap_or_else(|e| {
		panic!("failed to save SVG to {output_path:?}: {e}");
	});
}
