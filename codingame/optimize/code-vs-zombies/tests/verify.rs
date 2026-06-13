use std::path::PathBuf;

use code_vs_zombies::{Referee, parse_solution_file, parse_validator_packs};

#[test]
fn verify_17_horde_score() {
	let cwd = std::env::current_dir().unwrap();
	let val = parse_validator_packs(&cwd.join("validator/17_horde.txt"))
		.expect("validator")
		.into_iter()
		.last()
		.unwrap();
	let sol = parse_solution_file(&cwd.join("output/17_horde/solution.txt")).expect("solution");
	let score_text = std::fs::read_to_string(cwd.join("output/17_horde/score.txt")).expect("score");
	let claimed_score: u32 = score_text.trim().parse().expect("parse score");

	let referee = Referee::new(&val, sol.len()).expect("init referee");
	let score_list = referee.run(&[sol]).expect("run");
	assert_eq!(
		score_list[0], claimed_score,
		"17_horde score mismatch: got {} expected {}",
		score_list[0], claimed_score
	);
}

#[test]
fn verify_all_outputs() {
	let cwd = std::env::current_dir().unwrap();
	let output_dir: PathBuf = cwd.join("output");
	let entry_list: Vec<_> = std::fs::read_dir(&output_dir)
		.expect("read output dir")
		.filter_map(|e| e.ok())
		.collect();

	let mut total: u64 = 0;
	let mut failed: Vec<String> = Vec::new();
	for entry in entry_list {
		let name = entry.file_name().to_string_lossy().to_string();
		let dir = entry.path();
		let score_path = dir.join("score.txt");
		let sol_path = dir.join("solution.txt");
		let val_path = cwd.join(format!("validator/{name}.txt"));
		if !score_path.exists() || !sol_path.exists() || !val_path.exists() {
			continue;
		}
		let claimed: u32 = std::fs::read_to_string(&score_path)
			.unwrap()
			.trim()
			.parse()
			.unwrap();
		let sol = parse_solution_file(&sol_path).unwrap();
		let val = parse_validator_packs(&val_path)
			.unwrap()
			.into_iter()
			.last()
			.unwrap();
		if sol.is_empty() {
			eprintln!("{name}: empty solution");
			failed.push(name.clone());
			continue;
		}
		let referee = Referee::new(&val, sol.len()).unwrap();
		let score_list = referee.run(&[sol]).unwrap();
		if score_list[0] != claimed {
			eprintln!(
				"{name}: claimed {} but referee says {}",
				claimed, score_list[0]
			);
			failed.push(name.clone());
		}
		total += claimed as u64;
	}
	println!("verified total: {total}");
	assert!(failed.is_empty(), "failed validators: {failed:?}");
}
