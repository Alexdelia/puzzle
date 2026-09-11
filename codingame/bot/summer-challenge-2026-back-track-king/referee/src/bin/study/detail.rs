use crate::view::View;
use btk::replay::board::as_seen_by;
use btk::replay::pair_name;

pub fn turn(view: &View, turn: i32) {
	let Some(record) = view.run.at_turn(turn) else {
		println!("\n== turn {turn} == not in the log");
		return;
	};
	println!("\n== turn {turn} ==");
	for player in view.sides() {
		println!(
			"{:<3} paint {}/3  gained {:>4}  score {:>5}  {}",
			view.name(player),
			record.paint_spent(player),
			record.gained[player],
			record.score[player],
			record.answers[player]
		);
	}
	print!("{}", as_seen_by(&record.board, view.me));

	println!("active connections");
	for active in &record.pairs {
		println!(
			"  {} len {:>3} us {:>3} foe {:>3}",
			pair_name(active.pair),
			active.length,
			view.mine(active.owned),
			view.theirs(active.owned)
		);
	}

	let mut pressed: Vec<_> = record
		.before
		.iter()
		.filter(|(_, look)| !look.inked && look.instability > 0)
		.collect();
	pressed.sort_by_key(|(_, look)| -look.instability);
	if pressed.is_empty() {
		return;
	}
	println!("regions under pressure before this turn");
	for (zone, look) in pressed {
		let tracks = [look.tracks[0], look.tracks[1]];
		println!(
			"  region {zone:<3} instability {} cells {} trackUs {} trackFoe {} pairs {}",
			look.instability,
			look.cells,
			view.mine(tracks),
			view.theirs(tracks),
			look.pairs.len()
		);
	}
}

pub fn boards(view: &View, every: usize) {
	for record in view.turns().iter().step_by(every) {
		println!("\n== board turn {} ==", record.turn);
		print!("{}", as_seen_by(&record.board, view.me));
	}
}
