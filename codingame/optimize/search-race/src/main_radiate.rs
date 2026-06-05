mod dist;
mod output_repr;
mod output_solution;
mod parse;
mod referee;
mod segment;
mod solve;
#[cfg(feature = "visualize")]
mod visualize;

use radiate::*;

use crate::{
	dist::dist,
	output_repr::{MAX_THRUST, MAX_TILT_CHANGE, MIN_THRUST, MIN_TILT_CHANGE, Step},
	referee::{
		car::Car,
		env::{Coord, MAX_STEP},
		intersect,
		process_step::process_step,
	},
	segment::Segment,
	solve::{SOLUTION_PER_GENERATION, get_score::get_score},
};

const INPUT_COUNT: usize = 8;
const TREE_DEPTH: usize = 7;
const STEPS_PER_CHECKPOINT: usize = 64;

fn build_input(car: &Car, checkpoint: Coord) -> [f32; INPUT_COUNT] {
	let dx = checkpoint.x - car.x;
	let dy = checkpoint.y - car.y;
	let distance = (dx * dx + dy * dy).sqrt();
	let angle_to = dy.atan2(dx);
	let (sin, cos) = car.angle.sin_cos();
	[
		(dx / distance) as f32,
		(dy / distance) as f32,
		(distance / 1000.0) as f32,
		(angle_to - car.angle) as f32,
		car.sx as f32 / 100.0,
		car.sy as f32 / 100.0,
		sin as f32,
		cos as f32,
	]
}

fn simulate_with_trees(
	trees: &[Tree<Op<f32>>],
	checkpoint_list: &[Coord],
	car_init_state: Car,
) -> f32 {
	let mut car = car_init_state;
	let mut checkpoint_index = 0;
	let mut reached_at_step = 0;
	let mut closest_to_checkpoint = f64::INFINITY;

	for step_index in 0..MAX_STEP {
		let current_checkpoint = checkpoint_list[checkpoint_index];
		let input = build_input(&car, current_checkpoint);

		let tilt_raw = trees[0].eval(&input);
		let thrust_raw = trees[1].eval(&input);

		let tilt = (tilt_raw.round() as i8).clamp(MIN_TILT_CHANGE, MAX_TILT_CHANGE);
		let thrust = thrust_raw
			.round()
			.clamp(MIN_THRUST as f32, MAX_THRUST as f32) as u8;

		let step = Step { tilt, thrust };

		let from = Coord { x: car.x, y: car.y };
		let moved_to = process_step(&mut car, &step);
		let traveled = Segment {
			a: from,
			b: moved_to,
		};

		if step_index > reached_at_step + STEPS_PER_CHECKPOINT {
			break;
		}

		let d = dist(car.x, car.y, current_checkpoint.x, current_checkpoint.y);
		if d < closest_to_checkpoint {
			closest_to_checkpoint = d;
		}

		if intersect(current_checkpoint, traveled.a, traveled.b) {
			checkpoint_index += 1;
			reached_at_step = step_index;
			closest_to_checkpoint = f64::INFINITY;

			if checkpoint_index == checkpoint_list.len() {
				return get_score(
					checkpoint_list,
					checkpoint_index,
					closest_to_checkpoint,
					step_index + 1,
					None,
				);
			}
		}
	}

	get_score(
		checkpoint_list,
		checkpoint_index,
		closest_to_checkpoint,
		MAX_STEP,
		None,
	)
}

fn main() -> Result<(), String> {
	let path = parse::get_path()?;
	let _validator_name = parse::get_validator_name(&path);
	let (car_init_state, checkpoint_list) = parse::parse(&path)?;

	let store = vec![
		(
			NodeType::Vertex,
			vec![
				Op::add(),
				Op::sub(),
				Op::mul(),
				Op::div(),
				Op::abs(),
				Op::neg(),
				Op::min(),
				Op::max(),
				Op::sin(),
				Op::cos(),
				Op::weight(),
			],
		),
		(
			NodeType::Leaf,
			vec![
				Op::var(0),
				Op::var(1),
				Op::var(2),
				Op::var(3),
				Op::var(4),
				Op::var(5),
				Op::var(6),
				Op::var(7),
				Op::constant(0.0_f32),
				Op::constant(1.0_f32),
			],
		),
	];

	let codec = TreeCodec::multi_root(TREE_DEPTH, 2, store);

	let engine = GeneticEngine::builder()
		.codec(codec)
		.fitness_fn(move |trees: Vec<Tree<Op<f32>>>| {
			simulate_with_trees(&trees, &checkpoint_list, car_init_state)
		})
		.minimizing()
		.population_size(SOLUTION_PER_GENERATION)
		.alter(alters!(
			TreeCrossover::new(0.5),
			HoistMutator::new(0.02),
			OperationMutator::new(0.05, 0.03)
		))
		.parallel()
		.build();

	engine
		.iter()
		.logging()
		.until_seconds(600.0)
		.last()
		.inspect(|generation| {
			let score = generation.score().as_f32();
			eprintln!("gen {} score {score:.3}", generation.index());
			for (i, tree) in generation.value().iter().enumerate() {
				eprintln!("  tree[{i}]: {}", tree.format());
			}
		});

	Ok(())
}
