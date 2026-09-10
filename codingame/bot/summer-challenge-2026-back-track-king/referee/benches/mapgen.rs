#[path = "common.rs"]
mod common;

use btk::map::validate;
use btk::mapgen::generate;
use common::{LARGE, MEDIUM, SMALL, shape};

fn main() {
	divan::main()
}

#[divan::bench(args = [SMALL, MEDIUM, LARGE])]
fn generate_map(bencher: divan::Bencher, size: (usize, usize, usize)) {
	let shape = shape(size);
	let mut seed = 0u64;
	bencher.counter(1u32).bench_local(|| {
		seed += 1;
		divan::black_box(generate(seed, &shape))
	});
}

#[divan::bench(args = [SMALL, LARGE])]
fn validate_map(bencher: divan::Bencher, size: (usize, usize, usize)) {
	let map = generate(7, &shape(size));
	bencher.bench(|| divan::black_box(validate(&map, true)));
}

#[divan::bench(args = [4, 8, 12])]
fn generate_by_town_count(bencher: divan::Bencher, towns: usize) {
	let shape = shape((30, 20, towns));
	let mut seed = 0u64;
	bencher.bench_local(|| {
		seed += 1;
		divan::black_box(generate(seed, &shape))
	});
}
