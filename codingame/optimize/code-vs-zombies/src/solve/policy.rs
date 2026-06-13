use std::f64::consts::PI;

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::simulate::{MAX_H, MAX_W, PLAYER_RANGE_SQ, State, simulate};
use crate::{Axis, Coord, InitialState, Score};

const W: f64 = MAX_W as f64;
const H: f64 = MAX_H as f64;

// (lo, hi) bounds for each policy parameter.
const BOUNDS: &[(f64, f64)] = &[
	(0.0, 1.5),       // 0 w_centroid   fraction-pull toward zombie centroid
	(0.0, 1.5),       // 1 w_nearest    fraction-pull toward nearest zombie
	(1200.0, 3500.0), // 2 standoff_R  hold distance from nearest zombie
	(0.0, 3.0),       // 3 w_standoff   outward push when within standoff_R
	(0.0, W),         // 4 anchor_x     corral point x
	(0.0, H),         // 5 anchor_y     corral point y
	(0.0, 1.5),       // 6 w_corral     fraction-pull toward corral
	(0.0, 2.5),       // 7 w_protect    pull toward a human-threatening zombie
	(400.0, 3500.0),  // 8 protect_R   how close a threat must be to trigger protect
	(0.3, 1.0), // 9 trigger_frac dive when this fraction of zombies are within kill range of centroid
];

pub fn param_dim() -> usize {
	BOUNDS.len()
}

fn clamp(x: f64, lo: f64, hi: f64) -> f64 {
	x.max(lo).min(hi)
}

pub fn policy_rollout(init: &InitialState, theta: &[f64], turn_limit: usize) -> Vec<Coord> {
	let w_centroid = clamp(theta[0], BOUNDS[0].0, BOUNDS[0].1);
	let w_nearest = clamp(theta[1], BOUNDS[1].0, BOUNDS[1].1);
	let standoff_r = clamp(theta[2], BOUNDS[2].0, BOUNDS[2].1);
	let w_standoff = clamp(theta[3], BOUNDS[3].0, BOUNDS[3].1);
	let anchor = (
		clamp(theta[4], BOUNDS[4].0, BOUNDS[4].1),
		clamp(theta[5], BOUNDS[5].0, BOUNDS[5].1),
	);
	let w_corral = clamp(theta[6], BOUNDS[6].0, BOUNDS[6].1);
	let w_protect = clamp(theta[7], BOUNDS[7].0, BOUNDS[7].1);
	let protect_r = clamp(theta[8], BOUNDS[8].0, BOUNDS[8].1);
	let trigger_frac = clamp(theta[9], BOUNDS[9].0, BOUNDS[9].1);

	let mut s = State::from_initial(init);
	let mut moves = Vec::with_capacity(turn_limit);

	for _ in 0..turn_limit {
		if s.over {
			moves.push((s.player.0 as Axis, s.player.1 as Axis));
			continue;
		}
		let (px, py) = s.player;

		let mut cx = 0.0;
		let mut cy = 0.0;
		let mut n = 0;
		let mut nearest = (px, py);
		let mut nearest_d2 = f64::INFINITY;
		for (z, &(zx, zy)) in s.zombie_list.iter().enumerate() {
			if !s.zombie_alive_list[z] {
				continue;
			}
			cx += zx;
			cy += zy;
			n += 1;
			let d2 = (zx - px) * (zx - px) + (zy - py) * (zy - py);
			if d2 < nearest_d2 {
				nearest_d2 = d2;
				nearest = (zx, zy);
			}
		}
		if n == 0 {
			moves.push((px as Axis, py as Axis));
			s.step((px as Axis, py as Axis));
			continue;
		}
		cx /= n as f64;
		cy /= n as f64;

		let mut in_range = 0;
		for (z, &(zx, zy)) in s.zombie_list.iter().enumerate() {
			if !s.zombie_alive_list[z] {
				continue;
			}
			let d2 = (zx - cx) * (zx - cx) + (zy - cy) * (zy - cy);
			if d2 <= PLAYER_RANGE_SQ {
				in_range += 1;
			}
		}

		let target = if in_range as f64 >= trigger_frac * n as f64 {
			(cx, cy)
		} else {
			let mut vx = w_centroid * (cx - px) + w_nearest * (nearest.0 - px);
			let mut vy = w_centroid * (cy - py) + w_nearest * (nearest.1 - py);

			let dn = nearest_d2.sqrt();
			if dn > 1.0 && dn < standoff_r {
				let push = w_standoff * (standoff_r - dn);
				vx += (px - nearest.0) / dn * push;
				vy += (py - nearest.1) / dn * push;
			}

			vx += w_corral * (anchor.0 - px);
			vy += w_corral * (anchor.1 - py);

			if w_protect > 0.0 {
				let mut threat: Option<(f64, f64)> = None;
				let mut threat_d2 = protect_r * protect_r;
				for (z, &(zx, zy)) in s.zombie_list.iter().enumerate() {
					if !s.zombie_alive_list[z] {
						continue;
					}
					let da = (zx - px) * (zx - px) + (zy - py) * (zy - py);
					let mut dh = f64::INFINITY;
					for (h, &(hx, hy)) in s.human_list.iter().enumerate() {
						if !s.human_alive_list[h] {
							continue;
						}
						let d = (zx - hx) * (zx - hx) + (zy - hy) * (zy - hy);
						if d < dh {
							dh = d;
						}
					}
					if dh < da && dh < threat_d2 {
						threat_d2 = dh;
						threat = Some((zx, zy));
					}
				}
				if let Some((tzx, tzy)) = threat {
					vx += w_protect * (tzx - px);
					vy += w_protect * (tzy - py);
				}
			}

			(px + vx, py + vy)
		};

		let t = (
			clamp(target.0, 0.0, W - 1.0) as Axis,
			clamp(target.1, 0.0, H - 1.0) as Axis,
		);
		moves.push(t);
		s.step(t);
	}
	moves
}

fn gaussian(rng: &mut Xoshiro256PlusPlus) -> f64 {
	let u1: f64 = rng.random::<f64>().max(1e-12);
	let u2: f64 = rng.random::<f64>();
	(-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
}

pub fn cem_optimize(
	init: &InitialState,
	turn_limit: usize,
	generations: usize,
	lambda: usize,
	seed: u64,
) -> (Score, Vec<f64>) {
	let d = BOUNDS.len();
	let mu = (lambda / 4).max(4);
	let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);

	let mut mean: Vec<f64> = BOUNDS.iter().map(|&(lo, hi)| (lo + hi) / 2.0).collect();
	let mut std: Vec<f64> = BOUNDS.iter().map(|&(lo, hi)| (hi - lo) / 4.0).collect();

	let mut best_score: Score = 0;
	let mut best_theta = mean.clone();

	for _ in 0..generations {
		let mut pop: Vec<(Score, Vec<f64>)> = Vec::with_capacity(lambda);
		for _ in 0..lambda {
			let theta: Vec<f64> = (0..d)
				.map(|i| {
					clamp(
						mean[i] + std[i] * gaussian(&mut rng),
						BOUNDS[i].0,
						BOUNDS[i].1,
					)
				})
				.collect();
			let moves = policy_rollout(init, &theta, turn_limit);
			let (score, _) = simulate(init, &moves);
			pop.push((score, theta));
		}
		pop.sort_by_key(|p| std::cmp::Reverse(p.0));
		if pop[0].0 > best_score {
			best_score = pop[0].0;
			best_theta = pop[0].1.clone();
		}

		for i in 0..d {
			let m: f64 = pop[..mu].iter().map(|(_, t)| t[i]).sum::<f64>() / mu as f64;
			let var: f64 = pop[..mu]
				.iter()
				.map(|(_, t)| (t[i] - m) * (t[i] - m))
				.sum::<f64>()
				/ mu as f64;
			let floor = (BOUNDS[i].1 - BOUNDS[i].0) * 0.02;
			mean[i] = m;
			std[i] = var.sqrt().max(floor);
		}
	}

	(best_score, best_theta)
}
