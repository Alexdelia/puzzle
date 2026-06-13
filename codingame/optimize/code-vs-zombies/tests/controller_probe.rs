use std::path::Path;

use code_vs_zombies::simulate::State;
use code_vs_zombies::solve::{gather_and_nuke, shepherd};
use code_vs_zombies::{Coord, InitialState, parse_validator_packs};

fn validator(name: &str) -> InitialState {
	parse_validator_packs(Path::new(&format!("validator/{name}.txt")))
		.unwrap()
		.into_iter()
		.last()
		.unwrap()
}

fn score_and_max_combo(init: &InitialState, moves: &[Coord]) -> (u32, usize, usize) {
	let mut s = State::from_initial(init);
	let mut max_combo = 0;
	for &m in moves {
		let before = s.alive_z;
		s.step(m);
		max_combo = max_combo.max(before - s.alive_z);
		if s.over {
			break;
		}
	}
	(s.score, max_combo, s.alive_h)
}

fn anchors(init: &InitialState) -> Vec<(f64, f64)> {
	let w = 16000.0;
	let h = 9000.0;
	let mut a = vec![
		(0.0, 0.0),
		(w - 1.0, 0.0),
		(0.0, h - 1.0),
		(w - 1.0, h - 1.0),
		(w / 2.0, h / 2.0),
	];
	let nz = init.zombie_list.len().max(1) as f64;
	let (zx, zy) = init
		.zombie_list
		.iter()
		.fold((0.0, 0.0), |(ax, ay), &(x, y)| {
			(ax + x as f64, ay + y as f64)
		});
	let zc = (zx / nz, zy / nz);
	if !init.human_list.is_empty() {
		let nh = init.human_list.len() as f64;
		let (hx, hy) = init
			.human_list
			.iter()
			.fold((0.0, 0.0), |(ax, ay), &(x, y)| {
				(ax + x as f64, ay + y as f64)
			});
		let hc = (hx / nh, hy / nh);
		let (dx, dy) = (zc.0 - hc.0, zc.1 - hc.1);
		let d = (dx * dx + dy * dy).sqrt().max(1.0);
		a.push((zc.0 + dx / d * 6000.0, zc.1 + dy / d * 6000.0));
	}
	a
}

#[test]
#[ignore]
fn controller_probe() {
	let turn_limit = 200;
	for name in [
		"17_horde",
		"18_flanked",
		"13_rescue_mission",
		"10_cross",
		"08_rows_to_defend_redux",
	] {
		let init = validator(name);
		let stored = std::fs::read_to_string(format!("output/{name}/score.txt"))
			.ok()
			.and_then(|s| s.trim().parse::<u32>().ok())
			.unwrap_or(0);
		let mut best = (0u32, 0usize, 0usize, (0.0, 0.0), 0.0, 0.0);
		for anchor in anchors(&init) {
			for standoff in [2050.0, 2250.0, 2500.0, 2900.0] {
				for tf in [0.6, 0.85, 1.0] {
					let moves = gather_and_nuke(&init, anchor, standoff, tf, turn_limit);
					let (sc, combo, ah) = score_and_max_combo(&init, &moves);
					if sc > best.0 {
						best = (sc, combo, ah, anchor, standoff, tf);
					}
				}
			}
		}
		let mut shep = (0u32, 0usize, 0usize, (0.0, 0.0), 0.0);
		for corral in anchors(&init) {
			for nf in [0.6, 0.8, 0.95, 1.0] {
				let moves = shepherd(&init, corral, nf, turn_limit);
				let (sc, combo, ah) = score_and_max_combo(&init, &moves);
				if sc > shep.0 {
					shep = (sc, combo, ah, corral, nf);
				}
			}
		}
		println!(
			"{name:24} SHEPHERD best={:>9} maxcombo={:>2} humans_saved={}/{} (corral={:.0},{:.0} nf={})",
			shep.0,
			shep.1,
			shep.2,
			init.human_list.len(),
			shep.3.0,
			shep.3.1,
			shep.4
		);
		println!(
			"{name:24} stored={stored:>8} controller_best={:>8} maxcombo={:>2} humans_saved={}/{} (anchor={:.0},{:.0} standoff={} tf={})",
			best.0,
			best.1,
			best.2,
			init.human_list.len(),
			best.3.0,
			best.3.1,
			best.4,
			best.5
		);
	}
}
