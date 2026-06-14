mod env;

use std::time::Duration;

use crate::simulation::{
	Cell, Engine, MAP_AREA, MAP_HEIGHT, MAP_WIDTH, MAX_ROBOTS, NONE, Robot, Tile, char_to_tile,
};
use crate::solve::{Knobs, Strategy};

const DEFAULT_CHAIN_COUNT: usize = 1 << 12;

pub struct Config {
	pub name: String,
	pub engine: Engine,
	pub chain_count: usize,
	pub duration: Duration,
	pub fresh: bool,
	pub seed: u64,
	pub phase_list: Vec<Strategy>,
	pub knobs: Knobs,
}

pub fn parse() -> Result<Config, String> {
	let mut argument = std::env::args().skip(1);

	let path = argument
		.next()
		.ok_or_else(|| "missing input file path argument".to_string())?;

	let chain_count = match argument.next() {
		Some(value) => value
			.parse::<usize>()
			.map_err(|e| format!("invalid chain count: {e}"))?,
		None => DEFAULT_CHAIN_COUNT,
	};

	let name = validator_name(&path);
	let content = std::fs::read_to_string(&path).map_err(|e| format!("read {path}: {e}"))?;
	let engine = parse_map(&content)?;

	Ok(Config {
		name,
		engine,
		chain_count,
		duration: env::duration(),
		fresh: env::fresh(),
		seed: env::seed(),
		phase_list: env::phase_list(),
		knobs: env::knobs(),
	})
}

fn validator_name(path: &str) -> String {
	std::path::Path::new(path)
		.file_stem()
		.map(|stem| stem.to_string_lossy().to_string())
		.unwrap_or_default()
}

pub fn parse_map(content: &str) -> Result<Engine, String> {
	let row_list: Vec<&str> = content.lines().take(MAP_HEIGHT).collect();
	if row_list.len() < MAP_HEIGHT {
		return Err(format!(
			"expected {MAP_HEIGHT} rows, got {}",
			row_list.len()
		));
	}

	let mut base = [NONE; MAP_AREA];
	let mut robot_list = Vec::new();

	for (y, row) in row_list.iter().enumerate() {
		let column_list: Vec<char> = row.chars().collect();
		if column_list.len() < MAP_WIDTH {
			return Err(format!(
				"row {y} has {} columns, expected {MAP_WIDTH}",
				column_list.len()
			));
		}
		for (x, &c) in column_list.iter().take(MAP_WIDTH).enumerate() {
			let cell = y * MAP_WIDTH + x;
			let tile: Tile =
				char_to_tile(c).ok_or_else(|| format!("invalid char {c:?} at ({x}, {y})"))?;
			if c.is_ascii_uppercase() {
				base[cell] = NONE;
				robot_list.push(Robot {
					cell: cell as Cell,
					heading: tile,
				});
			} else {
				base[cell] = tile;
			}
		}
	}

	if robot_list.is_empty() {
		return Err("no robots found in map".to_string());
	}
	if robot_list.len() > MAX_ROBOTS {
		return Err(format!(
			"{} robots exceeds maximum of {MAX_ROBOTS}",
			robot_list.len()
		));
	}

	Ok(Engine { base, robot_list })
}
