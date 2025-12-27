mod first_generation;
mod simulate_generation;
use simulate_generation::simulate_generation;
mod breed_generation;
mod get_score;
use breed_generation::breed_generation;

use std::sync::mpsc;

use rayon::ThreadPoolBuilder;

#[cfg(feature = "visualize")]
use crate::referee::env::Coord;
#[cfg(feature = "visualize")]
use crate::visualize;
use crate::{
	output_repr::Solution, parse::get_iteration, referee::lander::Lander, segment::Segment,
};

pub const SOLUTION_PER_GENERATION: usize = 512;

pub const VALID_LANDING_INDEX: usize = 0;

pub type Score = i16;

struct ProcessOutput {
	index: usize,
	is_valid_landing: bool,
	score: Score,
	step_count: usize,
	#[cfg(feature = "visualize")]
	path: Vec<Coord>,
}

#[derive(Default)]
struct BestSolution {
	is_valid_landing: bool,
	solution: Solution,
	step_count: usize,
	#[cfg(feature = "visualize")]
	path: Vec<Coord>,
}

pub fn solve(
	validator_name: &str,
	landscape: &[Segment],
	lander_init_state: &Lander,
	#[cfg(feature = "visualize")] base_doc: svg::Document,
) -> Result<Solution, String> {
	let pool = ThreadPoolBuilder::new()
		.build()
		.map_err(|e| format!("failed to build thread pool: {e}"))?;

	let mut solution_list = first_generation::init_first_generation(validator_name)?;
	let mut score_list = [Score::default(); SOLUTION_PER_GENERATION];
	#[cfg(feature = "visualize")]
	let mut path_list: [Vec<Coord>; SOLUTION_PER_GENERATION] = (0..SOLUTION_PER_GENERATION)
		.map(|_| Vec::new())
		.collect::<Vec<_>>()
		.try_into()
		.expect("path list size mismatch");

	let mut best = (Score::MAX, BestSolution::default());

	let mut generation: usize = 0;
	let max_iteration = get_iteration()?;
	while !best.1.is_valid_landing || generation < max_iteration {
		let (tx, rx) = mpsc::channel::<ProcessOutput>();

		simulate_generation(&pool, tx, landscape, lander_init_state, &solution_list);

		#[cfg(feature = "visualize")]
		let mut doc = base_doc.clone();
		#[cfg(feature = "visualize")]
		{
			doc = doc.add(visualize::generation_number(generation));
		}

		for r in rx.iter().take(SOLUTION_PER_GENERATION) {
			score_list[r.index] = r.score;

			if r.score < best.0 {
				best = (
					r.score,
					BestSolution {
						is_valid_landing: r.is_valid_landing,
						solution: solution_list[r.index].clone(),
						step_count: r.step_count,
						#[cfg(feature = "visualize")]
						path: r.path.clone(),
					},
				);
			}

			#[cfg(feature = "visualize")]
			{
				doc = doc.add(visualize::solution(&r.path, r.is_valid_landing, false));
				path_list[r.index] = r.path;
			}
		}

		#[cfg(feature = "visualize")]
		{
			doc = doc.add(visualize::solution(
				&best.1.path,
				best.1.is_valid_landing,
				true,
			));
			visualize::write_doc(validator_name, &doc, generation);
		}

		solution_list = breed_generation(solution_list, score_list);

		if generation % 128 == 0 {
			eprint!("\r{generation} {best_score}", best_score = best.0);
		}

		generation += 1;
	}

	eprintln!("\r{generation} {best_score}", best_score = best.0);

	let mut solution = best.1.solution;
	solution.truncate(best.1.step_count);
	Ok(solution)
}
