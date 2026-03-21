use crate::{
	output_repr::Step,
	referee::{car::Car, env::FRICTION},
};

/// https://github.com/Illedan/CGSearchRace/blob/55bca1e6893d4e69bb74224eea22dac1ba013e8c/SearchRace/config/statement_en.html#L70-L74
/// The car rotates to face the target point, with a maximum of 18 degrees.
/// The car's facing vector is multiplied by the given thrust value. The result is added to the current speed vector.
/// The speed vector is added to the position of the car.
/// The current speed vector is multiplied by 0.85
/// The speed's values are truncated, angles converted to degrees and rounded and the position's values are truncated.
pub fn process_step(car: &mut Car, step: &Step) {
	car.angle = (car.angle + (step.tilt as i16)) % 360;

	let angle_radians = (car.angle as f64).to_radians();
	let thrust = step.thrust as f64;

	car.sx += thrust * angle_radians.cos();
	car.sy += thrust * angle_radians.sin();

	car.x += car.sx;
	car.y += car.sy;

	car.sx *= FRICTION;
	car.sy *= FRICTION;

	car.x = car.x.trunc();
	car.y = car.y.trunc();
	car.sx = car.sx.trunc();
	car.sy = car.sy.trunc();
}
