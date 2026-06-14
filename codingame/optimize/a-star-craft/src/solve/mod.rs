use std::time::Instant;

use rand::{RngExt, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::parse::Config;
use crate::simulation::{
	Cell, GpuSim, MAP_AREA, NONE, PinnedBuf, Score, SimOutput, Solution, Tile, build_next_table,
	simulate_batch,
};

pub fn solve(config: Config) -> Result<(), String> {
	let Config {
		name,
		engine,
		batch,
	} = config;
	let next = build_next_table();

	println!("map {name}: {} robots", engine.robot_count());

	let mut gpu = GpuSim::new(batch, &engine, &next)?;
	let mut generation: PinnedBuf<Solution> = gpu.alloc_pinned(batch)?;
	let mut output_list: PinnedBuf<SimOutput> = gpu.alloc_pinned(batch)?;

	let placeable_cell_list: Vec<Cell> = (0..MAP_AREA)
		.filter(|&cell| engine.base[cell] == NONE)
		.map(|cell| cell as Cell)
		.collect();

	let mut rng = Xoshiro256PlusPlus::seed_from_u64(0xA5A5_1234_DEAD_BEEF);
	for solution in generation.as_mut_slice() {
		solution.arrow = [NONE; MAP_AREA];
		for &cell in &placeable_cell_list {
			let pick = rng.random_range(0..8u8);
			solution.arrow[cell as usize] = if pick < 4 { pick as Tile } else { NONE };
		}
	}

	gpu.submit_async(&generation, &mut output_list)?;
	gpu.wait()?;

	let gpu_start = Instant::now();
	gpu.submit_async(&generation, &mut output_list)?;
	gpu.wait()?;
	let gpu_elapsed = gpu_start.elapsed();

	let mut best_score: Score = 0;
	let mut best_index = 0usize;
	for (index, output) in output_list.iter().enumerate() {
		if output.score > best_score {
			best_score = output.score;
			best_index = index;
		}
	}

	println!(
		"gpu : {batch} solutions in {:.3} ms ({:.1} ns/solution)",
		gpu_elapsed.as_secs_f64() * 1e3,
		gpu_elapsed.as_nanos() as f64 / batch as f64
	);
	println!(
		"best: score {best_score} (#{best_index}, game length {})",
		output_list[best_index].game_length
	);

	let mut cpu_output_list = vec![SimOutput::default(); batch];
	let cpu_start = Instant::now();
	simulate_batch(&engine, &next, generation.as_slice(), &mut cpu_output_list);
	let cpu_elapsed = cpu_start.elapsed();

	println!(
		"cpu : {batch} solutions in {:.3} ms ({:.1} ns/solution), speedup {:.0}x",
		cpu_elapsed.as_secs_f64() * 1e3,
		cpu_elapsed.as_nanos() as f64 / batch as f64,
		cpu_elapsed.as_secs_f64() / gpu_elapsed.as_secs_f64()
	);

	let mut mismatch_count = 0;
	for index in 0..batch {
		if cpu_output_list[index] != output_list[index] {
			if mismatch_count < 5 {
				eprintln!(
					"mismatch #{index}: cpu {:?} gpu {:?}",
					cpu_output_list[index], output_list[index]
				);
			}
			mismatch_count += 1;
		}
	}

	if mismatch_count == 0 {
		println!("cross-check: {batch}/{batch} solutions match");
		Ok(())
	} else {
		Err(format!("cross-check failed: {mismatch_count} mismatches"))
	}
}
