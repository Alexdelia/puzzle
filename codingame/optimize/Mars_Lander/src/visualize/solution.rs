use svg::node::element::{Path, path::Data};

use super::*;
use crate::referee::env::Coord;

const GOOD_LANDER_PATH_COLOR: &str = "green";
const DEAD_LANDER_PATH_COLOR: &str = "red";

pub fn solution(lander_path: &[Coord], dead: bool) -> Path {
	let mut d = Data::new().move_to((
		lander_path[0].x * CONVERSION_WIDTH,
		SVG_HEIGHT as Axis - lander_path[0].y * CONVERSION_HEIGHT,
	));

	for coord in &lander_path[1..] {
		d = d.line_to((
			coord.x * CONVERSION_WIDTH,
			SVG_HEIGHT as Axis - coord.y * CONVERSION_HEIGHT,
		));
	}

	Path::new()
		.set("d", d)
		.set("fill", "none")
		.set(
			"stroke",
			if dead {
				DEAD_LANDER_PATH_COLOR
			} else {
				GOOD_LANDER_PATH_COLOR
			},
		)
		.set("stroke-opacity", 1.0 / 3.0)
		.set("stroke-width", 2)
}
