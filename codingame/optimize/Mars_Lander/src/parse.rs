use std::str::FromStr;

use crate::{
	referee::{
		env::{Angle, Axis, Coord},
		lander::{Fuel, Lander, Power},
	},
	segment::Segment,
	solve::VALID_LANDING_INDEX,
};

pub fn get_path() -> Result<String, String> {
	let args = std::env::args().collect::<Vec<String>>();
	if args.len() < 2 {
		return Err("missing input file path argument".to_string());
	}
	Ok(args[1].clone())
}

pub fn get_iteration() -> Result<usize, String> {
	option_env!("ITERATION").map_or(Ok(1000), |s| s.parse::<usize>().map_err(|e| e.to_string()))
}

pub fn parse(path: &str) -> Result<(Lander, Vec<Segment>), String> {
	let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

	let lines = content.lines().collect::<Vec<&str>>();
	if lines.len() < 2 {
		return Err(
			"input file must contain at least lander line and one landscape line".to_string(),
		);
	}

	Ok((parse_lander(lines[0])?, parse_landsacpe(&lines[1..])?))
}

fn parse_n<T>(key: &str, s: Option<&str>) -> Result<T, String>
where
	T: FromStr,
	T::Err: ToString,
{
	s.ok_or(format!("missing {key}"))?
		.trim()
		.parse::<T>()
		.map_err(|e| e.to_string())
}

fn parse_lander(line: &str) -> Result<Lander, String> {
	let mut split = line.split_whitespace();
	Ok(Lander {
		x: parse_n::<Axis>("x coordinate", split.next())?,
		y: parse_n::<Axis>("y coordinate", split.next())?,
		sx: parse_n::<Axis>("horizontal speed", split.next())?,
		sy: parse_n::<Axis>("vertical speed", split.next())?,
		fuel: parse_n::<Fuel>("fuel", split.next())?,
		rotate: parse_n::<Angle>("rotate angle", split.next())?,
		power: parse_n::<Power>("thrust power", split.next())?,
	})
}

fn parse_landsacpe(lines: &[&str]) -> Result<Vec<Segment>, String> {
	let point_list: Vec<Coord> = lines
		.iter()
		.map(|line| {
			let mut split = line.split_whitespace();
			Ok(Coord {
				x: parse_n::<Axis>("x coordinate", split.next())?,
				y: parse_n::<Axis>("y coordinate", split.next())?,
			})
		})
		.collect::<Result<Vec<Coord>, String>>()?;

	let segment_list: Vec<Segment> = point_list
		.windows(2)
		.map(|segment_point| Segment {
			a: segment_point[0],
			b: segment_point[1],
		})
		.collect();

	Ok(segment_list)
}
