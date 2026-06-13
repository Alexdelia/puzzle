use std::time::Duration;

use crate::{Coord, Score};

#[derive(Debug, Clone)]
pub struct SearchConfig {
	pub seed: u64,
	pub population: usize,
	pub elite: usize,
	pub turn_limit: usize,
	pub time_limit: Duration,
	pub human_weight: i64,
}

impl Default for SearchConfig {
	fn default() -> Self {
		Self {
			seed: 0xC0DEC0DE,
			population: 2 << 13,
			elite: 64,
			turn_limit: 64,
			time_limit: Duration::from_mins(1),
			human_weight: 0,
		}
	}
}

pub struct SearchState {
	pub population: Vec<Vec<Coord>>,
	pub best_score: Score,
	pub best_solution: Vec<Coord>,
	pub generation: u64,
	pub last_improvement_gen: u64,
}
