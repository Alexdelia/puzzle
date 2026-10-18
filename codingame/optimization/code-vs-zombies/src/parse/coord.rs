use crate::{Axis, Coord};

pub fn parse_coord(s: &str) -> Result<Coord, String> {
	let mut part_list = s.split_whitespace();
	let x = part_list
		.next()
		.ok_or_else(|| format!("missing x in {s:?}"))?
		.parse::<Axis>()
		.map_err(|e| format!("bad x in {s:?}: {e}"))?;
	let y = part_list
		.next()
		.ok_or_else(|| format!("missing y in {s:?}"))?
		.parse::<Axis>()
		.map_err(|e| format!("bad y in {s:?}: {e}"))?;
	Ok((x, y))
}

pub fn parse_coord_list(s: &str) -> Result<Vec<Coord>, String> {
	s.split(';').map(|p| parse_coord(p.trim())).collect()
}
