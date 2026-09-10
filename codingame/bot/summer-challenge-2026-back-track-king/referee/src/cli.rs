use crate::driver::{Driver, make_driver};
use crate::game::{Overflow, Rules};
use crate::map::{CG_HEIGHT, CG_TOWNS, CG_WIDTH, Map, format_map, parse_map, validate};
use crate::mapgen::{GenParams, generate};
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
	/// map seed: the same seed always builds the same map
	#[arg(long, default_value_t = 1)]
	pub seed: u64,

	/// read a fixed map from a file instead of generating one
	#[arg(long, value_name = "FILE", conflicts_with_all = ["width", "height", "towns"])]
	pub map: Option<String>,

	/// accept a --map file outside the CodinGame size ranges
	#[arg(long, requires = "map")]
	pub loose_map: bool,

	/// force the map width instead of drawing one from the seed
	#[arg(long, value_name = "W", value_parser = ranged(CG_WIDTH))]
	pub width: Option<usize>,

	/// force the map height instead of drawing one from the seed
	#[arg(long, value_name = "H", value_parser = ranged(CG_HEIGHT))]
	pub height: Option<usize>,

	/// force the town count instead of drawing one from the seed
	#[arg(long, value_name = "N", value_parser = ranged(CG_TOWNS))]
	pub towns: Option<usize>,
}

fn ranged(allowed: std::ops::RangeInclusive<usize>) -> clap::builder::RangedU64ValueParser<usize> {
	clap::builder::RangedU64ValueParser::new()
		.range(*allowed.start() as u64..=*allowed.end() as u64)
}

impl MapArgs {
	pub fn shape(&self) -> GenParams {
		GenParams {
			width: self.width,
			height: self.height,
			towns: self.towns,
		}
	}

	pub fn build(&self, seed: u64) -> Result<Map, String> {
		let Some(path) = &self.map else {
			return Ok(generate(seed, &self.shape()));
		};
		let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
		let map = parse_map(&text).map_err(|e| format!("{path}: {e}"))?;
		if !self.loose_map {
			validate(&map, true)
				.map_err(|e| format!("{path}: {e} (--loose-map accepts it anyway)"))?;
		}
		Ok(map)
	}

	pub fn dump(&self) -> Result<String, String> {
		self.build(self.seed).map(|map| format_map(&map))
	}
}

#[derive(Args, Clone, Debug)]
pub struct RuleArgs {
	/// turn limit
	#[arg(long, value_name = "N", default_value_t = crate::game::MAX_TURNS)]
	pub turns: usize,

	/// paint points handed out each turn
	#[arg(long, value_name = "N", default_value_t = crate::game::PAINT_PER_TURN)]
	pub paint: u32,

	/// instability that inks a region out
	#[arg(long, value_name = "N", default_value_t = crate::game::INK_THRESHOLD)]
	pub ink: u8,

	/// let DISRUPT actually ink regions out (silver league and up)
	#[arg(long)]
	pub disruption: bool,

	/// let neutral tracks score for both players
	#[arg(long)]
	pub neutral_scores: bool,

	/// treat a refused action as a loss instead of a no-op
	#[arg(long)]
	pub strict: bool,

	/// what to do with an action that costs more paint than is left
	#[arg(long, value_enum, default_value_t = Overflow::Stop)]
	pub overflow: Overflow,
}

impl RuleArgs {
	pub fn rules(&self) -> Rules {
		Rules {
			max_turns: self.turns,
			paint: self.paint,
			ink_threshold: self.ink,
			disruption: self.disruption,
			neutral_scores: self.neutral_scores,
			strict: self.strict,
			overflow: self.overflow,
		}
	}
}

pub fn die<T>(message: impl std::fmt::Display) -> T {
	eprintln!("{message}");
	std::process::exit(2)
}
