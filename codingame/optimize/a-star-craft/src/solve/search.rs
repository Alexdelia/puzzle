use std::time::{Duration, Instant};

use rand::{RngExt, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use super::persist;
use super::strategy::{ChainMeta, Plan, Strategy};
use crate::simulation::{
	Cell, Engine, GpuSim, MAP_AREA, NONE, PinnedBuf, Score, SimOutput, Solution, Spot, Tile, Turn,
	placeable_spot_list,
};

const MAX_MOVE: usize = 16;
const LOG_PERIOD: u64 = 512;

#[derive(Clone, Copy)]
pub struct Knobs {
	pub refocus_period: u64,
	pub refocus_fraction: f32,
	pub refocus_kick_min: usize,
	pub refocus_kick_max: usize,
}

impl Default for Knobs {
	fn default() -> Self {
		Self {
			refocus_period: 256,
			refocus_fraction: 0.6,
			refocus_kick_min: 1,
			refocus_kick_max: 4,
		}
	}
}

#[derive(Clone, Copy)]
struct MovePending {
	cell: [Cell; MAX_MOVE],
	old: [Tile; MAX_MOVE],
	len: u8,
	force_accept: bool,
}

const EMPTY_PENDING: MovePending = MovePending {
	cell: [0; MAX_MOVE],
	old: [NONE; MAX_MOVE],
	len: 0,
	force_accept: false,
};

pub struct Search {
	gpu: GpuSim,
	spot_list: Vec<Spot>,
	rng: Xoshiro256PlusPlus,
	chain_count: usize,
	knobs: Knobs,
	base: [Tile; MAP_AREA],
	name: String,

	current: PinnedBuf<Solution>,
	output_list: PinnedBuf<SimOutput>,
	current_score: Vec<Score>,
	meta: Vec<ChainMeta>,
	pending: Vec<MovePending>,

	best: Solution,
	best_score: Score,
	best_game_length: Turn,
	disk_best: Score,

	round: u64,
	start_time: Instant,
	last_log_time: Instant,
	last_log_round: u64,
	refocus_count: u64,
}

fn pick_change(candidate: &[Tile], previous: Tile, rng: &mut Xoshiro256PlusPlus) -> Tile {
	match candidate.iter().position(|&tile| tile == previous) {
		Some(skip) => {
			let roll = rng.random_range(0..candidate.len() - 1);
			candidate[if roll < skip { roll } else { roll + 1 }]
		}
		None => candidate[rng.random_range(0..candidate.len())],
	}
}

impl Search {
	pub fn new(
		name: String,
		engine: &Engine,
		next: &[Cell],
		chain_count: usize,
		seed: u64,
		disk_best: Score,
		knobs: Knobs,
	) -> Result<Self, String> {
		let spot_list = placeable_spot_list(&engine.base, next, &engine.robot_list);

		let gpu = GpuSim::new(chain_count, engine, next)?;
		let current = gpu.alloc_pinned::<Solution>(chain_count)?;
		let output_list = gpu.alloc_pinned::<SimOutput>(chain_count)?;

		let now = Instant::now();
		Ok(Self {
			gpu,
			spot_list,
			rng: Xoshiro256PlusPlus::seed_from_u64(seed),
			chain_count,
			knobs,
			base: engine.base,
			name,
			current,
			output_list,
			current_score: vec![0; chain_count],
			meta: vec![
				ChainMeta {
					temperature: 1.0,
					stagnant: 0,
				};
				chain_count
			],
			pending: vec![EMPTY_PENDING; chain_count],
			best: Solution::empty(),
			best_score: 0,
			best_game_length: 0,
			disk_best,
			round: 0,
			start_time: now,
			last_log_time: now,
			last_log_round: 0,
			refocus_count: 0,
		})
	}

	pub fn placeable_count(&self) -> usize {
		self.spot_list.len()
	}

	pub fn init_chains(&mut self, stored: Option<Solution>) {
		for i in 0..self.chain_count {
			self.current[i].arrow = [NONE; MAP_AREA];
			let density: f32 = self.rng.random();
			for index in 0..self.spot_list.len() {
				if self.rng.random::<f32>() < density {
					let spot = self.spot_list[index];
					let pick = self.rng.random_range(0..spot.alive_count as usize);
					self.current[i].arrow[spot.cell as usize] = spot.candidate[pick];
				}
			}
		}
		if let Some(solution) = stored {
			self.current[0] = solution;
		}
	}

	pub fn seed_scores(&mut self) -> Result<(), String> {
		self.start_time = Instant::now();
		self.last_log_time = self.start_time;
		self.evaluate()?;
		for i in 0..self.chain_count {
			let score = self.output_list[i].score;
			self.current_score[i] = score;
			if score > self.best_score {
				self.best = self.current[i];
				self.best_score = score;
				self.best_game_length = self.output_list[i].game_length;
			}
		}
		self.record_best();
		Ok(())
	}

	pub fn run(&mut self, strategy: &Strategy, budget: Duration) -> Result<(), String> {
		for slot in &mut self.meta {
			*slot = strategy.init_meta();
		}
		let phase_start = Instant::now();
		loop {
			let refocus = self.knobs.refocus_period > 0
				&& self.round > 0
				&& self.round.is_multiple_of(self.knobs.refocus_period);
			self.propose(strategy, refocus);
			self.evaluate()?;
			self.accept(strategy);
			self.record_best();

			self.round += 1;
			if self.round.is_multiple_of(LOG_PERIOD) {
				self.log(strategy);
			}
			if phase_start.elapsed() >= budget {
				break;
			}
		}
		Ok(())
	}

	fn evaluate(&mut self) -> Result<(), String> {
		let Self {
			gpu,
			current,
			output_list,
			..
		} = self;
		gpu.submit_async(current.as_slice(), output_list.as_mut_slice())?;
		gpu.wait()
	}

	fn propose(&mut self, strategy: &Strategy, refocus: bool) {
		let refocus_threshold = (self.best_score as f32 * self.knobs.refocus_fraction) as Score;
		let spot_count = self.spot_list.len();

		for i in 0..self.chain_count {
			let reset = refocus && self.current_score[i] < refocus_threshold;
			let plan = if reset {
				self.current[i].arrow = self.best.arrow;
				self.meta[i] = strategy.init_meta();
				self.refocus_count += 1;
				Plan {
					move_size: self
						.rng
						.random_range(self.knobs.refocus_kick_min..=self.knobs.refocus_kick_max),
					force_accept: true,
				}
			} else {
				strategy.plan(&mut self.meta[i], &mut self.rng)
			};

			let move_size = plan.move_size.min(MAX_MOVE);
			self.pending[i].len = move_size as u8;
			self.pending[i].force_accept = plan.force_accept;
			for m in 0..move_size {
				let spot = self.spot_list[self.rng.random_range(0..spot_count)];
				let previous = self.current[i].arrow[spot.cell as usize];
				let value = pick_change(
					&spot.candidate[..=spot.alive_count as usize],
					previous,
					&mut self.rng,
				);
				self.pending[i].cell[m] = spot.cell;
				self.pending[i].old[m] = previous;
				self.current[i].arrow[spot.cell as usize] = value;
			}
		}
	}

	fn accept(&mut self, strategy: &Strategy) {
		for i in 0..self.chain_count {
			let new = self.output_list[i].score;
			let cur = self.current_score[i];
			let delta = new as i32 - cur as i32;
			let accepted = self.pending[i].force_accept
				|| strategy.accept(delta, &self.meta[i], &mut self.rng);
			let improved = accepted && new > cur;

			if accepted {
				self.current_score[i] = new;
				if new > self.best_score {
					self.best = self.current[i];
					self.best_score = new;
					self.best_game_length = self.output_list[i].game_length;
				}
			} else {
				let len = self.pending[i].len as usize;
				for m in (0..len).rev() {
					let cell = self.pending[i].cell[m] as usize;
					self.current[i].arrow[cell] = self.pending[i].old[m];
				}
			}
			strategy.after_round(&mut self.meta[i], improved);
		}
	}

	fn record_best(&mut self) {
		if self.best_score > self.disk_best {
			if let Err(e) = persist::write_best(&self.name, &self.best, &self.base, self.best_score)
			{
				eprintln!("write best: {e}");
			}
			self.disk_best = self.best_score;
		}
	}

	pub fn report(&self) -> (Score, Turn) {
		(self.best_score, self.best_game_length)
	}

	fn log(&mut self, strategy: &Strategy) {
		let now = Instant::now();
		let since = (now - self.last_log_time).as_secs_f64().max(1e-9);
		let rounds_since = (self.round - self.last_log_round) as f64;
		let elapsed = (now - self.start_time).as_secs();

		let mut sum: u64 = 0;
		let mut population_max: Score = 0;
		for &score in &self.current_score {
			sum += score as u64;
			if score > population_max {
				population_max = score;
			}
		}
		let mean = sum as f64 / self.chain_count as f64;

		let rounds_per_sec = rounds_since / since;
		let moves_per_sec = rounds_per_sec * self.chain_count as f64;
		let nanos_per_eval = since * 1e9 / (rounds_since * self.chain_count as f64);

		let elapsed_str = if elapsed >= 60 {
			format!(
				"\x1b[0;34m{}\x1b[2mm \x1b[0;36m{:>2}\x1b[2ms",
				elapsed / 60,
				elapsed % 60
			)
		} else {
			format!("\x1b[0;36m{elapsed}\x1b[2ms")
		};

		eprint!(
			"\r\x1b[4A\x1b[1;32m{best:>5}\x1b[0m \x1b[2mgame\x1b[0m {game:>3} \x1b[2mdisk\x1b[0m {disk:<5} \x1b[0;35m{strategy}\x1b[0m\x1b[K
\x1b[2mpop\x1b[0m \x1b[0;36m{mean:>7.1}\x1b[2m mean\x1b[0m \x1b[0;36m{population_max:>5}\x1b[2m max\x1b[0m \x1b[0;33m{refocus}\x1b[2m refocus\x1b[0m\x1b[K
\x1b[0;38;2;52;235;198m{nanos_per_eval:>6.1}\x1b[2mns/eval \x1b[0;96m{moves_per_sec:>8.0}\x1b[2m mv/s \x1b[0;94m{rounds_per_sec:>6.0}\x1b[2m rnd/s\x1b[0m\x1b[K
{elapsed_str}\x1b[K
\x1b[0;1m{round}\x1b[0m \x1b[2mrounds\x1b[0m\x1b[K",
			best = self.best_score,
			game = self.best_game_length,
			disk = self.disk_best,
			strategy = strategy.name(),
			refocus = self.refocus_count,
			round = self.round,
		);

		self.last_log_time = now;
		self.last_log_round = self.round;
	}

	pub fn finish(&mut self, strategy: &Strategy) {
		self.log(strategy);
		eprintln!();
	}
}
