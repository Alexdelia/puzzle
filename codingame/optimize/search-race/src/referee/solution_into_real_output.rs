use crate::output_repr::{Solution, Step};

pub fn solution_into_real_output(solution: &Solution) -> String {
	solution
		.iter()
		.map(|Step { tilt, thrust }| format!("EXPERT {tilt} {thrust}"))
		.collect::<Vec<_>>()
		.join("\n")
}
