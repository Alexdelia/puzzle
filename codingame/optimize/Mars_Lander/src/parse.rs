use crate::{
	referee::env::{Axis, Coord},
	segment::Segment,
};

pub fn get_path() -> Result<String, String> {
	let args = std::env::args().collect::<Vec<String>>();
	if args.len() < 2 {
		return Err("missing input file path argument".to_string());
	}
	Ok(args[1].clone())
}

pub fn parse(path: &str) -> Result<Vec<Segment>, String> {
	let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

	let point_list: Vec<Coord> = content
		.lines()
		.map(|line| {
			let mut split = line.split_whitespace();
			Ok(Coord {
				x: split
					.next()
					.ok_or("missing x coordinate".to_string())?
					.parse::<Axis>()
					.map_err(|e| e.to_string())?,
				y: split
					.next()
					.ok_or("missing y coordinate".to_string())?
					.parse::<Axis>()
					.map_err(|e| e.to_string())?,
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
