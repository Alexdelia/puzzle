use std::fs;
use std::path::Path;

use crate::{Coord, coord::parse_coord};

pub fn parse_solution_file(path: &Path) -> Result<Vec<Coord>, String> {
	let content = fs::read_to_string(path).map_err(|e| format!("failed to read {path:?}: {e}"))?;
	let mut out = Vec::new();
	for (i, line) in content.lines().enumerate() {
		let trimmed = line.trim();
		if trimmed.is_empty() {
			continue;
		}
		let xy = parse_coord(trimmed).map_err(|e| format!("{path:?} line {}: {e}", i + 1))?;
		out.push(xy);
	}
	Ok(out)
}
