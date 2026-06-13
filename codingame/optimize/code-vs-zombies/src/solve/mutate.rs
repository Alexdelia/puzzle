use rand::Rng;

use crate::simulate::{MAX_H, MAX_W, State};
use crate::{Axis, Coord, InitialState};

pub(crate) fn shift(c: Axis, d: i32, max: Axis) -> Axis {
	(c as i32 + d).clamp(0, max as i32 - 1) as Axis
}

pub fn pad_solution(sol: &mut Vec<Coord>, fallback: Coord, max_turns: usize) {
	if sol.len() >= max_turns {
		sol.truncate(max_turns);
		return;
	}
	let pad = *sol.last().unwrap_or(&fallback);
	sol.resize(max_turns, pad);
}

pub fn mutate(sol: &mut [Coord], initial: &InitialState, rng: &mut impl Rng, rate: f32) {
	let len = sol.len();
	for i in 0..len {
		if rng.random::<f32>() >= rate {
			continue;
		}
		mutate_one(sol, i, initial, rng);
	}
}

pub(crate) fn mutate_one(sol: &mut [Coord], i: usize, initial: &InitialState, rng: &mut impl Rng) {
	let op: u8 = rng.random_range(0..20);
	match op {
		0 => {
			sol[i] = (rng.random_range(0..MAX_W), rng.random_range(0..MAX_H));
		}
		1..=4 => {
			let sigma: i32 = rng.random_range(50..=500);
			let dx = rng.random_range(-sigma..=sigma);
			let dy = rng.random_range(-sigma..=sigma);
			sol[i].0 = shift(sol[i].0, dx, MAX_W);
			sol[i].1 = shift(sol[i].1, dy, MAX_H);
		}
		5 | 6 => {
			let sigma: i32 = rng.random_range(500..=2500);
			let dx = rng.random_range(-sigma..=sigma);
			let dy = rng.random_range(-sigma..=sigma);
			sol[i].0 = shift(sol[i].0, dx, MAX_W);
			sol[i].1 = shift(sol[i].1, dy, MAX_H);
		}
		7 => {
			if !initial.zombie_list.is_empty() {
				let z = initial.zombie_list[rng.random_range(0..initial.zombie_list.len())];
				let dx = rng.random_range(-1000..=1000);
				let dy = rng.random_range(-1000..=1000);
				sol[i] = (shift(z.0, dx, MAX_W), shift(z.1, dy, MAX_H));
			}
		}
		8 => {
			if !initial.human_list.is_empty() {
				let h = initial.human_list[rng.random_range(0..initial.human_list.len())];
				let dx = rng.random_range(-1000..=1000);
				let dy = rng.random_range(-1000..=1000);
				sol[i] = (shift(h.0, dx, MAX_W), shift(h.1, dy, MAX_H));
			}
		}
		9 => {
			let len = sol.len();
			let block = rng.random_range(2..=10).min(len - i);
			let value = sol[i];
			for slot in sol[i..(i + block).min(len)].iter_mut() {
				*slot = value;
			}
		}
		10 => {
			let len = sol.len();
			let block = rng.random_range(2..=8).min(len - i);
			if block > 1 {
				let v = sol[i];
				for slot in sol[(i + 1)..(i + block).min(len)].iter_mut() {
					*slot = v;
					let dx = rng.random_range(-200..=200);
					let dy = rng.random_range(-200..=200);
					slot.0 = shift(slot.0, dx, MAX_W);
					slot.1 = shift(slot.1, dy, MAX_H);
				}
			}
		}
		11 => {
			let len = sol.len();
			let block = rng.random_range(2..=10).min(len - i);
			if i + 1 < len && block > 1 {
				let saved = sol[i];
				for k in i..(i + block - 1).min(len - 1) {
					sol[k] = sol[k + 1];
				}
				sol[(i + block - 1).min(len - 1)] = saved;
			}
		}
		12 => {
			if i > 0 {
				sol[i] = sol[i - 1];
			}
		}
		13 => {
			let len = sol.len();
			if i + 1 < len {
				sol[i] = sol[i + 1];
			}
		}
		14 => {
			if initial.zombie_list.len() >= 2 {
				let a = initial.zombie_list[rng.random_range(0..initial.zombie_list.len())];
				let b = initial.zombie_list[rng.random_range(0..initial.zombie_list.len())];
				let mid = ((a.0 + b.0) / 2, (a.1 + b.1) / 2);
				let block = rng.random_range(3..=10).min(sol.len() - i);
				for k in i..(i + block).min(sol.len()) {
					sol[k] = mid;
				}
			}
		}
		15 => {
			if !initial.human_list.is_empty() {
				let h = initial.human_list[rng.random_range(0..initial.human_list.len())];
				let block = rng.random_range(3..=10).min(sol.len() - i);
				for k in i..(i + block).min(sol.len()) {
					sol[k] = h;
				}
			}
		}
		16 => {
			let len = sol.len();
			let l = rng.random_range(2..=8).min(len - i);
			if l >= 2 {
				let end = (i + l).min(len);
				sol[i..end].reverse();
			}
		}
		17 => {
			let len = sol.len();
			if i + 1 < len {
				let v = (rng.random_range(0..MAX_W), rng.random_range(0..MAX_H));
				sol.copy_within(i..(len - 1), i + 1);
				sol[i] = v;
			}
		}
		18 => {
			let len = sol.len();
			if i + 1 < len {
				sol.copy_within((i + 1)..len, i);
				let pad = sol[len - 2];
				sol[len - 1] = pad;
			}
		}
		_ => {
			let len = sol.len();
			let max_off = (len - i).clamp(1, 20);
			let off = rng.random_range(0..max_off);
			let dest = (i + off).min(len - 1);
			if dest != i {
				sol.swap(i, dest);
			}
		}
	}
}

pub fn smart_mutate_kill_focus(sol: &mut [Coord], initial: &InitialState, rng: &mut impl Rng) {
	let len = sol.len();
	if len == 0 {
		return;
	}
	let turn = rng.random_range(0..len);
	let mut state = State::from_initial(initial);
	for &m in sol.iter().take(turn) {
		state.step(m);
		if state.over {
			return;
		}
	}
	let mut sum_x = 0.0;
	let mut sum_y = 0.0;
	let mut n = 0;
	for (z, &(zx, zy)) in state.zombie_list.iter().enumerate() {
		if state.zombie_alive_list[z] {
			sum_x += zx;
			sum_y += zy;
			n += 1;
		}
	}
	if n == 0 {
		return;
	}
	let cx = (sum_x / n as f64) as Axis;
	let cy = (sum_y / n as f64) as Axis;
	let dx = rng.random_range(-800..=800);
	let dy = rng.random_range(-800..=800);
	sol[turn] = (shift(cx, dx, MAX_W), shift(cy, dy, MAX_H));
}
