use crate::view::{View, section};
use btk::game::PASSIVE_INCOME;

pub fn paint(view: &View) {
	let run = view.run;
	let totals = &run.totals;
	let budget = run.turns.len() as i32 * PASSIVE_INCOME;
	section("paint");
	view.heading();
	view.count_row("tracks placed", totals.placed);
	view.count_row("  on plains", totals.plains);
	view.count_row("  on river", totals.river);
	view.count_row("  on mountain", totals.mountain);
	view.row(
		"paint spent",
		totals.paint_spent.map(|spent| format!("{spent}/{budget}")),
	);
	view.count_row("paint wasted", totals.paint_lost);
	view.count_row("tracks lost to ink", lost_to_ink(view));
	view.count_row(
		"tracks left at the end",
		[run.tracks_left(0), run.tracks_left(1)],
	);
	println!("{:<24} {:>10}", "neutral tracks left", run.tracks_left(2));
	view.row(
		"first idle build turn",
		[0, 1].map(|player| first_idle(view, player)),
	);
}

fn lost_to_ink(view: &View) -> [i32; 2] {
	let mut lost = [0i32; 2];
	for record in view.turns() {
		for counts in record.wiped.values() {
			lost[0] += counts[0];
			lost[1] += counts[1];
		}
	}
	lost
}

fn first_idle(view: &View, player: usize) -> String {
	view.turns()
		.iter()
		.find(|record| record.placed[player] == 0)
		.map_or("-".to_string(), |record| record.turn.to_string())
}

pub fn disruption(view: &View) {
	let totals = &view.run.totals;
	section("disruption");
	view.heading();
	view.count_row("disrupts landed", totals.disrupts);
	view.count_row("disrupt turns wasted", totals.blots_lost);
	view.count_row("regions inked", totals.inked_by);
	view.count_row("own track washed", totals.wiped_own);
	view.count_row("foe track washed", totals.wiped_foe);

	println!("\nwhat each disrupt was aimed at, as the board looked when it was chosen");
	println!("turn side region inst cells  trackUs trackFoe  pathUs pathFoe  pairs  inked");
	for record in view.turns() {
		for player in 0..2 {
			let Some(zone) = record.disrupted[player] else {
				continue;
			};
			let look = &record.before[&zone];
			let tracks = [look.tracks[0], look.tracks[1]];
			let on_path = [look.path_cells[0], look.path_cells[1]];
			println!(
				"{:>4} {:<4} {:>6} {:>4} {:>5} {:>8} {:>8} {:>7} {:>7} {:>6}  {}",
				record.turn,
				view.name(player),
				zone,
				look.instability,
				look.cells,
				view.mine(tracks),
				view.theirs(tracks),
				view.mine(on_path),
				view.theirs(on_path),
				look.pairs.len(),
				if record.inked.contains(&zone) {
					"yes"
				} else {
					""
				}
			);
		}
	}
}

const INCOME_ROWS: usize = 20;

pub fn income(view: &View) {
	section("income");
	println!("turn  pairs  cellsUs cellsFoe cellsNeu   +us  +foe    us   foe    gap");
	let step = (view.turns().len() / INCOME_ROWS).max(1);
	let mut shown: Vec<_> = view.turns().iter().step_by(step).collect();
	if let Some(last) = view.turns().last()
		&& shown.last().map(|record| record.turn) != Some(last.turn)
	{
		shown.push(last);
	}
	for record in shown {
		let cells = [record.active_cells[0], record.active_cells[1]];
		println!(
			"{:>4} {:>6} {:>8} {:>8} {:>8} {:>5} {:>5} {:>5} {:>5} {:>6}",
			record.turn,
			record.pairs.len(),
			view.mine(cells),
			view.theirs(cells),
			record.active_cells[2],
			view.mine(record.gained),
			view.theirs(record.gained),
			view.mine(record.score),
			view.theirs(record.score),
			view.mine(record.score) - view.theirs(record.score)
		);
	}
}
