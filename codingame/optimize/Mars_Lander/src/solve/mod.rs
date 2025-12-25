mod first_generation;
mod simulate_generation;
use simulate_generation::simulate_generation;
mod get_score;
use get_score::get_score;
mod breed_generation;
use breed_generation::breed_generation;

use std::sync::mpsc;

use rayon::ThreadPoolBuilder;

#[cfg(feature = "visualize")]
use crate::visualize;
use crate::{
	output_repr::Solution,
	parse::get_iteration,
	referee::{env::Coord, lander::Lander},
	segment::Segment,
};

pub const SOLUTION_PER_GENERATION: usize = 512;

pub const VALID_LANDING_INDEX: usize = 0;

pub type Score = i16;

struct ProcessOutput {
	index: usize,
	lander: Lander,
	is_valid_landing: bool,
	#[cfg(feature = "visualize")]
	path: Vec<Coord>,
}

#[derive(Default)]
struct BestSolution {
	is_valid_landing: bool,
	solution: Solution,
	#[cfg(feature = "visualize")]
	path: Vec<Coord>,
}

pub fn solve(
	landscape: &[Segment],
	lander_init_state: &Lander,
	#[cfg(feature = "visualize")] base_doc: svg::Document,
	#[cfg(feature = "visualize")] validator_name: &str,
) -> Result<Solution, String> {
	let landing_segment = &landscape[VALID_LANDING_INDEX];

	let pool = ThreadPoolBuilder::new()
		.build()
		.map_err(|e| format!("failed to build thread pool: {e}"))?;

	let mut solution_list = first_generation::init_first_generation();
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
		eprint!("\r{generation}");

		let (tx, rx) = mpsc::channel::<ProcessOutput>();

		simulate_generation(&pool, tx, landscape, lander_init_state, &solution_list);

		#[cfg(feature = "visualize")]
		let mut doc = base_doc.clone();

		for r in rx.iter().take(SOLUTION_PER_GENERATION) {
			let score = get_score(landing_segment, &r.lander, r.is_valid_landing);
			score_list[r.index] = score;

			if score < best.0 {
				best = (
					score,
					BestSolution {
						is_valid_landing: r.is_valid_landing,
						solution: solution_list[r.index].clone(),
						#[cfg(feature = "visualize")]
						path: r.path.clone(),
					},
				);
				// dbg!(&best.0, &best.1.is_valid_landing, &best.1.lander);
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

		generation += 1;

		eprint!(" {best_score}", best_score = best.0);
	}

	eprintln!("");

	Ok(best.1.solution)
}
