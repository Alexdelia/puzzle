use crate::{
	output_repr::{Solution, ThrustChange},
	referee::lander::Lander,
};

pub fn solution_into_real_output(solution: &Solution, lander_init_state: &Lander) -> String {
	let mut output = String::new();

	let mut rotate = lander_init_state.rotate;
	let mut power = lander_init_state.power;
	for step in solution.iter() {
		rotate = (rotate + step.tilt).clamp(-90, 90);
		match step.thrust {
			ThrustChange::Increase => {
				if power < 4 {
					power += 1;
				}
			}
			ThrustChange::Keep => {}
			ThrustChange::Decrease => {
				if power > 0 {
					power -= 1;
				}
			}
		}

		output.push_str(&format!("{rotate} {power}\n"));
	}

	output
}
