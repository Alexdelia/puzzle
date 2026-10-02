use block_the_spreading_fire::{Action, Game, Map, NO_FIRE, SAFE};
use std::fs;

const DEMO_MAP: &str = "validator/test/07_big_map_with_villages_and_wall.txt";
const DEMO_FIXTURE: &str = "tests/fixtures/demo_07.txt";

struct Frame {
	turn: u32,
	action: Action,
	remaining: u32,
	burnt: u32,
	cut: u32,
	active: usize,
	cooldown: u32,
	changes: Vec<(usize, i8)>,
}

fn parse_frames(map: &Map, text: &str) -> Vec<Frame> {
	let mut frames: Vec<Frame> = Vec::new();
	for line in text.lines() {
		let tokens = line.split_whitespace().collect::<Vec<_>>();
		if tokens[0] != "turn" {
			let x: usize = tokens[0].parse().unwrap();
			let y: usize = tokens[1].parse().unwrap();
			frames
				.last_mut()
				.unwrap()
				.changes
				.push((map.index(x, y), tokens[2].parse().unwrap()));
			continue;
		}
		let action_tokens = if tokens[2] == "WAIT" { 1 } else { 2 };
		let action = Action::parse(&tokens[2..2 + action_tokens].join(" "), map).unwrap();
		let stats = tokens[2 + action_tokens..]
			.iter()
			.map(|t| t.parse::<u32>().unwrap())
			.collect::<Vec<_>>();
		frames.push(Frame {
			turn: tokens[1].parse().unwrap(),
			action,
			remaining: stats[0],
			burnt: stats[1],
			cut: stats[2],
			active: stats[3] as usize,
			cooldown: stats[4],
			changes: Vec::new(),
		});
	}
	frames
}

fn assert_frame(game: &Game, frame: &Frame, expected: &[i8]) {
	let map = game.map();
	let mismatches = (0..expected.len())
		.filter(|&i| game.progress()[i] != expected[i])
		.map(|i| (map.coords(i), game.progress()[i], expected[i]))
		.take(5)
		.collect::<Vec<_>>();
	assert!(
		mismatches.is_empty(),
		"turn {turn}: (cell, got, expected) {mismatches:?}",
		turn = frame.turn
	);
	let stats = |g: &Game| {
		(
			g.remaining_value(),
			g.burnt_value(),
			g.cut_value(),
			g.active_fires(),
			g.cooldown(),
		)
	};
	assert_eq!(
		stats(game),
		(
			frame.remaining,
			frame.burnt,
			frame.cut,
			frame.active,
			frame.cooldown
		),
		"turn {turn}: (remaining, burnt, cut, active, cooldown)",
		turn = frame.turn
	);
}

#[test]
fn replays_codingame_demo_frame_by_frame() {
	let map = Map::parse(&fs::read_to_string(DEMO_MAP).unwrap()).unwrap();
	let frames = parse_frames(&map, &fs::read_to_string(DEMO_FIXTURE).unwrap());
	assert_eq!(frames.len(), 80);

	let mut expected = map
		.cells
		.iter()
		.map(|&c| {
			if c == block_the_spreading_fire::Cell::Safe {
				SAFE
			} else {
				NO_FIRE
			}
		})
		.collect::<Vec<_>>();
	let mut game = Game::new(&map);
	for frame in &frames {
		if frame.turn > 0 {
			game.play(frame.action)
				.unwrap_or_else(|fault| panic!("turn {turn}: {fault}", turn = frame.turn));
		}
		if let Action::Cut(i) = frame.action {
			expected[i] = SAFE;
		}
		for &(i, progress) in &frame.changes {
			expected[i] = progress;
		}
		assert_frame(&game, frame, &expected);
	}
	assert!(game.is_over());
	assert_eq!(game.remaining_value(), 6068);
}
