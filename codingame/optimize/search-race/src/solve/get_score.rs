use super::Score;
use crate::referee::env::{Axis, Coord, MAX_STEP};

/// (w^2 + h^2)^0.5
const MAX_DISTANCE: Axis = 18357.6;

const DIST_DIV_FACTOR: Score = 1.0;
const CHECKPOINT_FACTOR: Score = (MAX_DISTANCE as Score) / DIST_DIV_FACTOR;

pub fn get_score(
	checkpoint_list: &[Coord],
	current_checkpoint_index: usize,
	closest_to_checkpoint: f64,
	step_count: usize,
	turn_to_finish: Option<f64>,
) -> Score {
	if current_checkpoint_index == checkpoint_list.len() {
		let ttf = turn_to_finish.unwrap_or(step_count as f64);
		return (ttf as Score) - (MAX_STEP as Score);
	}

	let remaining_checkpoint_count = checkpoint_list.len() - current_checkpoint_index - 1;

	((remaining_checkpoint_count as Score) * CHECKPOINT_FACTOR)
		+ ((closest_to_checkpoint as Score) / DIST_DIV_FACTOR)
}
