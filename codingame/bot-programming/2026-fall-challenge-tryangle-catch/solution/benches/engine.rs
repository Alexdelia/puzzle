use engine::{Command, Map, Point, State, nodes_of};
use tryangle::mapgen;

const SEEDS: [i64; 5] = [1, 2, 3, 4, 5];
const SPAWN_ONE_IN: u64 = 3;

fn main() {
	divan::main()
}

struct Position {
	map_index: usize,
	state: State,
	commands: [Vec<Command>; 2],
}

fn spreading_commands(map: &Map, state: &State, player: usize, rng: &mut u64) -> Vec<Command> {
	let mut next = || {
		*rng ^= *rng << 13;
		*rng ^= *rng >> 7;
		*rng ^= *rng << 17;
		*rng
	};
	let mut commands = Vec::new();
	for from in 0..map.len() {
		let amount = state.units[from][player];
		let around: Vec<usize> = nodes_of(state.adjacency[from]).collect();
		if amount > 0 && !around.is_empty() {
			let to = around[next() as usize % around.len()];
			commands.push(Command::Move { from, to, amount });
		}
	}
	for triangle in state.owner[player].iter() {
		if next() % SPAWN_ONE_IN == 0 {
			let node = map.triangles[triangle][0];
			commands.push(Command::Spawn { triangle, node });
		}
	}
	commands
}

fn recorded_positions() -> (Vec<Map>, Vec<Position>) {
	let mut maps = Vec::new();
	let mut positions = Vec::new();
	for seed in SEEDS {
		let board = mapgen::generate(seed);
		let map = Map::new(
			board
				.points
				.iter()
				.map(|p| Point { x: p.x, y: p.y })
				.collect(),
		);
		let mut state = State::new(&map, &board.units, &board.edges());
		let mut rng = seed as u64 | 1;
		for _ in 0..engine::MAX_TURNS {
			let commands = [0, 1].map(|p| spreading_commands(&map, &state, p, &mut rng));
			positions.push(Position {
				map_index: maps.len(),
				state,
				commands: commands.clone(),
			});
			state.step(&map, [&commands[0], &commands[1]]);
		}
		maps.push(map);
	}
	(maps, positions)
}

#[divan::bench]
fn step_recorded_turns(bencher: divan::Bencher) {
	let (maps, positions) = recorded_positions();
	bencher
		.counter(divan::counter::ItemsCount::new(positions.len()))
		.bench_local(|| {
			for position in &positions {
				let mut state = position.state;
				let [mine, theirs] = &position.commands;
				state.step(&maps[position.map_index], [mine, theirs]);
				divan::black_box(state);
			}
		});
}

#[divan::bench]
fn copy_state(bencher: divan::Bencher) {
	let (_, positions) = recorded_positions();
	let state = positions[positions.len() / 2].state;
	bencher.bench_local(|| divan::black_box(divan::black_box(state)));
}

#[divan::bench]
fn build_map(bencher: divan::Bencher) {
	let board = mapgen::generate(SEEDS[0]);
	let points: Vec<Point> = board
		.points
		.iter()
		.map(|p| Point { x: p.x, y: p.y })
		.collect();
	bencher.bench_local(|| divan::black_box(Map::new(points.clone())));
}
