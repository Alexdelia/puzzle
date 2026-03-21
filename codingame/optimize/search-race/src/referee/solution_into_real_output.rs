use crate::output_repr::{Solution, Step};

// EXPERT output
pub fn solution_into_real_output(solution: &Solution) -> String {
	solution
		.iter()
		.map(|Step { tilt, thrust }| format!("{tilt} {thrust}"))
		.collect::<Vec<_>>()
		.join("\n")
}
