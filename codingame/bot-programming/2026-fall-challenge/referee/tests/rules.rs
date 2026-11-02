use tryangle::game::{Answer, Game, Phase};
use tryangle::jhash::string_hash;
use tryangle::jrand::JavaRandom;
use tryangle::mapgen::from_options;
use tryangle::task::java_split;

const DIAMOND_NODES: &str = "100,100_300,100_200,300_400,300_300,500";
const DIAMOND_EDGES: &str = "0,1_1,2_0,2_1,3_2,3_2,4_3,4";

fn diamond(units: &str, league: i32) -> Game {
	Game::new(
		from_options(DIAMOND_NODES, DIAMOND_EDGES, units).unwrap(),
		league,
	)
}

struct Turn {
	asked: [bool; 2],
	summary: Vec<String>,
}

fn until_ask(game: &mut Game) -> Turn {
	let mut summary = Vec::new();
	loop {
		let phase = game.begin_frame();
		if let Phase::Ask(_) = phase {
			return Turn {
				asked: phase.asked(),
				summary,
			};
		}
		summary.extend(game.summary.iter().cloned());
		assert!(!game.is_over(), "game ended before the next input");
	}
}

fn answer(game: &mut Game, turn: &Turn, commands: [&str; 2]) -> Vec<String> {
	let answers = [0, 1].map(|p| turn.asked[p].then(|| Answer::Line(commands[p].to_string())));
	game.end_frame(turn.asked, answers);
	game.summary.clone()
}

fn play(game: &mut Game, commands: [&str; 2]) -> Vec<String> {
	let turn = until_ask(game);
	let mut summary = turn.summary.clone();
	summary.extend(answer(game, &turn, commands));
	summary
}

fn triangle_owner(game: &Game, corners: [usize; 3]) -> Option<usize> {
	let id = game
		.board
		.triangles
		.iter()
		.copied()
		.find(|&t| game.board.triangle(t).nodes == corners)
		.expect("triangle exists");
	game.board.triangle(id).owner
}

#[test]
fn java_random_matches_known_sequences() {
	assert_eq!(JavaRandom::new(42).next_i32(), -1170105035);
	assert_eq!(JavaRandom::new(0).next_i32(), -1155484576);
	assert_eq!(JavaRandom::new(0).next_double(), 0.730967787376657);
}

#[test]
fn java_string_hash_matches() {
	assert_eq!(string_hash("hello"), 99162322);
	assert_eq!(string_hash(""), 0);
}

#[test]
fn split_drops_trailing_empties_only() {
	assert_eq!(java_split("a;;b;;", ';'), ["a", "", "b"]);
	assert_eq!(java_split(";;", ';'), Vec::<&str>::new());
	assert_eq!(java_split("", ';'), [""]);
	assert_eq!(java_split(";a", ';'), ["", "a"]);
}

#[test]
fn diamond_has_three_triangles() {
	let game = diamond("0,0_0,0_0,0_0,0_0,0", 1);
	let corners: Vec<[usize; 3]> = game
		.board
		.triangles
		.iter()
		.map(|&t| game.board.triangle(t).nodes)
		.collect();
	assert_eq!(corners, [[0, 1, 2], [1, 2, 3], [2, 3, 4]]);
}

#[test]
fn a_triangle_held_at_start_scores_before_the_first_turn() {
	let game = diamond("1,0_1,0_1,0_0,1_0,0", 1);
	assert_eq!(triangle_owner(&game, [0, 1, 2]), Some(0));
	assert_eq!(game.scores(), [1, 0]);
}

#[test]
fn moves_step_once_toward_the_target_by_lowest_id() {
	let mut game = diamond("2,0_0,0_0,0_0,0_0,1", 1);
	play(&mut game, ["MOVE 0 4 2", "WAIT"]);
	until_ask(&mut game);
	assert_eq!(game.board.units[0][0], 0);
	assert_eq!(game.board.units[2][0], 2, "0 reaches 4 through 2, not 1");
}

#[test]
fn moved_units_cannot_move_again_in_the_same_turn() {
	let mut game = diamond("1,0_0,0_0,0_0,0_0,1", 1);
	let summary = play(&mut game, ["MOVE 0 1 1;MOVE 1 3 1", "WAIT"]);
	assert_eq!(summary, ["$0: Task can't be applied: MOVE 1 3 1"]);
	until_ask(&mut game);
	assert_eq!(game.board.units[1][0], 1);
}

#[test]
fn spawning_frees_the_triangle_until_its_corners_are_empty() {
	let mut game = diamond("1,0_1,0_1,0_0,0_0,2", 1);
	let summary = play(&mut game, ["SPAWN 2 0 1", "WAIT"]);
	assert_eq!(
		summary,
		["$0 spawned 1 unit"],
		"no MOVE queued, so the spawn runs in the input frame"
	);
	let next = until_ask(&mut game);
	assert!(next.summary.is_empty());
	assert_eq!(game.board.units[2][0], 2);
	assert_eq!(triangle_owner(&game, [0, 1, 2]), None);
	let id = game.board.triangles[0];
	assert!(!game.board.triangle(id).can_capture[0]);
}

#[test]
fn surrounded_units_die() {
	let game = diamond("0,1_1,0_1,0_0,0_0,1", 1);
	assert_eq!(
		game.board.units[0][1], 0,
		"node 0 only touches nodes 1 and 2"
	);
	assert_eq!(
		game.board.owner(0),
		Some(0),
		"an empty node surrounded by one side belongs to it"
	);

	let mut game = diamond("0,0_1,0_1,0_0,0_0,2", 1);
	play(&mut game, ["WAIT", "MOVE 4 0 1"]);
	let next = until_ask(&mut game);
	assert!(next.summary.is_empty(), "{:?}", next.summary);
	assert_eq!(
		game.board.units[2][1], 1,
		"node 2 touches 3 and 4, not all held by side 0"
	);
}

#[test]
fn parser_follows_the_java_patterns() {
	let mut game = diamond("1,0_0,0_0,0_0,0_0,1", 1);
	let summary = play(
		&mut game,
		[
			"  move 0 1 1  ;MSG hello;WAIT;TEXT 1 -2 any text;LINE 0 0 1 1;MOVE 9 1 1;MOVE 0 1 0;;",
			"WAIT",
		],
	);
	assert_eq!(
		summary,
		[
			"$0: Invalid node: 9",
			"$0: Amount of units to move must be positive"
		]
	);
	assert_eq!(game.players[0].message, "hello");
	assert!(game.players[0].active);
}

#[test]
fn malformed_or_overflowing_commands_are_fatal() {
	for (command, message) in [
		("MOVE 0 1 1x", "Unknown command: MOVE 0 1 1x"),
		(
			"MOVE 0 1 99999999999",
			"Unknown command: MOVE 0 1 99999999999",
		),
		(" MSG late", "Unknown command:  MSG late"),
		("CIRCLE 1 2", "Unknown command: CIRCLE 1 2"),
	] {
		let mut game = diamond("1,0_0,0_0,0_0,0_0,1", 1);
		let summary = play(&mut game, [command, "WAIT"]);
		assert_eq!(summary, [format!("$0: {message}")], "{command}");
		assert_eq!(game.scores()[0], -1);
		assert!(game.ended);
		assert_eq!(game.tooltips, [format!("$0 {message}!")]);
	}
}

#[test]
fn errors_past_three_of_a_kind_are_folded() {
	let mut game = diamond("1,0_0,0_0,0_0,0_0,1", 1);
	let summary = play(
		&mut game,
		["MOVE 9 1 1;MOVE 8 1 1;MOVE 7 1 1;MOVE 6 1 1", "WAIT"],
	);
	assert_eq!(
		summary,
		[
			"$0: Invalid node: 9",
			"$0: Invalid node: 8",
			"$0: 2 more errors of that type"
		]
	);
}

#[test]
fn attack_needs_league_two() {
	let units = "1,0_1,0_1,0_0,2_0,0";
	let mut wood = diamond(units, 1);
	let summary = play(&mut wood, ["ATTACK 0 1 2 3", "WAIT"]);
	assert_eq!(
		summary,
		["$0: Command not available in your league: ATTACK 0 1 2 3"]
	);

	let mut silver = diamond(units, 2);
	play(&mut silver, ["ATTACK 0 1 2 3", "WAIT"]);
	assert_eq!(silver.board.units[3][1], 0);
	assert_eq!(
		silver.board.units[0][0], 0,
		"the attack costs one unit per corner"
	);
	assert_eq!(triangle_owner(&silver, [0, 1, 2]), None);
}

#[test]
fn paths_can_be_added_and_removed_from_league_three() {
	let units = "1,0_1,0_1,0_0,0_0,1";
	let mut game = diamond(units, 3);
	let turn = until_ask(&mut game);
	answer(&mut game, &turn, ["REMOVE_PATH 1 0 2 3", "WAIT"]);
	until_ask(&mut game);
	assert!(!game.board.connected(1, 3));
	let corners: Vec<[usize; 3]> = game
		.board
		.triangles
		.iter()
		.map(|&t| game.board.triangle(t).nodes)
		.collect();
	assert_eq!(corners, [[0, 1, 2], [2, 3, 4]]);
}

#[test]
fn linkable_paths_only_from_league_three() {
	let board = || {
		from_options(
			DIAMOND_NODES,
			"0,1_1,2_0,2_2,3_2,4_3,4",
			"1,0_0,0_0,0_0,0_0,1",
		)
		.unwrap()
	};
	assert!(tryangle::proto::linkable(&Game::new(board(), 1)).is_empty());
	let links = tryangle::proto::linkable(&Game::new(board(), 3));
	assert!(links.contains(&(1, 3)), "{links:?}");
	assert!(!links.contains(&(0, 4)), "longer than 400: {links:?}");
}
