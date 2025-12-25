mod output_repr;
mod parse;
mod referee;
mod segment;
mod solve;
#[cfg(feature = "visualize")]
mod visualize;

fn main() -> Result<(), String> {
	let path = parse::get_path()?;
	#[cfg(feature = "visualize")]
	let validator_name = visualize::get_validator_name(&path);

	let (lander_init_state, landscape) = parse::parse(&path)?;

	#[cfg(feature = "visualize")]
	let base_doc = visualize::landscape(&landscape);

	#[cfg(feature = "visualize")]
	{
		visualize::write_doc(&validator_name, &base_doc, 0);

		let mut doc = base_doc;

		use rand::Rng;
		let mut rng = rand::rng();

		// generate random solutions for visualization
		for i in 1..=10 {
			let mut solution = Vec::with_capacity(50);
			for _ in 0..50 {
				solution.push(crate::output_repr::Step {
					tilt: rng.random_range(-15..=15),
					thrust: match rng.random_range(0..3) {
						0 => output_repr::ThrustChange::Decrease,
						1 => output_repr::ThrustChange::Keep,
						_ => output_repr::ThrustChange::Increase,
					},
				});
			}

			let mut lander = referee::lander::Lander {
				x: 2500.0,
				y: 2500.0,
				sx: 0.0,
				sy: 0.0,
				fuel: 500,
				rotate: 0.0,
				power: 0,
			};
			let mut lander_path = Vec::new();
			lander_path.push(referee::env::Coord {
				x: lander.x,
				y: lander.y,
			});

			for step in &solution {
				referee::process_step::process_step(&mut lander, step);
				lander_path.push(referee::env::Coord {
					x: lander.x,
					y: lander.y,
				});
			}

			doc = doc.add(visualize::solution(
				&lander_path,
				rng.random_bool(1.0 / 2.0),
			));
			visualize::write_doc(&validator_name, &doc, i);
		}
	}

	solve::solve(
		&landscape,
		lander_init_state,
		#[cfg(feature = "visualize")]
		base_doc,
	)
}
