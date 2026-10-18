use btk::game::{DEFAULT_LEAGUE, Game};
use btk::grid::{Coord, coord, parse};

const BOARD: &str = "\
size 5 4
terrain
.....
.....
.....
.....
regions
0 0 0 0 0
0 0 0 0 0
0 0 0 0 0
0 0 0 0 0
towns 3
town 0 0 0 1,2
town 1 4 3 x
town 2 4 0 x
";

fn board(red: &[Coord], blue: &[Coord]) -> Game {
	let mut game = Game::new(parse(BOARD).unwrap(), DEFAULT_LEAGUE);
	for (owner, cells) in [(0i8, red), (1, blue)] {
		for &at in cells {
			game.grid.tile_mut(at).track = owner;
		}
	}
	game
}

fn connections(game: &Game, at: Coord) -> Vec<(u8, u8)> {
	game.grid.tile(at).connections.clone()
}

#[test]
fn statement_example_one() {
	let red = [coord(0, 1), coord(0, 2), coord(0, 3), coord(4, 1)];
	let blue = [coord(1, 3), coord(2, 3), coord(3, 3), coord(4, 2)];
	let mut game = board(&red, &blue);
	game.take_commands(0, "WAIT");
	game.take_commands(1, "WAIT");
	let report = game.perform_update();

	assert_eq!(report.gained, [7, 7]);
	assert_eq!(game.grid.towns[0].active, vec![1, 2]);
	assert_eq!(game.grid.towns[0].paths[&1].len(), 8);
	assert_eq!(game.grid.towns[0].paths[&2].len(), 11);
	assert_eq!(connections(&game, coord(0, 3)), vec![(0, 1), (0, 2)]);
	assert_eq!(connections(&game, coord(4, 1)), vec![(0, 2)]);
	assert_eq!(connections(&game, coord(2, 0)), vec![]);
}

#[test]
fn statement_example_two() {
	let red = [
		coord(0, 1),
		coord(0, 2),
		coord(0, 3),
		coord(4, 1),
		coord(1, 0),
		coord(2, 0),
		coord(3, 0),
	];
	let blue = [coord(1, 3), coord(2, 3), coord(3, 3), coord(4, 2)];
	let mut game = board(&red, &blue);
	game.take_commands(0, "WAIT");
	game.take_commands(1, "WAIT");
	let report = game.perform_update();

	assert_eq!(report.gained, [7, 1]);
	assert_eq!(game.grid.towns[0].paths[&2].len(), 5);
	assert_eq!(
		game.grid.towns[0].paths[&1],
		vec![
			coord(0, 0),
			coord(1, 0),
			coord(2, 0),
			coord(3, 0),
			coord(4, 0),
			coord(4, 1),
			coord(4, 2),
			coord(4, 3),
		]
	);
}

#[test]
fn paint_runs_out_within_the_turn() {
	let mut game = Game::new(parse(BOARD).unwrap(), DEFAULT_LEAGUE);
	game.take_commands(
		0,
		"PLACE_TRACKS 1 1;PLACE_TRACKS 2 1;PLACE_TRACKS 3 1;PLACE_TRACKS 1 2",
	);
	game.take_commands(1, "WAIT");
	game.perform_update();

	assert_eq!(game.grid.tile(coord(1, 1)).track, 0);
	assert_eq!(game.grid.tile(coord(3, 1)).track, 0);
	assert_eq!(game.grid.tile(coord(1, 2)).track, -1);
	assert!(
		game.summary
			.iter()
			.any(|line| line.contains("Not enough track points"))
	);
}

#[test]
fn a_contested_cell_turns_neutral() {
	let mut game = Game::new(parse(BOARD).unwrap(), DEFAULT_LEAGUE);
	game.take_commands(0, "PLACE_TRACKS 2 2");
	game.take_commands(1, "PLACE_TRACKS 2 2");
	game.perform_update();
	assert_eq!(game.grid.tile(coord(2, 2)).track, 2);
}
