use std::path::Path;

use code_vs_zombies::simulate::State;
use code_vs_zombies::solve::{cem_optimize, policy_rollout};
use code_vs_zombies::{Coord, InitialState, parse_validator_packs};

fn validator(name: &str) -> InitialState {
	parse_validator_packs(Path::new(&format!("validator/{name}.txt")))
		.unwrap()
		.into_iter()
		.last()
		.unwrap()
}

fn max_combo_and_humans(init: &InitialState, moves: &[Coord]) -> (usize, usize) {
	let mut s = State::from_initial(init);
	let mut mc = 0;
	for &m in moves {
		let bz = s.alive_z;
		s.step(m);
		mc = mc.max(bz - s.alive_z);
		if s.over {
			break;
		}
	}
	(mc, s.alive_h)
}

#[test]
#[ignore]
fn policy_probe() {
	let turn_limit = 160;
	for name in [
		"13_rescue_mission",
		"21_devastation",
		"18_flanked",
		"17_horde",
	] {
		let init = validator(name);
		let stored = std::fs::read_to_string(format!("output/{name}/score.txt"))
			.ok()
			.and_then(|s| s.trim().parse::<u32>().ok())
			.unwrap_or(0);
		let mut best_score = 0;
		let mut best_theta = vec![];
		for restart in 0..6 {
			let (sc, theta) = cem_optimize(&init, turn_limit, 120, 96, 1234 + restart * 7919);
			if sc > best_score {
				best_score = sc;
				best_theta = theta;
			}
		}
		let moves = policy_rollout(&init, &best_theta, turn_limit);
		let (mc, ah) = max_combo_and_humans(&init, &moves);
		println!(
			"{name:24} stored={stored:>8} policy_best={best_score:>8} maxcombo={mc:>2} humans={ah}/{}",
			init.human_list.len()
		);
	}
}
