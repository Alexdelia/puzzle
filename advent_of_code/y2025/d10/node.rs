use crate::{Distance, State};

pub struct Node {
	pub state: State,
	pub dist: Distance,
}

impl PartialEq for Node {
	fn eq(&self, other: &Self) -> bool {
		self.dist.eq(&other.dist)
	}
}

impl Eq for Node {}

impl PartialOrd for Node {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(other.dist.cmp(&self.dist))
	}
}

impl Ord for Node {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		other.dist.cmp(&self.dist)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::collections::BinaryHeap;

	#[test]
	fn test_node_ordering() {
		let mut heap = BinaryHeap::from([
			Node {
				state: 0b0000,
				dist: 5,
			},
			Node {
				state: 0b0000,
				dist: 10,
			},
			Node {
				state: 0b0000,
				dist: 3,
			},
		]);

		assert_eq!(heap.pop().unwrap().dist, 3);
		assert_eq!(heap.pop().unwrap().dist, 5);
		assert_eq!(heap.pop().unwrap().dist, 10);
	}
}
