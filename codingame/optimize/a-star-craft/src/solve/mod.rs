mod persist;
mod report;
mod search;
mod strategy;

pub use search::Knobs;
pub use strategy::Strategy;

use crate::parse::Config;
use crate::simulation::build_next_table;
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
	} = config;

	let next = build_next_table();
	let disk_best = persist::read_best_score(&name).unwrap_or(0);
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
	)?;

	report::announce(
		&name,
		engine.robot_count(),
		disk_best,
		search.spot_list(),
		search.forced_list(),
	);

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
