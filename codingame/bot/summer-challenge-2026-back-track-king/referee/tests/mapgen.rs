use btk::{grid, gridmaker};

fn replay(seeds: &[i64]) -> String {
	let mut out = String::new();
	for &seed in seeds {
		out.push_str(&format!("seed {seed}\n"));
		out.push_str(&grid::describe(&gridmaker::make(seed)));
	}
	out
}

#[test]
fn matches_the_official_generator() {
	let golden = include_str!("golden/maps.txt");
	let seeds: Vec<i64> = golden
		.lines()
		.filter_map(|line| line.strip_prefix("seed "))
		.map(|seed| seed.parse().unwrap())
		.collect();
	assert!(!seeds.is_empty(), "golden map file has no seeds");
	let ours = replay(&seeds);
	for (line, (want, got)) in golden.lines().zip(ours.lines()).enumerate() {
		assert_eq!(want, got, "line {}", line + 1);
	}
	assert_eq!(golden.lines().count(), ours.lines().count());
}
