use svg::Document;

use crate::{referee::env::Axis, segment::Segment};

const MAX_WIDTH: Axis = 7000.0;
const MAX_HEIGHT: Axis = 3000.0;

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

pub fn landscape(segment_list: &[Segment]) -> Document {
	let mut document = Document::new()
		.set("viewBox", (0, 0, SVG_WIDTH, SVG_HEIGHT))
		.set("width", SVG_WIDTH)
		.set("height", SVG_HEIGHT);

	for segment in segment_list {
		let line = svg::node::element::Line::new()
			.set("x1", segment.a.x * CONVERSION_WIDTH)
			.set("y1", SVG_HEIGHT as f64 - segment.a.y * CONVERSION_HEIGHT)
			.set("x2", segment.b.x * CONVERSION_WIDTH)
			.set("y2", SVG_HEIGHT as f64 - segment.b.y * CONVERSION_HEIGHT)
			.set("stroke", "gray")
			.set("stroke-width", 2);
		document = document.add(line);
	}

	document
}
