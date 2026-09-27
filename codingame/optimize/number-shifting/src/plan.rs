use crate::{Level, Move};

pub const ROOT: usize = usize::MAX;
const BRUTE_FORCE_CHILDREN: usize = 12;

pub struct Cell {
	pub x: usize,
	pub y: usize,
	pub value: i32,
	pub candidates: Vec<(usize, i32)>,
}

pub fn cells(level: &Level) -> Vec<Cell> {
	let mut cells = Vec::new();
	let mut index = vec![ROOT; level.w * level.h];
	for y in 0..level.h {
		for x in 0..level.w {
			if level.at(x, y) != 0 {
				index[y * level.w + x] = cells.len();
				cells.push(Cell {
					x,
					y,
					value: level.at(x, y),
					candidates: Vec::new(),
				});
			}
		}
	}
	for (i, cell) in cells.iter_mut().enumerate() {
		let (x, y) = (cell.x, cell.y);
		let row = (0..level.w).map(|other_x| (index[y * level.w + other_x], other_x.abs_diff(x)));
		let column =
			(0..level.h).map(|other_y| (index[other_y * level.w + x], other_y.abs_diff(y)));
		cell.candidates = row
			.chain(column)
			.filter(|&(j, _)| j != ROOT && j != i)
			.map(|(j, d)| (j, d as i32))
			.collect();
	}
	cells
}

pub fn direction(from: &Cell, to: &Cell) -> usize {
	match (to.x.cmp(&from.x), to.y.cmp(&from.y)) {
		(std::cmp::Ordering::Equal, std::cmp::Ordering::Greater) => 0,
		(std::cmp::Ordering::Greater, _) => 1,
		(std::cmp::Ordering::Equal, _) => 2,
		_ => 3,
	}
}

pub struct Rooted<'a> {
	pub cells: &'a [Cell],
	pub parent: Vec<usize>,
	pub target: Vec<i32>,
	pub children: Vec<Vec<usize>>,
}

impl Rooted<'_> {
	fn emit(&self, node: usize, moves: &mut Vec<Move>) -> Result<(), usize> {
		let distances = self.children[node]
			.iter()
			.map(|&c| self.target[c])
			.collect::<Vec<_>>();
		let steps =
			plan(self.cells[node].value, &distances, self.target[node]).map_err(|_| node)?;
		let mut sum = self.cells[node].value;
		for (i, delta) in steps {
			let child = self.children[node][i];
			self.emit(child, moves)?;
			let add = (sum > 0) == (delta > 0);
			sum += delta;
			let (c, p) = (&self.cells[child], &self.cells[node]);
			moves.push(Move {
				x: c.x,
				y: c.y,
				dir: direction(c, p),
				add,
			});
		}
		Ok(())
	}

	pub fn solution(&self) -> Result<Vec<Move>, usize> {
		let mut moves = Vec::new();
		for root in (0..self.cells.len()).filter(|&c| self.parent[c] == ROOT) {
			self.emit(root, &mut moves)?;
		}
		Ok(moves)
	}
}

pub fn signed_sums(value: i32, distances: &[i32]) -> Vec<i32> {
	let bound = value + distances.iter().sum::<i32>();
	let width = 2 * bound as usize + 1;
	let mut reachable = vec![false; width];
	reachable[(value + bound) as usize] = true;
	for &d in distances {
		let mut next = vec![false; width];
		for (i, _) in reachable.iter().enumerate().filter(|(_, r)| **r) {
			next[i + d as usize] = true;
			next[i - d as usize] = true;
		}
		reachable = next;
	}
	reachable
		.iter()
		.enumerate()
		.filter(|(_, r)| **r)
		.map(|(i, _)| i as i32 - bound)
		.collect()
}

fn no_early_zero(value: i32, steps: &[i32]) -> bool {
	let mut sum = value;
	steps[..steps.len() - 1].iter().all(|&d| {
		sum += d;
		sum != 0
	})
}

fn find_order(
	value: i32,
	steps: &mut Vec<i32>,
	used: &mut [bool],
	order: &mut Vec<usize>,
	all: &[i32],
) -> bool {
	if order.len() == all.len() {
		return true;
	}
	let sum = value + steps.iter().sum::<i32>();
	for i in 0..all.len() {
		if used[i] || (sum + all[i] == 0 && order.len() + 1 < all.len()) {
			continue;
		}
		used[i] = true;
		steps.push(all[i]);
		order.push(i);
		if find_order(value, steps, used, order, all) {
			return true;
		}
		order.pop();
		steps.pop();
		used[i] = false;
	}
	false
}

pub fn order_steps(value: i32, steps: &[i32]) -> Option<Vec<usize>> {
	let mut order = (0..steps.len()).collect::<Vec<_>>();
	order.sort_by_key(|&i| (steps[i] < 0, -steps[i].abs()));
	let sorted = order.iter().map(|&i| steps[i]).collect::<Vec<_>>();
	if no_early_zero(value, &sorted) {
		return Some(order);
	}
	if steps.len() > 8 {
		return None;
	}
	let mut order = Vec::new();
	let found = find_order(
		value,
		&mut Vec::new(),
		&mut vec![false; steps.len()],
		&mut order,
		steps,
	);
	found.then_some(order)
}

fn signed_steps(distances: &[i32], mask: u32) -> Vec<i32> {
	distances
		.iter()
		.enumerate()
		.map(|(i, &d)| if mask >> i & 1 == 1 { d } else { -d })
		.collect()
}

pub fn plan(value: i32, distances: &[i32], target: i32) -> Result<Vec<(usize, i32)>, i32> {
	if distances.is_empty() {
		return match target {
			0 => Err(value),
			t if t == value => Ok(Vec::new()),
			t => Err((value - t).abs()),
		};
	}
	if distances.len() > BRUTE_FORCE_CHILDREN {
		let best = signed_sums(value, distances)
			.into_iter()
			.map(|s| (s.abs() - target).abs())
			.min()
			.unwrap();
		return Err(best.max(1));
	}
	let mut best = i32::MAX;
	let mut crossing = None;
	for mask in 0..1u32 << distances.len() {
		let sum = distances.iter().enumerate().fold(value, |acc, (i, &d)| {
			acc + if mask >> i & 1 == 1 { d } else { -d }
		});
		if sum == target {
			let steps = signed_steps(distances, mask);
			let order =
				order_steps(value, &steps).expect("positive-final plan is always orderable");
			return Ok(order.into_iter().map(|i| (i, steps[i])).collect());
		}
		if sum == -target && crossing.is_none() {
			crossing = Some(mask);
		}
		best = best.min((sum.abs() - target).abs());
	}
	for mask in (0..1u32 << distances.len()).filter(|_| crossing.is_some()) {
		let sum = distances.iter().enumerate().fold(value, |acc, (i, &d)| {
			acc + if mask >> i & 1 == 1 { d } else { -d }
		});
		if sum != -target {
			continue;
		}
		let steps = signed_steps(distances, mask);
		if let Some(order) = order_steps(value, &steps) {
			return Ok(order.into_iter().map(|i| (i, steps[i])).collect());
		}
	}
	Err(best.max(1))
}

const SUM_WORDS: usize = 8;
const SUM_OFFSET: i32 = (SUM_WORDS * 32) as i32;
const ABS_WORDS: usize = SUM_WORDS / 2 + 1;

#[derive(Clone, Copy)]
pub struct SumSet {
	bits: [u64; SUM_WORDS],
}

impl SumSet {
	pub fn new(value: i32) -> SumSet {
		let mut set = SumSet {
			bits: [0; SUM_WORDS],
		};
		set.insert(value);
		set
	}

	pub fn of(value: i32, distances: impl IntoIterator<Item = i32>) -> SumSet {
		let mut set = SumSet::new(value);
		for d in distances {
			set.add(d);
		}
		set
	}

	fn insert(&mut self, sum: i32) {
		let bit = (sum + SUM_OFFSET) as usize;
		self.bits[bit / 64] |= 1 << (bit % 64);
	}

	pub fn contains(&self, sum: i32) -> bool {
		let bit = sum + SUM_OFFSET;
		if bit < 0 || bit >= (SUM_WORDS * 64) as i32 {
			return false;
		}
		let bit = bit as usize;
		self.bits[bit / 64] >> (bit % 64) & 1 == 1
	}

	pub fn contains_abs(&self, value: i32) -> bool {
		self.contains(value) || self.contains(-value)
	}

	pub fn add(&mut self, d: i32) {
		let (words, shift) = (d as usize / 64, d as u32 % 64);
		let mut next = [0u64; SUM_WORDS];
		for i in 0..SUM_WORDS {
			let w = self.bits[i];
			if w == 0 {
				continue;
			}
			if i + words < SUM_WORDS {
				next[i + words] |= w << shift;
				if shift > 0 && i + words + 1 < SUM_WORDS {
					next[i + words + 1] |= w >> (64 - shift);
				}
			}
			if i >= words {
				next[i - words] |= w >> shift;
				if shift > 0 && i > words {
					next[i - words - 1] |= w << (64 - shift);
				}
			}
		}
		self.bits = next;
	}

	pub fn union(&mut self, other: &SumSet) {
		for (mine, theirs) in self.bits.iter_mut().zip(other.bits) {
			*mine |= theirs;
		}
	}

	fn abs_mask(&self) -> [u64; ABS_WORDS] {
		let half = SUM_WORDS / 2;
		let mut mask = [0u64; ABS_WORDS];
		for k in 0..half {
			let mirrored = self.bits[half - 1 - k].reverse_bits();
			mask[k] |= self.bits[half + k] | mirrored << 1;
			mask[k + 1] |= mirrored >> 63;
		}
		mask
	}

	pub fn abs_values(&self) -> impl Iterator<Item = i32> + use<> {
		let mask = self.abs_mask();
		(0..ABS_WORDS).flat_map(move |k| {
			let mut rest = mask[k];
			std::iter::from_fn(move || {
				(rest != 0).then(|| {
					let bit = rest.trailing_zeros();
					rest &= rest - 1;
					(k * 64) as i32 + bit as i32
				})
			})
		})
	}

	pub fn min_abs(&self) -> i32 {
		self.abs_values().next().unwrap_or(SUM_OFFSET)
	}
}
