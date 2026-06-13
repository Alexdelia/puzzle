use rand::Rng;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::simulate::{MAX_H, MAX_W, State};
use crate::{Axis, Coord, InitialState, Referee, Score, Solution};

#[derive(Clone)]
struct BeamMember {
	move_list: Vec<Coord>,
	state: State,
}

fn greedy_default(state: &mut State, max_turns: usize, move_list: &mut Vec<Coord>) {
	while move_list.len() < max_turns {
		if state.over {
			move_list.push((state.player.0 as Axis, state.player.1 as Axis));
			continue;
		}
		let mut best_d2 = f64::INFINITY;
		let mut target = (state.player.0 as Axis, state.player.1 as Axis);
		for (z, &(zx, zy)) in state.zombie_list.iter().enumerate() {
			if !state.zombie_alive_list[z] {
				continue;
			}
			let dx = zx - state.player.0;
			let dy = zy - state.player.1;
			let d2 = dx * dx + dy * dy;
			if d2 < best_d2 {
				best_d2 = d2;
				target = (zx as Axis, zy as Axis);
			}
		}
		move_list.push(target);
		state.step(target);
	}
}

fn centroid_default(state: &mut State, max_turns: usize, move_list: &mut Vec<Coord>) {
	while move_list.len() < max_turns {
		if state.over {
			move_list.push((state.player.0 as Axis, state.player.1 as Axis));
			continue;
		}
		let mut sx = 0.0;
		let mut sy = 0.0;
		let mut n = 0;
		for (z, &(zx, zy)) in state.zombie_list.iter().enumerate() {
			if !state.zombie_alive_list[z] {
				continue;
			}
			sx += zx;
			sy += zy;
			n += 1;
		}
		let target = if n > 0 {
			((sx / n as f64) as Axis, (sy / n as f64) as Axis)
		} else {
			(state.player.0 as Axis, state.player.1 as Axis)
		};
		move_list.push(target);
		state.step(target);
	}
}

fn candidates_at_state(
	state: &State,
	initial: &InitialState,
	rng: &mut impl Rng,
	n: usize,
) -> Vec<Coord> {
	let mut out: Vec<Coord> = Vec::with_capacity(n);
	for (z, &(zx, zy)) in state.zombie_list.iter().enumerate() {
		if state.zombie_alive_list[z] {
			out.push((zx as Axis, zy as Axis));
		}
	}
	for (h, &(hx, hy)) in state.human_list.iter().enumerate() {
		if state.human_alive_list[h] {
			out.push((hx as Axis, hy as Axis));
		}
	}
	let alive_zombie_list: Vec<(f64, f64)> = state
		.zombie_list
		.iter()
		.enumerate()
		.filter(|(z, _)| state.zombie_alive_list[*z])
		.map(|(_, &z)| z)
		.collect();
	if alive_zombie_list.len() >= 2 {
		for i in 0..alive_zombie_list.len().min(6) {
			for j in (i + 1)..alive_zombie_list.len().min(6) {
				let mx = (alive_zombie_list[i].0 + alive_zombie_list[j].0) / 2.0;
				let my = (alive_zombie_list[i].1 + alive_zombie_list[j].1) / 2.0;
				out.push((mx as Axis, my as Axis));
			}
		}
	}
	if !alive_zombie_list.is_empty() {
		let cx =
			alive_zombie_list.iter().map(|z| z.0).sum::<f64>() / alive_zombie_list.len() as f64;
		let cy =
			alive_zombie_list.iter().map(|z| z.1).sum::<f64>() / alive_zombie_list.len() as f64;
		out.push((cx as Axis, cy as Axis));
	}
	out.push((state.player.0 as Axis, state.player.1 as Axis));
	for _ in 0..6 {
		out.push((rng.random_range(0..MAX_W), rng.random_range(0..MAX_H)));
	}
	for &z in initial.zombie_list.iter().take(6) {
		out.push(z);
	}
	while out.len() > n {
		let pick = rng.random_range(0..out.len());
		out.swap_remove(pick);
	}
	while out.len() < n {
		let dx = rng.random_range(-1500..=1500);
		let dy = rng.random_range(-1500..=1500);
		out.push((
			(state.player.0 as i32 + dx).clamp(0, MAX_W as i32 - 1) as Axis,
			(state.player.1 as i32 + dy).clamp(0, MAX_H as i32 - 1) as Axis,
		));
	}
	out
}

pub fn beam_search(
	initial: &InitialState,
	referee: &Referee,
	max_turns: usize,
	beam_size: usize,
	candidate_count_per_turn: usize,
	rng_seed: u64,
) -> (Score, Solution) {
	let mut rng = Xoshiro256PlusPlus::seed_from_u64(rng_seed);
	let mut beam_list: Vec<BeamMember> = vec![BeamMember {
		move_list: Vec::new(),
		state: State::from_initial(initial),
	}];

	let mut best_score: Score = 0;
	let mut best_solution: Solution = vec![initial.player; max_turns];

	for t in 0..max_turns {
		if beam_list.is_empty() {
			break;
		}
		if beam_list.iter().all(|b| b.state.over) {
			break;
		}

		let mut candidate_list: Vec<(usize, Coord)> = Vec::new();
		let mut sim_list: Vec<Solution> = Vec::new();

		for (bi, beam) in beam_list.iter().enumerate() {
			let cand_set =
				candidates_at_state(&beam.state, initial, &mut rng, candidate_count_per_turn);
			for cand in cand_set {
				let mut move_list = beam.move_list.clone();
				move_list.push(cand);
				let mut state_after = beam.state.clone();
				state_after.step(cand);
				if rng.random::<f32>() < 0.5 {
					greedy_default(&mut state_after, max_turns, &mut move_list);
				} else {
					centroid_default(&mut state_after, max_turns, &mut move_list);
				}
				while move_list.len() < max_turns {
					move_list.push((state_after.player.0 as Axis, state_after.player.1 as Axis));
				}
				candidate_list.push((bi, cand));
				sim_list.push(move_list);
			}
		}

		if sim_list.is_empty() {
			break;
		}
		let score_list = referee.run(&sim_list).expect("beam eval");

		let mut idx: Vec<usize> = (0..sim_list.len()).collect();
		idx.sort_by(|&a, &b| score_list[b].cmp(&score_list[a]));

		let top = idx[0];
		if score_list[top] > best_score {
			best_score = score_list[top];
			best_solution = sim_list[top].clone();
		}

		let mut new_beam_list: Vec<BeamMember> = Vec::with_capacity(beam_size);
		for &i in idx.iter() {
			if new_beam_list.len() >= beam_size {
				break;
			}
			let (bi, cand) = candidate_list[i];
			let mut move_list = beam_list[bi].move_list.clone();
			move_list.push(cand);
			let mut state = beam_list[bi].state.clone();
			state.step(cand);
			let is_dup = new_beam_list.iter().any(|b| {
				b.move_list.len() == move_list.len()
					&& b.state.score == state.score
					&& b.state.player.0 == state.player.0
					&& b.state.player.1 == state.player.1
					&& b.state.alive_z == state.alive_z
					&& b.state.alive_h == state.alive_h
			});
			if !is_dup {
				new_beam_list.push(BeamMember { move_list, state });
			}
		}
		if new_beam_list.is_empty() {
			break;
		}
		beam_list = new_beam_list;
		let _ = t;
	}

	(best_score, best_solution)
}
