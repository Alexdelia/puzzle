use std::collections::HashMap;

use crate::{Distance, Joltage, State};

pub struct LightNode {
	pub state: State,
	pub dist: Distance,
}

impl PartialEq for LightNode {
	fn eq(&self, other: &Self) -> bool {
		self.dist == other.dist
	}
}

impl Eq for LightNode {}

impl PartialOrd for LightNode {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for LightNode {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		other.dist.cmp(&self.dist)
	}
}

pub struct JoltageNode {
	pub state: HashMap<usize, (Joltage, Vec<usize>)>,
	pub dist: Distance,
}

impl PartialEq for JoltageNode {
	fn eq(&self, other: &Self) -> bool {
		self.dist == other.dist && self.state.len() == other.state.len()
	}
}

impl Eq for JoltageNode {}

impl PartialOrd for JoltageNode {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for JoltageNode {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		other
			.state
			.iter()
			.map(|(_, (_, button_list))| button_list.len())
			.sum::<usize>()
			.cmp(
				&self
					.state
					.iter()
					.map(|(_, (_, button_list))| button_list.len())
					.sum::<usize>(),
			)
			.then_with(|| {
				other
					.state
					.iter()
					.map(|(_, (joltage, _))| *joltage as usize)
					.sum::<usize>()
					.cmp(
						&self
							.state
							.iter()
							.map(|(_, (joltage, _))| *joltage as usize)
							.sum::<usize>(),
					)
			})
			.then_with(|| other.dist.cmp(&self.dist))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::collections::BinaryHeap;

	#[test]
	fn test_light_node_ordering() {
		let mut heap = BinaryHeap::<LightNode>::from([
			LightNode {
				state: 0b0000,
				dist: 5,
			},
			LightNode {
				state: 0b0000,
				dist: 10,
			},
			LightNode {
				state: 0b0000,
				dist: 3,
			},
		]);

		assert_eq!(heap.pop().unwrap().dist, 3);
		assert_eq!(heap.pop().unwrap().dist, 5);
		assert_eq!(heap.pop().unwrap().dist, 10);
	}

	#[test]
	fn test_joltage_node_ordering() {
		let mut heap = BinaryHeap::<JoltageNode>::from([
			JoltageNode {
				state: HashMap::from([(0, (0, vec![0])), (1, (1, vec![1]))]),
				dist: 5,
			},
			JoltageNode {
				state: HashMap::from([(0, (0, vec![0]))]),
				dist: 10,
			},
			JoltageNode {
				state: HashMap::from([(0, (0, vec![0])), (1, (1, vec![1])), (2, (2, vec![2]))]),
				dist: 3,
			},
			JoltageNode {
				state: HashMap::from([(0, (0, vec![0]))]),
				dist: 2,
			},
		]);

		assert_eq!(heap.pop().unwrap().dist, 2);
		assert_eq!(heap.pop().unwrap().dist, 10);
		assert_eq!(heap.pop().unwrap().dist, 5);
		assert_eq!(heap.pop().unwrap().dist, 3);
	}
}
