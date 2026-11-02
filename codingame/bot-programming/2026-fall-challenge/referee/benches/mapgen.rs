#[path = "common.rs"]
mod common;

use common::SEEDS;
use tryangle::mapgen::{generate, options};

fn main() {
	divan::main()
}

#[divan::bench(args = SEEDS)]
fn generate_board(bencher: divan::Bencher, seed: i64) {
	let mut next = seed;
	bencher.counter(1u32).bench_local(|| {
		next += 1000;
		divan::black_box(generate(next))
	});
}

#[divan::bench]
fn describe_board(bencher: divan::Bencher) {
	let board = generate(SEEDS[0]);
	bencher.bench(|| divan::black_box(options(&board)));
}
