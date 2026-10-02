use block_the_spreading_fire::{Action, Fault, Game, Map, score_plan};
use std::fs;

fn load(path: &str) -> Map {
	Map::parse(&fs::read_to_string(path).unwrap()).unwrap()
}

fn corridor() -> Map {
	load("validator/test/03_each_cell_matters.txt")
}

#[test]
fn init_input_round_trips_test_file() {
	let text = fs::read_to_string("validator/test/08_big_random_map.txt").unwrap();
	assert_eq!(
		Map::parse(&text).unwrap().init_input(),
		text.trim_end().to_string() + "\n"
	);
}

#[test]
fn cooldown_seen_next_turn_is_cut_duration_minus_one() {
	let map = corridor();
	let mut game = Game::new(&map);
	game.play(Action::Cut(map.index(9, 1))).unwrap();
	let seen = (0..4).map(|_| {
		let cooldown = game.cooldown();
		game.play(Action::Wait).unwrap();
		cooldown
	});
	assert_eq!(
		seen.collect::<Vec<_>>(),
		[map.tree.cut_duration - 1, 2, 1, 0]
	);
}

#[test]
fn rejects_invalid_cuts() {
	let map = corridor();
	let parse = |line: &str| Action::parse(line, &map);
	assert_eq!(parse("WAIT"), Ok(Action::Wait));
	assert_eq!(parse("hello"), Err(Fault::BadOutput("hello".into())));
	assert_eq!(parse("1 1 1"), Err(Fault::BadOutput("1 1 1".into())));
	assert_eq!(parse("15 1"), Err(Fault::OutOfMap(15, 1)));
	assert_eq!(parse("-1 1"), Err(Fault::OutOfMap(-1, 1)));

	let mut game = Game::new(&map);
	assert_eq!(game.check(parse("0 0").unwrap()), Err(Fault::CutSafe(0, 0)));
	assert_eq!(
		game.check(parse("7 1").unwrap()),
		Err(Fault::CutOnFire(7, 1))
	);
	game.play(parse("9 1").unwrap()).unwrap();
	assert_eq!(
		game.check(parse("10 1").unwrap()),
		Err(Fault::CutOnCooldown(3))
	);
	assert_eq!(
		game.check(parse("9 1").unwrap()),
		Err(Fault::CutOnCooldown(3))
	);
	while game.cooldown() > 0 {
		game.play(Action::Wait).unwrap();
	}
	assert_eq!(game.check(parse("9 1").unwrap()), Err(Fault::CutSafe(9, 1)));
	assert_eq!(
		game.check(parse("7 1").unwrap()),
		Err(Fault::CutBurnt(7, 1))
	);
}

#[test]
fn burnt_cell_spreads_to_neighbours_same_turn() {
	let map = corridor();
	let mut game = Game::new(&map);
	let (start, left, right) = (map.index(7, 1), map.index(6, 1), map.index(8, 1));
	game.play(Action::Wait).unwrap();
	assert_eq!([game.progress()[start], game.progress()[left]], [1, -1]);
	game.play(Action::Wait).unwrap();
	assert_eq!(
		[
			game.progress()[start],
			game.progress()[left],
			game.progress()[right]
		],
		[2, 0, 0]
	);
}

#[test]
fn plan_cuts_as_soon_as_cooldown_allows() {
	let map = corridor();
	assert_eq!(score_plan(&map, &[map.index(8, 1)]), Ok(5 * map.tree.value));
	assert_eq!(
		score_plan(&map, &[map.index(8, 1), map.index(2, 1)]),
		Ok(6 * map.tree.value)
	);
	assert_eq!(
		score_plan(&map, &[map.index(9, 1), map.index(6, 1)]),
		Err((1, Fault::CutBurnt(6, 1)))
	);
	assert_eq!(
		score_plan(&map, &[map.index(7, 1)]),
		Err((0, Fault::CutOnFire(7, 1)))
	);
}

#[test]
fn plan_reproduces_demo_score() {
	let map = load("validator/test/07_big_map_with_villages_and_wall.txt");
	let demo_cuts = [
		(32, 22),
		(33, 21),
		(39, 16),
		(38, 17),
		(37, 18),
		(36, 19),
		(35, 20),
		(34, 21),
		(32, 23),
		(39, 15),
		(38, 14),
		(31, 25),
		(32, 24),
		(30, 25),
		(29, 25),
		(39, 13),
		(28, 25),
		(40, 12),
		(27, 25),
		(40, 11),
		(25, 24),
		(26, 25),
		(24, 23),
	];
	let cuts = demo_cuts.map(|(x, y)| map.index(x, y));
	assert_eq!(score_plan(&map, &cuts), Ok(6068));
}
