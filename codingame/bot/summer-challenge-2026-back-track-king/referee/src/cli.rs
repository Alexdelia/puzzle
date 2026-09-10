use crate::driver::{Driver, make_driver};
use crate::game::DEFAULT_LEAGUE;
use crate::grid::{Grid, parse};
use crate::gridmaker;
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
}

#[derive(Args, Clone, Debug)]
pub struct MapArgs {
	/// CodinGame seed: the same seed builds the same board as the official referee
	#[arg(long, default_value_t = 1, allow_negative_numbers = true)]
	pub seed: i64,

	/// read a fixed board from a file instead of generating one
	#[arg(long, value_name = "FILE")]
	pub map: Option<String>,

	/// league level: 1 and 2 are the solo tutorials, 3 and up the real game
	#[arg(long, value_name = "N", default_value_t = DEFAULT_LEAGUE)]
	pub league: i32,
}

impl MapArgs {
	pub fn build(&self, seed: i64) -> Result<Grid, String> {
		let Some(path) = &self.map else {
			if seed == 0 {
				return Err(
					"seed 0 is not reproducible: the CodinGame engine ignores it and seeds itself \
					 from the system"
						.into(),
				);
			}
			return Ok(gridmaker::make(seed));
		};
		let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
		parse(&text).map_err(|e| format!("{path}: {e}"))
	}
}

pub fn die<T>(message: impl std::fmt::Display) -> T {
	eprintln!("{message}");
	std::process::exit(2)
}
