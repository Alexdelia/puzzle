use std::time::{Duration, Instant};

use super::persist;
use super::report;
use crate::simulation::{
	Cell, Engine, ForcedArrow, GpuSim, MAP_AREA, NONE, PinnedBuf, Score, SimOutput, Solution, Spot,
	Tile, Turn,
};

const LOG_PERIOD: Duration = Duration::from_millis(250);

pub struct BruteForce {
	gpu: GpuSim,
	generation: PinnedBuf<Solution>,
	output_list: PinnedBuf<SimOutput>,
	batch_capacity: usize,
	spot_list: Vec<Spot>,
	forced_list: Vec<ForcedArrow>,
	base: [Tile; MAP_AREA],
	name: String,
	best: Solution,
	best_score: Score,
	best_game_length: Turn,
	disk_best: Score,
}

fn advance(digit: &mut [u8], arrow: &mut [Tile; MAP_AREA], spot_list: &[Spot]) -> bool {
	for (k, spot) in spot_list.iter().enumerate() {
		let radix = spot.alive_count + spot.removable as u8;
		digit[k] += 1;
		if digit[k] < radix {
			arrow[spot.cell as usize] = spot.candidate[digit[k] as usize];
			return true;
		}
		digit[k] = 0;
		arrow[spot.cell as usize] = spot.candidate[0];
	}
	false
}

impl BruteForce {
	pub fn new(
		name: String,
		engine: &Engine,
		next: &[Cell],
		batch_capacity: usize,
		disk_best: Score,
		spot_list: Vec<Spot>,
		forced_list: Vec<ForcedArrow>,
	) -> Result<Self, String> {
		let gpu = GpuSim::new(batch_capacity, engine, next)?;
		let generation = gpu.alloc_pinned::<Solution>(batch_capacity)?;
		let output_list = gpu.alloc_pinned::<SimOutput>(batch_capacity)?;
		Ok(Self {
			gpu,
			generation,
			output_list,
			batch_capacity,
			spot_list,
			forced_list,
			base: engine.base,
			name,
			best: Solution::empty(),
			best_score: 0,
			best_game_length: 0,
			disk_best,
		})
	}

	pub fn run(&mut self, total: u64) -> Result<(), String> {
		let mut digit = vec![0u8; self.spot_list.len()];
		let mut arrow = [NONE; MAP_AREA];
		for forced in &self.forced_list {
			arrow[forced.cell as usize] = forced.direction;
		}
		for spot in &self.spot_list {
			arrow[spot.cell as usize] = spot.candidate[0];
		}

		let start = Instant::now();
		let mut last_log = start;
		let mut done: u64 = 0;
		let mut exhausted = false;

		while !exhausted {
			let mut batch = 0;
			while batch < self.batch_capacity && !exhausted {
				self.generation[batch].arrow = arrow;
				batch += 1;
				exhausted = !advance(&mut digit, &mut arrow, &self.spot_list);
			}

			let Self {
				gpu,
				generation,
				output_list,
				..
			} = self;
			gpu.submit_async(generation.as_slice(), output_list.as_mut_slice())?;
			gpu.wait()?;

			for i in 0..batch {
				let output = self.output_list[i];
				if output.score > self.best_score {
					self.best = self.generation[i];
					self.best_score = output.score;
					self.best_game_length = output.game_length;
					self.record_best();
				}
			}

			done += batch as u64;
			let now = Instant::now();
			if now.duration_since(last_log) >= LOG_PERIOD || exhausted {
				report::brute_progress(
					done,
					total,
					now.duration_since(start).as_secs_f64(),
					self.best_score,
					self.best_game_length,
				);
				last_log = now;
			}
		}

		report::brute_finish();
		Ok(())
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
}
