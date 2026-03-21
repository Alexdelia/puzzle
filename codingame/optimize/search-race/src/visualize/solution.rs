use svg::node::element::{Path, path::Data};

use crate::{
	referee::env::Coord,
	visualize::scale::{scale_x, scale_y},
};

const GOOD_LANDER_PATH_COLOR: &str = "green";
const DEAD_LANDER_PATH_COLOR: &str = "red";
const BEST_LANDER_PATH_COLOR: &str = "gold";

pub fn solution(car_path: &[Coord], finished: bool, best: bool) -> Path {
	let mut d = Data::new().move_to((scale_x(car_path[0].x), scale_y(car_path[0].y)));

	for coord in &car_path[1..] {
		d = d.line_to((scale_x(coord.x), scale_y(coord.y)));
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
