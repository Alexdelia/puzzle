use std::sync::mpsc;

use crate::{
	output_repr::Solution,
	referee::{
		env::{Coord, MAX_HEIGHT, MAX_WIDTH},
		intersect,
		lander::Lander,
		process_step::process_step,
	},
	segment::Segment,
};

use super::{ProcessOutput, VALID_LANDING_INDEX};

pub fn simulate_generation(
	pool: &rayon::ThreadPool,
	tx: mpsc::Sender<ProcessOutput>,
	landscape: &[Segment],
	lander_init_state: &Lander,
	solution_list: &[Solution],
) {
	pool.scope(|s| {
		for (i, solution) in solution_list.iter().enumerate() {
			let tx = tx.clone();
			let mut lander = *lander_init_state;

			s.spawn(move |_| {
				#[cfg(feature = "visualize")]
				let mut path = Vec::with_capacity(solution.len() + 1);
				#[cfg(feature = "visualize")]
				path.push(Coord {
					x: lander.x,
					y: lander.y,
				});

				for i in 0..solution.len() {
					let step = &solution[i];

					let from = Coord {
						x: lander.x,
						y: lander.y,
					};

					process_step(&mut lander, step);

					if lander.x < 0.0
						|| lander.x > MAX_WIDTH
						|| lander.y < 0.0 || lander.y > MAX_HEIGHT
					{
						tx.send(ProcessOutput {
							index: i,
							lander,
							is_valid_landing: false,
							#[cfg(feature = "visualize")]
							path,
						})
						.expect("failed to send result");
						return;
					}

					let traveled = Segment {
						a: from,
						b: Coord {
							x: lander.x,
							y: lander.y,
						},
					};
					#[cfg(feature = "visualize")]
					path.push(traveled.b);

					for (segment_index, segment) in landscape.iter().enumerate() {
						if intersect(&traveled, segment) {
							tx.send(ProcessOutput {
								index: i,
								lander,
								is_valid_landing: segment_index == VALID_LANDING_INDEX
									&& lander.valid_landing_condition(),
								#[cfg(feature = "visualize")]
								path,
							})
							.expect("failed to send result");
							return;
						}
					}
				}

				tx.send(ProcessOutput {
					index: i,
					lander,
					is_valid_landing: false,
					#[cfg(feature = "visualize")]
					path,
				})
				.expect("failed to send result");
			});
		}
	});
}
