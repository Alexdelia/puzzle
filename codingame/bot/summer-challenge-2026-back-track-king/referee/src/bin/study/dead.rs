use crate::overview::print_grid;
use crate::view::{View, percent, section};

const WIDE: usize = 34;
use btk::grid::{Coord, Grid, coord};
use btk::replay::{Pair, TurnRecord, town_char};
use std::collections::BTreeMap;

const UNREACHED: i32 = i32::MAX;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fate {
	Orphan,
	Beaten(i32),
	TieLost,
}

impl Fate {
	fn key(self) -> (i32, i32) {
		match self {
			Fate::Orphan => (0, 0),
			Fate::Beaten(excess) => (1, -excess),
			Fate::TieLost => (2, 0),
		}
	}

	fn glyph(self) -> char {
		match self {
			Fate::Orphan => 'o',
			Fate::Beaten(_) => 'b',
			Fate::TieLost => 't',
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Origin {
	Fresh,
	OwnTrack,
	FoeTrack,
}

struct DeadCell {
	owner: usize,
	turn: i32,
	paint: i32,
	origin: Origin,
	fate: Fate,
	beaten_by_foe: bool,
}

pub fn dead_claims(view: &View) {
	let run = view.run;
	section("why each dead claim died");
	println!("a cell is paid only when the referee income walk crosses it, so a claim that is off");
	println!("every chosen path pays nothing however long it is held");

	let mut cells = collect(view);
	for record in view.turns() {
		let dist = town_distances(&run.grid, record);
		for (&at, cell) in cells.iter_mut() {
			if record.owner(at) != cell.owner as i8 {
				continue;
			}
			verdict(&run.grid, record, &dist, at, cell);
		}
	}

	report(view, &cells);
	runs(view, &cells);
	fate_map(view, &cells);
}

const MIN_RUN: usize = 3;
const TOP_RUNS: usize = 8;

fn runs(view: &View, cells: &BTreeMap<Coord, DeadCell>) {
	let run = view.run;
	let mine: BTreeMap<Coord, &DeadCell> = cells
		.iter()
		.filter(|(_, cell)| cell.owner == view.me)
		.map(|(&at, cell)| (at, cell))
		.collect();
	let mut seen: std::collections::BTreeSet<Coord> = std::collections::BTreeSet::new();
	let mut found: Vec<(usize, i32, i32, i32, bool)> = Vec::new();
	for &start in mine.keys() {
		if !seen.insert(start) {
			continue;
		}
		let mut queue = std::collections::VecDeque::from([start]);
		let mut size = 0usize;
		let mut first = i32::MAX;
		let mut last = 0;
		let mut paint = 0;
		let mut near_town = false;
		while let Some(at) = queue.pop_front() {
			let cell = mine[&at];
			size += 1;
			first = first.min(cell.turn);
			last = last.max(cell.turn);
			paint += cell.paint;
			for next in run.grid.neighbours(at) {
				near_town |= run.grid.tile(next).is_town();
				if mine.contains_key(&next) && seen.insert(next) {
					queue.push_back(next);
				}
			}
		}
		found.push((size, first, last, paint, near_town));
	}
	found.sort_unstable_by(|a, b| b.cmp(a));
	let runs_of = |least: usize| found.iter().filter(|entry| entry.0 >= least).count();
	println!(
		"\n{} dead runs, {} of them {MIN_RUN} cells or more",
		found.len(),
		runs_of(MIN_RUN)
	);
	println!("cells  built over  paint  touches a town");
	for &(size, first, last, paint, near_town) in found.iter().take(TOP_RUNS) {
		if size < MIN_RUN {
			break;
		}
		println!(
			"{size:>5}  {:>10}  {paint:>5}  {}",
			format!("t{first}-{last}"),
			if near_town { "yes" } else { "no" }
		);
	}
}

fn collect(view: &View) -> BTreeMap<Coord, DeadCell> {
	let run = view.run;
	let mut cells = BTreeMap::new();
	for claim in &run.totals.claims {
		let Some(owner) = claim.player() else {
			continue;
		};
		if run.totals.paid_for(claim.at)[owner] != 0 {
			continue;
		}
		let origin = match (
			claim.touching[owner],
			claim.touching[1 - owner] || claim.touching[2],
		) {
			(_, true) => Origin::FoeTrack,
			(true, false) => Origin::OwnTrack,
			(false, false) => Origin::Fresh,
		};
		cells.entry(claim.at).or_insert(DeadCell {
			owner,
			turn: claim.turn,
			paint: run.grid.tile(claim.at).rail_cost(),
			origin,
			fate: Fate::Orphan,
			beaten_by_foe: false,
		});
	}
	cells
}

fn verdict(grid: &Grid, record: &TurnRecord, dist: &[Vec<i32>], at: Coord, cell: &mut DeadCell) {
	let index = grid.index(at);
	for town in &grid.towns {
		for &other in &town.desired {
			let goal = grid.index(grid.towns[other].coord);
			let span = dist[town.id][goal];
			let here = dist[town.id][index];
			let back = dist[other][index];
			if span == UNREACHED || here == UNREACHED || back == UNREACHED {
				continue;
			}
			let excess = here + back - span;
			let fate = if excess == 0 {
				Fate::TieLost
			} else {
				Fate::Beaten(excess)
			};
			if fate.key() > cell.fate.key() {
				cell.fate = fate;
				cell.beaten_by_foe = foe_owns(record, (town.id, other), cell.owner);
			}
		}
	}
}

fn foe_owns(record: &TurnRecord, pair: Pair, owner: usize) -> bool {
	let Some(path) = record.paths.get(&pair) else {
		return false;
	};
	let owned = record.owners_of(path);
	owned[1 - owner] > owned[owner]
}

fn town_distances(grid: &Grid, record: &TurnRecord) -> Vec<Vec<i32>> {
	let passable: Vec<bool> = (0..grid.cells())
		.map(|cell| {
			let at = grid.coord_of(cell);
			grid.tile(at).is_town() || record.owner(at) != -1
		})
		.collect();
	grid.towns
		.iter()
		.map(|town| flood(grid, &passable, town.coord))
		.collect()
}

fn flood(grid: &Grid, passable: &[bool], from: Coord) -> Vec<i32> {
	let mut dist = vec![UNREACHED; grid.cells()];
	let mut queue = std::collections::VecDeque::from([from]);
	dist[grid.index(from)] = 0;
	while let Some(at) = queue.pop_front() {
		let step = dist[grid.index(at)] + 1;
		for next in grid.neighbours(at) {
			let cell = grid.index(next);
			if passable[cell] && dist[cell] == UNREACHED {
				dist[cell] = step;
				queue.push_back(next);
			}
		}
	}
	dist
}

fn report(view: &View, cells: &BTreeMap<Coord, DeadCell>) {
	let run = view.run;
	view.heading();
	let mut dead = [0i32; 2];
	let mut orphan = [0i32; 2];
	let mut beaten = [0i32; 2];
	let mut tie = [0i32; 2];
	let mut foe_took = [0i32; 2];
	let mut excess = [Vec::new(), Vec::new()];
	let mut when = [Vec::new(), Vec::new()];
	let mut sunk = [0i32; 2];
	for cell in cells.values() {
		let side = cell.owner;
		dead[side] += 1;
		when[side].push(cell.turn);
		sunk[side] += cell.paint;
		match cell.fate {
			Fate::Orphan => orphan[side] += 1,
			Fate::Beaten(steps) => {
				beaten[side] += 1;
				excess[side].push(steps);
			}
			Fate::TieLost => tie[side] += 1,
		}
		if cell.beaten_by_foe {
			foe_took[side] += 1;
		}
	}

	view.count_row("dead claims", dead);
	view.count_row("  never on a linked pair", orphan);
	view.count_row("  on a beaten route", beaten);
	view.count_row("  on a tie-losing path", tie);
	view.row(
		"  reachable but unpaid",
		[0, 1].map(|side| {
			format!(
				"{:.0}%",
				percent((beaten[side] + tie[side]) as i64, dead[side] as i64)
			)
		}),
	);
	view.row(
		"median extra steps",
		[0, 1].map(|side| median(&excess[side])),
	);
	view.row(
		"winning path was the foe's",
		[0, 1].map(|side| {
			format!(
				"{:.0}%",
				percent(foe_took[side] as i64, (beaten[side] + tie[side]) as i64)
			)
		}),
	);
	view.row("median claim turn", [0, 1].map(|side| median(&when[side])));
	view.count_row("paint sunk in dead claims", sunk);
	view.row(
		"  share of paint spent",
		[0, 1].map(|side| {
			format!(
				"{:.0}%",
				percent(sunk[side] as i64, run.totals.paint_spent[side] as i64)
			)
		}),
	);

	println!("\nwhere the dead claims were laid, by fate");
	println!(
		"{:<WIDE$} {:>9} {:>9} {:>9}",
		"", "fresh", "own track", "foe track"
	);
	for (label, want) in [
		("off every linked pair", 0),
		("beaten route", 1),
		("tie lost", 2),
	] {
		for side in view.sides() {
			let mut counts = [0i32; 3];
			for cell in cells.values().filter(|cell| cell.owner == side) {
				if cell.fate.key().0 != want {
					continue;
				}
				counts[match cell.origin {
					Origin::Fresh => 0,
					Origin::OwnTrack => 1,
					Origin::FoeTrack => 2,
				}] += 1;
			}
			println!(
				"{:<WIDE$} {:>9} {:>9} {:>9}",
				format!("{label} ({})", view.name(side)),
				counts[0],
				counts[1],
				counts[2]
			);
		}
	}
}

fn median(values: &[i32]) -> String {
	if values.is_empty() {
		return "-".to_string();
	}
	let mut sorted = values.to_vec();
	sorted.sort_unstable();
	sorted[sorted.len() / 2].to_string()
}

fn fate_map(view: &View, cells: &BTreeMap<Coord, DeadCell>) {
	println!("dead claims by fate (o = off every linked pair, b = beaten route, t = tie lost;");
	println!("lowercase ours, uppercase theirs)");
	print_grid(&view.run.grid, |x, y| {
		let at = coord(x, y);
		let tile = view.run.grid.tile(at);
		if tile.is_town() {
			return town_char(tile.town as usize);
		}
		match cells.get(&at) {
			Some(cell) if cell.owner == view.me => cell.fate.glyph(),
			Some(cell) => cell.fate.glyph().to_ascii_uppercase(),
			None => '.',
		}
	});
}
