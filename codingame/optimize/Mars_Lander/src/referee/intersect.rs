use crate::segment::Segment;

pub fn intersect(a: &Segment, b: &Segment) -> bool {
	let sa_x = a.b.x - a.a.x;
	let sa_y = a.b.y - a.a.y;
	let sb_x = b.b.x - b.a.x;
	let sb_y = b.b.y - b.a.y;

	let k = 1.0 / (-sb_x * sa_y + sa_x * sb_y);

	let s = (-sa_y * (a.a.x - b.a.x) + sa_x * (a.a.y - b.a.y)) * k;
	let t = (sb_x * (a.a.y - b.a.y) - sb_y * (a.a.x - b.a.x)) * k;

	s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::referee::env::Coord;

	#[test]
	fn test_intersect() {
		assert!(intersect(
			&Segment {
				a: Coord { x: 0.0, y: 0.0 },
				b: Coord { x: 4.0, y: 4.0 },
			},
			&Segment {
				a: Coord { x: 0.0, y: 4.0 },
				b: Coord { x: 4.0, y: 0.0 },
			}
		));
	}

	#[test]
	fn test_no_intersect() {
		assert!(!intersect(
			&Segment {
				a: Coord { x: 0.0, y: 0.0 },
				b: Coord { x: 2.0, y: 2.0 },
			},
			&Segment {
				a: Coord { x: 3.0, y: 3.0 },
				b: Coord { x: 4.0, y: 4.0 },
			}
		));
	}
}
