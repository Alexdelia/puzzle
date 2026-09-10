#[path = "common.rs"]
mod common;

use btk::arena::run_match;
use btk::driver::make_driver;
use common::{LARGE, MEDIUM, SMALL, bot, fresh};
use std::time::Duration;

fn main() {
	divan::main()
}

fn duel(seed: i64, p1: &str, p2: &str) {
	let mut game = fresh(seed);
	let timeout = Duration::from_secs(10);
	let mut d0 = make_driver(p1, timeout).unwrap();
	let mut d1 = make_driver(p2, timeout).unwrap();
	divan::black_box(run_match(&mut game, [d0.as_mut(), d1.as_mut()], None, None));
}

#[divan::bench(args = [SMALL, MEDIUM, LARGE])]
fn greedy_vs_bronze(bencher: divan::Bencher, seed: i64) {
	let (p1, p2) = (bot("greedy"), bot("bronze"));
	let mut next = seed;
	bencher.counter(1u32).bench_local(|| {
		next += 1000;
		duel(next, &p1, &p2)
	});
}

#[divan::bench(args = ["wait", "bronze", "greedy", "fuzz"])]
fn mirror_match(bencher: divan::Bencher, name: &str) {
	let command = bot(name);
	let mut seed = LARGE;
	bencher.counter(1u32).bench_local(|| {
		seed += 1000;
		duel(seed, &command, &command)
	});
}
