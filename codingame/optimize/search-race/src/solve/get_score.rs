use super::Score;
use crate::{
	dist::dist,
	referee::{
		car::Car,
		env::{Axis, Coord, MAX_STEP},
	},
};

/// (w^2 + h^2)^0.5
const MAX_DISTANCE: Axis = 18357.6;

const DIST_DIV_FACTOR: Score = 128;
const CHECKPOINT_FACTOR: Score = (MAX_DISTANCE as Score) / DIST_DIV_FACTOR;

pub fn get_score(
	checkpoint_list: &[Coord],
	current_checkpoint_index: usize,
	car: &Car,
	step_count: usize,
	last_checkpoint_reached_at_step: usize,
) -> Score {
	if current_checkpoint_index == checkpoint_list.len() {
		// TODO: calculate at what % of the turn it was finished
		return (step_count as Score) - (MAX_STEP as Score);
	}

	let remaining_checkpoint_count = checkpoint_list.len() - current_checkpoint_index - 1;

	let current_checkpoint = checkpoint_list[current_checkpoint_index];

	let d = dist(car.x, car.y, current_checkpoint.x, current_checkpoint.y);

	((remaining_checkpoint_count as Score) * CHECKPOINT_FACTOR)
		+ ((d as Score) / DIST_DIV_FACTOR)
		+ (((last_checkpoint_reached_at_step as Score) / (step_count as Score))
			* (remaining_checkpoint_count as Score))
}
