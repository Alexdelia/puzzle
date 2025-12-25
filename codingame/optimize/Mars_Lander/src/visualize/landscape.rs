use svg::Document;

use super::*;
use crate::segment::Segment;

const BACKGROUND_COLOR: &str = "black";

const LANDSCAPE_BORDER_COLOR: &str = "lightgray";
const LANDSCAPE_FILL_COLOR: &str = "gray";
const LANDING_SEGMENT_COLOR: &str = "green";

pub fn landscape(segment_list: &[Segment]) -> Document {
	let mut document = Document::new()
		.set("viewBox", (0, 0, SVG_WIDTH, SVG_HEIGHT))
		.set("width", SVG_WIDTH)
		.set("height", SVG_HEIGHT);

	document = document.add(
		svg::node::element::Rectangle::new()
			.set("x", 0)
			.set("y", 0)
			.set("width", SVG_WIDTH)
			.set("height", SVG_HEIGHT)
			.set("fill", BACKGROUND_COLOR),
	);

	let mut point_list = Vec::with_capacity(segment_list.len() + 2);
	point_list.push((0.0, MAX_HEIGHT * CONVERSION_HEIGHT));
	for segment in segment_list {
		point_list.push((
			segment.a.x * CONVERSION_WIDTH,
			SVG_HEIGHT as Axis - segment.a.y * CONVERSION_HEIGHT,
		));
	}
	point_list.push((
		segment_list.last().unwrap().b.x * CONVERSION_WIDTH,
		SVG_HEIGHT as Axis - segment_list.last().unwrap().b.y * CONVERSION_HEIGHT,
	));
	point_list.push((MAX_WIDTH * CONVERSION_WIDTH, MAX_HEIGHT * CONVERSION_HEIGHT));

	document = document.add(
		svg::node::element::Polygon::new()
			.set(
				"points",
				point_list
					.iter()
					.map(|(x, y)| format!("{x},{y}"))
					.collect::<Vec<String>>()
					.join(" "),
			)
			.set("fill", LANDSCAPE_FILL_COLOR),
	);

	for segment in segment_list {
		let is_landing = segment.a.y == segment.b.y;

		let line = svg::node::element::Line::new()
			.set("x1", segment.a.x * CONVERSION_WIDTH)
			.set("y1", SVG_HEIGHT as Axis - segment.a.y * CONVERSION_HEIGHT)
			.set("x2", segment.b.x * CONVERSION_WIDTH)
			.set("y2", SVG_HEIGHT as Axis - segment.b.y * CONVERSION_HEIGHT)
			.set(
				"stroke",
				if is_landing {
					LANDING_SEGMENT_COLOR
				} else {
					LANDSCAPE_BORDER_COLOR
				},
			)
			.set("stroke-width", 3);
		document = document.add(line);
	}

	document
}
