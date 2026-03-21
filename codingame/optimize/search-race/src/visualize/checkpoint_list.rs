use svg::Document;

use crate::{
	referee::env::{CHECKPOINT_RADIUS, Coord},
	visualize::scale::scale_x,
};

use super::*;

const BACKGROUND_COLOR: &str = "black";

const CHECKPOINT_COLOR: &str = "green";
const CHECKPOINT_STROKE_WIDTH: usize = 2;

pub fn checkpoint_list(checkpoint_list: &[Coord]) -> Document {
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

	for c in checkpoint_list {
		document = document.add(
			svg::node::element::Circle::new()
				.set("cx", scale_x(c.x))
				.set("cy", scale_x(c.y))
				.set("r", scale_x(CHECKPOINT_RADIUS))
				.set("fill", "none")
				.set("stroke", CHECKPOINT_COLOR)
				.set("stroke-width", CHECKPOINT_STROKE_WIDTH),
		);
	}

	document
}
