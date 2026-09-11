use crate::view::{View, base36, section, short_answer};
use btk::grid::{Grid, TERRAIN_CHAR, coord};
use btk::replay::{Log, town_char};

pub fn header(view: &View, log: &Log, swapped: bool) {
	let run = view.run;
	println!("== log ==");
	println!("file          {}", log.path);
	println!("seed          {}", log.seed);
	println!(
		"map           {}x{}, {} towns",
		log.width,
		log.height,
		log.towns.len()
	);
	println!("we are        p{} (us), foe p{} (foe)", view.me, view.foe());
	println!(
		"answers       {} turns logged, first stream is p{}",
		log.marked_turns,
		usize::from(swapped)
	);
	report_score(view, log);
	println!(
		"turns         {} replayed, log marked {}",
		run.turns.len(),
		log.marked_total
			.map_or("?".to_string(), |total| total.to_string())
	);
	let gap = view.mine(run.score) - view.theirs(run.score);
	let winner = match gap.cmp(&0) {
		std::cmp::Ordering::Greater => "we win",
		std::cmp::Ordering::Less => "we lose",
		std::cmp::Ordering::Equal => "draw",
	};
	println!("outcome       {winner} by {}", gap.abs());
	notes(view);
}

fn report_score(view: &View, log: &Log) {
	let run = view.run;
	let replayed = format!("p0 {} p1 {}", run.score[0], run.score[1]);
	match log.final_score {
		Some(want) => {
			let verdict = if want == run.score {
				"matches the log"
			} else {
				"DIFFERS from the log"
			};
			println!(
				"score         {replayed} ({verdict}: p0 {} p1 {})",
				want[0], want[1]
			);
		}
		None => println!("score         {replayed} (log carries no final score)"),
	}
}

fn notes(view: &View) {
	let run = view.run;
	if !run.ended {
		println!("note          the log ran out before the referee ended the game");
	}
	for player in 0..2 {
		if let Some(why) = &run.disqualified[player] {
			println!(
				"note          p{player} ({}) was disqualified: {why}",
				view.name(player)
			);
		}
	}
	let errors: usize = run.turns.iter().map(|record| record.errors.len()).sum();
	if errors > 0 {
		println!("note          {errors} referee complaints, see the timeline");
	}
}

pub fn map(grid: &Grid, regions: bool) {
	section("map");
	print_grid(grid, |x, y| {
		let tile = grid.tile(coord(x, y));
		if tile.is_town() {
			town_char(tile.town as usize)
		} else {
			TERRAIN_CHAR[tile.kind as usize] as char
		}
	});
	for town in &grid.towns {
		let wants: Vec<String> = town
			.desired
			.iter()
			.map(|&other| {
				let to = grid.towns[other].coord;
				format!("{}{}", town_char(other), town.coord.manhattan_to(to))
			})
			.collect();
		println!(
			"town {} {:>2} at ({:>2},{:>2}) region {:<3} wants {}",
			town_char(town.id),
			town.id,
			town.coord.x,
			town.coord.y,
			grid.zone_of(town.coord).id,
			if wants.is_empty() {
				"-".into()
			} else {
				wants.join(" ")
			}
		);
	}
	if regions {
		println!("regions (base 36)");
		print_grid(grid, |x, y| base36(grid.tile(coord(x, y)).zone as usize));
	}
}

pub fn print_grid(grid: &Grid, mut glyph: impl FnMut(i32, i32) -> char) {
	for y in 0..grid.height {
		let line: String = (0..grid.width).map(|x| glyph(x, y)).collect();
		println!("{line}");
	}
}

pub fn timeline(view: &View) {
	section("timeline");
	println!("turn side  paint  +pts  score  actions");
	for record in view.turns() {
		for player in view.sides() {
			println!(
				"{:>4} {:<4} {:>2}/3  {:>5} {:>6}  {}",
				record.turn,
				view.name(player),
				record.paint_spent(player),
				record.gained[player],
				record.score[player],
				short_answer(&record.answers[player])
			);
		}
		for (&zone, counts) in &record.wiped {
			let by: Vec<&str> = (0..2)
				.filter(|&player| record.disrupted[player] == Some(zone))
				.map(|player| view.name(player))
				.collect();
			println!(
				"       INK region {zone} by {} washed away us {} foe {} neutral {}",
				by.join("+"),
				view.mine([counts[0], counts[1]]),
				view.theirs([counts[0], counts[1]]),
				counts[2]
			);
		}
		for line in &record.errors {
			println!(
				"       ERR {}",
				line.replace("¤RED¤", "").replace("§RED§", "")
			);
		}
	}
}
