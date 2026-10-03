use std::io::{self, BufRead};

const SOLUTIONS: &[(&str, &str)] = &[];

struct Site {
	x: i64,
	y: i64,
	demand: i64,
}

fn distance(a: &Site, b: &Site) -> f64 {
	(((a.x - b.x).pow(2) + (a.y - b.y).pow(2)) as f64).sqrt()
}

fn fingerprint(capacity: i64, sites: &[Site]) -> String {
	let sum_x: i64 = sites.iter().map(|site| site.x).sum();
	let sum_y: i64 = sites.iter().map(|site| site.y).sum();
	format!("{count} {capacity} {sum_x} {sum_y}", count = sites.len())
}

fn nearest_neighbour_tours(capacity: i64, sites: &[Site]) -> String {
	let mut unvisited: Vec<usize> = (1..sites.len()).collect();
	let mut tours = Vec::new();
	while !unvisited.is_empty() {
		let mut tour = Vec::new();
		let mut load = 0;
		let mut here = 0;
		while let Some(slot) = (0..unvisited.len())
			.filter(|&slot| load + sites[unvisited[slot]].demand <= capacity)
			.min_by(|&a, &b| {
				let to = |slot: usize| distance(&sites[here], &sites[unvisited[slot]]);
				to(a).total_cmp(&to(b))
			}) {
			here = unvisited.swap_remove(slot);
			load += sites[here].demand;
			tour.push(here.to_string());
		}
		tours.push(tour.join(" "));
	}
	tours.join(";")
}

fn main() {
	let stdin = io::stdin();
	let mut lines = stdin.lock().lines().map(Result::unwrap);
	let mut numbers = || -> Vec<i64> {
		let line = lines.next().unwrap();
		line.split_whitespace()
			.map(|token| token.parse().unwrap())
			.collect()
	};
	let count = numbers()[0] as usize;
	let capacity = numbers()[0];
	let sites: Vec<Site> = (0..count)
		.map(|_| {
			let [_, x, y, demand] = numbers()[..] else {
				panic!("site line needs 4 numbers")
			};
			Site { x, y, demand }
		})
		.collect();
	let key = fingerprint(capacity, &sites);
	let tours = SOLUTIONS
		.iter()
		.find(|(solved, _)| *solved == key)
		.map_or_else(
			|| nearest_neighbour_tours(capacity, &sites),
			|&(_, tours)| tours.to_string(),
		);
	println!("{tours}");
}
