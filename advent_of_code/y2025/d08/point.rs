use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
	pub x: PointUnit,
	pub y: PointUnit,
	pub z: PointUnit,
}

pub type PointUnit = u32;

impl FromStr for Point {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let coords: Vec<&str> = s.trim().split(',').collect();
		if coords.len() != 3 {
			return Err(format!("Invalid point format: {s}"));
		}

		Ok(Point {
			x: coords[0]
				.parse::<PointUnit>()
				.map_err(|e| format!("Invalid x coordinate: {e}"))?,
			y: coords[1]
				.parse::<PointUnit>()
				.map_err(|e| format!("Invalid y coordinate: {e}"))?,
			z: coords[2]
				.parse::<PointUnit>()
				.map_err(|e| format!("Invalid z coordinate: {e}"))?,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_point() {
		let point_str = "162,817,812";
		let point = point_str.parse::<Point>();
		assert_eq!(
			point,
			Ok(Point {
				x: 162,
				y: 817,
				z: 812
			})
		);
	}
}
