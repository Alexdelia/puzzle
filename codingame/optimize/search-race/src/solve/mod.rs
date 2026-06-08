mod first_generation;
mod simulate_generation;
use simulate_generation::{GpuSim, SimOutput, simulate_solution};
mod breed_generation;
#[cfg(feature = "extra-log")]
mod extra_log;
pub mod get_score;
use breed_generation::breed_generation;

use std::time::{Duration, Instant};

#[cfg(feature = "visualize")]
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
const MAX_STEP_TO_CHECKPOINT_LIMIT: usize = 48;
const STAGNANT_GENERATIONS_BEFORE_WIDENING: usize = 512;

const MAX_SEARCH_DURATION: Duration = Duration::from_secs(60 * 15);

const CHECKPOINT_LOOKBACK: usize = 2;
const _: () = assert!(CHECKPOINT_LOOKBACK >= 1);

pub type Score = f32;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
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

#[derive(Default)]
struct BestSolution {
	finished: bool,
	solution: Solution,
	step_count: usize,
	turn_to_finish: Option<f64>,
	reached_checkpoint_count: usize,
	#[cfg(feature = "visualize")]
	path: Vec<Coord>,
	#[cfg(feature = "extra-log")]
	max_step_gap: Option<usize>,
}

fn boxed_filled<T: Clone>(value: T, n: usize) -> Box<[T]> {
	vec![value; n].into_boxed_slice()
}

pub fn solve(
	validator_name: &str,
	checkpoint_list: &[Coord],
	car_init_state: &Car,
	#[cfg(feature = "visualize")] base_doc: svg::Document,
) -> Result<(), String> {
	#[cfg(feature = "visualize")]
	let pool = ThreadPoolBuilder::new()
		.build()
		.map_err(|e| format!("failed to build thread pool: {e}"))?;

	let mut gpu = GpuSim::new(SOLUTION_PER_GENERATION, checkpoint_list)?;

	let checkpoint_count = checkpoint_list.len();

	let fresh = crate::parse::is_fresh();
	let mut best_disk_ttf = read_turn_to_finish(validator_name);
	let (mut solution_list, loaded) =
		first_generation::init_first_generation(validator_name, fresh)?;
	let mut frozen_list: Box<[FrozenPrefix]> = boxed_filled(
		FrozenPrefix::from_scratch(car_init_state),
		SOLUTION_PER_GENERATION,
	);
	let mut sim_outputs: Box<[SimOutput]> =
		boxed_filled(SimOutput::default(), SOLUTION_PER_GENERATION);

	let mut best = (Score::MAX, BestSolution::default());

	let mut step_to_checkpoint_limit = INITIAL_STEP_TO_CHECKPOINT_LIMIT;
	let mut best_frontier = 0;
	let mut stagnant_generation_count = 0;
	let mut optimize_end = false;
	let mut previous_best_score = Score::MAX;

	if loaded {
		let loaded_run = simulate_solution(
			checkpoint_list,
			car_init_state,
			&solution_list[0],
			&FrozenPrefix::from_scratch(car_init_state),
			MAX_STEP,
		);
		best_frontier = loaded_run.reached_checkpoint_count as usize;
		frozen_list[0] = loaded_run.frozen;
		if loaded_run.is_finished() {
			optimize_end = true;
		}
	}

	#[cfg(feature = "extra-log")]
	eprint!("\n\n\n\n");
	#[cfg(not(feature = "extra-log"))]
	eprint!("\n\n\n");

	let start_time = Instant::now();
	let mut generation: usize = 0;
	let max_iteration = get_iteration()?;
	while !best.1.finished || optimize_end || generation < max_iteration {
		let generation_start = Instant::now();

		gpu.run(
			&solution_list,
			&frozen_list,
			car_init_state,
			step_to_checkpoint_limit,
			&mut sim_outputs,
		)?;

		#[cfg(feature = "visualize")]
		let path_list = compute_paths_for_visualize(
			&pool,
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

		for i in 0..SOLUTION_PER_GENERATION {
			let r = sim_outputs[i];
			let r_finished = r.is_finished();
			frozen_list[i] = if r_finished && !optimize_end {
				FrozenPrefix::from_scratch(car_init_state)
			} else {
				r.frozen
			};

			if r.score < best.0 {
				best = (
					r.score,
					BestSolution {
						finished: r_finished,
						solution: solution_list[i],
						step_count: r.step_count as usize,
						turn_to_finish: r.turn_to_finish_opt(),
						reached_checkpoint_count: r.reached_checkpoint_count as usize,
						#[cfg(feature = "visualize")]
						path: path_list[i].clone(),
						#[cfg(feature = "extra-log")]
						max_step_gap: extra_log::compute_max_step_gap(
							checkpoint_list,
							car_init_state,
							&solution_list[i],
						),
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
		}

		#[cfg(feature = "visualize")]
		{
			for (i, path) in path_list.iter().enumerate() {
				doc = doc.add(visualize::solution(
					path,
					sim_outputs[i].is_finished(),
					false,
				));
			}
			doc = doc.add(visualize::solution(&best.1.path, best.1.finished, true));
			visualize::write_doc(validator_name, &doc, generation);
		}

		let mut burst = false;
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
						burst = true;
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
				burst = true;
			}
		}
		best_frontier = best.1.reached_checkpoint_count;
		previous_best_score = best.0;
		let best_finished_step_count = if best.1.finished && !optimize_end {
			Some(best.1.step_count)
		} else {
			None
		};

		let score_list: Box<[Score]> = sim_outputs.iter().map(|o| o.score).collect();
		let step_count_list: Box<[usize]> =
			sim_outputs.iter().map(|o| o.step_count as usize).collect();

		(solution_list, frozen_list) = breed_generation(
			solution_list,
			&score_list,
			&step_count_list,
			frozen_list,
			car_init_state,
			best_finished_step_count,
			step_to_checkpoint_limit,
			burst,
		);

		let generation_elapsed = generation_start.elapsed();
		if generation.is_multiple_of(128) {
			if generation_elapsed > MAX_SEARCH_DURATION {
				break;
			}

			log_generation(
				generation,
				&best,
				checkpoint_count,
				step_to_checkpoint_limit,
				generation_elapsed,
				start_time.elapsed(),
			);
		}

		generation += 1;
	}

	log_generation(
		generation,
		&best,
		checkpoint_count,
		step_to_checkpoint_limit,
		Duration::ZERO,
		start_time.elapsed(),
	);
	println!();

	Ok(())
}

#[cfg(feature = "visualize")]
fn compute_paths_for_visualize(
	pool: &rayon::ThreadPool,
	checkpoint_list: &[Coord],
	car_init_state: &Car,
	solution_list: &[Solution],
	frozen_list: &[FrozenPrefix],
	step_to_checkpoint_limit: usize,
) -> Box<[Vec<Coord>]> {
	use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

	let mut paths: Box<[Vec<Coord>]> = vec![Vec::new(); SOLUTION_PER_GENERATION].into_boxed_slice();
	pool.install(|| {
		paths.par_iter_mut().enumerate().for_each(|(i, p)| {
			*p = simulate_generation::compute_path(
				checkpoint_list,
				car_init_state,
				&solution_list[i],
				&frozen_list[i],
				step_to_checkpoint_limit,
			);
		});
	});
	paths
}

#[inline]
fn log_generation(
	generation: usize,
	best: &(Score, BestSolution),
	checkpoint_count: usize,
	step_to_checkpoint_limit: usize,
	generation_elapsed: Duration,
	total_elapsed: Duration,
) {
	let (progress, progress_color) = if best.1.finished {
		(600.0 + best.0, "\x1b[1;32m")
	} else {
		(best.0, "\x1b[1;35m")
	};

	let elapsed_sec = total_elapsed.as_secs();
	let elapsed_str = if elapsed_sec >= 60 {
		format!(
			"\x1b[0;34m{min}\x1b[2mm \x1b[0;36m{sec:>2}\x1b[2ms",
			min = elapsed_sec / 60,
			sec = elapsed_sec % 60
		)
	} else {
		format!("\x1b[0;36m{elapsed_sec}\x1b[2ms")
	};

	#[cfg(feature = "extra-log")]
	let extra_line = match best.1.max_step_gap {
		Some(g) => format!("\nmax_step_between_checkpoints: \x1b[0;33m{g}\x1b[0m"),
		None => "\nmax_step_between_checkpoints: \x1b[2m-\x1b[0m".to_string(),
	};
	#[cfg(not(feature = "extra-log"))]
	let extra_line = "";
	let cursor_up = if cfg!(feature = "extra-log") { 3 } else { 2 };

	eprint!(
		"\r\x1b[{cursor_up}A{progress_color}{progress:>11.3} \x1b[0;32m{best_checkpoint_reached:>2}\x1b[0;2m/{checkpoint_count} \x1b[0;2m{best_step_count:>3}\x1b[0;33m+{step_to_checkpoint_limit:<2}{extra_line}
\x1b[0;38;2;52;235;198m{average_nano:>5.2}\x1b[2mμ \x1b[0;96m{generation_ms:>6.3}\x1b[2mms {elapsed_str}
\x1b[0;1m{generation}\x1b[0m",
		best_checkpoint_reached = best.1.reached_checkpoint_count,
		best_step_count = best.1.step_count,
		average_nano = (generation_elapsed / SOLUTION_PER_GENERATION as u32).as_nanos() as f32 / 1_000.0,
		generation_ms = generation_elapsed.as_micros() as f32 / 1_000.0,
	);
}
