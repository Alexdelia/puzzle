use number_shifting::{Level, Move};
use std::fs;

fn main() {
	let args = std::env::args().skip(1).collect::<Vec<_>>();
	let [level_path, solution_path] = &args[..] else {
		panic!("usage: verify <level> <solution>");
	};
	let level = Level::parse(&fs::read_to_string(level_path).unwrap());
	let moves = fs::read_to_string(solution_path)
		.unwrap()
		.lines()
		.filter_map(Move::parse)
		.collect::<Vec<_>>();
	match level.check(&moves) {
		Ok(()) => println!("OK {} moves", moves.len()),
		Err(e) => {
			println!("FAIL {e}");
			std::process::exit(1);
		}
	}
}
