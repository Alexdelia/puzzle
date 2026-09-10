#[path = "common.rs"]
mod common;

use btk::pathfind::{autobuild, terrain_reachable, train_path};
use common::{LARGE, after};

fn main() {
	divan::main()
}

#[divan::bench(args = [0, 50, 100])]
fn train_paths(bencher: divan::Bencher, turns: i32) {
	let game = after(LARGE, turns);
	bencher.bench_local(|| {
		for town in &game.grid.towns {
			for &other in &town.desired {
				divan::black_box(train_path(
					&game.grid,
					town.coord,
					game.grid.towns[other].coord,
				));
			}
		}
	});
}

#[divan::bench(args = [0, 50])]
fn autoplace_between_two_towns(bencher: divan::Bencher, turns: i32) {
	let game = after(LARGE, turns);
	let from = game.grid.towns[0].coord;
	let to = game.grid.towns[1].coord;
	bencher.bench_local(|| divan::black_box(autobuild(&game.grid, from, to)));
}

#[divan::bench]
fn game_over_check(bencher: divan::Bencher) {
	let game = after(LARGE, 50);
	bencher.bench_local(|| {
		for town in &game.grid.towns {
			for &other in &town.desired {
				divan::black_box(terrain_reachable(
					&game.grid,
					town.coord,
					game.grid.towns[other].coord,
				));
			}
		}
	});
}

#[divan::bench(args = [0, 50])]
fn one_turn(bencher: divan::Bencher, turns: i32) {
	let mut game = after(LARGE, turns);
	let from = game.grid.towns[0].coord;
	let to = game.grid.towns[1].coord;
	let line = format!("AUTOPLACE {} {} {} {}", from.x, from.y, to.x, to.y);
	bencher.bench_local(|| {
		game.reset_turn_data();
		game.take_commands(0, &line);
		game.take_commands(1, "DISRUPT 0");
		divan::black_box(game.perform_update())
	});
}

#[divan::bench]
fn an_idle_turn(bencher: divan::Bencher) {
	let mut game = after(LARGE, 40);
	bencher.bench_local(|| {
		game.reset_turn_data();
		game.take_commands(0, "WAIT");
		game.take_commands(1, "WAIT");
		divan::black_box(game.perform_update())
	});
}
