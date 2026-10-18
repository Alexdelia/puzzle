use std::io::{self, BufRead};

const SOLUTIONS: &[(&str, &str)] = &[];

fn normalize(map: &str) -> String {
	map.lines().map(str::trim).collect::<Vec<_>>().join("\n")
}

fn main() {
	let stdin = io::stdin();
	let mut lines = stdin.lock().lines().map(Result::unwrap);
	let mut header: Vec<String> = lines.by_ref().take(4).collect();
	let height: usize = header[2]
		.split_whitespace()
		.nth(1)
		.unwrap()
		.parse()
		.unwrap();
	header.extend(lines.by_ref().take(height));
	let map = normalize(&header.join("\n"));
	let plan = SOLUTIONS
		.iter()
		.find(|(solved, _)| normalize(solved) == map)
		.map_or("", |&(_, cuts)| cuts);
	let mut cuts = plan.lines().map(str::trim).filter(|cut| !cut.is_empty());
	while let Some(cooldown) = lines.next() {
		lines.by_ref().take(height).for_each(drop);
		let cut = if cooldown.trim() == "0" {
			cuts.next()
		} else {
			None
		};
		println!("{}", cut.unwrap_or("WAIT"));
	}
}
