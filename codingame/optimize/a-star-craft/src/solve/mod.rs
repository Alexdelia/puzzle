mod brute;
mod persist;
mod report;
mod search;
mod strategy;

pub use search::Knobs;
pub use strategy::Strategy;

use crate::parse::Config;
use crate::simulation::{Placement, Spot, build_next_table, placement};
use brute::BruteForce;
use search::Search;

pub fn solve(config: Config) -> Result<(), String> {
	let Config {
		name,
		engine,
		chain_count,
		duration,
		fresh,
		seed,
		phase_list,
		knobs,
		brute_limit,
	} = config;

	let next = build_next_table();
	let disk_best = persist::read_best_score(&name).unwrap_or(0);

	let Placement {
		spot_list,
		forced_list,
	} = placement(&engine.base, &next, &engine.robot_list);
	let exhaustive = exhaustive_total(&spot_list, brute_limit);

	report::announce(
		&name,
		engine.robot_count(),
		disk_best,
		&spot_list,
		&forced_list,
		exhaustive,
	);

	if let Some(total) = exhaustive {
		let mut brute = BruteForce::new(
			name.clone(),
			&engine,
			&next,
			chain_count,
			disk_best,
			spot_list,
			forced_list,
		)?;
		brute.run(total)?;
		let (best_score, best_game_length) = brute.report();
		persist::mark_complete(&name, best_score)?;
		println!("{name} best {best_score} game {best_game_length}");
		return Ok(());
	}

	let stored = if fresh {
		None
	} else {
		persist::read_stored_solution(&name, &engine.base)
	};

	let mut search = Search::new(
		name.clone(),
		&engine,
		&next,
		chain_count,
		seed,
		disk_best,
		knobs,
		Placement {
			spot_list,
			forced_list,
		},
	)?;

	eprint!("\n\n\n\n\n");

	search.init_chains(stored);
	search.seed_scores()?;

	let phase_budget = duration / phase_list.len() as u32;
	for strategy in &phase_list {
		search.run(strategy, phase_budget)?;
	}

	if let Some(strategy) = phase_list.last() {
		search.finish(strategy);
	}

	let (best_score, best_game_length) = search.report();
	println!("{name} best {best_score} game {best_game_length}");

	Ok(())
}

fn exhaustive_total(spot_list: &[Spot], limit: u64) -> Option<u64> {
	let mut total: u64 = 1;
	for spot in spot_list {
		let radix = spot.alive_count as u64 + spot.removable as u64;
		total = total.checked_mul(radix)?;
		if total > limit {
			return None;
		}
	}
	Some(total)
}
