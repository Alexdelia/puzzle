use std::str::FromStr;

use crate::referee::{
	car::Car,
	env::{Axis, Coord, Degree},
};

const DEFAULT_ITERATION: usize = 128;

pub fn get_path() -> Result<String, String> {
	let args = std::env::args().collect::<Vec<String>>();
	if args.len() < 2 {
		return Err("missing input file path argument".to_string());
	}
	Ok(args[1].clone())
}

pub fn get_iteration() -> Result<usize, String> {
	option_env!("ITERATION").map_or(Ok(DEFAULT_ITERATION), |s| {
		s.parse::<usize>().map_err(|e| e.to_string())
	})
}

pub fn parse(path: &str) -> Result<(Car, Vec<Coord>), String> {
	let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

	let lines = content.lines().collect::<Vec<&str>>();
	if lines.len() < 2 {
		return Err(
			"input file must contain at least car line and one checkpoint line".to_string(),
		);
	}

	Ok((parse_car(lines[0])?, parse_checkpoint_list(&lines[1..])?))
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

fn parse_car(line: &str) -> Result<Car, String> {
	let mut split = line.split_whitespace();
	Ok(Car {
		x: parse_n::<Axis>("x coordinate", split.next())?,
		y: parse_n::<Axis>("y coordinate", split.next())?,
		sx: parse_n::<Axis>("horizontal speed", split.next())?,
		sy: parse_n::<Axis>("vertical speed", split.next())?,
		angle: parse_n::<Degree>("angle", split.next())?,
	})
}

fn parse_checkpoint_list(lines: &[&str]) -> Result<Vec<Coord>, String> {
	lines
		.iter()
		.map(|line| {
			let mut split = line.split_whitespace();
			Ok(Coord {
				x: parse_n::<Axis>("x coordinate", split.next())?,
				y: parse_n::<Axis>("y coordinate", split.next())?,
			})
		})
		.collect::<Result<Vec<Coord>, String>>()
}
