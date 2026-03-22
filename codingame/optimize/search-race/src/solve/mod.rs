mod first_generation;
mod simulate_generation;
use simulate_generation::simulate_generation;
mod breed_generation;
mod get_score;
use breed_generation::breed_generation;

use std::sync::mpsc;

use rayon::ThreadPoolBuilder;

#[cfg(feature = "visualize")]
use crate::visualize;

use crate::{
	output_repr::Solution,
	output_solution::output_solution,
	parse::get_iteration,
	referee::{car::Car, env::Coord},
};

pub const SOLUTION_PER_GENERATION: usize = 512;

pub type Score = i16;

struct ProcessOutput {
	index: usize,
	finished: bool,
	score: Score,
	step_count: usize,
	#[cfg(feature = "visualize")]
	path: Vec<Coord>,
}

#[derive(Default)]
struct BestSolution {
	finished: bool,
	solution: Solution,
	step_count: usize,
	#[cfg(feature = "visualize")]
	path: Vec<Coord>,
}

pub fn solve(
	validator_name: &str,
	checkpoint_list: &[Coord],
	car_init_state: &Car,
	#[cfg(feature = "visualize")] base_doc: svg::Document,
) -> Result<Solution, String> {
	let pool = ThreadPoolBuilder::new()
		.build()
		.map_err(|e| format!("failed to build thread pool: {e}"))?;

	let mut solution_list = first_generation::init_first_generation(validator_name)?;
	let mut score_list = [Score::default(); SOLUTION_PER_GENERATION];
	let mut step_count_list = [usize::default(); SOLUTION_PER_GENERATION];
	#[cfg(feature = "visualize")]
	let mut path_list: [Vec<Coord>; SOLUTION_PER_GENERATION] = (0..SOLUTION_PER_GENERATION)
		.map(|_| Vec::new())
		.collect::<Vec<_>>()
		.try_into()
		.expect("path list size mismatch");

	let mut best = (Score::MAX, BestSolution::default());

	let mut generation: usize = 0;
	let max_iteration = get_iteration()?;
	while !best.1.finished || generation < max_iteration {
		let (tx, rx) = mpsc::channel::<ProcessOutput>();

		simulate_generation(&pool, tx, checkpoint_list, car_init_state, &solution_list);

		#[cfg(feature = "visualize")]
		let mut doc = base_doc.clone();
		#[cfg(feature = "visualize")]
		{
			doc = doc.add(visualize::generation_number(generation));
		}

		for r in rx.iter().take(SOLUTION_PER_GENERATION) {
			score_list[r.index] = r.score;
			step_count_list[r.index] = r.step_count;

			if r.score < best.0 {
				best = (
					r.score,
					BestSolution {
						finished: r.finished,
						solution: solution_list[r.index].clone(),
						step_count: r.step_count,
						#[cfg(feature = "visualize")]
						path: r.path.clone(),
					},
				);

				output_solution(&best.1.solution, validator_name)?;
			}

			#[cfg(feature = "visualize")]
			{
				doc = doc.add(visualize::solution(&r.path, r.finished, false));
				path_list[r.index] = r.path;
			}
		}

		#[cfg(feature = "visualize")]
		{
			doc = doc.add(visualize::solution(&best.1.path, best.1.finished, true));
			visualize::write_doc(validator_name, &doc, generation);
		}

		solution_list = breed_generation(solution_list, score_list, step_count_list);

		if generation.is_multiple_of(128) {
			eprint!(
				"\r{generation} {best_score} {best_step_count}",
				best_score = best.0,
				best_step_count = best.1.step_count
			);
		}

		generation += 1;
	}

	eprintln!(
		"\r{generation} {best_score} {best_step_count}",
		best_score = best.0,
		best_step_count = best.1.step_count
	);

	let mut solution = best.1.solution;
	solution.truncate(best.1.step_count);
	Ok(solution)
}
