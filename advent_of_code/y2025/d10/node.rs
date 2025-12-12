use crate::Distance;

type Priority = u16;

pub struct Node<T> {
	pub t: T,
	pub dist: Distance,
	pub priority: Priority,
}

impl<T> PartialEq for Node<T> {
	fn eq(&self, other: &Self) -> bool {
		self.dist == other.dist && self.priority == other.priority
	}
}

impl<T> Eq for Node<T> {}

impl<T> PartialOrd for Node<T> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<T> Ord for Node<T> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.priority
			.cmp(&other.priority)
			.then_with(|| other.dist.cmp(&self.dist))
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
			Node {
				t: 0b0000,
				dist: 5,
				priority: 0,
			},
			Node {
				t: 0b0000,
				dist: 10,
				priority: 0,
			},
			Node {
				t: 0b0000,
				dist: 3,
				priority: 0,
			},
		]);

		assert_eq!(heap.pop().unwrap().dist, 3);
		assert_eq!(heap.pop().unwrap().dist, 5);
		assert_eq!(heap.pop().unwrap().dist, 10);
	}

	#[test]
	fn test_node_ordering_with_priority() {
		let mut heap = BinaryHeap::<Node<State>>::from([
			Node {
				t: 0b0000,
				dist: 5,
				priority: 2,
			},
			Node {
				t: 0b0000,
				dist: 5,
				priority: 1,
			},
			Node {
				t: 0b0000,
				dist: 3,
				priority: 0,
			},
		]);

		let first = heap.pop().unwrap();
		assert_eq!(first.dist, 5);
		assert_eq!(first.priority, 2);
		let second = heap.pop().unwrap();
		assert_eq!(second.dist, 5);
		assert_eq!(second.priority, 1);
		let third = heap.pop().unwrap();
		assert_eq!(third.dist, 3);
		assert_eq!(third.priority, 0);
	}
}
