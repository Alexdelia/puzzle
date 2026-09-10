use btk::game::{Action, Game, NEUTRAL, NO_TRACK, Rules, parse_actions};
use btk::map::{Map, PLAIN, Town, format_map, parse_map, validate};
use btk::mapgen::{GenParams, generate};
use btk::proto::{init_text, turn_text};
use btk::rng::Rng;

const SEEDS: u64 = 20000;

#[test]
fn generated_maps_respect_every_stated_constraint() {
	for seed in 0..SEEDS {
		let map = generate(seed, &GenParams::default());
		validate(&map, true).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
	}
}

#[test]
fn generation_is_reproducible() {
	for seed in [0, 1, 42, 9999] {
		let a = format_map(&generate(seed, &GenParams::default()));
		let b = format_map(&generate(seed, &GenParams::default()));
		assert_eq!(a, b, "seed {seed}");
	}
}

#[test]
fn maps_survive_a_text_round_trip() {
	for seed in 0..200 {
		let map = generate(seed, &GenParams::default());
		let text = format_map(&map);
		let back = parse_map(&text).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
		assert_eq!(text, format_map(&back), "seed {seed}");
	}
}

#[test]
fn forced_shapes_are_honoured() {
	for width in [21, 25, 30] {
		for height in [14, 17, 20] {
			for towns in 4..=12 {
				let shape = GenParams {
					width: Some(width),
					height: Some(height),
					towns: Some(towns),
				};
				for seed in 0..40 {
					let map = generate(seed, &shape);
					assert_eq!(
						(map.width, map.height, map.towns.len()),
						(width, height, towns)
					);
					validate(&map, true)
						.unwrap_or_else(|e| panic!("{width}x{height} t{towns} seed {seed}: {e}"));
				}
			}
		}
	}
}

#[test]
fn protocol_blocks_have_the_documented_shape() {
	let map = generate(3, &GenParams::default());
	let game = Game::new(map, Rules::default());
	let init_block = init_text(&game.map, 1);
	let init: Vec<&str> = init_block.lines().collect();
	assert_eq!(init.len(), 3 + game.map.cells() + 1 + game.map.towns.len());
	assert_eq!(init[0], "1");
	assert_eq!(init[1], game.map.width.to_string());
	assert_eq!(init[2], game.map.height.to_string());

	let mut text = String::new();
	turn_text(&game.map, &game.state, &game.conn, 0, &mut text);
	let turn: Vec<&str> = text.lines().collect();
	assert_eq!(turn.len(), 2 + game.map.cells());
	for row in &turn[2..] {
		assert_eq!(row.split_whitespace().count(), 4);
	}
}

#[test]
fn a_turn_never_spends_more_than_the_paint_budget() {
	let mut rng = Rng::new(77);
	for seed in 0..300 {
		let map = generate(seed, &GenParams::default());
		let budget = Rules::default().paint;
		let mut game = Game::new(map, Rules::default());
		while !game.over() {
			let town = &game.map.towns[rng.below(game.map.towns.len())];
			let other = &game.map.towns[rng.below(game.map.towns.len())];
			let actions = vec![Action::Autoplace(town.x, town.y, other.x, other.y)];
			let before: u32 = spent(&game);
			game.step([&actions, &[Action::Wait]]).unwrap();
			assert!(spent(&game) - before <= budget, "seed {seed}");
		}
	}
}

fn spent(game: &Game) -> u32 {
	(0..game.map.cells())
		.filter(|&c| game.state.owner[c] != NO_TRACK)
		.map(|c| game.map.cost(c))
		.sum()
}

#[test]
fn tracks_never_land_on_towns_or_overwrite_each_other() {
	let mut rng = Rng::new(11);
	for seed in 0..300 {
		let map = generate(seed, &GenParams::default());
		let mut game = Game::new(map, Rules::default());
		let mut owned = vec![NO_TRACK; game.map.cells()];
		while !game.over() {
			let actions: [Vec<Action>; 2] = std::array::from_fn(|_| {
				let a = &game.map.towns[rng.below(game.map.towns.len())];
				let b = &game.map.towns[rng.below(game.map.towns.len())];
				vec![Action::Autoplace(a.x, a.y, b.x, b.y)]
			});
			game.step([&actions[0], &actions[1]]).unwrap();
			for (cell, was) in owned.iter_mut().enumerate() {
				let now = game.state.owner[cell];
				assert!(now == NO_TRACK || game.map.town_at[cell] < 0, "seed {seed}");
				if *was != NO_TRACK {
					assert_eq!(now, *was, "seed {seed} cell {cell} changed hands");
				}
				*was = now;
			}
		}
	}
}

fn duel_map() -> Map {
	Map::new(
		5,
		1,
		vec![PLAIN; 5],
		vec![0, 1, 2, 3, 4],
		vec![
			Town {
				x: 0,
				y: 0,
				desired: vec![1],
			},
			Town {
				x: 4,
				y: 0,
				desired: vec![],
			},
		],
	)
}

#[test]
fn a_cell_both_players_paint_turns_neutral() {
	let map = duel_map();
	let mut game = Game::new(map, Rules::default());
	let both = vec![Action::Place(2, 0)];
	let report = game.step([&both, &both]).unwrap();
	assert_eq!(game.state.owner[2], NEUTRAL);
	assert_eq!(report.placed, [vec![2], vec![2]]);
}

#[test]
fn a_refused_placement_is_a_no_op_but_a_foul_under_strict() {
	let mut game = Game::new(duel_map(), Rules::default());
	let onto_town = vec![Action::Place(0, 0)];
	let report = game.step([&onto_town, &[Action::Wait]]).unwrap();
	assert!(report.placed[0].is_empty());
	assert_eq!(report.skipped[0].len(), 1);

	let strict = Rules {
		strict: true,
		..Rules::default()
	};
	let mut game = Game::new(duel_map(), strict);
	let foul = game.step([&onto_town, &[Action::Wait]]).unwrap_err();
	assert_eq!(foul.player, 0);
}

#[test]
fn autoplace_stops_once_a_path_exists() {
	let mut game = Game::new(duel_map(), Rules::default());
	let build = vec![Action::Autoplace(0, 0, 4, 0)];
	let first = game.step([&build, &[Action::Wait]]).unwrap();
	assert_eq!(first.placed[0], vec![1, 2, 3]);
	assert_eq!(game.state.score, [3, 0]);

	let second = game.step([&build, &[Action::Wait]]).unwrap();
	assert!(second.placed[0].is_empty());
	assert_eq!(game.state.score, [6, 0]);
}

#[test]
fn paint_runs_out_mid_path() {
	let mut terrain = vec![PLAIN; 5];
	terrain[2] = 2;
	let map = Map::new(
		5,
		1,
		terrain,
		vec![0, 1, 2, 3, 4],
		vec![
			Town {
				x: 0,
				y: 0,
				desired: vec![1],
			},
			Town {
				x: 4,
				y: 0,
				desired: vec![],
			},
		],
	);
	let mut game = Game::new(map, Rules::default());
	let build = vec![Action::Autoplace(0, 0, 4, 0)];
	let first = game.step([&build, &[Action::Wait]]).unwrap();
	assert_eq!(first.placed[0], vec![1]);
	let second = game.step([&build, &[Action::Wait]]).unwrap();
	assert_eq!(second.placed[0], vec![2]);
	let third = game.step([&build, &[Action::Wait]]).unwrap();
	assert_eq!(third.placed[0], vec![3]);
	assert_eq!(game.state.score, [3, 0]);
}

#[test]
fn disruption_inks_a_region_and_washes_its_tracks() {
	let rules = Rules {
		disruption: true,
		..Rules::default()
	};
	let mut game = Game::new(duel_map(), rules);
	game.step([&[Action::Place(2, 0)], &[Action::Wait]])
		.unwrap();
	assert_eq!(game.state.owner[2], 0);

	let disrupt = vec![Action::DisruptRegion(2)];
	for _ in 0..2 {
		let report = game.step([&disrupt, &disrupt]).unwrap();
		let _ = report;
	}
	assert!(game.state.inked[2]);
	assert_eq!(game.state.owner[2], NO_TRACK);

	let refused = game
		.step([&[Action::DisruptRegion(0)], &[Action::Wait]])
		.unwrap();
	assert_eq!(refused.skipped[0].len(), 1);
}

#[test]
fn disruption_is_inert_by_default() {
	let mut game = Game::new(duel_map(), Rules::default());
	let disrupt = vec![Action::DisruptRegion(2)];
	for _ in 0..6 {
		game.step([&disrupt, &disrupt]).unwrap();
	}
	assert_eq!(game.state.instability[2], 0);
	assert!(!game.state.inked[2]);
}

#[test]
fn the_action_grammar_matches_the_statement() {
	assert_eq!(parse_actions("WAIT").unwrap(), vec![Action::Wait]);
	assert_eq!(
		parse_actions("PLACE_TRACKS 3 4").unwrap(),
		vec![Action::Place(3, 4)]
	);
	assert_eq!(
		parse_actions("DISRUPT 7").unwrap(),
		vec![Action::DisruptRegion(7)]
	);
	assert_eq!(
		parse_actions("DISRUPT 7 8").unwrap(),
		vec![Action::DisruptCell(7, 8)]
	);
	assert_eq!(
		parse_actions("AUTOPLACE 1 2 3 4;MESSAGE hi there").unwrap(),
		vec![
			Action::Autoplace(1, 2, 3, 4),
			Action::Message("hi there".into())
		]
	);
	assert!(parse_actions("AUTOPLACE 1 2 3 4;AUTOPLACE 5 6 7 8").is_err());
	assert!(parse_actions("").is_err());
	assert!(parse_actions("PLACE_TRACKS 1").is_err());
	assert!(parse_actions("PLACE_TRACKS -1 2").is_err());
	assert!(parse_actions("FLY 1 2").is_err());
}

#[test]
fn broken_map_files_are_rejected_not_panicked_on() {
	let cases = [
		"size 5 1\nterrain\n.....\nregions\n0 0 0 0 0\ntowns 1\n0 0 5\n",
		"size 5 1\nterrain\n.....\nregions\n0 0 0 0 0\ntowns 1\n0 0 0\n",
		"size 5 1\nterrain\n..^..\nregions\n0 0 0 0 0\ntowns 2\n2 0 1\n4 0 0\n",
		"size 5 1\nterrain\n.....\nregions\n0 0 0 0 0\ntowns 2\n0 0 1\n9 0 0\n",
		"size 5 1\nterrain\n.....\nregions\n0 1 0 1 0\ntowns 1\n0 0 x\n",
	];
	for case in cases {
		assert!(
			parse_map(case).is_err(),
			"should have been rejected:\n{case}"
		);
	}
}

#[test]
fn running_out_of_paint_leaves_the_disruption_point_and_the_message_alone() {
	let map = Map::new(
		8,
		1,
		vec![PLAIN; 8],
		vec![0, 1, 2, 3, 4, 5, 6, 7],
		vec![
			Town {
				x: 0,
				y: 0,
				desired: vec![1],
			},
			Town {
				x: 7,
				y: 0,
				desired: vec![],
			},
		],
	);
	let rules = Rules {
		disruption: true,
		..Rules::default()
	};
	let mut game = Game::new(map, rules);
	let line = vec![
		Action::Autoplace(0, 0, 7, 0),
		Action::DisruptRegion(5),
		Action::Message("gg".into()),
	];
	let report = game.step([&line, &[Action::Wait]]).unwrap();
	assert_eq!(report.placed[0], vec![1, 2, 3]);
	assert_eq!(game.state.instability[5], 1);
	assert_eq!(report.messages[0].as_deref(), Some("gg"));
}

#[test]
fn an_idle_turn_still_scores_the_standing_connections() {
	let mut game = Game::new(duel_map(), Rules::default());
	game.step([&[Action::Autoplace(0, 0, 4, 0)], &[Action::Wait]])
		.unwrap();
	assert_eq!(game.state.score, [3, 0]);
	for turn in 2..=5 {
		let report = game.step([&[Action::Wait], &[Action::Wait]]).unwrap();
		assert_eq!(report.gained, [3, 0], "turn {turn}");
	}
	assert_eq!(game.state.score, [15, 0]);
}

#[test]
fn generated_regions_always_hold_at_least_two_cells() {
	for seed in 0..2000 {
		let map = generate(seed, &GenParams::default());
		let mut size = vec![0usize; map.region_count];
		for cell in 0..map.cells() {
			size[map.region_of(cell)] += 1;
		}
		assert!(size.iter().all(|&n| n >= 2), "seed {seed}: {size:?}");
	}
}
