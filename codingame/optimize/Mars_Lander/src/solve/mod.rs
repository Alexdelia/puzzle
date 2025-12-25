mod first_generation;

use std::sync::mpsc;

use rayon::ThreadPoolBuilder;

use crate::{
	output_repr::Solution,
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
	let solution_list = first_generation::init_first_generation();
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

		process_generation(&pool, tx, landscape, lander_init_state, &solution_list);

		#[cfg(feature = "visualize")]
		let mut doc = base_doc.clone();

		for r in rx.iter().take(SOLUTION_PER_GENERATION) {
			let score = get_score(landing_segment, &lander_list[r.index], r.is_valid_landing);

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
	}

	Ok(())
}

fn process_generation(
	pool: &rayon::ThreadPool,
	tx: mpsc::Sender<ProcessOutput>,
	landscape: &[Segment],
	lander_init_state: Lander,
	solution_list: &[Solution],
) {
	pool.scope(|s| {
		for (i, solution) in solution_list.iter().enumerate() {
			let tx = tx.clone();
			let mut lander = lander_init_state;

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
						tx.send(ProcessOutput {
							index: i,
							is_valid_landing: false,
							#[cfg(feature = "visualize")]
							path,
						})
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
							tx.send(ProcessOutput {
								index: i,
								is_valid_landing: segment_index == VALID_LANDING_INDEX
									&& lander.valid_landing_condition(),
								#[cfg(feature = "visualize")]
								path,
							})
							.expect("failed to send result");
							return;
						}
					}
				}

				tx.send(ProcessOutput {
					index: i,
					is_valid_landing: false,
					#[cfg(feature = "visualize")]
					path,
				})
				.expect("failed to send result");
			});
		}
	});
}

fn get_score(landing_segment: &Segment, lander: &Lander, is_valid_landing: bool) -> Score {
	if is_valid_landing {
		return -(lander.fuel as Score);
	}

	let y_diff = lander.y - landing_segment.a.y;

	let x_diff = if lander.x < landing_segment.a.x {
		landing_segment.a.x - lander.x
	} else if lander.x > landing_segment.b.x {
		lander.x - landing_segment.b.x
	} else {
		0.0
	};

	(x_diff as Score) + (y_diff as Score)
}
