mod dist;
mod output_repr;
mod output_solution;
mod parse;
mod referee;
mod segment;
mod solve;
#[cfg(feature = "visualize")]
mod visualize;

use std::ops::Range;

use radiate::*;

use crate::{
	dist::dist,
	output_repr::{
		MAX_THRUST, MAX_TILT_CHANGE, MIN_THRUST, MIN_TILT_CHANGE, Solution, Step, TiltChange,
	},
	referee::{
		car::Car,
		env::{Coord, MAX_STEP},
		intersect,
		process_step::process_step,
	},
	segment::Segment,
	solve::{SOLUTION_PER_GENERATION, get_score::get_score},
};

type GeneTiltChange = StepGene;
const GENE_OFFSET_TILT_CHANGE: TiltChange = -MIN_TILT_CHANGE;
const GENE_MIN_TILT_CHANGE: GeneTiltChange =
	(MIN_TILT_CHANGE + GENE_OFFSET_TILT_CHANGE) as GeneTiltChange;
const GENE_MAX_TILT_CHANGE: GeneTiltChange =
	(MAX_TILT_CHANGE + GENE_OFFSET_TILT_CHANGE) as GeneTiltChange;

const CHROMOSOME_SIZE: usize = MAX_STEP / 2;

const TILT_RANGE: Range<StepGene> = GENE_MIN_TILT_CHANGE..GENE_MAX_TILT_CHANGE + 1;
const THRUST_RANGE: Range<StepGene> = MIN_THRUST..MAX_THRUST + 1;

// impl Gene for Step {
// type Allele = (GeneTiltChange, Thrust);
// }
//
// type StepChromosome = [(GeneTiltChange, Thrust); CHROMOSOME_SIZE];

type StepGene = u8;

#[derive(Clone, PartialEq)]
struct StepChromosome {
	step: [IntGene<StepGene>; 2],
}

impl Chromosome for StepChromosome {
	type Gene = IntGene<StepGene>;

	fn as_slice(&self) -> &[Self::Gene] {
		&self.step
	}

	fn as_mut_slice(&mut self) -> &mut [Self::Gene] {
		&mut self.step
	}
}

impl Valid for StepChromosome {
	fn is_valid(&self) -> bool {
		self.step[0].is_valid() && self.step[1].is_valid()
	}
}

impl StepChromosome {
	fn new() -> Self {
		StepChromosome {
			step: [
				IntGene::<StepGene>::from((TILT_RANGE, TILT_RANGE)),
				IntGene::<StepGene>::from((THRUST_RANGE, THRUST_RANGE)),
			],
		}
	}
}

fn solve(
	validator_name: &str,
	checkpoint_list: Vec<Coord>,
	car_init_state: Car,
) -> Result<Solution, String> {
	let codec: FnCodec<StepChromosome, Solution> = FnCodec::new()
		.with_encoder(|| Genotype::from(vec![StepChromosome::new(); CHROMOSOME_SIZE]))
		.with_decoder(|genotype| {
			let mut solution = Vec::with_capacity(CHROMOSOME_SIZE);
			for chromosome in genotype.iter() {
				let tilt_change = (*chromosome.step[0].allele() as TiltChange)
					- GENE_OFFSET_TILT_CHANGE as TiltChange;
				let thrust = *chromosome.step[1].allele();
				solution.push(Step {
					tilt: tilt_change,
					thrust,
				});
			}
			solution
		});

	let engine = GeneticEngine::builder()
		.codec(codec)
		.fitness_fn(move |solution: Vec<Step>| {
			let mut car = car_init_state;

			let mut checkpoint_index = 0;
			let mut reached_at_step = MAX_STEP;
			let mut closest_to_checkpoint = f64::INFINITY;

			for (step_index, step) in solution.iter().enumerate() {
				let from = Coord { x: car.x, y: car.y };

				process_step(&mut car, step);

				let traveled = Segment {
					a: from,
					b: Coord { x: car.x, y: car.y },
				};

				if reached_at_step + 64 < step_index {
					break;
				}

				let current_checkpoint = checkpoint_list[checkpoint_index];

				let d = dist(car.x, car.y, current_checkpoint.x, current_checkpoint.y);
				if d < closest_to_checkpoint {
					closest_to_checkpoint = d;
				}

				if intersect(current_checkpoint, traveled.a, traveled.b) {
					checkpoint_index += 1;
					reached_at_step = step_index;
					closest_to_checkpoint = f64::INFINITY;

					let step_count = step_index + 1;

					if checkpoint_index == checkpoint_list.len() {
						return get_score(
							&checkpoint_list,
							checkpoint_index,
							closest_to_checkpoint,
							step_count,
						);
					}
				}
			}

			let step_count = solution.len();
			get_score(
				&checkpoint_list,
				checkpoint_index,
				closest_to_checkpoint,
				step_count,
			)
		})
		.minimizing()
		.population_size(SOLUTION_PER_GENERATION)
		.parallel()
		.build();

	/*
	let mut best = (Score::MAX, Solution::default());

	for generation in engine.iter() {
		let score = generation.score().as_f32();
		if score < best.0 {
			best = (score, generation.value().clone());
		}

		if generation.index().is_multiple_of(128) {
			eprint!(
				"\r{index} {best_score} {best_step_count}",
				index = generation.index(),
				best_score = best.0,
				best_step_count = best.1.len()
			);
		}

		if generation.time().as_secs() >= 60 {
			break;
		}
	}

	Ok(best.1)
	*/

	engine
		.iter()
		.logging()
		.until_seconds(600.0)
		.last()
		.inspect(|generation| {
			let score = generation.score().as_f32();
			eprintln!(
				"\r{index} {score} {step_count}",
				index = generation.index(),
				step_count = generation.value().len()
			);
		});

	Err("not implemented".to_string())
}

fn main() -> Result<(), String> {
	let path = parse::get_path()?;
	let validator_name = parse::get_validator_name(&path);

	let (car_init_state, checkpoint_list) = parse::parse(&path)?;

	let solution = solve(&validator_name, checkpoint_list, car_init_state)?;

	// output_solution::output_solution(&solution, &validator_name)
	Ok(())
}
