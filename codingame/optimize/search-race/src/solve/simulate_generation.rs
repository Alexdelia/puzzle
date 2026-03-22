use std::sync::mpsc;

use crate::{
	output_repr::Solution,
	referee::{
		car::Car,
		env::{Coord, MAX_HEIGHT, MAX_STEP, MAX_WIDTH},
		intersect,
		process_step::process_step,
	},
	segment::Segment,
	solve::get_score::get_score,
};

use super::ProcessOutput;

pub fn simulate_generation(
	pool: &rayon::ThreadPool,
	tx: mpsc::Sender<ProcessOutput>,
	checkpoint_list: &[Coord],
	car_init_state: &Car,
	solution_list: &[Solution],
) {
	pool.scope(|s| {
		for (i, solution) in solution_list.iter().enumerate() {
			let tx = tx.clone();
			let mut car = *car_init_state;

			s.spawn(move |_| {
				#[cfg(feature = "visualize")]
				let mut path = Vec::with_capacity(solution.len() + 1);
				#[cfg(feature = "visualize")]
				path.push(Coord { x: car.x, y: car.y });

				let mut checkpoint_index = 0;
				let mut reached_at_step = MAX_STEP;

				for (step_index, step) in solution.iter().enumerate() {
					let from = Coord { x: car.x, y: car.y };

					process_step(&mut car, step);

					let traveled = Segment {
						a: from,
						b: Coord { x: car.x, y: car.y },
					};
					#[cfg(feature = "visualize")]
					path.push(traveled.b);

					if reached_at_step + 32 < step_index {
						break;
					}
					if traveled.b.x < 0.0
						|| traveled.b.x > MAX_WIDTH
						|| traveled.b.y < 0.0
						|| traveled.b.y > MAX_HEIGHT
					{
						break;
					}

					let current_checkpoint = checkpoint_list[checkpoint_index];

					if intersect(current_checkpoint, traveled.a, traveled.b) {
						checkpoint_index += 1;
						reached_at_step = step_index;

						let step_count = step_index + 1;

						if checkpoint_index == checkpoint_list.len() {
							tx.send(ProcessOutput {
								index: i,
								finished: true,
								score: get_score(
									checkpoint_list,
									checkpoint_index,
									&car,
									step_count,
									reached_at_step,
								),
								step_count,
								#[cfg(feature = "visualize")]
								path,
							})
							.expect("failed to send result");
							return;
						}
					}
				}

				let step_count = solution.len();

				tx.send(ProcessOutput {
					index: i,
					finished: false,
					step_count,
					score: get_score(
						checkpoint_list,
						checkpoint_index,
						&car,
						step_count,
						reached_at_step,
					),
					#[cfg(feature = "visualize")]
					path,
				})
				.expect("failed to send result");
			});
		}
	});
}
