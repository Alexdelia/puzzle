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
pub fn process_step(car: &mut Car, step: &Step) {
	let new_angle_deg = car.angle.to_degrees() + step.tilt as f64;
	car.angle = new_angle_deg.to_radians();

	let (sin, cos) = car.angle.sin_cos();
	car.sx += cos * step.thrust as f64;
	car.sy += sin * step.thrust as f64;

	car.x += car.sx;
	car.y += car.sy;

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
}

#[cfg(test)]
mod tests {
	use super::*;

	fn car_state_string(car: &Car) -> String {
		let angle_deg = car.angle.to_degrees().round() as i32;
		format!(
			"{} {} {} {} {}",
			car.x as i64, car.y as i64, car.sx as i64, car.sy as i64, angle_deg
		)
	}

	#[test]
	fn simulate_first_steps() {
		let mut car = Car {
			x: 1000.0,
			y: 1000.0,
			sx: 0.0,
			sy: 0.0,
			angle: (27.0_f64).to_radians(),
		};

		let steps: &[(i8, u8)] = &[
			(-5, 200),
			(-18, 200),
			(0, 200),
			(3, 200),
			(18, 200),
			(-2, 200),
			(-5, 200),
			(-18, 200),
			(9, 200),
			(9, 200),
		];

		println!("turn 0: {}", car_state_string(&car));
		for (i, &(tilt, thrust)) in steps.iter().enumerate() {
			let step = Step { tilt, thrust };
			process_step(&mut car, &step);
			println!("turn {}: {}", i + 1, car_state_string(&car));
		}
	}
}
