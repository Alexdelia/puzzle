#[path = "common.rs"]
mod common;

use btk::grid::describe;
use btk::gridmaker::make;
use common::{LARGE, MEDIUM, SMALL};

fn main() {
	divan::main()
}

#[divan::bench(args = [SMALL, MEDIUM, LARGE])]
fn generate_board(bencher: divan::Bencher, seed: i64) {
	let mut next = seed;
	bencher.counter(1u32).bench_local(|| {
		next += 1000;
		divan::black_box(make(next))
	});
}

#[divan::bench]
fn describe_board(bencher: divan::Bencher) {
	let grid = make(LARGE);
	bencher.bench(|| divan::black_box(describe(&grid)));
}
