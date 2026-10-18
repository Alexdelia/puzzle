use crate::overview::print_grid;
use crate::view::{View, base36, percent, section};
use btk::grid::{Coord, coord};
use btk::replay::board::as_seen_by;
use btk::replay::{Pair, TurnRecord, pair_name, town_char};
use std::collections::BTreeSet;

pub fn connections(view: &View) {
	let run = view.run;
	section("connections");
	println!("pair   turns  avg len  slack  paid us  paid foe  share us");
	let mut rows: Vec<_> = run.totals.pay.iter().collect();
	rows.sort_by_key(|(_, pay)| -pay.total());
	let mut paid = [0i64; 2];
	for (&pair, pay) in rows {
		paid[0] += pay.paid[0];
		paid[1] += pay.paid[1];
		println!(
			"{}  {:>5} {:>8.1} {:>6.1} {:>8} {:>9} {:>8.0}%",
			pair_name(pair),
			pay.turns,
			pay.mean_length(),
			pay.mean_length() - straight(run, pair) as f64,
			view.mine(pay.paid),
			view.theirs(pay.paid),
			percent(view.mine(pay.paid), pay.total())
		);
	}
	println!("total  {:>26} {:>9}", view.mine(paid), view.theirs(paid));

	let never: Vec<String> = run
		.grid
		.towns
		.iter()
		.flat_map(|town| {
			town.desired
				.iter()
				.filter(move |&&other| !run.totals.pay.contains_key(&(town.id, other)))
				.map(move |&other| pair_name((town.id, other)))
		})
		.collect();
	if !never.is_empty() {
		println!("never connected: {}", never.join(" "));
	}
}

fn straight(run: &btk::replay::Replay, pair: (usize, usize)) -> i32 {
	let from = run.grid.towns[pair.0].coord;
	let to = run.grid.towns[pair.1].coord;
	from.manhattan_to(to) + 1
}

pub fn paid_cells(view: &View, top: usize) {
	let run = view.run;
	section("who owns the paid cells");
	let mut rows: Vec<_> = run.totals.cell_pay.iter().collect();
	rows.sort_by_key(|(_, pay)| -(pay[0] + pay[1]));
	println!("cell        paid  owner");
	for (at, pay) in rows.iter().take(top) {
		let owner = run.grid.tile(**at).track;
		println!(
			"({:>2},{:>2}) {:>9}  {}",
			at.x,
			at.y,
			pay[0] + pay[1],
			match owner {
				0 | 1 => view.name(owner as usize),
				2 => "neutral",
				_ => "washed",
			}
		);
	}

	println!("payout map (digits are log10 of the points that cell paid, . = never paid)");
	print_grid(&run.grid, |x, y| {
		let at = coord(x, y);
		let tile = run.grid.tile(at);
		let paid: i64 = run.totals.paid_for(at).iter().sum();
		if tile.is_town() {
			town_char(tile.town as usize)
		} else if paid == 0 {
			'.'
		} else {
			base36(((paid as f64).log10().floor() as usize).min(9))
		}
	});

	println!("final board (o = us, x = foe, * = neutral, : = inked)");
	let last = view.turns().last().map(|record| record.board.as_str());
	print!("{}", as_seen_by(last.unwrap_or_default(), view.me));
}

pub fn openings(view: &View) {
	section("who completes each connection");
	println!("the split the first turn a pair is active, and the split at its best-paying turn");
	println!("pair  turn  len   us  foe   peak turn  len   us  foe");
	let mut seen: BTreeSet<Pair> = BTreeSet::new();
	for record in view.turns() {
		for (&pair, path) in &record.paths {
			if !seen.insert(pair) {
				continue;
			}
			let owned = record.owners_of(path);
			let peak = peak_turn(view, pair);
			println!(
				"{} {:>5} {:>4} {:>4} {:>4}   {:>8} {:>4} {:>4} {:>4}",
				pair_name(pair),
				record.turn,
				path.len(),
				view.mine(owned),
				view.theirs(owned),
				peak.map_or(0, |(turn, _, _)| turn),
				peak.map_or(0, |(_, len, _)| len as i32),
				peak.map_or(0, |(_, _, owned)| view.mine(owned)),
				peak.map_or(0, |(_, _, owned)| view.theirs(owned))
			);
		}
	}
}

fn peak_turn(view: &View, pair: Pair) -> Option<(i32, usize, [i32; 2])> {
	view.turns()
		.iter()
		.filter_map(|record| record.paths.get(&pair).map(|path| (record, path)))
		.max_by_key(|(record, path)| view.theirs(record.owners_of(path)) + path.len() as i32)
		.map(|(record, path)| (record.turn, path.len(), record.owners_of(path)))
}

pub fn takeovers(view: &View, threshold: f64) {
	section("path takeovers");
	println!("a pair whose ownership share jumps by {threshold:.0} points or more in one turn");
	println!("turn pair   len      us->    foe->   share  cells the mover added that turn");
	for window in view.turns().windows(2) {
		let (before, after) = (&window[0], &window[1]);
		for (&pair, path) in &after.paths {
			let Some(was) = before.paths.get(&pair) else {
				continue;
			};
			let owned_now = after.owners_of(path);
			let owned_was = before.owners_of(was);
			let moved = view.share(owned_now) - view.share(owned_was);
			if moved.abs() < threshold {
				continue;
			}
			println!(
				"{:>4} {} {:>3}->{:<3} {:>3}->{:<3} {:>3}->{:<3} {:>+5.0}  {}",
				after.turn,
				pair_name(pair),
				was.len(),
				path.len(),
				view.mine(owned_was),
				view.mine(owned_now),
				view.theirs(owned_was),
				view.theirs(owned_now),
				moved,
				added_cells(view, after, path, was)
			);
		}
	}
}

fn added_cells(view: &View, after: &TurnRecord, path: &[Coord], was: &[Coord]) -> String {
	let old: BTreeSet<_> = was.iter().collect();
	path.iter()
		.filter(|at| !old.contains(at))
		.map(|at| format!("({},{}){}", at.x, at.y, view.owner_name(after.owner(*at))))
		.collect::<Vec<_>>()
		.join(" ")
}

pub fn one_pair(view: &View, turn: i32, wanted: &str) {
	let Some(record) = view.run.at_turn(turn) else {
		return;
	};
	let Some(pair) = parse_pair(wanted) else {
		println!("\n== pair {wanted} == not a pair, write it as A-C or 0-2");
		return;
	};
	let Some(path) = record.paths.get(&pair) else {
		println!("\n== pair {wanted} at turn {turn} == not an active connection");
		return;
	};
	let owned = record.owners_of(path);
	println!(
		"\n== pair {} at turn {turn} == len {} us {} foe {} ({:.0}% ours)",
		pair_name(pair),
		path.len(),
		view.mine(owned),
		view.theirs(owned),
		view.share(owned)
	);
	let on_path: BTreeSet<_> = path.iter().copied().collect();
	print_grid(&view.run.grid, |x, y| {
		let at = coord(x, y);
		let tile = view.run.grid.tile(at);
		if tile.is_town() {
			town_char(tile.town as usize)
		} else if on_path.contains(&at) {
			match view.owner_name(record.owner(at)) {
				"us" => 'O',
				"foe" => 'X',
				"neu" => '+',
				_ => '?',
			}
		} else if (0..=2).contains(&record.owner(at)) {
			','
		} else {
			' '
		}
	});
}

fn parse_pair(wanted: &str) -> Option<Pair> {
	let mut ids = wanted.split('-').map(town_id);
	match (ids.next(), ids.next(), ids.next()) {
		(Some(Some(from)), Some(Some(to)), None) => Some((from, to)),
		_ => None,
	}
}

fn town_id(word: &str) -> Option<usize> {
	word.parse::<usize>().ok().or_else(|| {
		let letter = word.chars().next()?;
		letter
			.is_ascii_uppercase()
			.then(|| (letter as u8 - b'A') as usize)
	})
}
