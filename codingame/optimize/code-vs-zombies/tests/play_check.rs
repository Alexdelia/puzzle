use std::path::Path;

use code_vs_zombies::parse::solution::parse_solution_file;
use code_vs_zombies::simulate::State;
use code_vs_zombies::{Coord, InitialState};

fn parse_pts(s: &str) -> Vec<Coord> {
	s.trim()
		.split(';')
		.filter(|t| !t.trim().is_empty())
		.map(|t| {
			let mut it = t.split_whitespace();
			(
				it.next().unwrap().parse().unwrap(),
				it.next().unwrap().parse().unwrap(),
			)
		})
		.collect()
}

fn packs(name: &str) -> Vec<InitialState> {
	let txt = std::fs::read_to_string(format!("validator/{name}.txt")).unwrap();
	let body = txt.lines().skip(1).collect::<Vec<_>>().join("\n");
	body.split("\n\n")
		.map(str::trim)
		.filter(|p| !p.is_empty())
		.map(|p| {
			let l: Vec<&str> = p.lines().collect();
			InitialState {
				player: parse_pts(l[0])[0],
				human_list: parse_pts(l[1]),
				zombie_list: parse_pts(l[2]),
			}
		})
		.collect()
}

fn dist2(a: Coord, b: Coord) -> u64 {
	let dx = a.0 as i64 - b.0 as i64;
	let dy = a.1 as i64 - b.1 as i64;
	(dx * dx + dy * dy) as u64
}

fn live(state: &State) -> (Coord, Vec<Coord>, Vec<Coord>) {
	let player = (state.player.0 as u16, state.player.1 as u16);
	let humans = state
		.human_list
		.iter()
		.zip(&state.human_alive_list)
		.filter(|&(_, &a)| a)
		.map(|(&(x, y), _)| (x as u16, y as u16))
		.collect();
	let zombies = state
		.zombie_list
		.iter()
		.zip(&state.zombie_alive_list)
		.filter(|&(_, &a)| a)
		.map(|(&(x, y), _)| (x as u16, y as u16))
		.collect();
	(player, humans, zombies)
}

fn greedy_target(player: Coord, humans: &[Coord], zombies: &[Coord]) -> Coord {
	let mut best = None;
	let mut best_key = u64::MAX;
	for &z in zombies {
		let key = if humans.is_empty() {
			dist2(z, player)
		} else {
			humans.iter().map(|&h| dist2(z, h)).min().unwrap()
		};
		if key < best_key {
			best_key = key;
			best = Some(z);
		}
	}
	best.unwrap_or(player)
}

fn play_out(init: &InitialState, moves: &[Coord], cap: usize) -> (bool, usize, usize, usize, i64) {
	let mut s = State::from_initial(init);
	let mut t = 0;
	while !s.over && t < cap {
		let target = if t < moves.len() {
			moves[t]
		} else {
			let (p, h, z) = live(&s);
			greedy_target(p, &h, &z)
		};
		s.step(target);
		t += 1;
	}
	let won = s.over && s.alive_z == 0 && s.alive_h > 0;
	(won, s.alive_h, s.alive_z, t, s.score as i64)
}

fn check(name: &str) -> (i64, i64, i64) {
	let pk = packs(name);
	let val = pk.last().unwrap();
	let sol = parse_solution_file(Path::new(&format!("output/{name}/solution.txt"))).unwrap();
	let offline = std::fs::read_to_string(format!("output/{name}/score.txt"))
		.unwrap()
		.trim()
		.parse::<i64>()
		.unwrap();

	let (won, _ah, _az, _turns, score) = play_out(val, &sol, sol.len() + 80);
	assert!(won, "{name} did not win");
	println!(
		"{name:32} stored={offline:>8} validator={score:>8} loss={:>8}",
		offline - score
	);
	(offline, score, offline - score)
}

#[test]
fn play_check() {
	let mut to = 0i64;
	let mut tv = 0i64;
	for n in [
		"01_simple",
		"02_2_zombies",
		"03_2_zombies_redux",
		"04_scared_human",
		"05_3_vs_3",
		"06_combo_opportunity",
		"07_rows_to_defend",
		"08_rows_to_defend_redux",
		"09_rectangle",
		"10_cross",
		"11_unavoidable_deaths",
		"12_columns_of_death",
		"13_rescue_mission",
		"14_triangle",
		"15_grave_danger",
		"16_grid",
		"17_horde",
		"18_flanked",
		"19_split_second_reflex",
		"20_swervy_pattern",
		"21_devastation",
	] {
		let (o, v, _) = check(n);
		to += o;
		tv += v;
	}
	println!("\nTOTAL offline={to} validator={tv} loss={}", to - tv);
}
