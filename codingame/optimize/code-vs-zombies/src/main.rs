use code_vs_zombies::{parse_config, parse_validator, solve};

fn main() -> Result<(), String> {
	let config = parse_config()?;
	let validator = parse_validator(&config.path.validator)?;

	solve(config, validator)
}
