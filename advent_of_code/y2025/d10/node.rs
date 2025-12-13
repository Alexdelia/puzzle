use crate::{Distance, JoltageButton, JoltageList, State};

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

pub type ButtonMask = u16;
pub struct JoltageNode {
	pub active_joltage: Vec<usize>,
	pub joltage_list: JoltageList,
	pub joltage_button_list: [JoltageButton; 16],
	pub dist: Distance,
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
}
