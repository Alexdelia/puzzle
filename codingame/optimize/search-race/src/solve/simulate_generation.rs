use std::sync::mpsc;

use crate::{
	dist::dist,
	output_repr::Solution,
	referee::{car::Car, env::Coord, intersect, process_step::process_step},
	segment::Segment,
	solve::get_score::get_score,
};

use super::{FrozenPrefix, ProcessOutput};

pub fn simulate_generation(
	pool: &rayon::ThreadPool,
	tx: mpsc::Sender<ProcessOutput>,
	checkpoint_list: &[Coord],
	car_init_state: &Car,
	solution_list: &[Solution],
	frozen_list: &[FrozenPrefix],
	step_to_new_checkpoint_limit: usize,
) {
	pool.scope(|s| {
		for (i, (solution, frozen)) in solution_list.iter().zip(frozen_list.iter()).enumerate() {
			let tx = tx.clone();
			let frozen = *frozen;

			s.spawn(move |_| {
				tx.send(simulate_solution(
					i,
					checkpoint_list,
					car_init_state,
					solution,
					&frozen,
					step_to_new_checkpoint_limit,
				))
				.expect("failed to send result");
			});
		}
	});
}

pub fn simulate_solution(
	index: usize,
	checkpoint_list: &[Coord],
	car_init_state: &Car,
	solution: &Solution,
	frozen: &FrozenPrefix,
	step_to_new_checkpoint_limit: usize,
) -> ProcessOutput {
	let resuming = frozen.resume_from_step > 0;
	let (mut car, mut checkpoint_index) = if resuming {
		(frozen.car, frozen.checkpoint_index)
	} else {
		(*car_init_state, 0)
	};

	let new_checkpoint = if resuming {
		frozen.checkpoint_index + 1
	} else {
		0
	};

	#[cfg(feature = "visualize")]
	let mut path = Vec::with_capacity(solution.len() + 1);
	#[cfg(feature = "visualize")]
	path.push(Coord { x: car.x, y: car.y });

	let mut reached_at_step = frozen.resume_from_step.saturating_sub(1);
	let mut window_start = reached_at_step;
	let mut window_len = frozen.reentry_step_count + step_to_new_checkpoint_limit;

	let mut closest_to_checkpoint = f64::INFINITY;

	let mut last_crossing: Option<FrozenPrefix> = None;
	let mut pre_last_crossing: Option<FrozenPrefix> = None;

	for (step_index, step) in solution.iter().enumerate().skip(frozen.resume_from_step) {
		let from = Coord { x: car.x, y: car.y };

		process_step(&mut car, step);

		let traveled = Segment {
			a: from,
			b: Coord { x: car.x, y: car.y },
		};
		#[cfg(feature = "visualize")]
		path.push(traveled.b);

		if window_start + window_len < step_index {
			break;
		}

		let current_checkpoint = checkpoint_list[checkpoint_index];

		let d = dist(car.x, car.y, current_checkpoint.x, current_checkpoint.y);
		if d < closest_to_checkpoint {
			closest_to_checkpoint = d;
		}

		if intersect(current_checkpoint, traveled.a, traveled.b) {
			let crossed_segment_step_count = step_index - reached_at_step;
			pre_last_crossing = last_crossing.map(|crossing| FrozenPrefix {
				reentry_step_count: crossed_segment_step_count,
				..crossing
			});
			last_crossing = Some(FrozenPrefix {
				resume_from_step: step_index + 1,
				car,
				checkpoint_index: checkpoint_index + 1,
				reentry_step_count: 0,
			});
			reached_at_step = step_index;

			let crossed_checkpoint = checkpoint_index;
			checkpoint_index += 1;
			closest_to_checkpoint = f64::INFINITY;

			if crossed_checkpoint >= new_checkpoint {
				window_start = step_index;
				window_len = step_to_new_checkpoint_limit;
			}

			if checkpoint_index == checkpoint_list.len() {
				let step_count = step_index + 1;
				return ProcessOutput {
					index,
					finished: true,
					score: get_score(
						checkpoint_list,
						checkpoint_index,
						closest_to_checkpoint,
						step_count,
					),
					step_count,
					reached_checkpoint_count: checkpoint_index,
					frozen: pre_last_crossing.unwrap_or(*frozen),
					#[cfg(feature = "visualize")]
					path,
				};
			}
		}
	}

	let step_count = solution.len();

	ProcessOutput {
		index,
		finished: false,
		step_count,
		reached_checkpoint_count: checkpoint_index,
		frozen: pre_last_crossing.unwrap_or(*frozen),
		score: get_score(
			checkpoint_list,
			checkpoint_index,
			closest_to_checkpoint,
			step_count,
		),
		#[cfg(feature = "visualize")]
		path,
	}
}
