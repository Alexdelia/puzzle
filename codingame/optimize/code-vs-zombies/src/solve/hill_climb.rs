use std::time::{Duration, Instant};

use rand::Rng;

use crate::simulate::{MAX_H, MAX_W, State};
use crate::{Axis, Coord, InitialState, Referee, Score};

use super::mutate::{mutate_one, shift};

pub fn hill_climb_pass(
	initial: &InitialState,
	referee: &Referee,
	best: &mut Vec<Coord>,
	best_score: &mut Score,
	rng: &mut impl Rng,
	time_limit: Duration,
	mut on_improvement: impl FnMut(Score, &[Coord]),
) {
	let start = Instant::now();
	let mut current = best.clone();
	let mut current_score = *best_score;
	let len = current.len();
	loop {
		if start.elapsed() >= time_limit {
			break;
		}
		let snapshot_list = simulate_snapshots(initial, &current);
		let mut improved = false;
		for turn_i in 0..len {
			if start.elapsed() >= time_limit {
				break;
			}
			let original = current[turn_i];
			let snapshot = snapshot_list.get(turn_i);
			let mut candidate_list: Vec<Coord> = Vec::with_capacity(128);
			candidate_list.push(original);
			for _ in 0..12 {
				let sigma = rng.random_range(100..=2000);
				let dx = rng.random_range(-sigma..=sigma);
				let dy = rng.random_range(-sigma..=sigma);
				candidate_list.push((shift(original.0, dx, MAX_W), shift(original.1, dy, MAX_H)));
			}
			for _ in 0..8 {
				candidate_list.push((rng.random_range(0..MAX_W), rng.random_range(0..MAX_H)));
			}
			for &z in initial.zombie_list.iter().take(12) {
				let dx = rng.random_range(-400..=400);
				let dy = rng.random_range(-400..=400);
				candidate_list.push((shift(z.0, dx, MAX_W), shift(z.1, dy, MAX_H)));
			}
			if let Some(snap) = snapshot {
				let nz = snap.zombie_list.len();
				let nh = snap.human_list.len();
				let mut alive_zombie_list: Vec<(f64, f64)> = Vec::new();
				for z in 0..nz {
					if snap.zombie_alive_list[z] {
						alive_zombie_list.push(snap.zombie_list[z]);
					}
				}
				let mut alive_human_list: Vec<(f64, f64)> = Vec::new();
				for h in 0..nh {
					if snap.human_alive_list[h] {
						alive_human_list.push(snap.human_list[h]);
					}
				}
				for &z in alive_zombie_list.iter().take(20) {
					candidate_list.push((z.0 as Axis, z.1 as Axis));
					let dx = rng.random_range(-300..=300);
					let dy = rng.random_range(-300..=300);
					candidate_list
						.push((shift(z.0 as Axis, dx, MAX_W), shift(z.1 as Axis, dy, MAX_H)));
				}
				for &h in alive_human_list.iter().take(10) {
					candidate_list.push((h.0 as Axis, h.1 as Axis));
				}
				if !alive_zombie_list.is_empty() {
					let cx = alive_zombie_list.iter().map(|z| z.0).sum::<f64>()
						/ alive_zombie_list.len() as f64;
					let cy = alive_zombie_list.iter().map(|z| z.1).sum::<f64>()
						/ alive_zombie_list.len() as f64;
					candidate_list.push((cx as Axis, cy as Axis));
				}
				for i in 0..alive_zombie_list.len().min(8) {
					for j in (i + 1)..alive_zombie_list.len().min(8) {
						let mx = (alive_zombie_list[i].0 + alive_zombie_list[j].0) / 2.0;
						let my = (alive_zombie_list[i].1 + alive_zombie_list[j].1) / 2.0;
						candidate_list.push((mx as Axis, my as Axis));
					}
				}
			}
			let sim_list: Vec<Vec<Coord>> = candidate_list
				.iter()
				.map(|&c| {
					let mut s = current.clone();
					s[turn_i] = c;
					s
				})
				.collect();
			let score_list = referee.run(&sim_list).expect("hc referee");
			let (top_idx, top_score) = score_list
				.iter()
				.enumerate()
				.max_by_key(|(_, s)| **s)
				.map(|(i, &s)| (i, s))
				.unwrap();
			if top_score > current_score {
				current = sim_list[top_idx].clone();
				current_score = top_score;
				improved = true;
				if current_score > *best_score {
					*best_score = current_score;
					*best = current.clone();
					on_improvement(*best_score, best);
				}
			}
		}
		if !improved {
			let mutation_count = (current.len() / 6).max(3);
			let trial_list: Vec<Vec<Coord>> = (0..32)
				.map(|_| {
					let mut p = best.clone();
					for _ in 0..mutation_count {
						let i = rng.random_range(0..p.len());
						mutate_one(&mut p, i, initial, rng);
					}
					p
				})
				.collect();
			let score_list = referee.run(&trial_list).expect("hc perturb");
			let (top_idx, top_score) = score_list
				.iter()
				.enumerate()
				.max_by_key(|(_, s)| **s)
				.map(|(i, &s)| (i, s))
				.unwrap();
			current = trial_list[top_idx].clone();
			current_score = top_score;
			if top_score > *best_score {
				*best = current.clone();
				*best_score = top_score;
				on_improvement(*best_score, best);
			}
		}
	}
}

fn simulate_snapshots(initial: &InitialState, move_list: &[Coord]) -> Vec<State> {
	let mut state = State::from_initial(initial);
	let mut snapshot_list: Vec<State> = Vec::with_capacity(move_list.len());
	for &m in move_list.iter() {
		snapshot_list.push(state.clone());
		if state.over {
			continue;
		}
		state.step(m);
	}
	snapshot_list
}

pub fn hill_climb_window(
	initial: &InitialState,
	referee: &Referee,
	best: &mut Vec<Coord>,
	best_score: &mut Score,
	rng: &mut impl Rng,
	time_limit: Duration,
	mut on_improvement: impl FnMut(Score, &[Coord]),
) {
	let start = Instant::now();
	let mut current = best.clone();
	let mut current_score = *best_score;
	let len = current.len();
	loop {
		if start.elapsed() >= time_limit {
			break;
		}
		let mut improved = false;
		for start_t in 0..len {
			if start.elapsed() >= time_limit {
				break;
			}
			let window_len = rng.random_range(2..=6).min(len - start_t);
			if window_len < 2 {
				continue;
			}
			let mut candidate_list: Vec<Vec<Coord>> = Vec::with_capacity(64);
			candidate_list.push(current[start_t..start_t + window_len].to_vec());
			for _ in 0..16 {
				let p = (rng.random_range(0..MAX_W), rng.random_range(0..MAX_H));
				candidate_list.push(vec![p; window_len]);
			}
			for &z in initial.zombie_list.iter().take(10) {
				candidate_list.push(vec![z; window_len]);
			}
			for &h in initial.human_list.iter().take(5) {
				candidate_list.push(vec![h; window_len]);
			}
			for _ in 0..16 {
				let mut w = Vec::with_capacity(window_len);
				for _ in 0..window_len {
					let dx = rng.random_range(-1500..=1500);
					let dy = rng.random_range(-1500..=1500);
					let v = current[start_t];
					w.push((shift(v.0, dx, MAX_W), shift(v.1, dy, MAX_H)));
				}
				candidate_list.push(w);
			}
			let sim_list: Vec<Vec<Coord>> = candidate_list
				.iter()
				.map(|w| {
					let mut s = current.clone();
					for (k, &v) in w.iter().enumerate() {
						s[start_t + k] = v;
					}
					s
				})
				.collect();
			let score_list = referee.run(&sim_list).expect("hcw referee");
			let (top_idx, top_score) = score_list
				.iter()
				.enumerate()
				.max_by_key(|(_, s)| **s)
				.map(|(i, &s)| (i, s))
				.unwrap();
			if top_score > current_score {
				current = sim_list[top_idx].clone();
				current_score = top_score;
				improved = true;
				if current_score > *best_score {
					*best_score = current_score;
					*best = current.clone();
					on_improvement(*best_score, best);
				}
			}
		}
		if !improved {
			let mutation_count = (current.len() / 5).max(3);
			let trial_list: Vec<Vec<Coord>> = (0..32)
				.map(|_| {
					let mut p = best.clone();
					for _ in 0..mutation_count {
						let i = rng.random_range(0..p.len());
						mutate_one(&mut p, i, initial, rng);
					}
					p
				})
				.collect();
			let score_list = referee.run(&trial_list).expect("hcw perturb");
			let (top_idx, top_score) = score_list
				.iter()
				.enumerate()
				.max_by_key(|(_, s)| **s)
				.map(|(i, &s)| (i, s))
				.unwrap();
			current = trial_list[top_idx].clone();
			current_score = top_score;
			if top_score > *best_score {
				*best = current.clone();
				*best_score = top_score;
				on_improvement(*best_score, best);
			}
		}
	}
}
