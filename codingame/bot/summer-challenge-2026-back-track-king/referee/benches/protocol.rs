#[path = "common.rs"]
mod common;

use btk::action::parse;
use btk::proto::{init_lines, turn_lines};
use common::{LARGE, SMALL, after};

fn main() {
	divan::main()
}

#[divan::bench(args = [SMALL, LARGE])]
fn write_init_block(bencher: divan::Bencher, seed: i64) {
	let game = after(seed, 0);
	let mut out = String::new();
	init_lines(&game.grid, 0, &mut out);
	bencher
		.counter(divan::counter::BytesCount::of_str(&out))
		.bench_local(|| init_lines(&game.grid, 0, &mut out));
}

#[divan::bench(args = [0, 50, 100])]
fn write_turn_block(bencher: divan::Bencher, turns: i32) {
	let game = after(LARGE, turns);
	let mut out = String::new();
	turn_lines(&game, 0, &mut out);
	bencher
		.counter(divan::counter::BytesCount::of_str(&out))
		.bench_local(|| turn_lines(&game, 0, &mut out));
}

#[divan::bench(args = [
	"WAIT",
	"PLACE_TRACKS 12 7",
	"AUTOPLACE 1 2 27 18;MESSAGE all aboard",
	"PLACE_TRACKS 1 1;PLACE_TRACKS 2 1;PLACE_TRACKS 3 1;DISRUPT 4 5",
])]
fn parse_an_action_line(bencher: divan::Bencher, line: &str) {
	bencher.bench(|| divan::black_box(parse(line)));
}
