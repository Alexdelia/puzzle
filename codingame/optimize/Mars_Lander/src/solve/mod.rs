mod first_generation;
mod simulate_generation;
use simulate_generation::simulate_generation;
mod get_score;
use get_score::get_score;
mod breed_generation;
use breed_generation::breed_generation;

use std::sync::mpsc;

use rayon::ThreadPoolBuilder;

use crate::{
	referee::{env::Coord, lander::Lander},
	segment::Segment,
	visualize,
};

pub const SOLUTION_PER_GENERATION: usize = 128;

pub const VALID_LANDING_INDEX: usize = 0;

pub type Score = i16;

struct ProcessOutput {
	index: usize,
	is_valid_landing: bool,
	#[cfg(feature = "visualize")]
	path: Vec<Coord>,
}

#[derive(Default)]
struct BestSolution {
	is_valid_landing: bool,
	#[cfg(feature = "visualize")]
	path: Vec<Coord>,
}

pub fn solve(
	landscape: &[Segment],
	lander_init_state: Lander,
	#[cfg(feature = "visualize")] base_doc: svg::Document,
	#[cfg(feature = "visualize")] validator_name: &str,
) -> Result<(), String> {
	let landing_segment = &landscape[VALID_LANDING_INDEX];

	let pool = ThreadPoolBuilder::new()
		.build()
		.map_err(|e| format!("failed to build thread pool: {e}"))?;

	let lander_list = [lander_init_state; SOLUTION_PER_GENERATION];
	let mut solution_list = first_generation::init_first_generation();
	let mut score_list = [Score::default(); SOLUTION_PER_GENERATION];
	#[cfg(feature = "visualize")]
	let mut path_list: [Vec<Coord>; SOLUTION_PER_GENERATION] = (0..SOLUTION_PER_GENERATION)
		.map(|_| Vec::new())
		.collect::<Vec<_>>()
		.try_into()
		.expect("path list size mismatch");

	let mut best = (Score::MAX, BestSolution::default());

	for generation in 0..(crate::parse::get_iteration()?) {
		dbg!(generation);

		let (tx, rx) = mpsc::channel::<ProcessOutput>();

		simulate_generation(&pool, tx, landscape, lander_init_state, &solution_list);

		#[cfg(feature = "visualize")]
		let mut doc = base_doc.clone();

		for r in rx.iter().take(SOLUTION_PER_GENERATION) {
			let score = get_score(landing_segment, &lander_list[r.index], r.is_valid_landing);
			score_list[r.index] = score;

			if score < best.0 {
				best = (
					score,
					BestSolution {
						is_valid_landing: r.is_valid_landing,
						#[cfg(feature = "visualize")]
						path: r.path.clone(),
					},
				)
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

		// breed next generation
		breed_generation(&mut solution_list, &score_list);
	}

	Ok(())
}
