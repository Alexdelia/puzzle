use number_shifting::generate;
use std::fs;

fn main() {
	let args = std::env::args()
		.skip(1)
		.map(|a| a.parse::<usize>().unwrap())
		.collect::<Vec<_>>();
	let [first, last, seeds] = args[..] else {
		panic!("usage: gen <first_level> <last_level> <seeds>");
	};
	fs::create_dir_all("level").unwrap();
	for level in first..=last {
		for seed in 0..seeds {
			let (grid, solution) = generate(level, seed as u64);
			grid.check(&solution).unwrap();
			let name = format!("level/{level:03}_{seed:02}");
			fs::write(format!("{name}.txt"), grid.to_string()).unwrap();
			let moves = solution
				.iter()
				.map(|m| format!("{m}\n"))
				.collect::<String>();
			fs::write(format!("{name}.sol"), moves).unwrap();
		}
	}
}
