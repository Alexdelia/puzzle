mod parse;
mod simulation;
mod solve;

fn main() -> Result<(), String> {
	let config = parse::parse()?;
	solve::solve(config)
}
