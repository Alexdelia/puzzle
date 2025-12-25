mod first_generation;

use std::sync::mpsc;

use rayon::ThreadPoolBuilder;

use crate::{
	referee::{
		env::{Coord, MAX_HEIGHT, MAX_WIDTH},
		intersect,
		lander::Lander,
		process_step::process_step,
	},
	segment::Segment,
	visualize,
};

pub const SOLUTION_PER_GENERATION: usize = 128;

pub const VALID_LANDING_INDEX: usize = 0;

pub type Score = i16;

pub fn solve(
	landscape: &[Segment],
	lander_init_state: Lander,
	#[cfg(feature = "visualize")] base_doc: svg::Document,
) -> Result<(), String> {
	let pool = ThreadPoolBuilder::new()
		.build()
		.map_err(|e| format!("failed to build thread pool: {e}"))?;

	let mut lander_list = [lander_init_state; SOLUTION_PER_GENERATION];
	let mut solution_list = first_generation::init_first_generation();
	#[cfg(feature = "visualize")]
	let mut path_list: [Vec<Coord>; SOLUTION_PER_GENERATION] = (0..SOLUTION_PER_GENERATION)
		.map(|_| Vec::new())
		.collect::<Vec<_>>()
		.try_into()
		.expect("path list size mismatch");

	let mut best: (Score, usize) = (Score::MAX, 0);

	for generation in 0..(crate::parse::get_iteration()?) {
		dbg!(generation);

		// run all solution in parallel
		// keep the solution with the best score
		// produce the next generation

		#[cfg(feature = "visualize")]
		let (tx, rx) = mpsc::channel::<(usize, bool, Vec<Coord>)>();
		#[cfg(not(feature = "visualize"))]
		let (tx, rx) = mpsc::channel::<(usize, bool)>();

		pool.scope(|s| {
			for i in 0..SOLUTION_PER_GENERATION {
				let tx = tx.clone();
				let landscape = landscape;
				let solution = &solution_list[i];
				let mut lander = lander_init_state.clone();
				s.spawn(move |_| {
					#[cfg(feature = "visualize")]
					let mut path = Vec::with_capacity(solution.len() + 1);
					#[cfg(feature = "visualize")]
					path.push(Coord {
						x: lander.x,
						y: lander.y,
					});

					for step in solution {
						let from = Coord {
							x: lander.x,
							y: lander.y,
						};

						process_step(&mut lander, step);

						if lander.x < 0.0
							|| lander.x > MAX_WIDTH
							|| lander.y < 0.0 || lander.y > MAX_HEIGHT
						{
							tx.send((
								i,
								false,
								#[cfg(feature = "visualize")]
								path,
							))
							.expect("failed to send result");
							return;
						}

						let traveled = Segment {
							a: from,
							b: Coord {
								x: lander.x,
								y: lander.y,
							},
						};
						#[cfg(feature = "visualize")]
						path.push(traveled.b);

						for (segment_index, segment) in landscape.iter().enumerate() {
							if intersect(&traveled, segment) {
								tx.send((
									i,
									segment_index == VALID_LANDING_INDEX,
									#[cfg(feature = "visualize")]
									path,
								))
								.expect("failed to send result");
								return;
							}
						}
					}

					tx.send((
						i,
						false,
						#[cfg(feature = "visualize")]
						path,
					))
					.expect("failed to send result");
				});
			}
		});

		#[cfg(feature = "visualize")]
		let mut doc = base_doc.clone();

		for r in rx.iter().take(SOLUTION_PER_GENERATION) {
			let i = r.0;
			let is_valid_landing = r.1;
			#[cfg(feature = "visualize")]
			let path = r.2;

			#[cfg(feature = "visualize")]
			{
				doc = doc.add(visualize::solution(&path, !is_valid_landing, false));
			}

			// FIXME: compute correct score
			let score = if is_valid_landing { 0 } else { 1 };

			if score < best.0 {
				best = (score, i);
			}
		}
	}

	Ok(())
}
