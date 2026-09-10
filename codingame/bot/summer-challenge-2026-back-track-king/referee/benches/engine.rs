#[path = "common.rs"]
mod common;

use btk::game::{Action, Rules};
use common::{LARGE, MEDIUM, SMALL, after, fresh};

fn main() {
	divan::main()
}

#[divan::bench(args = [0, 25, 50, 100])]
fn recompute_connections(bencher: divan::Bencher, turns: usize) {
	let mut game = after(1, LARGE, turns);
	let (map, state, conn) = (&game.map, &game.state, &mut game.conn);
	bencher.bench_local(|| conn.recompute(map, state));
}

#[divan::bench(args = [0, 50, 100])]
fn score_connections(bencher: divan::Bencher, turns: usize) {
	let game = after(1, LARGE, turns);
	let rules = Rules::default();
	bencher.bench(|| divan::black_box(game.conn.gains(&game.state, &rules)));
}

#[divan::bench(args = [SMALL, MEDIUM, LARGE])]
fn autoplace_on_an_empty_board(bencher: divan::Bencher, size: (usize, usize, usize)) {
	bencher
		.with_inputs(|| {
			let game = fresh(1, size);
			let (a, b) = (&game.map.towns[0], &game.map.towns[1]);
			let line = vec![Action::Autoplace(a.x, a.y, b.x, b.y)];
			(game, line)
		})
		.bench_local_refs(|(game, line)| {
			game.step([line, &[Action::Wait]]).unwrap();
		});
}

#[divan::bench(args = [0, 50])]
fn a_turn_both_sides_contest(bencher: divan::Bencher, turns: usize) {
	let mut game = after(3, LARGE, turns);
	let towns = &game.map.towns;
	let first = vec![Action::Autoplace(
		towns[0].x, towns[0].y, towns[1].x, towns[1].y,
	)];
	let last = towns.len() - 1;
	let second = vec![Action::Autoplace(
		towns[last].x,
		towns[last].y,
		towns[0].x,
		towns[0].y,
	)];
	bencher.bench_local(|| game.step([&first, &second]).unwrap());
}

#[divan::bench]
fn an_idle_turn(bencher: divan::Bencher) {
	let mut game = after(5, LARGE, 40);
	game.rules.max_turns = usize::MAX;
	let idle = [Action::Wait];
	bencher.bench_local(|| divan::black_box(game.step([&idle, &idle]).unwrap()));
}
