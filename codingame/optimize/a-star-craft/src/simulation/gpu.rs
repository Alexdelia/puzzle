use std::sync::Arc;

use cudarc::driver::{
	CudaContext, CudaFunction, CudaModule, CudaSlice, CudaStream, DeviceRepr, LaunchConfig,
	PushKernelArg, ValidAsZeroBits,
	result::{free_host, malloc_host},
};
use cudarc::nvrtc::{CompileOptions, compile_ptx_with_opts};

use super::{Cell, Engine, Robot, RobotCount, SimOutput, Solution, SolutionCount, Tile};

const KERNEL_SOURCE: &str = include_str!("kernel.cu");
const BLOCK: u32 = 256;

unsafe impl DeviceRepr for Robot {}
unsafe impl ValidAsZeroBits for Robot {}
unsafe impl DeviceRepr for Solution {}
unsafe impl ValidAsZeroBits for Solution {}
unsafe impl DeviceRepr for SimOutput {}
unsafe impl ValidAsZeroBits for SimOutput {}

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
	ctx: Arc<CudaContext>,
	stream: Arc<CudaStream>,
	#[allow(dead_code)]
	module: Arc<CudaModule>,
	func: CudaFunction,
	base_buf: CudaSlice<Tile>,
	next_buf: CudaSlice<Cell>,
	robot_buf: CudaSlice<Robot>,
	generation_buf: CudaSlice<Solution>,
	output_buf: CudaSlice<SimOutput>,
	robot_count: RobotCount,
	capacity: usize,
}

impl GpuSim {
	pub fn new(capacity: usize, engine: &Engine, next: &[Cell]) -> Result<Self, String> {
		let ctx = CudaContext::new(0).map_err(|e| format!("CudaContext::new: {e}"))?;
		let stream = ctx.default_stream();

		let opts = CompileOptions {
			arch: Some("compute_75"),
			maxrregcount: Some(64),
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

		let base_buf = stream
			.clone_htod(&engine.base[..])
			.map_err(|e| format!("upload base grid: {e}"))?;
		let next_buf = stream
			.clone_htod(next)
			.map_err(|e| format!("upload next table: {e}"))?;
		let robot_buf = stream
			.clone_htod(&engine.robot_list)
			.map_err(|e| format!("upload robot list: {e}"))?;
		let generation_buf = stream
			.alloc_zeros::<Solution>(capacity)
			.map_err(|e| format!("alloc generation buffer: {e}"))?;
		let output_buf = stream
			.alloc_zeros::<SimOutput>(capacity)
			.map_err(|e| format!("alloc output buffer: {e}"))?;

		Ok(Self {
			ctx,
			stream,
			module,
			func,
			base_buf,
			next_buf,
			robot_buf,
			generation_buf,
			output_buf,
			robot_count: engine.robot_count(),
			capacity,
		})
	}

	pub fn alloc_pinned<T: ValidAsZeroBits>(&self, len: usize) -> Result<PinnedBuf<T>, String> {
		PinnedBuf::new(self.ctx.clone(), len)
	}

	pub fn submit_async(
		&mut self,
		generation: &[Solution],
		output_list: &mut [SimOutput],
	) -> Result<(), String> {
		let count = generation.len();
		assert_eq!(count, output_list.len());
		assert!(count <= self.capacity);

		self.stream
			.memcpy_htod(generation, &mut self.generation_buf)
			.map_err(|e| format!("upload generation: {e}"))?;

		let n: SolutionCount = count as SolutionCount;
		let robot_count = self.robot_count;

		let mut builder = self.stream.launch_builder(&self.func);
		builder.arg(&self.base_buf);
		builder.arg(&self.next_buf);
		builder.arg(&self.robot_buf);
		builder.arg(&robot_count);
		builder.arg(&self.generation_buf);
		builder.arg(&mut self.output_buf);
		builder.arg(&n);

		let cfg = LaunchConfig {
			grid_dim: (n.div_ceil(BLOCK), 1, 1),
			block_dim: (BLOCK, 1, 1),
			shared_mem_bytes: 0,
		};
		unsafe { builder.launch(cfg) }.map_err(|e| format!("kernel launch: {e}"))?;

		self.stream
			.memcpy_dtoh(&self.output_buf, output_list)
			.map_err(|e| format!("download output: {e}"))?;

		Ok(())
	}

	pub fn wait(&self) -> Result<(), String> {
		self.stream
			.synchronize()
			.map_err(|e| format!("stream sync: {e}"))
	}
}
