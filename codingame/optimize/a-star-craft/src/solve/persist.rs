use std::fmt::Write;
use std::path::{Path, PathBuf};

use crate::simulation::{
	MAP_AREA, MAP_WIDTH, NONE, Score, Solution, Tile, char_to_tile, tile_to_char,
};

const OUTPUT_DIR: &str = "output";
const SCORE_FILE: &str = "score.txt";
const SOLUTION_FILE: &str = "solution.txt";
const COMPLETE_FLAG: &str = "complete.flag";

fn directory(name: &str) -> PathBuf {
	Path::new(OUTPUT_DIR).join(name)
}

pub fn read_best_score(name: &str) -> Option<Score> {
	let path = directory(name).join(SCORE_FILE);
	std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

pub fn read_stored_solution(name: &str, base: &[Tile]) -> Option<Solution> {
	let path = directory(name).join(SOLUTION_FILE);
	let content = std::fs::read_to_string(path).ok()?;

	let mut solution = Solution::empty();
	let mut token = content.split_whitespace();
	while let Some(x_token) = token.next() {
		let x: usize = x_token.parse().ok()?;
		let y: usize = token.next()?.parse().ok()?;
		let direction = char_to_tile(token.next()?.chars().next()?)?;
		let cell = y * MAP_WIDTH + x;
		if direction < NONE && cell < MAP_AREA && base[cell] == NONE {
			solution.arrow[cell] = direction;
		}
	}

	Some(solution)
}

pub fn format_solution(solution: &Solution, base: &[Tile]) -> String {
	let mut output = String::new();
	for (cell, &tile) in base.iter().enumerate().take(MAP_AREA) {
		if tile != NONE {
			continue;
		}
		let Some(direction) = tile_to_char(solution.arrow[cell]) else {
			continue;
		};
		if !output.is_empty() {
			output.push(' ');
		}
		let x = cell % MAP_WIDTH;
		let y = cell / MAP_WIDTH;
		let _ = write!(output, "{x} {y} {direction}");
	}
	output
}

pub fn write_best(
	name: &str,
	solution: &Solution,
	base: &[Tile],
	score: Score,
) -> Result<(), String> {
	let dir = directory(name);
	std::fs::create_dir_all(&dir).map_err(|e| format!("create {dir:?}: {e}"))?;
	std::fs::write(dir.join(SOLUTION_FILE), format_solution(solution, base))
		.map_err(|e| format!("write solution: {e}"))?;
	std::fs::write(dir.join(SCORE_FILE), score.to_string())
		.map_err(|e| format!("write score: {e}"))?;
	Ok(())
}

pub fn mark_complete(name: &str, score: Score) -> Result<(), String> {
	let dir = directory(name);
	std::fs::create_dir_all(&dir).map_err(|e| format!("create {dir:?}: {e}"))?;
	std::fs::write(dir.join(COMPLETE_FLAG), score.to_string())
		.map_err(|e| format!("write complete flag: {e}"))?;
	Ok(())
}
