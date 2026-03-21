use svg::node::element::{Path, path::Data};

use super::*;
use crate::referee::env::Coord;

const GOOD_LANDER_PATH_COLOR: &str = "green";
const DEAD_LANDER_PATH_COLOR: &str = "red";
const BEST_LANDER_PATH_COLOR: &str = "gold";

pub fn solution(car_path: &[Coord], finished: bool, best: bool) -> Path {
	let mut d = Data::new().move_to((
		car_path[0].x * CONVERSION_WIDTH,
		SVG_HEIGHT as Axis - car_path[0].y * CONVERSION_HEIGHT,
	));

	for coord in &car_path[1..] {
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
			if best {
				BEST_LANDER_PATH_COLOR
			} else if finished {
				GOOD_LANDER_PATH_COLOR
			} else {
				DEAD_LANDER_PATH_COLOR
			},
		)
		.set(
			"stroke-opacity",
			if best {
				0.9
			} else if finished {
				0.3
			} else {
				0.05
			},
		)
		.set("stroke-width", 2)
}
