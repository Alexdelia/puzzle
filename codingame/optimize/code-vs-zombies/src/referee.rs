use std::sync::Arc;

use cudarc::driver::{
	CudaContext, CudaFunction, CudaModule, CudaStream, DriverError, LaunchConfig, PushKernelArg,
};
use cudarc::nvrtc::{CompileOptions, compile_ptx_with_opts};

const KERNEL_SRC: &str = include_str!("./referee.cu");

#[derive(Clone, Debug)]
pub struct InitialState {
	pub player: (i32, i32),
	pub human_list: Vec<(i32, i32)>,
	pub zombie_list: Vec<(i32, i32)>,
}

#[derive(Debug)]
pub struct DebugLog {
	pub simulation_count: usize,
	pub num_states: usize,
	pub nh: usize,
	pub nz: usize,
	pub player: Vec<f64>,
	pub zombie_list: Vec<f64>,
	pub zombie_alive_list: Vec<i32>,
	pub human_alive_list: Vec<i32>,
	pub score_per_turn: Vec<i64>,
}

impl DebugLog {
	pub fn player_pos(&self, turn: usize, sim: usize) -> (f64, f64) {
		let i = (turn * self.simulation_count + sim) * 2;
		(self.player[i], self.player[i + 1])
	}
	pub fn zombie_pos(&self, turn: usize, sim: usize, z: usize) -> (f64, f64) {
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
	pub fn turn_score(&self, turn: usize, sim: usize) -> i64 {
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

	max_turns: usize,
	nh: usize,
	nz: usize,
}

impl Referee {
	pub fn new(initial: &InitialState, max_turns: usize) -> Result<Self, DriverError> {
		let ctx = CudaContext::new(0)?;
		let stream = ctx.default_stream();

		let nh = initial.human_list.len();
		let nz = initial.zombie_list.len();
		assert!(nh >= 1 && nz >= 1, "need at least one human and one zombie");

		let common_defs = vec![format!("-DNH={nh}"), format!("-DNZ={nz}")];

		let fast_opts = CompileOptions {
			options: common_defs.clone(),
			prec_div: Some(true),
			prec_sqrt: Some(true),
			fmad: Some(false),
			ftz: Some(false),
			..Default::default()
		};
		let fast_ptx =
			compile_ptx_with_opts(KERNEL_SRC, fast_opts).expect("fast kernel compile failure");
		let fast_module = ctx.load_module(fast_ptx)?;
		let fast_func = fast_module.load_function("simulate")?;

		let mut debug_options = common_defs.clone();
		debug_options.push("-DOUTPUT_STATE".into());
		let debug_opts = CompileOptions {
			options: debug_options,
			prec_div: Some(true),
			prec_sqrt: Some(true),
			fmad: Some(false),
			ftz: Some(false),
			..Default::default()
		};
		let debug_ptx =
			compile_ptx_with_opts(KERNEL_SRC, debug_opts).expect("debug kernel compile failure");
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
			max_turns,
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
	pub fn max_turns(&self) -> usize {
		self.max_turns
	}

	pub fn run(&self, solution: &[Vec<(i32, i32)>]) -> Result<Vec<i64>, DriverError> {
		let simulation_count = solution.len();
		assert!(simulation_count > 0);

		let solution_flat = pack_solution(solution, self.max_turns);
		let d_solution = self.stream.clone_htod(&solution_flat)?;
		let mut d_scores = self.stream.alloc_zeros::<i64>(simulation_count)?;

		let cfg = launch_config(simulation_count as u32);
		let mut builder = self.stream.launch_builder(&self.fast_func);
		let simulation_count_i = simulation_count as i32;
		let max_turns_i = self.max_turns as i32;
		builder.arg(&simulation_count_i);
		builder.arg(&max_turns_i);
		builder.arg(&d_solution);
		builder.arg(&mut d_scores);
		unsafe { builder.launch(cfg)? };

		let scores = self.stream.clone_dtoh(&d_scores)?;
		Ok(scores)
	}

	pub fn run_with_state(
		&self,
		solution: &[Vec<(i32, i32)>],
	) -> Result<(Vec<i64>, DebugLog), DriverError> {
		let simulation_count = solution.len();
		assert!(simulation_count > 0);
		let num_states = self.max_turns + 1;

		let solution_flat = pack_solution(solution, self.max_turns);
		let d_solution = self.stream.clone_htod(&solution_flat)?;
		let mut d_scores = self.stream.alloc_zeros::<i64>(simulation_count)?;
		let mut d_player = self
			.stream
			.alloc_zeros::<f64>(num_states * simulation_count * 2)?;
		let mut d_zombie_list = self
			.stream
			.alloc_zeros::<f64>(num_states * simulation_count * self.nz * 2)?;
		let mut d_z_alive = self
			.stream
			.alloc_zeros::<i32>(num_states * simulation_count * self.nz)?;
		let mut d_h_alive = self
			.stream
			.alloc_zeros::<i32>(num_states * simulation_count * self.nh)?;
		let mut d_score_per_turn = self
			.stream
			.alloc_zeros::<i64>(num_states * simulation_count)?;

		let cfg = launch_config(simulation_count as u32);
		let mut builder = self.stream.launch_builder(&self.debug_func);
		let simulation_count_i = simulation_count as i32;
		let max_turns_i = self.max_turns as i32;
		builder.arg(&simulation_count_i);
		builder.arg(&max_turns_i);
		builder.arg(&d_solution);
		builder.arg(&mut d_scores);
		builder.arg(&mut d_player);
		builder.arg(&mut d_zombie_list);
		builder.arg(&mut d_z_alive);
		builder.arg(&mut d_h_alive);
		builder.arg(&mut d_score_per_turn);
		unsafe { builder.launch(cfg)? };

		let scores = self.stream.clone_dtoh(&d_scores)?;
		let player = self.stream.clone_dtoh(&d_player)?;
		let zombie_list = self.stream.clone_dtoh(&d_zombie_list)?;
		let zombie_alive_list = self.stream.clone_dtoh(&d_z_alive)?;
		let human_alive_list = self.stream.clone_dtoh(&d_h_alive)?;
		let score_per_turn = self.stream.clone_dtoh(&d_score_per_turn)?;

		let log = DebugLog {
			simulation_count,
			num_states,
			nh: self.nh,
			nz: self.nz,
			player,
			zombie_list,
			zombie_alive_list,
			human_alive_list,
			score_per_turn,
		};
		Ok((scores, log))
	}
}

fn launch_config(simulation_count: u32) -> LaunchConfig {
	const BLOCK: u32 = 64;
	let num_blocks = simulation_count.div_ceil(BLOCK);
	LaunchConfig {
		grid_dim: (num_blocks, 1, 1),
		block_dim: (BLOCK, 1, 1),
		shared_mem_bytes: 0,
	}
}

fn pack_solution(solution: &[Vec<(i32, i32)>], max_turns: usize) -> Vec<f64> {
	let simulation_count = solution.len();
	let mut out = vec![0.0f64; max_turns * simulation_count * 2];
	for (sim_index, sim_solution) in solution.iter().enumerate() {
		for (turn_index, &(x, y)) in sim_solution.iter().enumerate() {
			if turn_index >= max_turns {
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
