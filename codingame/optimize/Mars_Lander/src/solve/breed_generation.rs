use super::{SOLUTION_PER_GENERATION, Score};
use crate::output_repr::Solution;

const KEEP_COUNT: usize = SOLUTION_PER_GENERATION / 8;

pub fn breed_generation(
	solution_list: &mut [Solution; SOLUTION_PER_GENERATION],
	score_list: &[Score; SOLUTION_PER_GENERATION],
) {
}
