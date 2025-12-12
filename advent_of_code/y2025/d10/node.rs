use crate::Distance;

pub struct Node<T> {
	pub t: T,
	pub dist: Distance,
}

impl<T> PartialEq for Node<T> {
	fn eq(&self, other: &Self) -> bool {
		self.dist.eq(&other.dist)
	}
}

impl<T> Eq for Node<T> {}

impl<T> PartialOrd for Node<T> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(other.dist.cmp(&self.dist))
	}
}

impl<T> Ord for Node<T> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		other.dist.cmp(&self.dist)
	}
}

#[cfg(test)]
mod tests {
	use crate::State;

	use super::*;

	use std::collections::BinaryHeap;

	#[test]
	fn test_node_ordering() {
		let mut heap = BinaryHeap::<Node<State>>::from([
			Node { t: 0b0000, dist: 5 },
			Node {
				t: 0b0000,
				dist: 10,
			},
			Node { t: 0b0000, dist: 3 },
		]);

		assert_eq!(heap.pop().unwrap().dist, 3);
		assert_eq!(heap.pop().unwrap().dist, 5);
		assert_eq!(heap.pop().unwrap().dist, 10);
	}
}
