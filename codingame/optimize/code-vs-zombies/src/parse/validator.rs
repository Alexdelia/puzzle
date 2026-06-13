use std::fs;
use std::path::Path;

use crate::InitialState;
use crate::parse::coord::{parse_coord, parse_coord_list};

pub fn parse_validator(path: &Path) -> Result<InitialState, String> {
	let content = fs::read_to_string(path).map_err(|e| format!("failed to read {path:?}: {e}"))?;
	let mut iter = content.lines();
	let _flip = iter.next().unwrap_or("").trim().to_string();

	let mut pack_line_list: Vec<&str> = Vec::new();
	for line in iter {
		let trimmed = line.trim();
		if trimmed.is_empty() {
			if !pack_line_list.is_empty() {
				break;
			}
			continue;
		}
		pack_line_list.push(trimmed);
	}

	if pack_line_list.len() < 3 {
		return Err(format!(
			"expected at least 3 lines in first pack of {path:?}, got {}",
			pack_line_list.len()
		));
	}

	let player =
		parse_coord(pack_line_list[0]).map_err(|e| format!("bad player line in {path:?}: {e}"))?;
	let human_list = parse_coord_list(pack_line_list[1])
		.map_err(|e| format!("bad human list in {path:?}: {e}"))?;
	let zombie_list = parse_coord_list(pack_line_list[2])
		.map_err(|e| format!("bad zombie list in {path:?}: {e}"))?;

	Ok(InitialState {
		player,
		human_list,
		zombie_list,
	})
}
