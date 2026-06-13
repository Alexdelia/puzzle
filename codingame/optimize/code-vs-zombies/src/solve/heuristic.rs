use rand::Rng;

use crate::simulate::{MAX_H, MAX_W, PLAYER_RANGE_SQ, State};
use crate::{Axis, Coord, InitialState};

use super::config::SearchConfig;
use super::mutate::{mutate, pad_solution};

fn alive_zombie_centroid(state: &State) -> Option<(f64, f64, usize)> {
	let mut sx = 0.0;
	let mut sy = 0.0;
	let mut n = 0;
	for (z, &(zx, zy)) in state.zombie_list.iter().enumerate() {
		if state.zombie_alive_list[z] {
			sx += zx;
			sy += zy;
			n += 1;
		}
	}
	if n == 0 {
		None
	} else {
		Some((sx / n as f64, sy / n as f64, n))
	}
}

pub fn gather_and_nuke(
	initial: &InitialState,
	anchor: (f64, f64),
	standoff: f64,
	trigger_frac: f64,
	max_turns: usize,
) -> Vec<Coord> {
	let mut state = State::from_initial(initial);
	let mut moves = Vec::with_capacity(max_turns);
	for _ in 0..max_turns {
		if state.over {
			moves.push((state.player.0 as Axis, state.player.1 as Axis));
			continue;
		}
		let target = match alive_zombie_centroid(&state) {
			None => (state.player.0 as Axis, state.player.1 as Axis),
			Some((cx, cy, n)) => {
				let in_range = state
					.zombie_list
					.iter()
					.enumerate()
					.filter(|(z, _)| state.zombie_alive_list[*z])
					.filter(|&(_, &(zx, zy))| {
						let dx = zx - cx;
						let dy = zy - cy;
						dx * dx + dy * dy <= PLAYER_RANGE_SQ
					})
					.count();
				if in_range as f64 >= trigger_frac * n as f64 {
					(cx as Axis, cy as Axis)
				} else {
					let dx = anchor.0 - cx;
					let dy = anchor.1 - cy;
					let d = (dx * dx + dy * dy).sqrt().max(1.0);
					let hx = (cx + dx / d * standoff).clamp(0.0, (MAX_W - 1) as f64);
					let hy = (cy + dy / d * standoff).clamp(0.0, (MAX_H - 1) as f64);
					(hx as Axis, hy as Axis)
				}
			}
		};
		moves.push(target);
		state.step(target);
	}
	moves
}

pub fn shepherd(
	initial: &InitialState,
	corral: (f64, f64),
	nuke_frac: f64,
	max_turns: usize,
) -> Vec<Coord> {
	let mut state = State::from_initial(initial);
	let mut moves = Vec::with_capacity(max_turns);
	for _ in 0..max_turns {
		if state.over {
			moves.push((state.player.0 as Axis, state.player.1 as Axis));
			continue;
		}
		let (px, py) = state.player;

		let mut worst_defector: Option<(f64, (f64, f64))> = None;
		for (z, &(zx, zy)) in state.zombie_list.iter().enumerate() {
			if !state.zombie_alive_list[z] {
				continue;
			}
			let da = (zx - px) * (zx - px) + (zy - py) * (zy - py);
			let mut dh = f64::INFINITY;
			for (h, &(hx, hy)) in state.human_list.iter().enumerate() {
				if !state.human_alive_list[h] {
					continue;
				}
				let d = (zx - hx) * (zx - hx) + (zy - hy) * (zy - hy);
				if d < dh {
					dh = d;
				}
			}
			let imminent = 1200.0 * 1200.0;
			if dh < da
				&& dh < imminent
				&& dh < worst_defector.map(|(d, _)| d).unwrap_or(f64::INFINITY)
			{
				worst_defector = Some((dh, (zx, zy)));
			}
		}

		let target = if let Some((_, (zx, zy))) = worst_defector {
			(zx as Axis, zy as Axis)
		} else {
			match alive_zombie_centroid(&state) {
				None => (px as Axis, py as Axis),
				Some((cx, cy, n)) => {
					let mut in_range = 0;
					let mut nearest = (cx, cy);
					let mut nearest_d2 = f64::INFINITY;
					for (z, &(zx, zy)) in state.zombie_list.iter().enumerate() {
						if !state.zombie_alive_list[z] {
							continue;
						}
						let dc2 = (zx - cx) * (zx - cx) + (zy - cy) * (zy - cy);
						if dc2 <= PLAYER_RANGE_SQ {
							in_range += 1;
						}
						let da2 = (zx - px) * (zx - px) + (zy - py) * (zy - py);
						if da2 < nearest_d2 {
							nearest_d2 = da2;
							nearest = (zx, zy);
						}
					}
					if in_range as f64 >= nuke_frac * n as f64 {
						(cx as Axis, cy as Axis)
					} else {
						let dx = corral.0 - cx;
						let dy = corral.1 - cy;
						let d = (dx * dx + dy * dy).sqrt().max(1.0);
						let ux = dx / d;
						let uy = dy / d;
						let hx = (nearest.0 + ux * 2050.0).clamp(0.0, (MAX_W - 1) as f64);
						let hy = (nearest.1 + uy * 2050.0).clamp(0.0, (MAX_H - 1) as f64);
						(hx as Axis, hy as Axis)
					}
				}
			}
		};
		moves.push(target);
		state.step(target);
	}
	moves
}

fn herding_anchors(initial: &InitialState) -> Vec<(f64, f64)> {
	let w = MAX_W as f64;
	let h = MAX_H as f64;
	let mut anchors = vec![
		(0.0, 0.0),
		(w - 1.0, 0.0),
		(0.0, h - 1.0),
		(w - 1.0, h - 1.0),
		(w / 2.0, h / 2.0),
	];
	let nz = initial.zombie_list.len().max(1) as f64;
	let (zx, zy) = initial
		.zombie_list
		.iter()
		.fold((0.0, 0.0), |(ax, ay), &(x, y)| {
			(ax + x as f64, ay + y as f64)
		});
	let zc = (zx / nz, zy / nz);
	if !initial.human_list.is_empty() {
		let nh = initial.human_list.len() as f64;
		let (hx, hy) = initial
			.human_list
			.iter()
			.fold((0.0, 0.0), |(ax, ay), &(x, y)| {
				(ax + x as f64, ay + y as f64)
			});
		let hc = (hx / nh, hy / nh);
		let dx = zc.0 - hc.0;
		let dy = zc.1 - hc.1;
		let d = (dx * dx + dy * dy).sqrt().max(1.0);
		anchors.push((
			(zc.0 + dx / d * 6000.0).clamp(0.0, w - 1.0),
			(zc.1 + dy / d * 6000.0).clamp(0.0, h - 1.0),
		));
	}
	anchors
}

fn push_herding_seeds(pop: &mut Vec<Vec<Coord>>, initial: &InitialState, turn_limit: usize) {
	for anchor in herding_anchors(initial) {
		for standoff in [2050.0, 2500.0, 2900.0] {
			for trigger_frac in [0.6, 0.85, 1.0] {
				pop.push(gather_and_nuke(
					initial,
					anchor,
					standoff,
					trigger_frac,
					turn_limit,
				));
			}
		}
	}
	for corral in herding_anchors(initial) {
		for nuke_frac in [0.6, 0.8, 0.95, 1.0] {
			pop.push(shepherd(initial, corral, nuke_frac, turn_limit));
		}
	}
}

pub fn random_solution(max_turns: usize, rng: &mut impl Rng) -> Vec<Coord> {
	(0..max_turns)
		.map(|_| (rng.random_range(0..MAX_W), rng.random_range(0..MAX_H)))
		.collect()
}

pub fn stand_still(initial: &InitialState, max_turns: usize) -> Vec<Coord> {
	vec![initial.player; max_turns]
}

pub fn target_static_point(point: Coord, max_turns: usize) -> Vec<Coord> {
	vec![point; max_turns]
}

pub fn greedy_nearest_zombie(initial: &InitialState, max_turns: usize) -> Vec<Coord> {
	let mut state = State::from_initial(initial);
	let mut move_list = Vec::with_capacity(max_turns);
	for _ in 0..max_turns {
		if state.over {
			move_list.push((state.player.0 as Axis, state.player.1 as Axis));
			continue;
		}
		let target = state
			.zombie_list
			.iter()
			.enumerate()
			.filter(|(z, _)| state.zombie_alive_list[*z])
			.min_by(|(_, a), (_, b)| {
				let da = (a.0 - state.player.0).powi(2) + (a.1 - state.player.1).powi(2);
				let db = (b.0 - state.player.0).powi(2) + (b.1 - state.player.1).powi(2);
				da.partial_cmp(&db).unwrap()
			})
			.map(|(_, &(x, y))| (x as Axis, y as Axis))
			.unwrap_or((state.player.0 as Axis, state.player.1 as Axis));
		move_list.push(target);
		state.step(target);
	}
	move_list
}

pub fn greedy_defend(initial: &InitialState, max_turns: usize) -> Vec<Coord> {
	let mut state = State::from_initial(initial);
	let mut move_list = Vec::with_capacity(max_turns);
	for _ in 0..max_turns {
		if state.over {
			move_list.push((state.player.0 as Axis, state.player.1 as Axis));
			continue;
		}
		let mut target = (state.player.0 as Axis, state.player.1 as Axis);
		let mut min_d2 = f64::INFINITY;
		for (h, &hpos) in state.human_list.iter().enumerate() {
			if !state.human_alive_list[h] {
				continue;
			}
			for (z, &zpos) in state.zombie_list.iter().enumerate() {
				if !state.zombie_alive_list[z] {
					continue;
				}
				let dx = hpos.0 - zpos.0;
				let dy = hpos.1 - zpos.1;
				let d2 = dx * dx + dy * dy;
				if d2 < min_d2 {
					min_d2 = d2;
					target = (hpos.0 as Axis, hpos.1 as Axis);
				}
			}
		}
		move_list.push(target);
		state.step(target);
	}
	move_list
}

pub fn target_zombie_idx(initial: &InitialState, idx: usize, max_turns: usize) -> Vec<Coord> {
	if idx >= initial.zombie_list.len() {
		return stand_still(initial, max_turns);
	}
	let z = initial.zombie_list[idx];
	vec![z; max_turns]
}

pub fn target_human_idx(initial: &InitialState, idx: usize, max_turns: usize) -> Vec<Coord> {
	if idx >= initial.human_list.len() {
		return stand_still(initial, max_turns);
	}
	let h = initial.human_list[idx];
	vec![h; max_turns]
}

pub fn track_zombie(initial: &InitialState, idx: usize, max_turns: usize) -> Vec<Coord> {
	let mut state = State::from_initial(initial);
	let mut move_list = Vec::with_capacity(max_turns);
	for _ in 0..max_turns {
		let target = if idx < state.zombie_list.len() && state.zombie_alive_list[idx] {
			let (x, y) = state.zombie_list[idx];
			(x as Axis, y as Axis)
		} else {
			(state.player.0 as Axis, state.player.1 as Axis)
		};
		move_list.push(target);
		state.step(target);
	}
	move_list
}

pub fn dynamic_centroid(initial: &InitialState, max_turns: usize) -> Vec<Coord> {
	let mut state = State::from_initial(initial);
	let mut move_list = Vec::with_capacity(max_turns);
	for _ in 0..max_turns {
		let target = if state.alive_z > 0 {
			let mut sx = 0.0;
			let mut sy = 0.0;
			for (z, &(zx, zy)) in state.zombie_list.iter().enumerate() {
				if state.zombie_alive_list[z] {
					sx += zx;
					sy += zy;
				}
			}
			let cx = (sx / state.alive_z as f64) as Axis;
			let cy = (sy / state.alive_z as f64) as Axis;
			(cx, cy)
		} else {
			(state.player.0 as Axis, state.player.1 as Axis)
		};
		move_list.push(target);
		state.step(target);
	}
	move_list
}

pub fn wait_then_target(
	_initial: &InitialState,
	wait_pos: Coord,
	wait_turn_count: usize,
	then_target: Coord,
	max_turns: usize,
) -> Vec<Coord> {
	let mut move_list = Vec::with_capacity(max_turns);
	for _ in 0..wait_turn_count.min(max_turns) {
		move_list.push(wait_pos);
	}
	while move_list.len() < max_turns {
		move_list.push(then_target);
	}
	move_list
}

pub fn defend_human(initial: &InitialState, h_idx: usize, max_turns: usize) -> Vec<Coord> {
	let mut state = State::from_initial(initial);
	let mut move_list = Vec::with_capacity(max_turns);
	for _ in 0..max_turns {
		if h_idx >= state.human_list.len() || !state.human_alive_list[h_idx] {
			move_list.push((state.player.0 as Axis, state.player.1 as Axis));
			state.step((state.player.0 as Axis, state.player.1 as Axis));
			continue;
		}
		let hpos = state.human_list[h_idx];
		let mut nearest_z: Option<(f64, f64)> = None;
		let mut best_d2 = f64::INFINITY;
		for (z, &(zx, zy)) in state.zombie_list.iter().enumerate() {
			if !state.zombie_alive_list[z] {
				continue;
			}
			let dx = hpos.0 - zx;
			let dy = hpos.1 - zy;
			let d2 = dx * dx + dy * dy;
			if d2 < best_d2 {
				best_d2 = d2;
				nearest_z = Some((zx, zy));
			}
		}
		let target = if let Some(z) = nearest_z {
			let mx = (z.0 + hpos.0) / 2.0;
			let my = (z.1 + hpos.1) / 2.0;
			(mx as Axis, my as Axis)
		} else {
			(hpos.0 as Axis, hpos.1 as Axis)
		};
		move_list.push(target);
		state.step(target);
	}
	move_list
}

pub fn build_initial_population(
	initial: &InitialState,
	cfg: &SearchConfig,
	seed_solution: Option<Vec<Coord>>,
	rng: &mut impl Rng,
) -> Vec<Vec<Coord>> {
	let mut pop: Vec<Vec<Coord>> = Vec::with_capacity(cfg.population);

	if let Some(mut s) = seed_solution {
		pad_solution(&mut s, initial.player, cfg.turn_limit);
		pop.push(s);
	}

	pop.push(stand_still(initial, cfg.turn_limit));
	pop.push(greedy_nearest_zombie(initial, cfg.turn_limit));
	pop.push(greedy_defend(initial, cfg.turn_limit));
	pop.push(dynamic_centroid(initial, cfg.turn_limit));

	push_herding_seeds(&mut pop, initial, cfg.turn_limit);

	for i in 0..initial.zombie_list.len().min(16) {
		pop.push(target_zombie_idx(initial, i, cfg.turn_limit));
		pop.push(track_zombie(initial, i, cfg.turn_limit));
	}
	for i in 0..initial.human_list.len().min(16) {
		pop.push(target_human_idx(initial, i, cfg.turn_limit));
		pop.push(defend_human(initial, i, cfg.turn_limit));
	}

	for wait_turn_count in [3, 5, 8, 12, 18] {
		for &h in initial.human_list.iter().take(4) {
			let target = if !initial.zombie_list.is_empty() {
				let z = initial.zombie_list[wait_turn_count % initial.zombie_list.len()];
				(z.0, z.1)
			} else {
				h
			};
			pop.push(wait_then_target(
				initial,
				h,
				wait_turn_count,
				target,
				cfg.turn_limit,
			));
		}
	}

	let centroid = {
		let n = initial.zombie_list.len() as f64;
		let (sx, sy) = initial
			.zombie_list
			.iter()
			.fold((0.0, 0.0), |(ax, ay), &(x, y)| {
				(ax + x as f64, ay + y as f64)
			});
		((sx / n) as Axis, (sy / n) as Axis)
	};
	pop.push(target_static_point(centroid, cfg.turn_limit));

	let map_center_list = [
		(MAX_W / 2, MAX_H / 2),
		(MAX_W / 4, MAX_H / 4),
		(3 * MAX_W / 4, MAX_H / 4),
		(MAX_W / 4, 3 * MAX_H / 4),
		(3 * MAX_W / 4, 3 * MAX_H / 4),
	];
	for c in map_center_list {
		pop.push(target_static_point(c, cfg.turn_limit));
	}

	let seed_count = pop.len();
	for i in 0..seed_count {
		for _ in 0..3 {
			let mut c = pop[i].clone();
			mutate(&mut c, initial, rng, 0.05);
			pop.push(c);
		}
	}

	while pop.len() < cfg.population {
		pop.push(random_solution(cfg.turn_limit, rng));
	}

	pop.truncate(cfg.population);
	pop
}
