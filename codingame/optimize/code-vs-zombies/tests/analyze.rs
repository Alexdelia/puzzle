use code_vs_zombies::{Referee, parse_solution_file, parse_validator};

#[test]
#[ignore]
fn analyze_18_flanked() {
	analyze_one("18_flanked");
}

#[test]
#[ignore]
fn analyze_13_rescue_mission() {
	analyze_one("13_rescue_mission");
}

#[test]
#[ignore]
fn analyze_21_devastation() {
	analyze_one("21_devastation");
}

#[test]
#[ignore]
fn analyze_17_horde() {
	analyze_one("17_horde");
}

fn analyze_one(name: &str) {
	let cwd = std::env::current_dir().unwrap();
	let val = parse_validator(&cwd.join(format!("validator/{name}.txt"))).expect("v");
	let sol = parse_solution_file(&cwd.join(format!("output/{name}/solution.txt"))).expect("s");
	let referee = Referee::new(&val.initial, sol.len()).expect("r");
	let (score_list, log) = referee
		.run_with_state(std::slice::from_ref(&sol))
		.expect("debug run");
	let turn_count = sol.len();
	let nh = log.nh;
	let nz = log.nz;
	println!("=== {name} ===");
	println!("NH={} NZ={} final_score={}", nh, nz, score_list[0]);
	let mut prev_score = 0;
	for (t, mv) in sol.iter().enumerate().take(turn_count) {
		let s = log.turn_score(t + 1, 0);
		let alive_h = (0..nh).filter(|&h| log.is_human_alive(t + 1, 0, h)).count();
		let alive_z = (0..nz)
			.filter(|&z| log.is_zombie_alive(t + 1, 0, z))
			.count();
		let delta = s - prev_score;
		let (px, py) = log.player_pos(t + 1, 0);
		println!(
			"t={:3} pos=({:5.0},{:5.0}) target=({:5},{:5}) score={:8} (+{:6}) alive_h={} alive_z={}",
			t, px, py, mv.0, mv.1, s, delta, alive_h, alive_z
		);
		prev_score = s;
	}
}
