use crate::overview::print_grid;
use crate::view::{View, percent, section};
use btk::grid::{Coord, coord};
use btk::replay::town_char;
use std::collections::BTreeMap;

pub fn gateways(view: &View) {
	let run = view.run;
	section("town gateways");
	println!("every path in or out of a town crosses one of its free neighbours");
	println!("town  wants  wanted  gate     claimed by  on turn   paid");
	let mut wanted_by = vec![0usize; run.grid.towns.len()];
	for town in &run.grid.towns {
		for &other in &town.desired {
			wanted_by[other] += 1;
		}
	}
	for town in &run.grid.towns {
		for gate in run.grid.neighbours(town.coord) {
			if run.grid.tile(gate).is_town() {
				continue;
			}
			let claim = run.totals.claim_of(gate);
			let paid: i64 = run.totals.paid_for(gate).iter().sum();
			println!(
				"{}  {:>5} {:>7}  ({:>2},{:>2})  {:>10} {:>8}  {:>5}",
				town_char(town.id),
				town.desired.len(),
				wanted_by[town.id],
				gate.x,
				gate.y,
				claim.map_or("free", |claim| view.owner_name(claim.owner)),
				claim.map_or("-".to_string(), |claim| claim.turn.to_string()),
				paid
			);
		}
	}
}

const BUCKET: i32 = 10;
const TOP_CELLS: usize = 10;

pub fn placements(view: &View) {
	let run = view.run;
	section("what each placement was worth");
	view.heading();

	let mut payouts = [Vec::new(), Vec::new()];
	for claim in &run.totals.claims {
		let Some(player) = claim.player() else {
			continue;
		};
		payouts[player].push(run.totals.paid_for(claim.at)[player]);
	}
	let claimed = [payouts[0].len() as i32, payouts[1].len() as i32];
	let dead = payouts
		.each_ref()
		.map(|paid| paid.iter().filter(|&&pay| pay == 0).count() as i32);
	let earned: [i64; 2] = payouts.each_ref().map(|paid| paid.iter().sum());

	view.count_row("cells claimed", claimed);
	view.count_row("never paid a point", dead);
	view.row(
		"share of claims dead",
		[0, 1].map(|player| {
			format!(
				"{:.0}%",
				percent(dead[player] as i64, claimed[player] as i64)
			)
		}),
	);
	view.row("points from own cells", earned.map(|sum| sum.to_string()));
	view.ratio_row(
		"points per claim",
		[0, 1].map(|player| earned[player] as f64 / claimed[player].max(1) as f64),
		1,
	);
	let paying = [0, 1].map(|player| claimed[player] - dead[player]);
	view.count_row("cells that paid", paying);
	view.ratio_row(
		"points per paying cell",
		[0, 1].map(|player| earned[player] as f64 / paying[player].max(1) as f64),
		1,
	);
	view.count_row("cells rebuilt after ink", rebuilt(view));
	yield_split(view);

	dead_by_bucket(view);
	dead_map(view);

	for player in view.sides() {
		let mut sorted = payouts[player].clone();
		sorted.sort_unstable_by(|a, b| b.cmp(a));
		let best: i64 = sorted.iter().take(TOP_CELLS).sum();
		println!(
			"{:<3} top {TOP_CELLS} cells carry {:>4} of {:>5} points ({:.0}%)",
			view.name(player),
			best,
			earned[player],
			percent(best, earned[player])
		);
	}
}

fn yield_split(view: &View) {
	let run = view.run;
	let mut held: BTreeMap<Coord, [i32; 2]> = BTreeMap::new();
	for record in view.turns() {
		let mut seen: std::collections::BTreeSet<Coord> = std::collections::BTreeSet::new();
		for path in record.paths.values() {
			seen.extend(path.iter().copied());
		}
		for at in seen {
			if let 0 | 1 = record.owner(at) {
				held.entry(at).or_default()[record.owner(at) as usize] += 1;
			}
		}
	}

	let mut turns = [Vec::new(), Vec::new()];
	let mut crossings = [Vec::new(), Vec::new()];
	let mut claimed: std::collections::BTreeSet<Coord> = std::collections::BTreeSet::new();
	for claim in &run.totals.claims {
		let Some(player) = claim.player() else {
			continue;
		};
		let paid = run.totals.paid_for(claim.at)[player];
		if paid == 0 || !claimed.insert(claim.at) {
			continue;
		}
		let alive = held
			.get(&claim.at)
			.map_or(0, |counts| counts[player])
			.max(1);
		turns[player].push(alive as f64);
		crossings[player].push(paid as f64 / alive as f64);
	}

	view.ratio_row(
		"  turns on a path",
		[0, 1].map(|player| mean(&turns[player])),
		1,
	);
	view.ratio_row(
		"  pairs a paid turn",
		[0, 1].map(|player| mean(&crossings[player])),
		2,
	);
	for player in view.sides() {
		let mut best: Vec<(i64, Coord)> = claimed
			.iter()
			.filter_map(|&at| {
				let paid = run.totals.paid_for(at)[player];
				(paid > 0).then_some((paid, at))
			})
			.collect();
		best.sort_unstable_by(|a, b| b.cmp(a));
		best.truncate(TOP_CELLS);
		let shown: Vec<String> = best
			.iter()
			.map(|&(paid, at)| {
				let alive = held.get(&at).map_or(0, |counts| counts[player]).max(1);
				format!("{paid}={alive}x{:.1}", paid as f64 / alive as f64)
			})
			.collect();
		println!(
			"{:<3} best cells, points=turns x pairs  {}",
			view.name(player),
			shown.join("  ")
		);
	}
}

fn mean(values: &[f64]) -> f64 {
	if values.is_empty() {
		return 0.0;
	}
	values.iter().sum::<f64>() / values.len() as f64
}

fn rebuilt(view: &View) -> [i32; 2] {
	let mut times: BTreeMap<Coord, [i32; 2]> = BTreeMap::new();
	for claim in &view.run.totals.claims {
		if let Some(player) = claim.player() {
			times.entry(claim.at).or_default()[player] += 1;
		}
	}
	let mut extra = [0i32; 2];
	for count in times.values() {
		for player in 0..2 {
			extra[player] += (count[player] - 1).max(0);
		}
	}
	extra
}

fn dead_by_bucket(view: &View) {
	let run = view.run;
	println!("\nclaims that never paid, by ten-turn bucket");
	println!("turns      us dead/claims   foe dead/claims   us paint/cell  foe paint/cell");

	let mut made: BTreeMap<i32, [i32; 2]> = BTreeMap::new();
	let mut dead: BTreeMap<i32, [i32; 2]> = BTreeMap::new();
	for claim in &run.totals.claims {
		let Some(player) = claim.player() else {
			continue;
		};
		let bucket = bucket_of(claim.turn);
		made.entry(bucket).or_default()[player] += 1;
		if run.totals.paid_for(claim.at)[player] == 0 {
			dead.entry(bucket).or_default()[player] += 1;
		}
	}
	let mut paint: BTreeMap<i32, [i32; 2]> = BTreeMap::new();
	for record in view.turns() {
		let entry = paint.entry(bucket_of(record.turn)).or_default();
		for (player, spent) in entry.iter_mut().enumerate() {
			*spent += record.paint_spent(player);
		}
	}

	for (&bucket, claims) in &made {
		let unpaid = dead.get(&bucket).copied().unwrap_or([0, 0]);
		let spent = paint.get(&bucket).copied().unwrap_or([0, 0]);
		let fraction = |player: usize| format!("{}/{}", unpaid[player], claims[player]);
		let per_cell =
			|player: usize| format!("{:.2}", spent[player] as f64 / claims[player].max(1) as f64);
		println!(
			"{:>3}-{:<3} {:>10} {:>18} {:>15} {:>15}",
			bucket,
			(bucket + BUCKET - 1).min(run.last_turn()),
			fraction(view.me),
			fraction(view.foe()),
			per_cell(view.me),
			per_cell(view.foe())
		);
	}
}

fn bucket_of(turn: i32) -> i32 {
	(turn - 1) / BUCKET * BUCKET + 1
}

fn dead_map(view: &View) {
	let run = view.run;
	println!("dead claims on the map (d = ours, D = foe's, towns as letters)");
	let mut dead: BTreeMap<Coord, char> = BTreeMap::new();
	for claim in &run.totals.claims {
		let Some(player) = claim.player() else {
			continue;
		};
		if run.totals.paid_for(claim.at)[player] == 0 {
			dead.insert(claim.at, if player == view.me { 'd' } else { 'D' });
		}
	}
	print_grid(&run.grid, |x, y| {
		let at = coord(x, y);
		let tile = run.grid.tile(at);
		if tile.is_town() {
			town_char(tile.town as usize)
		} else {
			dead.get(&at).copied().unwrap_or('.')
		}
	});
}

pub fn shelter(view: &View) {
	section("building out of reach of the ink");
	println!("a region holding a town can never be disrupted, so track there is permanent");
	view.heading();
	let run = view.run;
	let mut safe = [0i32; 2];
	let mut claims = [0i32; 2];
	for claim in &run.totals.claims {
		let Some(player) = claim.player() else {
			continue;
		};
		claims[player] += 1;
		let zone = run.grid.zone_of(claim.at);
		if !zone.towns.is_empty() {
			safe[player] += 1;
		}
	}
	view.count_row("claims", claims);
	view.count_row("  in a town region", safe);
	view.row(
		"  share safe",
		[0, 1].map(|player| {
			format!(
				"{:.0}%",
				percent(safe[player] as i64, claims[player] as i64)
			)
		}),
	);
	let mut lost = [0i32; 2];
	for record in view.turns() {
		for counts in record.wiped.values() {
			lost[0] += counts[0];
			lost[1] += counts[1];
		}
	}
	view.row(
		"lost to ink per 100 claims",
		[0, 1].map(|player| {
			format!(
				"{:.0}",
				100.0 * lost[player] as f64 / claims[player].max(1) as f64
			)
		}),
	);
	view.row(
		"track traded away by ink",
		[0, 1].map(|player| {
			format!(
				"{}:{}",
				run.totals.wiped_foe[player], run.totals.wiped_own[player]
			)
		}),
	);
}

pub fn network(view: &View) {
	let run = view.run;
	section("how the network gets built");
	view.heading();

	let mut fresh = [0i32; 2];
	let mut onto_own = [0i32; 2];
	let mut onto_foe = [0i32; 2];
	let mut near_town = [0i32; 2];
	for claim in &run.totals.claims {
		let Some(player) = claim.player() else {
			continue;
		};
		if claim.near_town {
			near_town[player] += 1;
		}
		let mine = claim.touching[player];
		let theirs = claim.touching[1 - player] || claim.touching[2];
		match (mine, theirs) {
			(_, true) => onto_foe[player] += 1,
			(true, false) => onto_own[player] += 1,
			(false, false) => fresh[player] += 1,
		}
	}
	view.count_row("claims starting fresh", fresh);
	view.count_row("claims extending own", onto_own);
	view.count_row("claims touching foe", onto_foe);
	view.count_row("claims next to a town", near_town);

	let runs = own_runs(view);
	view.row(
		"mean own run on a path",
		runs.each_ref().map(|lengths| match lengths.len() {
			0 => "-".to_string(),
			count => format!("{:.1}", lengths.iter().sum::<i32>() as f64 / count as f64),
		}),
	);
	view.row(
		"longest own run",
		runs.each_ref().map(|lengths| {
			lengths
				.iter()
				.max()
				.map_or("-".to_string(), |best| best.to_string())
		}),
	);
	view.ratio_row(
		"points per paint point",
		[0, 1]
			.map(|player| run.score[player] as f64 / run.totals.paint_spent[player].max(1) as f64),
		1,
	);
}

fn own_runs(view: &View) -> [Vec<i32>; 2] {
	let mut runs = [Vec::new(), Vec::new()];
	for record in view.turns() {
		for path in record.paths.values() {
			let mut owner = -1i8;
			let mut length = 0i32;
			let close = |owner: i8, length: i32, runs: &mut [Vec<i32>; 2]| {
				if owner == 0 || owner == 1 {
					runs[owner as usize].push(length);
				}
			};
			for &at in path {
				let here = record.owner(at);
				if here == owner && here != -1 {
					length += 1;
					continue;
				}
				close(owner, length, &mut runs);
				owner = here;
				length = 1;
			}
			close(owner, length, &mut runs);
		}
	}
	runs
}
