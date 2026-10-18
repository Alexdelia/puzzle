use crate::{
	output_repr::Solution,
	referee::{car::Car, env::Coord, intersect, process_step::process_step},
	segment::Segment,
};

pub fn compute_max_step_gap(
	checkpoint_list: &[Coord],
	car_init_state: &Car,
	solution: &Solution,
) -> Option<usize> {
	let mut car = *car_init_state;
	let mut checkpoint_index = 0;
	let mut prev_crossing_step: Option<usize> = None;
	let mut max_gap: Option<usize> = None;

	for (step_index, step) in solution.iter().enumerate() {
		if checkpoint_index >= checkpoint_list.len() {
			break;
		}

		let from = Coord { x: car.x, y: car.y };
		let moved_to = process_step(&mut car, step);
		let traveled = Segment {
			a: from,
			b: moved_to,
		};

		let current_checkpoint = checkpoint_list[checkpoint_index];
		if intersect(current_checkpoint, traveled.a, traveled.b) {
			let gap = match prev_crossing_step {
				Some(p) => step_index - p,
				None => step_index + 1,
			};
			max_gap = Some(max_gap.map_or(gap, |m| m.max(gap)));
			prev_crossing_step = Some(step_index);
			checkpoint_index += 1;
		}
	}

	max_gap
}
