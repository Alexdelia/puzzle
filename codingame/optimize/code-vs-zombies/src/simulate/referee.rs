use std::sync::Arc;

use cudarc::driver::{
	CudaContext, CudaFunction, CudaModule, CudaStream, DriverError, LaunchConfig, PushKernelArg,
};
use cudarc::nvrtc::{CompileOptions, compile_ptx_with_opts};

const KERNEL_SRC: &str = include_str!("./referee.cu");

type CudaBool = i32;

pub type Axis = u16;
pub type Coord = (Axis, Axis);

type FAxis = f64;
type FCoord = (FAxis, FAxis);

pub type Score = u32;
pub type Solution = Vec<Coord>;

#[derive(Clone, Debug)]
pub struct InitialState {
	pub player: Coord,
	pub human_list: Vec<Coord>,
	pub zombie_list: Vec<Coord>,
}

#[derive(Debug)]
pub struct DebugLog {
	pub simulation_count: usize,
	pub state_count: usize,
	pub nh: usize,
	pub nz: usize,
	pub player: Vec<f64>,
	pub zombie_list: Vec<f64>,
	pub zombie_alive_list: Vec<CudaBool>,
	pub human_alive_list: Vec<CudaBool>,
	pub score_per_turn: Vec<Score>,
}

impl DebugLog {
	pub fn player_pos(&self, turn: usize, sim: usize) -> FCoord {
		let i = (turn * self.simulation_count + sim) * 2;
		(self.player[i], self.player[i + 1])
	}
	pub fn zombie_pos(&self, turn: usize, sim: usize, z: usize) -> FCoord {
		let base = ((turn * self.simulation_count + sim) * self.nz + z) * 2;
		(self.zombie_list[base], self.zombie_list[base + 1])
	}
	pub fn is_zombie_alive(&self, turn: usize, sim: usize, z: usize) -> bool {
		let i = (turn * self.simulation_count + sim) * self.nz + z;
		self.zombie_alive_list[i] != 0
	}
	pub fn is_human_alive(&self, turn: usize, sim: usize, h: usize) -> bool {
		let i = (turn * self.simulation_count + sim) * self.nh + h;
		self.human_alive_list[i] != 0
	}
	pub fn turn_score(&self, turn: usize, sim: usize) -> Score {
		self.score_per_turn[turn * self.simulation_count + sim]
	}
}

pub struct Referee {
	#[allow(dead_code)]
	ctx: Arc<CudaContext>,
	stream: Arc<CudaStream>,

	#[allow(dead_code)]
	fast_module: Arc<CudaModule>,
	fast_func: CudaFunction,
	#[allow(dead_code)]
	debug_module: Arc<CudaModule>,
	debug_func: CudaFunction,

	turn_limit: usize,
	nh: usize,
	nz: usize,
}

impl Referee {
	pub fn new(initial: &InitialState, turn_limit: usize) -> Result<Self, DriverError> {
		let ctx = CudaContext::new(0)?;
		let stream = ctx.default_stream();

		let nh = initial.human_list.len();
		let nz = initial.zombie_list.len();
		assert!(nh >= 1 && nz >= 1, "need at least one human and one zombie");

		let common_define_list = vec![format!("-DNH={nh}"), format!("-DNZ={nz}")];

		let fast_compile_option = CompileOptions {
			options: common_define_list.clone(),
			prec_div: Some(true),
			prec_sqrt: Some(true),
			fmad: Some(false),
			ftz: Some(false),
			..Default::default()
		};
		let fast_ptx = compile_ptx_with_opts(KERNEL_SRC, fast_compile_option)
			.expect("fast kernel compile failure");
		let fast_module = ctx.load_module(fast_ptx)?;
		let fast_func = fast_module.load_function("simulate")?;

		let mut debug_define_list = common_define_list.clone();
		debug_define_list.push("-DOUTPUT_STATE".into());
		let debug_compile_option = CompileOptions {
			options: debug_define_list,
			prec_div: Some(true),
			prec_sqrt: Some(true),
			fmad: Some(false),
			ftz: Some(false),
			..Default::default()
		};
		let debug_ptx = compile_ptx_with_opts(KERNEL_SRC, debug_compile_option)
			.expect("debug kernel compile failure");
		let debug_module = ctx.load_module(debug_ptx)?;
		let debug_func = debug_module.load_function("simulate")?;

		upload_constants(&stream, &fast_module, initial)?;
		upload_constants(&stream, &debug_module, initial)?;

		Ok(Self {
			ctx,
			stream,
			fast_module,
			fast_func,
			debug_module,
			debug_func,
			turn_limit,
			nh,
			nz,
		})
	}

	pub fn nh(&self) -> usize {
		self.nh
	}
	pub fn nz(&self) -> usize {
		self.nz
	}
	pub fn turn_limit(&self) -> usize {
		self.turn_limit
	}

	pub fn run(&self, solution: &[Vec<Coord>]) -> Result<Vec<Score>, DriverError> {
		Ok(self.run_with_humans(solution)?.0)
	}

	pub fn run_with_humans(
		&self,
		solution: &[Vec<Coord>],
	) -> Result<(Vec<Score>, Vec<i32>), DriverError> {
		let simulation_count = solution.len();
		assert!(simulation_count > 0);

		let solution_flat = pack_solution(solution, self.turn_limit);
		let d_solution = self.stream.clone_htod(&solution_flat)?;
		let mut d_score_list = self.stream.alloc_zeros::<Score>(simulation_count)?;
		let mut d_alive_h = self.stream.alloc_zeros::<i32>(simulation_count)?;

		let cfg = launch_config(simulation_count as u32);
		let mut builder = self.stream.launch_builder(&self.fast_func);
		let simulation_count_i = simulation_count as i32;
		let turn_limit_i = self.turn_limit as i32;
		builder.arg(&simulation_count_i);
		builder.arg(&turn_limit_i);
		builder.arg(&d_solution);
		builder.arg(&mut d_score_list);
		builder.arg(&mut d_alive_h);
		unsafe { builder.launch(cfg)? };

		let score_list = self.stream.clone_dtoh(&d_score_list)?;
		let alive_h = self.stream.clone_dtoh(&d_alive_h)?;
		Ok((score_list, alive_h))
	}

	pub fn run_with_state(
		&self,
		solution: &[Vec<Coord>],
	) -> Result<(Vec<Score>, DebugLog), DriverError> {
		let simulation_count = solution.len();
		assert!(simulation_count > 0);
		let state_count = self.turn_limit + 1;

		let solution_flat = pack_solution(solution, self.turn_limit);
		let d_solution = self.stream.clone_htod(&solution_flat)?;
		let mut d_score_list = self.stream.alloc_zeros::<Score>(simulation_count)?;
		let mut d_alive_h = self.stream.alloc_zeros::<i32>(simulation_count)?;
		let mut d_player = self
			.stream
			.alloc_zeros::<f64>(state_count * simulation_count * 2)?;
		let mut d_zombie_list = self
			.stream
			.alloc_zeros::<f64>(state_count * simulation_count * self.nz * 2)?;
		let mut d_z_alive = self
			.stream
			.alloc_zeros::<CudaBool>(state_count * simulation_count * self.nz)?;
		let mut d_h_alive = self
			.stream
			.alloc_zeros::<CudaBool>(state_count * simulation_count * self.nh)?;
		let mut d_score_per_turn = self
			.stream
			.alloc_zeros::<Score>(state_count * simulation_count)?;

		let cfg = launch_config(simulation_count as u32);
		let mut builder = self.stream.launch_builder(&self.debug_func);
		let simulation_count_i = simulation_count as i32;
		let turn_limit_i = self.turn_limit as i32;
		builder.arg(&simulation_count_i);
		builder.arg(&turn_limit_i);
		builder.arg(&d_solution);
		builder.arg(&mut d_score_list);
		builder.arg(&mut d_alive_h);
		builder.arg(&mut d_player);
		builder.arg(&mut d_zombie_list);
		builder.arg(&mut d_z_alive);
		builder.arg(&mut d_h_alive);
		builder.arg(&mut d_score_per_turn);
		unsafe { builder.launch(cfg)? };

		let score_list = self.stream.clone_dtoh(&d_score_list)?;
		let player = self.stream.clone_dtoh(&d_player)?;
		let zombie_list = self.stream.clone_dtoh(&d_zombie_list)?;
		let zombie_alive_list = self.stream.clone_dtoh(&d_z_alive)?;
		let human_alive_list = self.stream.clone_dtoh(&d_h_alive)?;
		let score_per_turn = self.stream.clone_dtoh(&d_score_per_turn)?;

		let log = DebugLog {
			simulation_count,
			state_count,
			nh: self.nh,
			nz: self.nz,
			player,
			zombie_list,
			zombie_alive_list,
			human_alive_list,
			score_per_turn,
		};
		Ok((score_list, log))
	}
}

fn launch_config(simulation_count: u32) -> LaunchConfig {
	const BLOCK: u32 = 64;
	let block_count = simulation_count.div_ceil(BLOCK);
	LaunchConfig {
		grid_dim: (block_count, 1, 1),
		block_dim: (BLOCK, 1, 1),
		shared_mem_bytes: 0,
	}
}

fn pack_solution(solution: &[Vec<Coord>], turn_limit: usize) -> Vec<f64> {
	let simulation_count = solution.len();
	let mut out = vec![0.0f64; turn_limit * simulation_count * 2];
	for (sim_index, sim_solution) in solution.iter().enumerate() {
		for (turn_index, &(x, y)) in sim_solution.iter().enumerate() {
			if turn_index >= turn_limit {
				break;
			}
			let i = (turn_index * simulation_count + sim_index) * 2;
			out[i] = x as f64;
			out[i + 1] = y as f64;
		}
	}
	out
}

fn upload_constants(
	stream: &Arc<CudaStream>,
	module: &Arc<CudaModule>,
	init: &InitialState,
) -> Result<(), DriverError> {
	let player = [init.player.0 as f64, init.player.1 as f64];
	let human_list: Vec<f64> = init
		.human_list
		.iter()
		.flat_map(|&(x, y)| [x as f64, y as f64])
		.collect();
	let zombie_list: Vec<f64> = init
		.zombie_list
		.iter()
		.flat_map(|&(x, y)| [x as f64, y as f64])
		.collect();

	{
		let mut sym = module.get_global("c_player_init", stream)?;
		let mut view = unsafe { sym.transmute_mut::<f64>(2).unwrap() };
		stream.memcpy_htod(&player, &mut view)?;
	}
	{
		let mut sym = module.get_global("c_human_list_init", stream)?;
		let mut view = unsafe { sym.transmute_mut::<f64>(human_list.len()).unwrap() };
		stream.memcpy_htod(human_list.as_slice(), &mut view)?;
	}
	{
		let mut sym = module.get_global("c_zombie_list_init", stream)?;
		let mut view = unsafe { sym.transmute_mut::<f64>(zombie_list.len()).unwrap() };
		stream.memcpy_htod(zombie_list.as_slice(), &mut view)?;
	}

	Ok(())
}
