use std::time::Duration;

use crate::solve::{Knobs, Strategy};

const DEFAULT_DURATION: Duration = Duration::from_secs(30);
const DEFAULT_BRUTE_LIMIT: u64 = 2 << 31;

macro_rules! env_or {
	($name:expr, $fallback:expr) => {
		std::env::var($name)
			.ok()
			.and_then(|value| value.parse().ok())
			.unwrap_or($fallback)
	};
}

pub fn duration() -> Duration {
	std::env::var("DURATION")
		.ok()
		.and_then(|value| value.parse::<f64>().ok())
		.map(Duration::from_secs_f64)
		.unwrap_or(DEFAULT_DURATION)
}

pub fn brute_limit() -> u64 {
	env_or!("BRUTE_LIMIT", DEFAULT_BRUTE_LIMIT)
}

pub fn fresh() -> bool {
	matches!(std::env::var("FRESH").as_deref(), Ok("1" | "true" | "yes"))
}

pub fn seed() -> u64 {
	std::env::var("SEED")
		.ok()
		.and_then(|value| value.parse().ok())
		.unwrap_or_else(|| {
			std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.map(|elapsed| elapsed.as_nanos() as u64)
				.unwrap_or(0)
				^ 0xA5A5_1234_DEAD_BEEF
		})
}

pub fn knobs() -> Knobs {
	let default = Knobs::default();
	Knobs {
		refocus_period: env_or!("REFOCUS_PERIOD", default.refocus_period),
		refocus_fraction: env_or!("REFOCUS_FRACTION", default.refocus_fraction),
		refocus_kick_min: env_or!("REFOCUS_KICK_MIN", default.refocus_kick_min),
		refocus_kick_max: env_or!("REFOCUS_KICK_MAX", default.refocus_kick_max),
	}
}

pub fn phase_list() -> Vec<Strategy> {
	let spec = std::env::var("STRATEGY").unwrap_or_else(|_| "sa".to_string());
	let mut phase_list = Vec::new();
	for token in spec.split(',') {
		match token.trim() {
			"sa" | "anneal" => phase_list.push(anneal()),
			"ils" => phase_list.push(ils()),
			"" => {}
			other => eprintln!("unknown strategy {other:?}, ignoring"),
		}
	}
	if phase_list.is_empty() {
		phase_list.push(anneal());
		phase_list.push(ils());
	}
	phase_list
}

fn anneal() -> Strategy {
	Strategy::Anneal {
		start_temperature: env_or!("START_TEMP", 10.0),
		cooling: env_or!("COOLING", 0.9995),
		floor_temperature: env_or!("FLOOR_TEMP", 0.4),
	}
}

fn ils() -> Strategy {
	Strategy::IteratedLocalSearch {
		kick_threshold: 40,
		kick_min: 2,
		kick_max: 6,
	}
}
