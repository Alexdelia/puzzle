use crate::point::{Point, PointUnit};

#[derive(Debug, Clone, Copy)]
pub struct Connection {
	pub point: (Point, Point),
	pub distance: PointUnit,
}

impl Eq for Connection {}

impl PartialEq for Connection {
	fn eq(&self, other: &Self) -> bool {
		self.distance == other.distance
	}
}

impl Ord for Connection {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		other.distance.cmp(&self.distance)
	}
}

impl PartialOrd for Connection {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(other.distance.cmp(&self.distance))
	}
}

impl Connection {
	pub fn new(a: Point, b: Point) -> Self {
		let distance = Self::calc_distance(&a, &b);
		Self {
			point: (a, b),
			distance,
		}
	}

	/// euclidean distance
	fn calc_distance(a: &Point, b: &Point) -> PointUnit {
		(((a.x as i64 - b.x as i64).pow(2)
			+ (a.y as i64 - b.y as i64).pow(2)
			+ (a.z as i64 - b.z as i64).pow(2)) as f64)
			.sqrt() as PointUnit
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::BinaryHeap;

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
