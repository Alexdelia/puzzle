use aocd::*;
use std::collections::BinaryHeap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Point {
	x: PointUnit,
	y: PointUnit,
	z: PointUnit,
}

type PointUnit = u32;

struct Connection {
	point: (Point, Point),
	distance: PointUnit,
}

impl Eq for Connection {}

impl PartialEq for Connection {
	fn eq(&self, other: &Self) -> bool {
		self.distance == other.distance
	}
}

impl Ord for Connection {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.distance.cmp(&other.distance)
	}
}

impl PartialOrd for Connection {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Connection {
	fn new(a: Point, b: Point) -> Self {
		let distance = Self::_calc_distance(&a, &b);
		Self {
			point: (a, b),
			distance,
		}
	}

	fn _calc_distance(a: &Point, b: &Point) -> PointUnit {
		(a.x as i32 - b.x as i32).abs() as PointUnit
			+ (a.y as i32 - b.y as i32).abs() as PointUnit
			+ (a.z as i32 - b.z as i32).abs() as PointUnit
	}
}

#[aocd(2025, 8)]
fn main() {
	let lines: Vec<String> = input!().lines().map(|s| s.to_string()).collect();

	dbg!(lines.len());
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn heap_order() {
		let a = Point {
			x: 162,
			y: 817,
			z: 812,
		};
		let b = Point {
			x: 425,
			y: 690,
			z: 689,
		};
		let c = Point {
			x: 431,
			y: 825,
			z: 988,
		};

		let mut q: BinaryHeap<Connection> =
			BinaryHeap::from(vec![Connection::new(a, b), Connection::new(a, c)]);

		assert_eq!(q.pop().unwrap().point.1, b);
		assert_eq!(q.pop().unwrap().point.1, c);
	}
}
