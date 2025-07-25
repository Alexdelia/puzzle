use std::cmp::min;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::io::stdin;
use std::str::FromStr;

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

type Resource = u32;
type Strength = std::num::NonZeroU32;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum CellType {
	None,
	Egg,
	Crystal,
}

#[derive(Debug, Eq, PartialEq)]
struct Cell {
	r#type: CellType,
	resource: Resource,
	neighbor: [Option<usize>; 6],
	my_ant: Resource,
	opp_ant: Resource,
}

struct Env {
	cell: Vec<Cell>,
	init_crystal: Resource,
	remain_crystal: Resource,
	remain_ant: Resource,
	// n_base: usize,
	my_base: Vec<usize>,
	// opp_base: Vec<usize>,
	my_score: Resource,
	opp_score: Resource,
	my_ant: Resource,
	opp_ant: Resource,
	beacon: HashMap<usize, Strength>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Flood {
	path: Vec<usize>,
	score: u32,
}

impl Ord for Flood {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		other.score.cmp(&self.score)
	}
}

impl PartialOrd for Flood {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl FromStr for CellType {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"0" => Ok(Self::None),
			"1" => Ok(Self::Egg),
			"2" => Ok(Self::Crystal),
			_ => Err(()),
		}
	}
}

impl FromStr for Cell {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut i = s.split_whitespace();

		Ok(Self {
			r#type: i.next().unwrap().parse::<CellType>().unwrap(),
			resource: i.next().unwrap().parse::<Resource>().unwrap(),
			neighbor: [
				i.next().unwrap().parse::<usize>().ok(),
				i.next().unwrap().parse::<usize>().ok(),
				i.next().unwrap().parse::<usize>().ok(),
				i.next().unwrap().parse::<usize>().ok(),
				i.next().unwrap().parse::<usize>().ok(),
				i.next().unwrap().parse::<usize>().ok(),
			],
			my_ant: 0,
			opp_ant: 0,
		})
	}
}

#[inline]
fn gain_type(gain: (Resource, Resource), r#type: Option<CellType>) -> Resource {
	match r#type {
		Some(CellType::Crystal) => gain.0,
		Some(CellType::Egg) => gain.1,
		_ => gain.0 + gain.1,
	}
}

impl Env {
	fn new() -> Self {
		let mut buf = String::new();
		stdin().read_line(&mut buf).unwrap();
		let n_cell = parse_input!(buf, usize);

		let mut cell: Vec<Cell> = Vec::with_capacity(n_cell);
		let mut init_crystal = 0;

		for _ in 0..n_cell {
			let mut buf = String::new();
			stdin().read_line(&mut buf).unwrap();

			cell.push(buf.parse::<Cell>().unwrap());
			init_crystal += cell.last().unwrap().resource;
		}

		let mut buf = String::new();
		stdin().read_line(&mut buf).unwrap();
		// let n_base = parse_input!(buf, usize);

		let mut buf = String::new();
		stdin().read_line(&mut buf).unwrap();
		let my_base = buf
			.split_whitespace()
			.map(|s| s.parse::<usize>().unwrap())
			.collect();

		let mut buf = String::new();
		stdin().read_line(&mut buf).unwrap();
		/*
		let opp_base = buf
			.split_whitespace()
			.map(|s| s.parse::<usize>().unwrap())
			.collect();
		*/

		Env {
			cell,
			init_crystal,
			remain_crystal: init_crystal,
			remain_ant: 0,
			// n_base,
			my_base,
			// opp_base,
			my_score: 0,
			opp_score: 0,
			my_ant: 0,
			opp_ant: 0,
			beacon: HashMap::new(),
		}
	}

	fn update(&mut self) {
		self.my_ant = 0;
		self.opp_ant = 0;
		self.remain_crystal = 0;
		self.remain_ant = 0;

		let mut buf = String::new();
		stdin().read_line(&mut buf).unwrap();
		let mut sw = buf.split_whitespace();
		self.my_score = sw.next().unwrap().parse::<Resource>().unwrap();
		self.opp_score = sw.next().unwrap().parse::<Resource>().unwrap();

		for i in 0..self.cell.len() {
			let mut buf = String::new();
			stdin().read_line(&mut buf).unwrap();

			let mut sw = buf.split_whitespace();

			self.cell[i].resource = sw.next().unwrap().parse::<Resource>().unwrap();
			match self.cell[i].r#type {
				CellType::Crystal => self.remain_crystal += self.cell[i].resource,
				CellType::Egg => self.remain_ant += self.cell[i].resource,
				_ => (),
			}

			self.cell[i].my_ant = sw.next().unwrap().parse::<Resource>().unwrap();
			self.my_ant += self.cell[i].my_ant;
			self.cell[i].opp_ant = sw.next().unwrap().parse::<Resource>().unwrap();
			self.opp_ant += self.cell[i].opp_ant;
		}
	}

	fn output(&self) {
		let mut output = String::new();

		for (index, strength) in &self.beacon {
			output.push_str(&format!("BEACON {index} {strength};"));
		}

		if output.is_empty() {
			output.push_str("WAIT;");
		}

		let in_game_ant = self.my_ant + self.opp_ant;
		println!(
			"{output} MESSAGE 💎 {}%  |  🐜 {}%  |  {} {}% - 👤 {}%",
			self.remain_crystal * 100 / self.init_crystal,
			self.remain_ant * 100 / in_game_ant,
			if !self.endgame() { "🧙" } else { "🔥" },
			self.my_ant * 100 / in_game_ant,
			self.opp_ant * 100 / in_game_ant,
		);
	}

	/// return (Crystal, Ant)
	fn gain(&self, beacon: &HashMap<usize, Strength>, ant: Resource) -> (Resource, Resource) {
		if beacon.is_empty() {
			return (0, 0);
		}

		// weakest == ant / (sum of strength / lowest strength)
		let mut sum = 0;
		let mut lowest = u32::MAX;
		for i in beacon.values() {
			sum += i.get();
			if i.get() < lowest {
				lowest = i.get();
			}
		}

		let weakest = ant / (sum / lowest);
		let mut crystal = 0;
		let mut egg = 0;

		// does not yet calculate ant battle

		for i in beacon {
			match self.cell[*i.0].r#type {
				CellType::Crystal => crystal += min(self.cell[*i.0].resource, weakest),
				CellType::Egg => egg += min(self.cell[*i.0].resource, weakest),
				_ => (),
			}
		}

		(crystal, egg)
	}

	fn endgame(&self) -> bool {
		// endgame if:
		// need 10% of crystal to win
		// or
		// crystal to win < 50	// crystal < 100 if no score
		// or
		// less than 10% of ant left

		let endgame_score_threshold = (self.init_crystal as f32 / 2.0 * 0.9) as Resource;

		// self.my_score * 10 <= self.my_score + self.opp_score || self.opp_score * 10 <= self.my_score + self.opp_score
		self.my_score >= endgame_score_threshold
			|| self.opp_score >= endgame_score_threshold
			|| self.remain_crystal < 100
			|| self.remain_ant * 10 <= self.my_ant + self.opp_ant
	}

	fn ant_around(&self, index: usize) -> bool {
		for i in 0..6 {
			if let Some(i) = self.cell[index].neighbor[i] {
				if self.cell[i].my_ant > 0 {
					return true;
				}
			}
		}

		false
	}

	fn beacon_around(&self, index: usize) -> bool {
		for i in 0..6 {
			if let Some(i) = self.cell[index].neighbor[i] {
				if self.beacon.contains_key(&i) {
					return true;
				}
			}
		}

		false
	}

	// return path (+ Resource found as last element)
	fn beacon_flood(&self, r#type: Option<CellType>) -> Vec<Vec<usize>> {
		let mut queue: BinaryHeap<Flood> = BinaryHeap::new();
		let mut visited = vec![false; self.cell.len()];

		for i in self.beacon.iter() {
			queue.push(Flood {
				path: vec![*i.0],
				score: 0,
			});
			visited[*i.0] = true;
		}

		let mut found: Vec<Vec<usize>> = Vec::new();

		while let Some(f) = queue.pop() {
			let index = *f.path.last().unwrap();

			if self.cell[index].resource > 0
				&& !self.beacon.contains_key(&index)
				&& r#type
					.as_ref()
					.map_or(true, |t| self.cell[index].r#type == *t)
			{
				found.push(f.path);
				continue;
			}

			if !found.is_empty() && f.path.len() >= found[0].len() + 1 {
				// >= because another index is gonna be added this turn
				// + 1 because want to parse +1 index
				// (I do not use > in case I remove or modify the +1)
				break;
			}

			for i in 0..6 {
				if let Some(i) = self.cell[index].neighbor[i] {
					if !visited[i] {
						let mut flood = f.clone();
						flood.path.push(i);
						flood.score += if self.cell[i].my_ant > 0 || self.ant_around(i) {
							1
						} else {
							2
						};
						queue.push(flood);
						visited[i] = true;
					}
				}
			}
		}

		found
	}

	/// return (gain, best_beacon + self.beacon)
	fn best_beacon_list(
		&self,
		beacon: Vec<Vec<usize>>,
		only_calc: Option<CellType>,
		force: bool,
	) -> Option<(Resource, HashMap<usize, Strength>)> {
		if beacon.is_empty() {
			return None;
		}

		let current_gain = gain_type(self.gain(&self.beacon, self.my_ant), only_calc);

		let mut best: Option<(Resource, HashMap<usize, Strength>)> = None;

		for b in beacon.into_iter() {
			// calculate gain of self.beacon + b
			let b: HashMap<usize, Strength> = self
				.beacon
				.iter()
				.map(|(k, v)| (*k, *v))
				.chain(b.iter().map(|i| (*i, Strength::new(1).unwrap())))
				.collect();
			let gain = gain_type(self.gain(&b, self.my_ant), only_calc);

			if (force || gain > current_gain || gain == current_gain && b.len() > self.beacon.len())
				&& best.as_ref().map_or(true, |b| gain > b.0)
			{
				best = Some((gain, b));
			}
		}

		best
	}

	fn resource_group(&self, index: usize) -> Vec<usize> {
		let mut queue = VecDeque::new();
		let mut visited = vec![false; self.cell.len()];
		let mut group = Vec::new();

		queue.push_back(index);
		visited[index] = true;

		while let Some(index) = queue.pop_front() {
			if self.cell[index].resource > 0 {
				group.push(index);
			}

			for i in 0..6 {
				if let Some(i) = self.cell[index].neighbor[i] {
					if !visited[i] && self.cell[i].resource > 0 {
						queue.push_back(i);
						visited[i] = true;
					}
				}
			}
		}

		group
	}

	fn apply_best_beacon(
		&mut self,
		search: Option<CellType>,
		only_calc: Option<CellType>,
		force: bool,
	) -> Option<Resource> {
		if let Some((gain, beacon)) = self.best_beacon_list(
			self.beacon_flood(search)
				.into_iter()
				.map(|mut b| {
					b.extend(self.resource_group(*b.last().unwrap()));
					b
				})
				.collect::<Vec<_>>(),
			only_calc,
			force,
		) {
			self.beacon = beacon;

			Some(gain)
		} else {
			None
		}
	}

	fn on_crystal(&self) -> usize {
		self.beacon
			.iter()
			.filter(|(i, _)| {
				self.cell[**i].r#type == CellType::Crystal && self.cell[**i].resource > 0
			})
			.count()
	}

	fn compute_beacon(&mut self, clear_beacon: bool) {
		if clear_beacon {
			self.beacon.clear();
		}

		let endgame = self.endgame();

		for i in self.my_base.iter() {
			self.beacon.insert(
				*i,
				Strength::new(1).unwrap(),
				// Strength::new(
				//     (self.cell[*i].opp_ant as f32 / self.cell[*i].my_ant as f32).ceil() as u32,
				// )
				// .unwrap_or(Strength::new(1).unwrap()),
			);
		}

		if self.remain_ant > 0 && !endgame {
			if let Some(gain) = self.apply_best_beacon(Some(CellType::Egg), None, true) {
				dbg!(gain, self.beacon.len());
			}
		}

		if !endgame {
			while let Some(gain) = self.apply_best_beacon(None, None, false) {
				dbg!(gain, self.beacon.len());
			}
		}

		if endgame || self.my_ant > (self.opp_ant as f32 * 1.25) as Resource {
			let crystal_cell = self
				.cell
				.iter()
				.filter(|c| c.r#type == CellType::Crystal && c.resource > 0)
				.count();

			let objective = (crystal_cell as f32 / 0.66).ceil() as usize;

			while self.on_crystal() < objective {
				if let Some(gain) = self.apply_best_beacon(Some(CellType::Crystal), None, true) {
					dbg!(gain, self.beacon.len());
				} else {
					break;
				}
			}
		}

		for i in self.my_base.iter() {
			if !self.beacon_around(*i) {
				self.beacon.remove(i);
			}
		}
	}
}

fn main() {
	let mut env = Env::new();

	loop {
		env.update();

		env.compute_beacon(true);

		env.output();
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_cell_parse() {
		assert_eq!(
			"0 0 1 2 3 4 5 6".parse::<Cell>(),
			Ok(Cell {
				r#type: CellType::None,
				resource: 0,
				neighbor: [Some(1), Some(2), Some(3), Some(4), Some(5), Some(6)],
				my_ant: 0,
				opp_ant: 0
			})
		);

		assert_eq!(
			"1 0 -1 -1 8 -1 -1 -1".parse::<Cell>(),
			Ok(Cell {
				r#type: CellType::Egg,
				resource: 0,
				neighbor: [None, None, Some(8), None, None, None],
				my_ant: 0,
				opp_ant: 0
			})
		);

		assert_eq!(
			"2 42 13 14 15 2 1 0".parse::<Cell>(),
			Ok(Cell {
				r#type: CellType::Crystal,
				resource: 42,
				neighbor: [Some(13), Some(14), Some(15), Some(2), Some(1), Some(0)],
				my_ant: 0,
				opp_ant: 0
			})
		);
	}

	#[test]
	fn test_flood_binary_heap() {
		let mut heap: BinaryHeap<Flood> = BinaryHeap::new();

		heap.push(Flood {
			path: vec![0],
			score: 0,
		});
		heap.push(Flood {
			path: vec![2],
			score: 2,
		});
		heap.push(Flood {
			path: vec![1],
			score: 1,
		});
		heap.push(Flood {
			path: vec![3],
			score: 3,
		});

		assert_eq!(
			heap.pop(),
			Some(Flood {
				path: vec![0],
				score: 0
			})
		);
		assert_eq!(
			heap.pop(),
			Some(Flood {
				path: vec![1],
				score: 1
			})
		);
		assert_eq!(
			heap.pop(),
			Some(Flood {
				path: vec![2],
				score: 2
			})
		);
		assert_eq!(
			heap.pop(),
			Some(Flood {
				path: vec![3],
				score: 3
			})
		);
	}
}
