use crate::board::Board;
use crate::driver::{Driver, make_driver};
use crate::game::DEFAULT_LEAGUE;
use crate::mapgen;
use clap::Args;
use std::time::Duration;

#[derive(Args, Clone, Debug)]
pub struct BotArgs {
	/// side 0 bot: a path to an executable, or any shell command that speaks the CodinGame protocol
	#[arg(long, value_name = "SPEC")]
	pub p1: Option<String>,

	/// side 1 bot, same form as --p1
	#[arg(long, value_name = "SPEC")]
	pub p2: Option<String>,

	/// how long a bot may take to answer before it loses the game
	#[arg(long, value_name = "MS", default_value_t = 10000)]
	pub bot_timeout: u64,

	/// also fail a bot that breaks the CodinGame 1000ms/50ms budget
	#[arg(long)]
	pub enforce_cg_time: bool,
}

impl BotArgs {
	pub fn timeout(&self) -> Duration {
		Duration::from_millis(self.bot_timeout)
	}

	pub fn spec(&self, side: usize) -> &str {
		let given = if side == 0 { &self.p1 } else { &self.p2 };
		match given {
			Some(spec) => spec,
			None => die(format!("--p{} is required", side + 1)),
		}
	}

	pub fn driver(&self, side: usize) -> Result<Box<dyn Driver>, String> {
		make_driver(self.spec(side), self.timeout())
	}

	pub fn tagged_driver(&self, side: usize, game: usize) -> Result<Box<dyn Driver>, String> {
		let spec = self
			.spec(side)
			.replace("%GAME%", &format!("{game}-p{}", side + 1));
		make_driver(&spec, self.timeout())
	}
}

#[derive(Args, Clone, Debug)]
pub struct MapArgs {
	/// CodinGame seed: the same seed builds the same board as the official referee
	#[arg(long, default_value_t = 1, allow_negative_numbers = true)]
	pub seed: i64,

	/// read a board from a CodinGame log, whose `options` lines hold its nodes, edges and units
	#[arg(long, value_name = "FILE")]
	pub map: Option<String>,

	/// league level of the official referee: 1 MOVE and SPAWN (CodinGame Bronze and Silver), 2 adds ATTACK, 3 adds ADD_PATH and REMOVE_PATH
	#[arg(long, value_name = "N", default_value_t = DEFAULT_LEAGUE)]
	pub league: i32,
}

impl MapArgs {
	pub fn build(&self, seed: i64) -> Result<Board, String> {
		let Some(path) = &self.map else {
			return Ok(mapgen::generate(seed));
		};
		let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
		let log = crate::cglog::CgLog::parse(&text).map_err(|e| format!("{path}: {e}"))?;
		let field = |key: &str| {
			log.option(key)
				.ok_or_else(|| format!("{path}: no `{key}` option"))
		};
		mapgen::from_options(field("nodes")?, field("edges")?, field("units")?)
			.map_err(|e| format!("{path}: {e}"))
	}
}

pub fn die<T>(message: impl std::fmt::Display) -> T {
	eprintln!("{message}");
	std::process::exit(2)
}
