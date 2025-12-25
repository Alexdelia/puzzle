use crate::{
	output_repr::{Step, ThrustChange},
	referee::{
		env::GRAVITY,
		lander::{Fuel, Lander},
	},
};

pub fn process_step(lander: &mut Lander, step: &Step) {
	lander.rotate = (lander.rotate + step.tilt).clamp(-90, 90);

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

	let alpha = (lander.rotate as f64).to_radians();
	let pf = lander.power as f64;
	let ax = -pf * alpha.sin();
	let ay = pf * alpha.cos() - GRAVITY;

	lander.x += lander.sx + (ax * 0.5);
	lander.y += lander.sy + (ay * 0.5);

	lander.sx += ax;
	lander.sy += ay;
}
