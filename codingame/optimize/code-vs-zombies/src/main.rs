use code_vs_zombies::{parse_config, parse_validator_packs, solve};

fn main() -> Result<(), String> {
	let config = parse_config()?;
	let mut packs = parse_validator_packs(&config.path.validator)?;
	let test = packs[0].clone();
	let validator = packs.pop().unwrap();

	solve(config, test, validator)
}
