use super::Pair;
use super::board::render;
use super::probe::Probe;
use super::record::{ActivePair, Claim, Replay, Totals, TurnRecord, ZoneLook};
use crate::game::{DEFAULT_LEAGUE, Game, PASSIVE_INCOME, Stats, TurnReport};
use crate::grid::{Coord, Grid};
use std::collections::BTreeMap;

pub fn run(grid: Grid, answers: &[[String; 2]]) -> Replay {
	run_probed(grid, answers, None)
}

pub fn run_probed(grid: Grid, answers: &[[String; 2]], mut probe: Option<&mut Probe>) -> Replay {
	let mut game = Game::new(grid, DEFAULT_LEAGUE);
	let mut frame = String::new();
	if let Some(probe) = probe.as_deref_mut() {
		probe.start(&game.grid, &mut frame);
	}
	let mut turns = Vec::with_capacity(answers.len());
	let mut totals = Totals::default();
	let mut seen_errors = 0;
	let mut last_stats = game.stats.clone();

	for pair in answers {
		if game.ended || game.active_players() < 2 {
			break;
		}
		if let Some(probe) = probe.as_deref_mut() {
			probe.ask(&game, &mut frame);
		}
		let before = look_at_zones(&game);
		let tracks_before: Vec<i8> = game.grid.tiles.iter().map(|tile| tile.track).collect();

		game.reset_turn_data();
		for (player, line) in pair.iter().enumerate() {
			game.take_commands(player, line);
		}
		let report = game.perform_update();

		let wiped = count_wiped(&game, &report, &tracks_before);
		let active = collect_paths(&game, &mut totals);
		collect_claims(&game, &report, &tracks_before, &mut totals);
		let placed = add_turn_totals(&game, &report, &wiped, &last_stats, &mut totals);
		last_stats = game.stats.clone();

		turns.push(TurnRecord {
			turn: game.turn,
			answers: pair.clone(),
			placed,
			paint_left: [game.players[0].dosh, game.players[1].dosh],
			disrupted: report.disrupted,
			inked: report.inked.clone(),
			wiped,
			gained: report.gained,
			score: [game.players[0].score, game.players[1].score],
			errors: game.summary[seen_errors..].to_vec(),
			pairs: active.pairs,
			paths: active.paths,
			active_cells: active.cells,
			before,
			board: render(&game),
			tracks: game.grid.tiles.iter().map(|tile| tile.track).collect(),
			width: game.grid.width,
		});
		seen_errors = game.summary.len();
	}

	Replay {
		score: [game.players[0].score, game.players[1].score],
		ended: game.ended,
		disqualified: [
			game.players[0].deactivated_by.clone(),
			game.players[1].deactivated_by.clone(),
		],
		grid: game.grid,
		turns,
		totals,
	}
}

struct ActiveTurn {
	pairs: Vec<ActivePair>,
	paths: BTreeMap<Pair, Vec<Coord>>,
	cells: [i32; 4],
}

fn collect_paths(game: &Game, totals: &mut Totals) -> ActiveTurn {
	let mut pairs = Vec::new();
	let mut paths: BTreeMap<Pair, Vec<Coord>> = BTreeMap::new();
	let mut on_path: BTreeMap<Coord, i8> = BTreeMap::new();

	for town in &game.grid.towns {
		for (&other, path) in &town.paths {
			let pair = (town.id, other);
			let mut owned = [0i32; 2];
			for &at in path {
				let owner = game.grid.tile(at).track;
				on_path.insert(at, owner);
				if owner == 0 || owner == 1 {
					owned[owner as usize] += 1;
					totals.cell_pay.entry(at).or_default()[owner as usize] += 1;
				}
			}

			let entry = totals.pay.entry(pair).or_default();
			entry.turns += 1;
			entry.length_sum += path.len() as i64;
			entry.paid[0] += owned[0] as i64;
			entry.paid[1] += owned[1] as i64;

			pairs.push(ActivePair {
				pair,
				length: path.len(),
				owned,
			});
			paths.insert(pair, path.clone());
		}
	}

	let mut cells = [0i32; 4];
	for &owner in on_path.values() {
		cells[match owner {
			0..=2 => owner as usize,
			_ => 3,
		}] += 1;
	}
	ActiveTurn {
		pairs,
		paths,
		cells,
	}
}

fn collect_claims(game: &Game, report: &TurnReport, tracks_before: &[i8], totals: &mut Totals) {
	for &(at, owner) in &report.built {
		let mut touching = [false; 3];
		let mut near_town = false;
		for next in game.grid.neighbours(at) {
			let was = tracks_before[game.grid.index(next)];
			if (0..=2).contains(&was) {
				touching[was as usize] = true;
			}
			near_town |= game.grid.tile(next).is_town();
		}
		totals.claims.push(Claim {
			at,
			turn: game.turn,
			owner,
			touching,
			near_town,
		});
	}
}

fn count_wiped(
	game: &Game,
	report: &TurnReport,
	tracks_before: &[i8],
) -> BTreeMap<usize, [i32; 3]> {
	let mut wiped = BTreeMap::new();
	for &zone in &report.inked {
		let mut counts = [0i32; 3];
		for &at in &game.grid.zones[zone].coords {
			let owner = tracks_before[game.grid.index(at)];
			if (0..=2).contains(&owner) {
				counts[owner as usize] += 1;
			}
		}
		wiped.insert(zone, counts);
	}
	wiped
}

fn add_turn_totals(
	game: &Game,
	report: &TurnReport,
	wiped: &BTreeMap<usize, [i32; 3]>,
	before: &Stats,
	totals: &mut Totals,
) -> [i32; 2] {
	let now = &game.stats;
	let placed = [
		now.placed_tracks[0] - before.placed_tracks[0],
		now.placed_tracks[1] - before.placed_tracks[1],
	];
	for (player, &count) in placed.iter().enumerate() {
		totals.placed[player] += count;
		totals.plains[player] += now.on_plains[player] - before.on_plains[player];
		totals.river[player] += now.on_river[player] - before.on_river[player];
		totals.mountain[player] += now.on_mountains[player] - before.on_mountains[player];
		totals.paint_spent[player] += PASSIVE_INCOME - game.players[player].dosh;
		totals.paint_lost[player] += game.players[player].dosh;
		if report.disrupted[player].is_some() {
			totals.disrupts[player] += 1;
		} else {
			totals.blots_lost[player] += 1;
		}
	}
	for (&zone, counts) in wiped {
		for player in 0..2 {
			if report.disrupted[player] == Some(zone) {
				totals.inked_by[player] += 1;
				totals.wiped_own[player] += counts[player];
				totals.wiped_foe[player] += counts[1 - player];
			}
		}
	}
	placed
}

fn look_at_zones(game: &Game) -> BTreeMap<usize, ZoneLook> {
	let mut looks = BTreeMap::new();
	for zone in &game.grid.zones {
		let mut look = ZoneLook {
			instability: zone.instability,
			inked: zone.inked,
			has_town: !zone.towns.is_empty(),
			cells: zone.coords.len(),
			..ZoneLook::default()
		};
		for &at in &zone.coords {
			let tile = game.grid.tile(at);
			let owned = (0..=2).contains(&tile.track);
			if owned {
				look.tracks[tile.track as usize] += 1;
			}
			for &(from, to) in &tile.connections {
				let pair = (from as usize, to as usize);
				if !look.pairs.contains(&pair) {
					look.pairs.push(pair);
				}
				if owned {
					look.path_cells[tile.track as usize] += 1;
				}
			}
		}
		looks.insert(zone.id, look);
	}
	looks
}
