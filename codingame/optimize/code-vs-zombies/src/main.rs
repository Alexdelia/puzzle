use code_vs_zombies::{parse_config, parse_validator_packs, solve};

fn main() -> Result<(), String> {
	let config = parse_config()?;
	let packs = parse_validator_packs(&config.path.validator)?;
	let validator = packs.into_iter().last().unwrap();

	solve(config, validator)
}
