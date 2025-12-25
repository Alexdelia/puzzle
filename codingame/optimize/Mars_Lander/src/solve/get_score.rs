use super::Score;
use crate::{
	referee::{
		env::{VALID_X_SPEED_THRESHOLD, VALID_Y_SPEED_THRESHOLD},
		lander::Lander,
	},
	segment::Segment,
};

pub fn get_score(landing_segment: &Segment, lander: &Lander, is_valid_landing: bool) -> Score {
	if is_valid_landing {
		return -(lander.fuel as Score);
	}

	let min_a_x = landing_segment.a.x.min(landing_segment.b.x);
	let max_b_x = landing_segment.a.x.max(landing_segment.b.x);

	if lander.x < min_a_x {
		return (min_a_x - lander.x) as Score * 2 + 1000;
	}
	if lander.x > max_b_x {
		return (lander.x - max_b_x) as Score * 2 + 1000;
	}

	let speed_penalty = (lander.sx.abs() - (VALID_X_SPEED_THRESHOLD - 1.0)).max(0.0)
		+ (lander.sy.abs() - (VALID_Y_SPEED_THRESHOLD - 1.0)).max(0.0);
	let rotate_penalty = (lander.rotate.abs() as Score) * 2;

	speed_penalty as Score + rotate_penalty as Score
}
