use std::path::Path;

use code_vs_zombies::parse::solution::parse_solution_file;
use code_vs_zombies::simulate::State;
use code_vs_zombies::{Coord, InitialState, parse_validator_packs};

fn validator(name: &str) -> InitialState {
	parse_validator_packs(Path::new(&format!("validator/{name}.txt")))
		.unwrap()
		.into_iter()
		.last()
		.unwrap()
}

#[test]
#[ignore]
fn combo_hist() {
	let name = std::env::var("MAP").unwrap_or_else(|_| "17_horde".into());
	let solpath = std::env::var("SOL").unwrap_or_else(|_| format!("output/{name}/solution.txt"));
	let init = validator(&name);
	let moves: Vec<Coord> = parse_solution_file(Path::new(&solpath)).unwrap();
	let mut s = State::from_initial(&init);
	println!(
		"=== {name} ({} z, {} h) sol={solpath} ===",
		init.zombie_list.len(),
		init.human_list.len()
	);
	for (t, &m) in moves.iter().enumerate() {
		let bz = s.alive_z;
		let bh = s.alive_h;
		let bscore = s.score;
		s.step(m);
		let killed = bz - s.alive_z;
		if killed > 0 {
			println!(
				"turn {t:>3}: killed {killed:>2}  (humans_alive={bh}  base={}  +{} -> {})",
				bh * bh * 10,
				s.score - bscore,
				s.score
			);
		}
		if s.over {
			break;
		}
	}
	println!(
		"final score={} humans_alive={} zombies_alive={}",
		s.score, s.alive_h, s.alive_z
	);
}
