use std::time::{Duration, Instant};

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::{Coord, InitialState, Referee, Score};

use super::config::{SearchConfig, SearchState};
use super::crossover::crossover;
use super::heuristic::{build_initial_population, random_solution, track_zombie};
use super::hill_climb::{hill_climb_pass, hill_climb_window};
use super::mutate::{mutate, pad_solution, smart_mutate_gather, smart_mutate_kill_focus};

pub fn run_search(
	initial: &InitialState,
	referee: &Referee,
	cfg: &SearchConfig,
	seed_solution: Option<Vec<Coord>>,
	starting_best: Score,
	mut on_improvement: impl FnMut(Score, &[Coord]),
) -> SearchState {
	let mut rng = Xoshiro256PlusPlus::seed_from_u64(cfg.seed);
	let mut population = build_initial_population(initial, cfg, seed_solution, &mut rng);

	let mut best_score = starting_best;
	let mut best_solution: Vec<Coord> = vec![initial.player; cfg.turn_limit];
	let mut generation: u64 = 0;
	let mut last_improvement_gen: u64 = 0;

	let start = Instant::now();
	let mut last_log = start;
	let total_sec = cfg.time_limit.as_secs_f64();
	let hill_climb_reserve = total_sec * 0.20;
	let ga_budget = (total_sec - hill_climb_reserve).max(total_sec * 0.5);

	loop {
		if start.elapsed().as_secs_f64() >= ga_budget {
			break;
		}

		let w = cfg.human_weight;
		let (score_list, fitness): (Vec<Score>, Vec<i64>) = if w > 0 {
			let (sc, ah) = referee
				.run_with_humans(&population)
				.expect("referee.run_with_humans failed");
			let fit = sc
				.iter()
				.zip(&ah)
				.map(|(&s, &h)| s as i64 + h as i64 * w)
				.collect();
			(sc, fit)
		} else {
			let sc = referee.run(&population).expect("referee.run failed");
			let fit = sc.iter().map(|&s| s as i64).collect();
			(sc, fit)
		};

		let mut idx: Vec<usize> = (0..population.len()).collect();
		idx.sort_by(|&a, &b| fitness[b].cmp(&fitness[a]));

		let mut top_real = 0;
		let mut top_real_i = 0;
		for (i, &s) in score_list.iter().enumerate() {
			if s > top_real {
				top_real = s;
				top_real_i = i;
			}
		}
		if top_real > best_score {
			best_score = top_real;
			best_solution = population[top_real_i].clone();
			last_improvement_gen = generation;
			on_improvement(best_score, &best_solution);
		}

		if last_log.elapsed().as_secs_f64() >= 2.0 {
			eprintln!(
				"gen {generation} | top {top_real} | best {best_score} | top_fit {} | stagnant {} | elapsed {:.1}s",
				fitness[idx[0]],
				generation - last_improvement_gen,
				start.elapsed().as_secs_f64(),
			);
			last_log = Instant::now();
		}

		let elite: Vec<Vec<Coord>> = idx
			.iter()
			.take(cfg.elite)
			.map(|&i| population[i].clone())
			.collect();

		let stagnant = generation - last_improvement_gen;
		let mutation_rate = if stagnant < 50 {
			0.04
		} else if stagnant < 200 {
			0.08
		} else if stagnant < 500 {
			0.15
		} else {
			0.25
		};

		let fitness_clone = fitness.clone();
		let tournament = |rng: &mut Xoshiro256PlusPlus| -> usize {
			let k = 4;
			let mut best = rng.random_range(0..population.len());
			let mut bs = fitness_clone[best];
			for _ in 1..k {
				let c = rng.random_range(0..population.len());
				if fitness_clone[c] > bs {
					bs = fitness_clone[c];
					best = c;
				}
			}
			best
		};

		let mut new_pop = elite;
		while new_pop.len() < cfg.population {
			let r: f32 = rng.random();
			let parent_a = tournament(&mut rng);
			let child = if r < 0.20 {
				let mut c = population[parent_a].clone();
				mutate(&mut c, initial, &mut rng, mutation_rate);
				c
			} else if r < 0.55 {
				let parent_b = tournament(&mut rng);
				let mut c = crossover(&population[parent_a], &population[parent_b], &mut rng);
				mutate(&mut c, initial, &mut rng, mutation_rate);
				c
			} else if r < 0.75 {
				let parent_b = tournament(&mut rng);
				crossover(&population[parent_a], &population[parent_b], &mut rng)
			} else if r < 0.97 {
				let mut c = population[parent_a].clone();
				mutate(&mut c, initial, &mut rng, mutation_rate * 2.5);
				c
			} else {
				random_solution(cfg.turn_limit, &mut rng)
			};
			new_pop.push(child);
		}

		let smart_budget = (cfg.elite / 2).max(4);
		for slot in 0..smart_budget.min(new_pop.len() - cfg.elite) {
			let target_slot = cfg.elite + slot;
			let elite_idx = slot % cfg.elite;
			let mut c = new_pop[elite_idx].clone();
			if slot % 2 == 0 {
				smart_mutate_kill_focus(&mut c, initial, &mut rng);
			} else {
				smart_mutate_gather(&mut c, initial, &mut rng);
			}
			new_pop[target_slot] = c;
		}

		if stagnant > 0 && stagnant.is_multiple_of(300) {
			eprintln!("gen {generation}: stagnated, injecting fresh blood");
			let inject = cfg.population / 3;
			let random_from = cfg.population - inject;
			for slot in &mut new_pop[random_from..] {
				*slot = random_solution(cfg.turn_limit, &mut rng);
			}
			let mut tweaked = best_solution.clone();
			pad_solution(&mut tweaked, initial.player, cfg.turn_limit);
			let perturb_from = random_from.saturating_sub(inject);
			for slot in new_pop[perturb_from..random_from].iter_mut() {
				let mut c = tweaked.clone();
				mutate(&mut c, initial, &mut rng, 0.30);
				*slot = c;
			}
		}

		if stagnant > 0 && stagnant.is_multiple_of(1500) {
			eprintln!("gen {generation}: hard restart, reseeding");
			let keep = cfg.elite;
			for slot in new_pop[keep..].iter_mut() {
				let r: f32 = rng.random();
				*slot = if r < 0.3 {
					random_solution(cfg.turn_limit, &mut rng)
				} else if r < 0.7 {
					let mut c = best_solution.clone();
					pad_solution(&mut c, initial.player, cfg.turn_limit);
					mutate(&mut c, initial, &mut rng, 0.40);
					c
				} else {
					let zid = rng.random_range(0..initial.zombie_list.len().max(1));
					track_zombie(initial, zid, cfg.turn_limit)
				};
			}
		}

		if generation > 0 && generation.is_multiple_of(400) && best_score > 0 {
			let mut local_best = best_solution.clone();
			let mut local_score = best_score;
			let elapsed = start.elapsed().as_secs_f64();
			let memetic_budget = (ga_budget - elapsed).clamp(0.0, 2.0);
			if memetic_budget > 0.1 {
				hill_climb_pass(
					initial,
					referee,
					&mut local_best,
					&mut local_score,
					&mut rng,
					Duration::from_secs_f64(memetic_budget),
					|s, sol| {
						if s > best_score {
							best_score = s;
							best_solution = sol.to_vec();
							last_improvement_gen = generation;
							on_improvement(s, sol);
						}
					},
				);
				if local_score > best_score {
					best_score = local_score;
					best_solution = local_best.clone();
					last_improvement_gen = generation;
					on_improvement(best_score, &best_solution);
				}
				new_pop[0] = local_best;
			}
		}

		population = new_pop;
		generation += 1;
	}

	let remaining = (total_sec - start.elapsed().as_secs_f64()).max(0.0);
	if remaining > 0.5 && !best_solution.is_empty() {
		eprintln!(
			"GA done at gen {generation} ({:.1}s elapsed), hill-climbing for {:.1}s",
			start.elapsed().as_secs_f64(),
			remaining,
		);
		let hc_pass_budget = remaining * 0.6;
		let hc_win_budget = remaining - hc_pass_budget;
		hill_climb_pass(
			initial,
			referee,
			&mut best_solution,
			&mut best_score,
			&mut rng,
			Duration::from_secs_f64(hc_pass_budget),
			&mut on_improvement,
		);
		hill_climb_window(
			initial,
			referee,
			&mut best_solution,
			&mut best_score,
			&mut rng,
			Duration::from_secs_f64(hc_win_budget),
			&mut on_improvement,
		);
	}

	SearchState {
		population,
		best_score,
		best_solution,
		generation,
		last_improvement_gen,
	}
}
