#[path = "common.rs"]
mod common;

use btk::arena::run_match;
use btk::driver::make_driver;
use common::{LARGE, MEDIUM, SMALL, bot, fresh};
use std::time::Duration;

fn main() {
	divan::main()
}

fn duel(seed: u64, size: (usize, usize, usize), p1: &str, p2: &str) {
	let mut game = fresh(seed, size);
	let timeout = Duration::from_secs(10);
	let mut d0 = make_driver(p1, timeout).unwrap();
	let mut d1 = make_driver(p2, timeout).unwrap();
	divan::black_box(run_match(&mut game, [d0.as_mut(), d1.as_mut()], None, None));
}

#[divan::bench(args = [SMALL, MEDIUM, LARGE])]
fn greedy_vs_bronze(bencher: divan::Bencher, size: (usize, usize, usize)) {
	let (p1, p2) = (bot("greedy"), bot("bronze"));
	let mut seed = 0u64;
	bencher.counter(1u32).bench_local(|| {
		seed += 1;
		duel(seed, size, &p1, &p2)
	});
}

#[divan::bench(args = ["wait", "bronze", "greedy"])]
fn mirror_match(bencher: divan::Bencher, name: &str) {
	let command = bot(name);
	let mut seed = 0u64;
	bencher.counter(1u32).bench_local(|| {
		seed += 1;
		duel(seed, LARGE, &command, &command)
	});
}
