use crate::{
	output_repr::{Step, ThrustChange},
	referee::{
		env::{Angle, GRAVITY},
		lander::{Fuel, Lander},
	},
};

pub fn process_step(lander: &mut Lander, step: &Step) {
	lander.rotate += step.tilt as Angle;

	match step.thrust {
		ThrustChange::Decrease => {
			if lander.power > 0 {
				lander.power -= 1;
			}
		}
		ThrustChange::Keep => {}
		ThrustChange::Increase => {
			if lander.power < 4 {
				lander.power += 1;
			}
		}
	};

	lander.fuel = lander.fuel.saturating_sub(lander.power as Fuel);

	let alpha = lander.rotate.to_radians();
	let pf = lander.power as f64;
	let ax = -pf * alpha.sin();
	let ay = pf * alpha.cos() - GRAVITY;

	lander.x += lander.sx + (ax * 0.5);
	lander.y += lander.sy + (ay * 0.5);

	lander.sx += ax;
	lander.sy += ay;
}
