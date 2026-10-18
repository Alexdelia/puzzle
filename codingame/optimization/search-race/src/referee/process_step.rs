use std::f64::consts::TAU;

use crate::{
	output_repr::Step,
	referee::{
		car::Car,
		env::{FRICTION, truncate},
	},
};

/// https://github.com/Illedan/CGSearchRace/blob/55bca1e6893d4e69bb74224eea22dac1ba013e8c/SearchRace/config/statement_en.html#L70-L74
/// Car.handleExpertInput -> update angle, add thrust to velocity
/// Game.checkCollisions -> move by velocity (caller handles intersection)
/// Car.adjust / Unit.adjust -> truncate pos, friction+truncate vel, round angle
use crate::referee::env::Coord;

pub fn process_step(car: &mut Car, step: &Step) -> Coord {
	let new_angle_deg = car.angle.to_degrees() + step.tilt as f64;
	car.angle = new_angle_deg.to_radians();

	let (sin, cos) = car.angle.sin_cos();
	car.sx += cos * step.thrust as f64;
	car.sy += sin * step.thrust as f64;

	car.x += car.sx;
	car.y += car.sy;

	let moved_to = Coord { x: car.x, y: car.y };

	car.x = truncate(car.x);
	car.y = truncate(car.y);
	car.sx = truncate(car.sx * FRICTION);
	car.sy = truncate(car.sy * FRICTION);

	let degrees = car.angle.to_degrees().round();
	car.angle = degrees.to_radians();
	while car.angle > TAU {
		car.angle -= TAU;
	}
	while car.angle < 0.0 {
		car.angle += TAU;
	}

	moved_to
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn hl_step9_intersection() {
		use crate::dist::dist;
		use crate::referee::env::CHECKPOINT_RADIUS;
		use crate::referee::intersect;

		let mut car = Car {
			x: 1000.0,
			y: 4500.0,
			sx: 0.0,
			sy: 0.0,
			angle: (338.0_f64).to_radians(),
		};

		let steps: &[(i8, u8)] = &[
			(4, 181),
			(15, 190),
			(-6, 52),
			(16, 168),
			(13, 200),
			(0, 132),
			(-18, 128),
			(-17, 168),
			(-6, 114),
			(18, 105),
			(16, 177),
		];

		let checkpoint = Coord {
			x: 5500.0,
			y: 3905.0,
		};

		for (i, &(tilt, thrust)) in steps.iter().enumerate() {
			let from = Coord { x: car.x, y: car.y };
			let step = Step { tilt, thrust };
			let moved_to = process_step(&mut car, &step);
			let d = dist(car.x, car.y, checkpoint.x, checkpoint.y);
			let hit = intersect(checkpoint, from, moved_to);
			println!(
				"step {i}: ({}, {}) d={d:.2} hit={hit}",
				car.x as i64, car.y as i64
			);
		}

		let d_at_10 = dist(5536.0, 4504.0, 5500.0, 3905.0);
		println!("referee reports d=600.08 at step 10, we get d={d_at_10:.4}");
		assert!(
			d_at_10 > CHECKPOINT_RADIUS,
			"step 10 should NOT cross checkpoint (d={d_at_10} > {CHECKPOINT_RADIUS})"
		);
	}
}
