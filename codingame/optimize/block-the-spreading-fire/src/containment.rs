use crate::map::{Cell, Map};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

const UNREACHED: u32 = u32::MAX;

pub struct Containment {
	pub loss: u32,
	pub lateness: u32,
	pub region: Vec<usize>,
	pub cuts: Vec<usize>,
}

impl Containment {
	pub fn energy(&self, lateness_penalty: f64) -> f64 {
		self.loss as f64 + lateness_penalty * self.lateness as f64
	}

	pub fn is_feasible(&self) -> bool {
		self.lateness == 0
	}
}

pub fn contain(map: &Map, inside: &[bool]) -> Containment {
	let mut ignition = vec![UNREACHED; map.cells.len()];
	let mut deadline = vec![UNREACHED; map.cells.len()];
	let mut region = Vec::new();
	let mut cuts = Vec::new();
	let mut heap = BinaryHeap::from([Reverse((0, map.start))]);
	ignition[map.start] = 0;
	while let Some(Reverse((time, i))) = heap.pop() {
		if time > ignition[i] {
			continue;
		}
		region.push(i);
		let spread = time + map.params(i).fire_duration as u32;
		for n in map.neighbours(i) {
			if map.cells[n] == Cell::Safe {
				continue;
			}
			if inside[n] || n == map.start {
				if spread < ignition[n] {
					ignition[n] = spread;
					heap.push(Reverse((spread, n)));
				}
			} else {
				if deadline[n] == UNREACHED {
					cuts.push(n);
				}
				deadline[n] = deadline[n].min(spread);
			}
		}
	}
	cuts.sort_by_key(|&j| deadline[j] + map.params(j).cut_duration);
	let mut turn: u32 = 1;
	let mut lateness = 0;
	for &j in &cuts {
		lateness += turn.saturating_sub(deadline[j]);
		turn += map.params(j).cut_duration;
	}
	let loss = region
		.iter()
		.chain(&cuts)
		.map(|&i| map.params(i).value)
		.sum();
	Containment {
		loss,
		lateness,
		region,
		cuts,
	}
}
