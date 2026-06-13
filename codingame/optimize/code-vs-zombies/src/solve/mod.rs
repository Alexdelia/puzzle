pub mod beam;
mod config;
pub mod crossover;
pub mod ga;
pub mod heuristic;
pub mod hill_climb;
pub mod mutate;

pub use beam::*;
pub use config::*;
pub use crossover::*;
pub use ga::*;
pub use heuristic::*;
pub use hill_climb::*;
pub use mutate::*;

use std::fs;
use std::path::{Path, PathBuf};

use crate::parse_solution_file;
use crate::simulate::simulate;
use crate::{Config, Coord, InitialState, Mode, Referee, Score, Solution};

pub fn solve(config: Config, test: InitialState, validator: InitialState) -> Result<(), String> {
	let path = &config.path;
	fs::create_dir_all(&path.output_dir)
		.map_err(|e| format!("mkdir {:?}: {e}", path.output_dir))?;
	let score_path = &path.score;
	let solution_path = &path.solution;

	let cfg = &config.search_config;

	let referee = Referee::new(&validator, cfg.turn_limit)
		.map_err(|e| format!("failed to init referee: {e}"))?;

	let prior_solution = parse_solution_file(solution_path)
		.ok()
		.filter(|s| !s.is_empty());

	let (existing_solution, existing_best) =
		seed_for_validator(&test, &validator, &referee, prior_solution, cfg.turn_limit)?;
	if let Some(ref sol) = existing_solution {
		write_best(existing_best, sol, &validator, score_path, solution_path)?;
		eprintln!("seeded validator orientation (score {existing_best})");
	}

	let mut beam_result: Option<(Score, Solution)> = None;
	if matches!(config.mode, Mode::Beam | Mode::BeamThenGa) {
		let beam_seed = cfg.seed;
		let (beam_score, beam_sol) =
			beam_search(&validator, &referee, cfg.turn_limit, 128, 48, beam_seed);
		eprintln!("beam search: best={beam_score} (existing_best={existing_best})");
		if beam_score > existing_best {
			write_best(beam_score, &beam_sol, &validator, score_path, solution_path)?;
		}
		beam_result = Some((beam_score, beam_sol));
	}

	if config.mode == Mode::Beam {
		eprintln!("beam-only mode finished {}", config.validator_name);
		return Ok(());
	}

	let seed_for_ga = match (beam_result, existing_solution) {
		(Some((bs, bsol)), Some(esol)) => {
			if bs > existing_best {
				Some(bsol)
			} else {
				Some(esol)
			}
		}
		(Some((_, bsol)), None) => Some(bsol),
		(None, esol) => esol,
	};

	let validator_clone = validator.clone();
	let score_path_cb = score_path.clone();
	let solution_path_cb = solution_path.clone();
	let starting_best = existing_best.max(read_score(score_path));

	let result = run_search(
		&validator,
		&referee,
		cfg,
		seed_for_ga,
		starting_best,
		move |score, solution| {
			if let Err(e) = write_best(
				score,
				solution,
				&validator_clone,
				&score_path_cb,
				&solution_path_cb,
			) {
				eprintln!("failed to write best: {e}");
			}
		},
	);

	eprintln!(
		"finished {validator_name}: best={best_score} (improvement at gen {last_improvement_gen}, gen count {gen_count})",
		validator_name = config.validator_name,
		best_score = result.best_score,
		last_improvement_gen = result.last_improvement_gen,
		gen_count = result.generation,
	);

	Ok(())
}

fn apply_flip(p: Coord, ex: i32, tx: i32, ey: i32, ty: i32) -> Coord {
	((ex * p.0 as i32 + tx) as u16, (ey * p.1 as i32 + ty) as u16)
}

fn sorted_coords(mut v: Vec<Coord>) -> Vec<Coord> {
	v.sort();
	v
}

fn flips_test_to_val(test: &InitialState, val: &InitialState) -> Vec<(i32, i32, i32, i32)> {
	if test.human_list.len() != val.human_list.len()
		|| test.zombie_list.len() != val.zombie_list.len()
	{
		return Vec::new();
	}
	let vh = sorted_coords(val.human_list.clone());
	let vz = sorted_coords(val.zombie_list.clone());
	let mut out = Vec::new();
	for &(ex, ey) in &[(1, 1), (1, -1), (-1, 1), (-1, -1)] {
		let tx = val.player.0 as i32 - ex * test.player.0 as i32;
		let ty = val.player.1 as i32 - ey * test.player.1 as i32;
		let fh = sorted_coords(
			test.human_list
				.iter()
				.map(|&p| apply_flip(p, ex, tx, ey, ty))
				.collect(),
		);
		let fz = sorted_coords(
			test.zombie_list
				.iter()
				.map(|&p| apply_flip(p, ex, tx, ey, ty))
				.collect(),
		);
		if fh == vh && fz == vz {
			out.push((ex, tx, ey, ty));
		}
	}
	out
}

fn pad_seed(sol: &[Coord], turn_limit: usize) -> Vec<Coord> {
	let mut v = sol.to_vec();
	if let Some(&last) = v.last() {
		while v.len() < turn_limit {
			v.push(last);
		}
	}
	v.truncate(turn_limit);
	v
}

fn seed_for_validator(
	test: &InitialState,
	val: &InitialState,
	referee: &Referee,
	prior: Option<Vec<Coord>>,
	turn_limit: usize,
) -> Result<(Option<Vec<Coord>>, Score), String> {
	let Some(prior) = prior else {
		return Ok((None, 0));
	};

	let mut candidates = vec![pad_seed(&prior, turn_limit)];
	for (ex, tx, ey, ty) in flips_test_to_val(test, val) {
		let reflected: Vec<Coord> = prior
			.iter()
			.map(|&p| apply_flip(p, ex, tx, ey, ty))
			.collect();
		candidates.push(pad_seed(&reflected, turn_limit));
	}

	let scores = referee
		.run(&candidates)
		.map_err(|e| format!("seed referee.run: {e}"))?;
	let mut best_index = 0;
	let mut best_score = 0;
	for (i, &s) in scores.iter().enumerate() {
		if s > best_score {
			best_score = s;
			best_index = i;
		}
	}

	Ok((Some(candidates.swap_remove(best_index)), best_score))
}

fn read_score(path: &Path) -> Score {
	fs::read_to_string(path)
		.ok()
		.and_then(|s| s.trim().parse::<Score>().ok())
		.unwrap_or(0)
}

fn write_best(
	score: Score,
	solution: &[Coord],
	validator: &InitialState,
	score_path: &Path,
	solution_path: &Path,
) -> Result<(), String> {
	let trimmed = trim_solution(validator, solution);
	let solution_str: String = trimmed.iter().map(|(x, y)| format!("{x} {y}\n")).collect();
	write_atomic(score_path, &format!("{score}\n"))?;
	write_atomic(solution_path, &solution_str)?;
	eprintln!("  -> wrote new best: score={score} turns={}", trimmed.len());
	Ok(())
}

fn write_atomic(path: &Path, content: &str) -> Result<(), String> {
	let mut tmp = PathBuf::from(path);
	let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("out");
	tmp.set_file_name(format!("{file_name}.tmp"));
	fs::write(&tmp, content).map_err(|e| format!("write {tmp:?}: {e}"))?;
	fs::rename(&tmp, path).map_err(|e| format!("rename {tmp:?} -> {path:?}: {e}"))?;
	Ok(())
}

fn trim_solution(validator: &InitialState, move_list: &[Coord]) -> Solution {
	let (_, end) = simulate(validator, move_list);
	move_list[..end.max(1).min(move_list.len())].to_vec()
}
