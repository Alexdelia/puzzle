#[path = "common.rs"]
mod common;

use common::{SEEDS, Wanderer, after, fresh, play_out, step};
use tryangle::proto::input;

fn main() {
	divan::main()
}

#[divan::bench(args = SEEDS)]
fn whole_game(bencher: divan::Bencher, seed: i64) {
	bencher.bench_local(|| divan::black_box(play_out(seed)));
}

#[divan::bench(args = [0, 60, 200])]
fn one_frame(bencher: divan::Bencher, frames: i32) {
	let start = after(SEEDS[0], frames);
	bencher
		.with_inputs(|| (start.clone(), [Wanderer::new(3), Wanderer::new(5)]))
		.bench_local_values(|(mut game, mut wanderers)| {
			step(&mut game, &mut wanderers);
			game
		});
}

#[divan::bench(args = [0, 60, 200])]
fn turn_input(bencher: divan::Bencher, frames: i32) {
	let game = after(SEEDS[0], frames);
	bencher.bench(|| divan::black_box(input(&game, 0, false)));
}

#[divan::bench]
fn new_game_from_seed(bencher: divan::Bencher) {
	bencher.bench(|| divan::black_box(fresh(SEEDS[1])));
}
