#[path = "common.rs"]
mod common;

use btk::game::parse_actions;
use btk::proto::{init_text, turn_text};
use common::{LARGE, SMALL, after};

fn main() {
	divan::main()
}

#[divan::bench(args = [SMALL, LARGE])]
fn write_init_block(bencher: divan::Bencher, size: (usize, usize, usize)) {
	let game = after(1, size, 0);
	bencher
		.counter(divan::counter::BytesCount::of_str(&init_text(&game.map, 0)))
		.bench(|| divan::black_box(init_text(&game.map, 0)));
}

#[divan::bench(args = [0, 50, 100])]
fn write_turn_block(bencher: divan::Bencher, turns: usize) {
	let game = after(1, LARGE, turns);
	let mut out = String::new();
	turn_text(&game.map, &game.state, &game.conn, 0, &mut out);
	bencher
		.counter(divan::counter::BytesCount::of_str(&out))
		.bench_local(|| turn_text(&game.map, &game.state, &game.conn, 0, &mut out));
}

#[divan::bench(args = [
	"WAIT",
	"PLACE_TRACKS 12 7",
	"AUTOPLACE 1 2 27 18;MESSAGE all aboard",
	"PLACE_TRACKS 1 1;PLACE_TRACKS 2 1;PLACE_TRACKS 3 1;DISRUPT 4 5",
])]
fn parse_an_action_line(bencher: divan::Bencher, line: &str) {
	bencher.bench(|| divan::black_box(parse_actions(line).unwrap()));
}
