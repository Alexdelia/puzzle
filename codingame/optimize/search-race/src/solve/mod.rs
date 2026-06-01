mod first_generation;
mod simulate_generation;
use simulate_generation::{simulate_generation, simulate_solution};
mod breed_generation;
pub mod get_score;
use breed_generation::breed_generation;

use std::sync::mpsc;

use rayon::ThreadPoolBuilder;

#[cfg(feature = "visualize")]
use crate::visualize;

use crate::{
	output_repr::Solution,
	output_solution::{output_solution, output_turn_to_finish, read_turn_to_finish},
	parse::get_iteration,
	referee::{
		car::Car,
		env::{Coord, MAX_STEP},
	},
};

pub const SOLUTION_PER_GENERATION: usize = 512;

const INITIAL_STEP_TO_CHECKPOINT_LIMIT: usize = 3;
const MAX_STEP_TO_CHECKPOINT_LIMIT: usize = 64;
const STAGNANT_GENERATIONS_BEFORE_WIDENING: usize = 512;

pub type Score = f32;

#[derive(Clone, Copy, Debug)]
struct FrozenPrefix {
	resume_from_step: usize,
	car: Car,
	checkpoint_index: usize,
	reentry_step_count: usize,
}

impl FrozenPrefix {
	fn from_scratch(car: &Car) -> Self {
		Self {
			resume_from_step: 0,
			car: *car,
			checkpoint_index: 0,
			reentry_step_count: 0,
		}
	}
}

struct ProcessOutput {
	index: usize,
	finished: bool,
	score: Score,
	step_count: usize,
	turn_to_finish: Option<f64>,
	reached_checkpoint_count: usize,
	frozen: FrozenPrefix,
	#[cfg(feature = "visualize")]
	path: Vec<Coord>,
}

#[derive(Default)]
struct BestSolution {
	finished: bool,
	solution: Solution,
	step_count: usize,
	turn_to_finish: Option<f64>,
	reached_checkpoint_count: usize,
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

	let fresh = crate::parse::is_fresh();
	let mut best_disk_ttf = read_turn_to_finish(validator_name);
	let (mut solution_list, loaded) =
		first_generation::init_first_generation(validator_name, fresh)?;
	let mut score_list = [Score::default(); SOLUTION_PER_GENERATION];
	let mut step_count_list = [usize::default(); SOLUTION_PER_GENERATION];
	let mut frozen_list = [FrozenPrefix::from_scratch(car_init_state); SOLUTION_PER_GENERATION];
	#[cfg(feature = "visualize")]
	let mut path_list: [Vec<Coord>; SOLUTION_PER_GENERATION] = (0..SOLUTION_PER_GENERATION)
		.map(|_| Vec::new())
		.collect::<Vec<_>>()
		.try_into()
		.expect("path list size mismatch");

	let mut best = (Score::MAX, BestSolution::default());

	let mut step_to_checkpoint_limit = INITIAL_STEP_TO_CHECKPOINT_LIMIT;
	let mut best_frontier = 0;
	let mut stagnant_generation_count = 0;
	let mut optimize_end = false;
	let mut previous_best_score = Score::MAX;

	if loaded {
		let loaded_run = simulate_solution(
			0,
			checkpoint_list,
			car_init_state,
			&solution_list[0],
			&FrozenPrefix::from_scratch(car_init_state),
			MAX_STEP,
		);
		best_frontier = loaded_run.reached_checkpoint_count;
		frozen_list[0] = loaded_run.frozen;
		if loaded_run.finished {
			optimize_end = true;
		}
	}

	let mut generation: usize = 0;
	let max_iteration = get_iteration()?;
	while !best.1.finished || optimize_end || generation < max_iteration {
		let (tx, rx) = mpsc::channel::<ProcessOutput>();

		simulate_generation(
			&pool,
			tx,
			checkpoint_list,
			car_init_state,
			&solution_list,
			&frozen_list,
			step_to_checkpoint_limit,
		);

		#[cfg(feature = "visualize")]
		let mut doc = base_doc.clone();
		#[cfg(feature = "visualize")]
		{
			doc = doc.add(visualize::generation_number(generation));
		}

		for r in rx.iter().take(SOLUTION_PER_GENERATION) {
			score_list[r.index] = r.score;
			step_count_list[r.index] = r.step_count;
			frozen_list[r.index] = if r.finished && !optimize_end {
				FrozenPrefix::from_scratch(car_init_state)
			} else {
				r.frozen
			};

			if r.score < best.0 {
				best = (
					r.score,
					BestSolution {
						finished: r.finished,
						solution: solution_list[r.index].clone(),
						step_count: r.step_count,
						turn_to_finish: r.turn_to_finish,
						reached_checkpoint_count: r.reached_checkpoint_count,
						#[cfg(feature = "visualize")]
						path: r.path.clone(),
					},
				);

				let is_disk_best = best
					.1
					.turn_to_finish
					.is_some_and(|ttf| best_disk_ttf.is_none_or(|disk| ttf < disk));
				if is_disk_best {
					best_disk_ttf = best.1.turn_to_finish;
					output_solution(&best.1.solution, validator_name)?;
					output_turn_to_finish(best.1.turn_to_finish.unwrap(), validator_name)?;
				}
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

		if best.1.finished {
			if !optimize_end && best_frontier < checkpoint_list.len() {
				optimize_end = true;
				step_to_checkpoint_limit = INITIAL_STEP_TO_CHECKPOINT_LIMIT;
				stagnant_generation_count = 0;
			} else if optimize_end {
				if best.0 < previous_best_score {
					stagnant_generation_count = 0;
				} else {
					stagnant_generation_count += 1;
					if stagnant_generation_count >= STAGNANT_GENERATIONS_BEFORE_WIDENING {
						optimize_end = false;
						step_to_checkpoint_limit = MAX_STEP_TO_CHECKPOINT_LIMIT;
						stagnant_generation_count = 0;
					}
				}
			} else if best.0 < previous_best_score {
				optimize_end = true;
				step_to_checkpoint_limit = INITIAL_STEP_TO_CHECKPOINT_LIMIT;
				stagnant_generation_count = 0;
			}
		} else if best.1.reached_checkpoint_count > best_frontier {
			step_to_checkpoint_limit = INITIAL_STEP_TO_CHECKPOINT_LIMIT;
			stagnant_generation_count = 0;
		} else {
			stagnant_generation_count += 1;
			if stagnant_generation_count >= STAGNANT_GENERATIONS_BEFORE_WIDENING {
				step_to_checkpoint_limit =
					(step_to_checkpoint_limit + 1).min(MAX_STEP_TO_CHECKPOINT_LIMIT);
				stagnant_generation_count = 0;
			}
		}
		best_frontier = best.1.reached_checkpoint_count;
		previous_best_score = best.0;
		let best_finished_step_count = if best.1.finished && !optimize_end {
			Some(best.1.step_count)
		} else {
			None
		};
		(solution_list, frozen_list) = breed_generation(
			solution_list,
			score_list,
			step_count_list,
			frozen_list,
			car_init_state,
			best_finished_step_count,
			step_to_checkpoint_limit,
		);

		if generation.is_multiple_of(128) {
			log_generation(generation, &best, step_to_checkpoint_limit);
		}

		generation += 1;
	}

	log_generation(generation, &best, step_to_checkpoint_limit);
	println!();

	let mut solution = best.1.solution;
	solution.truncate(best.1.step_count);
	Ok(solution)
}

// TODO: prettier log
// TODO: also show elapsed
// TODO: also show elapsed of current generation
// TODO: also show average time per solution of the generation
#[inline]
fn log_generation(
	generation: usize,
	best: &(Score, BestSolution),
	step_to_checkpoint_limit: usize,
) {
	eprint!(
		"\r{generation} {best_score:.2} {best_step_count} {step_to_checkpoint_limit}",
		generation = generation,
		best_score = best.0,
		best_step_count = best.1.step_count,
		step_to_checkpoint_limit = step_to_checkpoint_limit,
	);
}
