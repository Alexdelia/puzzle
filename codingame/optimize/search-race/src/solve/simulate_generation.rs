use std::sync::Arc;

use cudarc::driver::{
	CudaContext, CudaFunction, CudaModule, CudaSlice, CudaStream, DeviceRepr, LaunchConfig,
	PushKernelArg, ValidAsZeroBits,
	result::{free_host, malloc_host},
};
use cudarc::nvrtc::{CompileOptions, compile_ptx_with_opts};

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

#[cfg(feature = "visualize")]
use crate::referee::env::MAX_STEP;

const CROSSING_LIST_SIZE: usize = CHECKPOINT_LOOKBACK + 1;

#[cfg(feature = "visualize")]
const PATH_BUF_LEN: usize = MAX_STEP + 1;

const KERNEL_SOURCE: &str = include_str!("kernel.cu");

#[cfg(feature = "visualize")]
#[derive(Clone, Copy)]
#[repr(C)]
pub struct PathBuf {
	pub coord_list: [Coord; PATH_BUF_LEN],
	pub len: u32,
}

#[cfg(feature = "visualize")]
impl PathBuf {
	#[inline]
	pub fn as_slice(&self) -> &[Coord] {
		&self.coord_list[..self.len as usize]
	}
}

#[cfg(feature = "visualize")]
unsafe impl DeviceRepr for PathBuf {}
#[cfg(feature = "visualize")]
unsafe impl ValidAsZeroBits for PathBuf {}

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

pub struct PinnedBuf<T> {
	ptr: *mut T,
	len: usize,
	_ctx: Arc<CudaContext>,
}

unsafe impl<T: Send> Send for PinnedBuf<T> {}
unsafe impl<T: Sync> Sync for PinnedBuf<T> {}

impl<T: ValidAsZeroBits> PinnedBuf<T> {
	pub fn new(ctx: Arc<CudaContext>, len: usize) -> Result<Self, String> {
		ctx.bind_to_thread().map_err(|e| format!("bind ctx: {e}"))?;
		let bytes = len * std::mem::size_of::<T>();
		let ptr =
			unsafe { malloc_host(bytes, 0).map_err(|e| format!("malloc_host: {e}"))? } as *mut T;
		unsafe {
			std::ptr::write_bytes(ptr, 0, len);
		}
		Ok(Self {
			ptr,
			len,
			_ctx: ctx,
		})
	}
}

impl<T> PinnedBuf<T> {
	#[inline]
	pub fn as_slice(&self) -> &[T] {
		unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
	}

	#[inline]
	pub fn as_mut_slice(&mut self) -> &mut [T] {
		unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
	}
}

impl<T> std::ops::Deref for PinnedBuf<T> {
	type Target = [T];
	#[inline]
	fn deref(&self) -> &[T] {
		self.as_slice()
	}
}

impl<T> std::ops::DerefMut for PinnedBuf<T> {
	#[inline]
	fn deref_mut(&mut self) -> &mut [T] {
		self.as_mut_slice()
	}
}

impl<T> Drop for PinnedBuf<T> {
	fn drop(&mut self) {
		unsafe {
			let _ = free_host(self.ptr as *mut _);
		}
	}
}

pub struct GpuSim {
	#[allow(dead_code)]
	ctx: Arc<CudaContext>,
	stream: Arc<CudaStream>,
	#[allow(dead_code)]
	module: Arc<CudaModule>,
	func: CudaFunction,
	solution_list_buf: CudaSlice<Solution>,
	frozen_list_buf: CudaSlice<FrozenPrefix>,
	checkpoint_list_buf: CudaSlice<Coord>,
	output_list_buf: CudaSlice<SimOutput>,
	#[cfg(feature = "visualize")]
	path_list_buf: CudaSlice<PathBuf>,
	#[cfg(feature = "visualize")]
	path_list_host: Box<[PathBuf]>,
	checkpoint_count: i32,
	capacity: usize,
}

impl GpuSim {
	pub fn new(capacity: usize, checkpoint_list: &[Coord]) -> Result<Self, String> {
		let ctx = CudaContext::new(0).map_err(|e| format!("CudaContext::new: {e}"))?;
		let stream = ctx.default_stream();
		let opts = CompileOptions {
			arch: Some("compute_75"),
			maxrregcount: Some(64),
			#[cfg(feature = "visualize")]
			options: vec!["-DEMIT_PATHS".into()],
			..Default::default()
		};
		let ptx = compile_ptx_with_opts(KERNEL_SOURCE, opts)
			.map_err(|e| format!("nvrtc compile: {e}"))?;
		let module = ctx
			.load_module(ptx)
			.map_err(|e| format!("load_module: {e}"))?;
		let func = module
			.load_function("simulate")
			.map_err(|e| format!("load_function: {e}"))?;

		let solution_list_buf = stream
			.alloc_zeros::<Solution>(capacity)
			.map_err(|e| format!("alloc solution list: {e}"))?;
		let frozen_list_buf = stream
			.alloc_zeros::<FrozenPrefix>(capacity)
			.map_err(|e| format!("alloc frozen list: {e}"))?;
		let output_list_buf = stream
			.alloc_zeros::<SimOutput>(capacity)
			.map_err(|e| format!("alloc output list: {e}"))?;
		let checkpoint_list_buf = stream
			.clone_htod(checkpoint_list)
			.map_err(|e| format!("upload checkpoint list: {e}"))?;

		#[cfg(feature = "visualize")]
		let path_list_buf = stream
			.alloc_zeros::<PathBuf>(capacity)
			.map_err(|e| format!("alloc path list: {e}"))?;
		#[cfg(feature = "visualize")]
		let path_list_host: Box<[PathBuf]> = {
			let mut v: Vec<PathBuf> = Vec::with_capacity(capacity);
			for _ in 0..capacity {
				v.push(unsafe { std::mem::zeroed() });
			}
			v.into_boxed_slice()
		};

		Ok(Self {
			ctx,
			stream,
			module,
			func,
			solution_list_buf,
			frozen_list_buf,
			checkpoint_list_buf,
			output_list_buf,
			#[cfg(feature = "visualize")]
			path_list_buf,
			#[cfg(feature = "visualize")]
			path_list_host,
			checkpoint_count: checkpoint_list.len() as i32,
			capacity,
		})
	}

	#[cfg(feature = "visualize")]
	pub fn path_list(&self) -> &[PathBuf] {
		&self.path_list_host
	}

	pub fn alloc_pinned<T: ValidAsZeroBits>(&self, len: usize) -> Result<PinnedBuf<T>, String> {
		PinnedBuf::new(self.ctx.clone(), len)
	}

	pub fn submit_async(
		&mut self,
		solution_list: &[Solution],
		frozen_list: &[FrozenPrefix],
		car_init: &Car,
		step_to_checkpoint_limit: usize,
		output_list: &mut [SimOutput],
	) -> Result<(), String> {
		let n = solution_list.len();
		assert_eq!(n, frozen_list.len());
		assert_eq!(n, output_list.len());
		assert!(n <= self.capacity);

		self.stream
			.memcpy_htod(solution_list, &mut self.solution_list_buf)
			.map_err(|e| format!("upload solution list: {e}"))?;
		self.stream
			.memcpy_htod(frozen_list, &mut self.frozen_list_buf)
			.map_err(|e| format!("upload frozen list: {e}"))?;

		let n_i32: i32 = n as i32;
		let step_limit_i32: i32 = step_to_checkpoint_limit as i32;
		let car = *car_init;
		let cp_count = self.checkpoint_count;

		let mut builder = self.stream.launch_builder(&self.func);
		builder.arg(&self.solution_list_buf);
		builder.arg(&self.frozen_list_buf);
		builder.arg(&self.checkpoint_list_buf);
		builder.arg(&cp_count);
		builder.arg(&car);
		builder.arg(&step_limit_i32);
		builder.arg(&mut self.output_list_buf);
		builder.arg(&n_i32);
		#[cfg(feature = "visualize")]
		builder.arg(&mut self.path_list_buf);

		const BLOCK: u32 = 256;
		let n_u32 = n as u32;
		let cfg = LaunchConfig {
			grid_dim: (n_u32.div_ceil(BLOCK), 1, 1),
			block_dim: (BLOCK, 1, 1),
			shared_mem_bytes: 0,
		};
		unsafe { builder.launch(cfg) }.map_err(|e| format!("kernel launch: {e}"))?;

		self.stream
			.memcpy_dtoh(&self.output_list_buf, output_list)
			.map_err(|e| format!("download output list: {e}"))?;
		#[cfg(feature = "visualize")]
		self.stream
			.memcpy_dtoh(&self.path_list_buf, &mut self.path_list_host[..n])
			.map_err(|e| format!("download path list: {e}"))?;

		Ok(())
	}

	pub fn wait(&self) -> Result<(), String> {
		self.stream
			.synchronize()
			.map_err(|e| format!("stream sync: {e}"))
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
