use std::env;
use std::str::FromStr;
use std::time::Duration;

use crate::parse::path::cwd;
use crate::{PathConfig, SearchConfig};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
	Ga,
	Beam,
	BeamThenGa,
}

impl FromStr for Mode {
	type Err = String;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"ga" => Ok(Mode::Ga),
			"beam" => Ok(Mode::Beam),
			"beam+ga" => Ok(Mode::BeamThenGa),
			other => Err(format!(
				"unknown MODE {other:?}, expected one of: ga, beam, beam+ga"
			)),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Config {
	pub validator_name: String,
	pub mode: Mode,
	pub search_config: SearchConfig,
	pub path: PathConfig,
}

pub const ENV_MODE: &str = "MODE";
pub const ENV_SEED: &str = "SEED";
pub const ENV_POPULATION: &str = "POPULATION";
pub const ENV_TIME_SEC_LIMIT: &str = "TIME_SEC_LIMIT";
pub const ENV_TURN_LIMIT: &str = "TURN_LIMIT";
pub const ENV_HUMAN_WEIGHT: &str = "HUMAN_WEIGHT";

pub fn parse_config() -> Result<Config, String> {
	let validator_name = env::args().nth(1).ok_or_else(usage_message)?;

	let mode = parse_env(ENV_MODE, Mode::Ga)?;
	let extra_seed = parse_env::<u64>(ENV_SEED, 0)?;
	let population = parse_env::<usize>(ENV_POPULATION, 8192)?;
	let turn_limit = parse_env::<usize>(ENV_TURN_LIMIT, 100)?;
	let time_sec_limit = parse_env::<f64>(ENV_TIME_SEC_LIMIT, 60.0)?;
	let human_weight = parse_env::<i64>(ENV_HUMAN_WEIGHT, 0)?;

	if population == 0 {
		return Err(format!("{ENV_POPULATION} must be > 0"));
	}
	if turn_limit == 0 {
		return Err(format!("{ENV_TURN_LIMIT} must be > 0"));
	}
	if !(time_sec_limit.is_finite() && time_sec_limit > 0.0) {
		return Err(format!(
			"{ENV_TIME_SEC_LIMIT} must be a positive finite number"
		));
	}

	let search_config = SearchConfig {
		seed: hash_str(&validator_name).wrapping_add(extra_seed.wrapping_mul(0x9E3779B97F4A7C15)),
		population,
		elite: (population / 64).clamp(8, 128),
		turn_limit,
		time_limit: Duration::from_secs_f64(time_sec_limit),
		human_weight,
	};

	let path = PathConfig::new(cwd()?, &validator_name);

	Ok(Config {
		validator_name,
		mode,
		search_config,
		path,
	})
}

fn hash_str(s: &str) -> u64 {
	let mut h: u64 = 0xCBF29CE484222325;
	for b in s.bytes() {
		h ^= b as u64;
		h = h.wrapping_mul(0x100000001B3);
	}
	h
}

pub fn usage_message() -> String {
	format!(
		"usage: code-vs-zombies <validator_name>
env var:
{ENV_MODE}=ga|beam|beam+ga, default ga
{ENV_SEED}=0
{ENV_POPULATION}=8192
{ENV_TIME_SEC_LIMIT}=60.0
{ENV_TURN_LIMIT}=100",
	)
}

fn parse_env<T: FromStr>(name: &str, default: T) -> Result<T, String>
where
	T::Err: std::fmt::Display,
{
	match env::var(name) {
		Ok(v) => v
			.parse::<T>()
			.map_err(|e| format!("invalid {name}={v:?}: {e}")),
		Err(_) => Ok(default),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn mode_from_str() {
		assert_eq!("ga".parse::<Mode>().unwrap(), Mode::Ga);
		assert_eq!("beam".parse::<Mode>().unwrap(), Mode::Beam);
		assert_eq!("beam+ga".parse::<Mode>().unwrap(), Mode::BeamThenGa);
		assert!("zoinks".parse::<Mode>().is_err());
	}
}
