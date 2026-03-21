mod landscape;
pub use landscape::landscape;
mod solution;
pub use solution::solution;
mod generation_number;
pub use generation_number::generation_number;

use crate::{
	OUTPUT_DIR,
	referee::env::{Axis, MAX_HEIGHT, MAX_WIDTH},
};

const VISUALIZE_OUTPUT_DIR: &str = "visualize";

const SVG_WIDTH: i32 = 1000;
const SVG_HEIGHT: i32 = (SVG_WIDTH as f64 * MAX_HEIGHT / MAX_WIDTH) as i32;

const CONVERSION_WIDTH: f64 = SVG_WIDTH as f64 / MAX_WIDTH;
const CONVERSION_HEIGHT: f64 = SVG_HEIGHT as f64 / MAX_HEIGHT;

pub fn clear_output(validator_name: &str) {
	let output_dir = std::path::Path::new(OUTPUT_DIR)
		.join(validator_name)
		.join(VISUALIZE_OUTPUT_DIR);
	if output_dir.exists() {
		std::fs::remove_dir_all(&output_dir).unwrap_or_else(|e| {
			panic!("failed to clear output directory {output_dir:?}: {e}");
		});
	}
}

pub fn write_doc(validator_name: &str, doc: &svg::Document, iteration: usize) {
	let output_dir = std::path::Path::new(OUTPUT_DIR)
		.join(validator_name)
		.join(VISUALIZE_OUTPUT_DIR);
	if !output_dir.exists() {
		std::fs::create_dir_all(&output_dir).unwrap_or_else(|e| {
			panic!("failed to create output directory {output_dir:?}: {e}");
		});
	}

	let output_path = output_dir.join(format!("{iteration:04}.svg"));

	svg::save(&output_path, doc).unwrap_or_else(|e| {
		panic!("failed to save SVG to {output_path:?}: {e}");
	});
}
