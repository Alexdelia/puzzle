use std::sync::Arc;

use cudarc::driver::{
	CudaContext, CudaFunction, CudaModule, CudaSlice, CudaStream, DeviceRepr, LaunchConfig,
	PushKernelArg, ValidAsZeroBits,
};
use cudarc::nvrtc::compile_ptx;

use crate::{
	dist::dist,
	output_repr::Solution,
	referee::{
		car::Car,
		env::{CHECKPOINT_RADIUS, Coord},
		intersect,
		process_step::process_step,
	},
	segment::Segment,
	solve::get_score::get_score,
};

use super::{CHECKPOINT_LOOKBACK, FrozenPrefix, Score};

const CROSSING_LIST_SIZE: usize = CHECKPOINT_LOOKBACK + 1;

const KERNEL_SOURCE: &str = include_str!("kernel.cu");

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SimOutput {
	pub finished: u32,
	pub score: Score,
	pub step_count: u32,
	pub reached_checkpoint_count: u32,
	/// NaN encodes `None`.
	pub turn_to_finish: f64,
	pub frozen: FrozenPrefix,
}

impl Default for SimOutput {
	fn default() -> Self {
		Self {
			finished: 0,
			score: 0.0,
			step_count: 0,
			reached_checkpoint_count: 0,
			turn_to_finish: f64::NAN,
			frozen: FrozenPrefix {
				resume_from_step: 0,
				car: Car::default(),
				checkpoint_index: 0,
				reentry_step_count: 0,
			},
		}
	}
}

impl SimOutput {
	#[inline]
	pub fn is_finished(&self) -> bool {
		self.finished != 0
	}

	#[inline]
	pub fn turn_to_finish_opt(self) -> Option<f64> {
		if self.turn_to_finish.is_nan() {
			None
		} else {
			Some(self.turn_to_finish)
		}
	}
}

unsafe impl DeviceRepr for SimOutput {}
unsafe impl ValidAsZeroBits for SimOutput {}
unsafe impl DeviceRepr for FrozenPrefix {}
unsafe impl ValidAsZeroBits for FrozenPrefix {}
unsafe impl DeviceRepr for Solution {}
unsafe impl ValidAsZeroBits for Solution {}
unsafe impl DeviceRepr for Coord {}
unsafe impl ValidAsZeroBits for Coord {}
unsafe impl DeviceRepr for Car {}
unsafe impl ValidAsZeroBits for Car {}

pub struct GpuSim {
	#[allow(dead_code)]
	ctx: Arc<CudaContext>,
	stream: Arc<CudaStream>,
	#[allow(dead_code)]
	module: Arc<CudaModule>,
	func: CudaFunction,
	solutions_buf: CudaSlice<Solution>,
	frozens_buf: CudaSlice<FrozenPrefix>,
	checkpoints_buf: CudaSlice<Coord>,
	outputs_buf: CudaSlice<SimOutput>,
	checkpoint_count: i32,
	capacity: usize,
}

impl GpuSim {
	pub fn new(capacity: usize, checkpoint_list: &[Coord]) -> Result<Self, String> {
		let ctx = CudaContext::new(0).map_err(|e| format!("CudaContext::new: {e}"))?;
		let stream = ctx.default_stream();
		let ptx = compile_ptx(KERNEL_SOURCE).map_err(|e| format!("nvrtc compile: {e}"))?;
		let module = ctx
			.load_module(ptx)
			.map_err(|e| format!("load_module: {e}"))?;
		let func = module
			.load_function("simulate")
			.map_err(|e| format!("load_function: {e}"))?;

		let solutions_buf = stream
			.alloc_zeros::<Solution>(capacity)
			.map_err(|e| format!("alloc solutions: {e}"))?;
		let frozens_buf = stream
			.alloc_zeros::<FrozenPrefix>(capacity)
			.map_err(|e| format!("alloc frozens: {e}"))?;
		let outputs_buf = stream
			.alloc_zeros::<SimOutput>(capacity)
			.map_err(|e| format!("alloc outputs: {e}"))?;
		let checkpoints_buf = stream
			.clone_htod(checkpoint_list)
			.map_err(|e| format!("upload checkpoints: {e}"))?;

		Ok(Self {
			ctx,
			stream,
			module,
			func,
			solutions_buf,
			frozens_buf,
			checkpoints_buf,
			outputs_buf,
			checkpoint_count: checkpoint_list.len() as i32,
			capacity,
		})
	}

	pub fn run(
		&mut self,
		solutions: &[Solution],
		frozens: &[FrozenPrefix],
		car_init: &Car,
		step_to_checkpoint_limit: usize,
		outputs: &mut [SimOutput],
	) -> Result<(), String> {
		let n = solutions.len();
		assert_eq!(n, frozens.len());
		assert_eq!(n, outputs.len());
		assert!(n <= self.capacity);

		self.stream
			.memcpy_htod(solutions, &mut self.solutions_buf)
			.map_err(|e| format!("upload solutions: {e}"))?;
		self.stream
			.memcpy_htod(frozens, &mut self.frozens_buf)
			.map_err(|e| format!("upload frozens: {e}"))?;

		let n_i32: i32 = n as i32;
		let step_limit_i32: i32 = step_to_checkpoint_limit as i32;
		let car = *car_init;
		let cp_count = self.checkpoint_count;

		let mut builder = self.stream.launch_builder(&self.func);
		builder.arg(&self.solutions_buf);
		builder.arg(&self.frozens_buf);
		builder.arg(&self.checkpoints_buf);
		builder.arg(&cp_count);
		builder.arg(&car);
		builder.arg(&step_limit_i32);
		builder.arg(&mut self.outputs_buf);
		builder.arg(&n_i32);

		const BLOCK: u32 = 128;
		let n_u32 = n as u32;
		let cfg = LaunchConfig {
			grid_dim: (n_u32.div_ceil(BLOCK), 1, 1),
			block_dim: (BLOCK, 1, 1),
			shared_mem_bytes: 0,
		};
		unsafe { builder.launch(cfg) }.map_err(|e| format!("kernel launch: {e}"))?;

		self.stream
			.memcpy_dtoh(&self.outputs_buf, outputs)
			.map_err(|e| format!("download outputs: {e}"))?;
		self.stream
			.synchronize()
			.map_err(|e| format!("stream sync: {e}"))?;

		Ok(())
	}
}

pub fn simulate_solution(
	checkpoint_list: &[Coord],
	car_init_state: &Car,
	solution: &Solution,
	frozen: &FrozenPrefix,
	step_to_checkpoint_limit: usize,
) -> SimOutput {
	let resuming = frozen.resume_from_step > 0;
	let (mut car, mut checkpoint_index) = if resuming {
		(frozen.car, frozen.checkpoint_index)
	} else {
		(*car_init_state, 0)
	};

	let mut reached_at_step = frozen.resume_from_step.saturating_sub(1);
	let mut window_start = reached_at_step;
	let mut window_len = step_to_checkpoint_limit;

	let mut closest_to_checkpoint = f64::INFINITY;

	let mut crossing_list: [Option<FrozenPrefix>; CROSSING_LIST_SIZE] = [None; CROSSING_LIST_SIZE];

	for (step_index, step) in solution.iter().enumerate().skip(frozen.resume_from_step) {
		let from = Coord { x: car.x, y: car.y };

		let moved_to = process_step(&mut car, step);

		let traveled = Segment {
			a: from,
			b: moved_to,
		};

		if window_start + window_len < step_index {
			break;
		}

		let current_checkpoint = checkpoint_list[checkpoint_index];

		let d = dist(car.x, car.y, current_checkpoint.x, current_checkpoint.y);
		if d < closest_to_checkpoint {
			closest_to_checkpoint = d;
		}

		if intersect(current_checkpoint, traveled.a, traveled.b) {
			let crossed_segment_step_count = step_index - reached_at_step;
			crossing_list.copy_within(0..CROSSING_LIST_SIZE - 1, 1);
			if let Some(previous) = &mut crossing_list[1] {
				previous.reentry_step_count = crossed_segment_step_count;
			}
			crossing_list[0] = Some(FrozenPrefix {
				resume_from_step: step_index + 1,
				car,
				checkpoint_index: checkpoint_index + 1,
				reentry_step_count: 0,
			});
			reached_at_step = step_index;

			checkpoint_index += 1;
			closest_to_checkpoint = f64::INFINITY;

			window_start = step_index;
			window_len = step_to_checkpoint_limit;

			if checkpoint_index == checkpoint_list.len() {
				let step_count = step_index + 1;
				let entry_t = checkpoint_entry_fraction(traveled.a, traveled.b, current_checkpoint);
				let turn_to_finish = step_index as f64 + entry_t;
				return SimOutput {
					finished: 1,
					score: get_score(
						checkpoint_list,
						checkpoint_index,
						closest_to_checkpoint,
						step_count,
						Some(turn_to_finish),
					),
					step_count: step_count as u32,
					reached_checkpoint_count: checkpoint_index as u32,
					turn_to_finish,
					frozen: crossing_list[CROSSING_LIST_SIZE - 1].unwrap_or(*frozen),
				};
			}
		}
	}

	let step_count = solution.len();

	if closest_to_checkpoint.is_infinite() {
		let current_checkpoint = checkpoint_list[checkpoint_index];
		closest_to_checkpoint = dist(car.x, car.y, current_checkpoint.x, current_checkpoint.y);
	}

	SimOutput {
		finished: 0,
		step_count: step_count as u32,
		reached_checkpoint_count: checkpoint_index as u32,
		turn_to_finish: f64::NAN,
		frozen: crossing_list[CROSSING_LIST_SIZE - 1].unwrap_or(*frozen),
		score: get_score(
			checkpoint_list,
			checkpoint_index,
			closest_to_checkpoint,
			step_count,
			None,
		),
	}
}

#[cfg(feature = "visualize")]
pub fn compute_path(
	checkpoint_list: &[Coord],
	car_init_state: &Car,
	solution: &Solution,
	frozen: &FrozenPrefix,
	step_to_checkpoint_limit: usize,
) -> Vec<Coord> {
	let resuming = frozen.resume_from_step > 0;
	let (mut car, mut checkpoint_index) = if resuming {
		(frozen.car, frozen.checkpoint_index)
	} else {
		(*car_init_state, 0)
	};

	let mut path = Vec::with_capacity(solution.len() + 1);
	path.push(Coord { x: car.x, y: car.y });

	let mut reached_at_step = frozen.resume_from_step.saturating_sub(1);
	let mut window_start = reached_at_step;
	let mut window_len = step_to_checkpoint_limit;

	for (step_index, step) in solution.iter().enumerate().skip(frozen.resume_from_step) {
		let from = Coord { x: car.x, y: car.y };
		let moved_to = process_step(&mut car, step);
		path.push(Coord { x: car.x, y: car.y });

		if window_start + window_len < step_index {
			break;
		}

		let current_checkpoint = checkpoint_list[checkpoint_index];
		let traveled = Segment {
			a: from,
			b: moved_to,
		};
		if intersect(current_checkpoint, traveled.a, traveled.b) {
			reached_at_step = step_index;
			checkpoint_index += 1;
			window_start = step_index;
			window_len = step_to_checkpoint_limit;
			if checkpoint_index == checkpoint_list.len() {
				break;
			}
		}
	}

	let _ = reached_at_step;
	path
}

fn checkpoint_entry_fraction(from: Coord, to: Coord, checkpoint: Coord) -> f64 {
	let dx = to.x - from.x;
	let dy = to.y - from.y;
	let fx = from.x - checkpoint.x;
	let fy = from.y - checkpoint.y;

	let a = dx * dx + dy * dy;
	if a == 0.0 {
		return 0.0;
	}

	let b = 2.0 * (fx * dx + fy * dy);
	let c = fx * fx + fy * fy - CHECKPOINT_RADIUS * CHECKPOINT_RADIUS;

	let discriminant = b * b - 4.0 * a * c;
	if discriminant < 0.0 {
		return 1.0;
	}

	let t = (-b - discriminant.sqrt()) / (2.0 * a);
	t.clamp(0.0, 1.0)
}
