use crate::view::{View, section};
use btk::action::{Action, parse};
use btk::grid::{Coord, Grid};
use std::collections::BTreeSet;

pub struct Answer {
	pub cells: BTreeSet<Coord>,
	pub disrupt: Option<i32>,
	pub line: String,
}

pub fn read_answer(grid: &Grid, line: &str) -> Answer {
	let parsed = parse(line);
	let mut cells = BTreeSet::new();
	let mut disrupt = None;
	for intent in parsed.intents {
		match intent {
			Action::Place { at, .. } => {
				cells.insert(at);
			}
			Action::Disrupt { zone } => disrupt = Some(zone),
			Action::DisruptAt { at } => {
				disrupt = grid.get(at).map(|tile| tile.zone as i32);
			}
			_ => {}
		}
	}
	Answer {
		cells,
		disrupt,
		line: line.trim().to_string(),
	}
}

pub fn agreement(view: &View, played: &[String], side: usize, show: usize) {
	section("imitation");
	println!(
		"a candidate bot answering the very frames p{side} ({}) was given, its answer compared",
		view.name(side)
	);
	println!("with what p{side} actually played; the game itself still follows the log");

	let recorded: Vec<&String> = view
		.turns()
		.iter()
		.map(|record| &record.answers[side])
		.collect();
	let turns = played.len().min(recorded.len());
	if turns == 0 {
		println!("the candidate answered nothing");
		return;
	}

	let mut same_line = 0;
	let mut same_cells = 0;
	let mut same_disrupt = 0;
	let mut shared = 0;
	let mut union = 0;
	let mut built_like = 0;
	let mut misses: Vec<String> = Vec::new();

	for turn in 0..turns {
		let want = read_answer(&view.run.grid, recorded[turn]);
		let got = read_answer(&view.run.grid, &played[turn]);
		same_line += usize::from(want.line == got.line);
		same_cells += usize::from(want.cells == got.cells);
		same_disrupt += usize::from(want.disrupt == got.disrupt);
		shared += want.cells.intersection(&got.cells).count();
		union += want.cells.union(&got.cells).count();
		built_like += usize::from(want.cells.len() == got.cells.len());
		if want.cells != got.cells || want.disrupt != got.disrupt {
			misses.push(format!(
				"{:>4}  log {:<44} bot {}",
				view.turns()[turn].turn,
				crate::view::short_answer(recorded[turn]),
				crate::view::short_answer(&played[turn])
			));
		}
	}

	let rate = |part: usize| 100.0 * part as f64 / turns as f64;
	println!("turns compared          {turns}");
	println!("identical answer        {:.0}%", rate(same_line));
	println!("same cells built        {:.0}%", rate(same_cells));
	println!("same region disrupted   {:.0}%", rate(same_disrupt));
	println!("same number of cells    {:.0}%", rate(built_like));
	println!(
		"cell overlap            {:.0}% ({shared} of {union})",
		100.0 * shared as f64 / union.max(1) as f64
	);
	if misses.is_empty() {
		return;
	}
	println!("\nfirst turns where they part ways");
	for line in misses.iter().take(show) {
		println!("{line}");
	}
	if misses.len() > show {
		println!("... and {} more", misses.len() - show);
	}
}
