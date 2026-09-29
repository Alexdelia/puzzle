use number_shifting::plan::{Cell, ROOT, Rooted, SumSet, cells, plan};
use number_shifting::{DX, DY, Level, Move, Rng};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

fn knob(name: &str, default: f64) -> f64 {
	std::env::var(name)
		.ok()
		.and_then(|s| s.parse().ok())
		.unwrap_or(default)
}

const REMAINING_WEIGHT: i64 = 64;
const ASSEMBLE_CANDIDATES: usize = 24;

#[derive(Clone, Copy)]
enum Delta {
	Add(i32),
	Remove(usize),
}

#[derive(Clone, Copy)]
struct Step {
	moved: usize,
	new_parent: usize,
	distance: i32,
}

struct ChainNode {
	prev: usize,
	step: Step,
}

enum Op {
	Link(usize, usize),
	Unlink(usize, usize, i32),
}

type Snapshot = (Vec<usize>, Vec<i32>);

struct Elite {
	score: i64,
	key: u64,
	state: Snapshot,
}

struct Pool {
	elites: Vec<Elite>,
	capacity: usize,
}

fn snapshot_key(state: &Snapshot) -> u64 {
	let mut hash = 0xcbf2_9ce4_8422_2325u64;
	for (&parent, &target) in state.0.iter().zip(&state.1) {
		hash = (hash ^ parent as u64).wrapping_mul(0x100_0000_01b3);
		hash = (hash ^ target as u64).wrapping_mul(0x100_0000_01b3);
	}
	hash
}

impl Pool {
	fn offer(&mut self, score: i64, state: &Snapshot) {
		let key = snapshot_key(state);
		if self.elites.iter().any(|e| e.key == key) {
			return;
		}
		let elite = Elite {
			score,
			key,
			state: state.clone(),
		};
		if self.elites.len() < self.capacity {
			self.elites.push(elite);
			return;
		}
		let worst = (0..self.elites.len())
			.max_by_key(|&i| self.elites[i].score)
			.unwrap();
		if score < self.elites[worst].score {
			self.elites[worst] = elite;
		}
	}

	fn pick(&self, rng: &mut Rng) -> Option<(i64, Snapshot)> {
		if self.elites.is_empty() {
			return None;
		}
		let a = &self.elites[rng.below(self.elites.len())];
		let b = &self.elites[rng.below(self.elites.len())];
		let winner = if a.score <= b.score { a } else { b };
		Some((winner.score, winner.state.clone()))
	}
}

struct Forest {
	width: usize,
	height: usize,
	cell_at: Vec<usize>,
	cells: Vec<Cell>,
	parent: Vec<usize>,
	target: Vec<i32>,
	children: Vec<Vec<usize>>,
	residual: Vec<i32>,
	remaining: Vec<usize>,
	remaining_slot: Vec<usize>,
	residual_sum: i64,
	weight: Vec<i64>,
	penalty_sum: i64,
	journal: Vec<Op>,
	sum_cache: Vec<SumSet>,
	neutral_tries: usize,
}

impl Forest {
	fn new(level: &Level) -> Forest {
		let cells = cells(level);
		let n = cells.len();
		let mut cell_at = vec![ROOT; level.w * level.h];
		for (i, c) in cells.iter().enumerate() {
			cell_at[c.y * level.w + c.x] = i;
		}
		let residual = cells.iter().map(|c| c.value).collect::<Vec<_>>();
		let base_weight = knob("WEIGHT_BASE", 64.0) as i64;
		Forest {
			penalty_sum: residual
				.iter()
				.map(|&r| base_weight * (REMAINING_WEIGHT + r as i64))
				.sum(),
			weight: vec![base_weight; n],
			width: level.w,
			height: level.h,
			cell_at,
			residual_sum: residual.iter().map(|&r| r as i64).sum(),
			residual,
			parent: vec![ROOT; n],
			target: vec![0; n],
			children: vec![Vec::new(); n],
			remaining: (0..n).collect(),
			remaining_slot: (0..n).collect(),
			journal: Vec::new(),
			sum_cache: cells.iter().map(|c| SumSet::new(c.value)).collect(),
			neutral_tries: knob("NEUTRAL", 8.0) as usize,
			cells,
		}
	}

	fn score(&self) -> i64 {
		self.penalty_sum
	}

	fn raw_score(&self) -> i64 {
		self.remaining.len() as i64 * REMAINING_WEIGHT + self.residual_sum
	}

	fn penalty(&self, node: usize, residual: i32) -> i64 {
		if residual > 0 {
			self.weight[node] * (REMAINING_WEIGHT + residual as i64)
		} else {
			0
		}
	}

	fn pick_remaining(&self, rng: &mut Rng) -> usize {
		self.remaining[rng.below(self.remaining.len())]
	}

	fn bump_weights(&mut self, step: i64) {
		for &node in &self.remaining {
			self.weight[node] += step;
			self.penalty_sum += step * (REMAINING_WEIGHT + self.residual[node] as i64);
		}
	}

	fn distances(&self, node: usize) -> Vec<i32> {
		self.children[node]
			.iter()
			.map(|&c| self.target[c])
			.collect()
	}

	fn sums(&self, node: usize) -> &SumSet {
		&self.sum_cache[node]
	}

	fn recompute_sums(&mut self, node: usize) {
		self.sum_cache[node] = SumSet::of(
			self.cells[node].value,
			self.children[node].iter().map(|&c| self.target[c]),
		);
	}

	fn effective_sums(&self, node: usize, delta: Option<Delta>) -> SumSet {
		match delta {
			None => self.sum_cache[node],
			Some(Delta::Add(d)) => {
				let mut sums = self.sum_cache[node];
				sums.add(d);
				sums
			}
			Some(Delta::Remove(removed)) => SumSet::of(
				self.cells[node].value,
				self.children[node]
					.iter()
					.filter(|&&c| c != removed)
					.map(|&c| self.target[c]),
			),
		}
	}

	fn fits(
		&self,
		node: usize,
		sums: &SumSet,
		target: i32,
		distances: impl FnOnce() -> Vec<i32>,
	) -> bool {
		sums.contains(target)
			|| (sums.contains(-target)
				&& plan(self.cells[node].value, &distances(), target).is_ok())
	}

	fn satisfied(&self, node: usize) -> bool {
		self.fits(node, self.sums(node), self.target[node], || {
			self.distances(node)
		})
	}

	fn root_residual(&self, node: usize) -> i32 {
		self.sums(node).min_abs()
	}

	fn set_residual(&mut self, node: usize, residual: i32) {
		let old = self.residual[node];
		self.residual_sum += (residual - old) as i64;
		self.penalty_sum += self.penalty(node, residual) - self.penalty(node, old);
		self.residual[node] = residual;
		match (old > 0, residual > 0) {
			(false, true) => {
				self.remaining_slot[node] = self.remaining.len();
				self.remaining.push(node);
			}
			(true, false) => {
				let slot = self.remaining_slot[node];
				self.remaining.swap_remove(slot);
				if let Some(&moved) = self.remaining.get(slot) {
					self.remaining_slot[moved] = slot;
				}
			}
			_ => {}
		}
	}

	fn refresh(&mut self, node: usize) {
		let residual = if self.parent[node] == ROOT {
			self.root_residual(node)
		} else {
			0
		};
		self.set_residual(node, residual);
	}

	fn raw_link(&mut self, child: usize, parent: usize, distance: i32) {
		self.parent[child] = parent;
		self.target[child] = distance;
		self.children[parent].push(child);
		self.recompute_sums(parent);
	}

	fn raw_unlink(&mut self, child: usize) -> (usize, i32) {
		let parent = self.parent[child];
		let distance = self.target[child];
		let slot = self.children[parent]
			.iter()
			.position(|&c| c == child)
			.unwrap();
		self.children[parent].swap_remove(slot);
		self.parent[child] = ROOT;
		self.target[child] = 0;
		self.recompute_sums(parent);
		(parent, distance)
	}

	fn link(&mut self, child: usize, parent: usize, distance: i32) {
		self.raw_link(child, parent, distance);
		self.journal.push(Op::Link(child, parent));
		self.refresh(child);
		self.refresh(parent);
	}

	fn link_checked(&mut self, child: usize, parent: usize, distance: i32) -> bool {
		self.link(child, parent, distance);
		if self.satisfied(child) {
			return true;
		}
		self.unlink(child);
		false
	}

	fn unlink(&mut self, child: usize) -> usize {
		let (parent, distance) = self.raw_unlink(child);
		self.journal.push(Op::Unlink(child, parent, distance));
		self.refresh(child);
		self.refresh(parent);
		parent
	}

	fn rollback(&mut self, mark: usize) {
		while self.journal.len() > mark {
			match self.journal.pop().unwrap() {
				Op::Link(child, parent) => {
					self.raw_unlink(child);
					self.refresh(child);
					self.refresh(parent);
				}
				Op::Unlink(child, parent, distance) => {
					self.raw_link(child, parent, distance);
					self.refresh(child);
					self.refresh(parent);
				}
			}
		}
	}

	fn in_subtree(&self, node: usize, root: usize) -> bool {
		let mut walker = node;
		while walker != ROOT {
			if walker == root {
				return true;
			}
			walker = self.parent[walker];
		}
		false
	}

	fn achievable(&self, node: usize) -> &SumSet {
		self.sums(node)
	}

	fn cell_toward(&self, node: usize, dir: usize, distance: i32) -> Option<usize> {
		let x = self.cells[node].x as i32 + DX[dir] * distance;
		let y = self.cells[node].y as i32 + DY[dir] * distance;
		if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
			return None;
		}
		Some(self.cell_at[y as usize * self.width + x as usize]).filter(|&c| c != ROOT)
	}

	fn parents_for(&self, node: usize) -> Vec<(usize, i32)> {
		let mut options = Vec::new();
		for distance in self.achievable(node).abs_values() {
			if distance == 0 {
				continue;
			}
			for dir in 0..4 {
				if let Some(p) = self.cell_toward(node, dir, distance)
					&& !self.in_subtree(p, node)
				{
					options.push((p, distance));
				}
			}
		}
		options
	}

	fn fix(&mut self, node: usize, depth: usize, rng: &mut Rng) {
		if node == ROOT || self.parent[node] == ROOT || self.satisfied(node) {
			return;
		}
		let options = if depth > 0 {
			self.parents_for(node)
		} else {
			Vec::new()
		};
		let old_parent = self.unlink(node);
		if !options.is_empty() {
			let (p, d) = self.pick_parent(&options, rng);
			if self.link_checked(node, p, d) {
				self.fix(p, depth - 1, rng);
			}
		}
		self.fix(old_parent, depth.saturating_sub(1), rng);
	}

	fn accepts(&self, parent: usize, distance: i32) -> bool {
		if self.parent[parent] == ROOT {
			return true;
		}
		let mut sums = *self.sums(parent);
		sums.add(distance);
		self.fits(parent, &sums, self.target[parent], || {
			let mut distances = self.distances(parent);
			distances.push(distance);
			distances
		})
	}

	fn pick_parent(&self, options: &[(usize, i32)], rng: &mut Rng) -> (usize, i32) {
		let offset = rng.below(options.len());
		for i in 0..options.len().min(self.neutral_tries) {
			let (p, d) = options[(offset + i) % options.len()];
			if self.accepts(p, d) {
				return (p, d);
			}
		}
		options[offset]
	}

	fn attach_move(&mut self, node: usize, depth: usize, rng: &mut Rng) -> bool {
		let options = self.parents_for(node);
		if options.is_empty() {
			return false;
		}
		let (p, d) = options[rng.below(options.len())];
		let old_parent = if self.parent[node] != ROOT {
			self.unlink(node)
		} else {
			ROOT
		};
		if self.link_checked(node, p, d) {
			self.fix(p, depth, rng);
		}
		self.fix(old_parent, depth, rng);
		true
	}

	fn detach_move(&mut self, node: usize, depth: usize, rng: &mut Rng) -> bool {
		if self.parent[node] == ROOT {
			return false;
		}
		let old_parent = self.unlink(node);
		self.fix(old_parent, depth, rng);
		true
	}

	fn assemble_candidates(&self, root: usize, rng: &mut Rng) -> Vec<(usize, i32)> {
		let mut free = self.cells[root]
			.candidates
			.iter()
			.copied()
			.filter(|&(z, d)| {
				let q = self.parent[z];
				q != root
					&& self.achievable(z).contains_abs(d)
					&& self.holds(q, Some(Delta::Remove(z)))
			})
			.collect::<Vec<_>>();
		for i in (1..free.len()).rev() {
			free.swap(i, rng.below(i + 1));
		}
		free.truncate(ASSEMBLE_CANDIDATES);
		free
	}

	fn subset_layers(&self, root: usize, free: &[(usize, i32)]) -> Vec<SumSet> {
		let mut layers = vec![*self.sums(root)];
		for &(_, d) in free {
			let mut next = *layers.last().unwrap();
			let mut shifted = next;
			shifted.add(d);
			next.union(&shifted);
			layers.push(next);
		}
		layers
	}

	fn subset_reaching(layers: &[SumSet], free: &[(usize, i32)], target: i32) -> Vec<(usize, i32)> {
		let mut chosen = Vec::new();
		let mut sum = target;
		for (k, &(z, d)) in free.iter().enumerate().rev() {
			let before = &layers[k];
			if before.contains(sum) {
				continue;
			}
			sum += if before.contains(sum - d) { -d } else { d };
			chosen.push((z, d));
		}
		chosen
	}

	fn assemble_move(&mut self, root: usize, depth: usize, rng: &mut Rng) -> bool {
		if self.parent[root] != ROOT {
			return false;
		}
		let free = self.assemble_candidates(root, rng);
		let layers = self.subset_layers(root, &free);
		if !layers.last().unwrap().contains(0) {
			return false;
		}
		let chosen = Self::subset_reaching(&layers, &free, 0);
		let mut old_parents = Vec::new();
		for &(z, d) in &chosen {
			if self.parent[z] != ROOT {
				old_parents.push(self.unlink(z));
			}
			self.link(z, root, d);
		}
		for &(z, _) in &chosen {
			self.fix(z, depth, rng);
		}
		for q in old_parents {
			self.fix(q, depth, rng);
		}
		true
	}

	fn neutral_detach(&mut self, rng: &mut Rng, tries: usize) -> bool {
		let n = self.cells.len();
		for _ in 0..tries {
			let node = rng.below(n);
			let parent = self.parent[node];
			if parent != ROOT && self.holds(parent, Some(Delta::Remove(node))) {
				self.unlink(node);
				return true;
			}
		}
		false
	}

	fn effective(&self, node: usize, delta: Option<Delta>) -> Vec<i32> {
		let mut distances = Vec::with_capacity(self.children[node].len() + 1);
		for &c in &self.children[node] {
			if !matches!(delta, Some(Delta::Remove(r)) if r == c) {
				distances.push(self.target[c]);
			}
		}
		if let Some(Delta::Add(d)) = delta {
			distances.push(d);
		}
		distances
	}

	fn holds(&self, node: usize, delta: Option<Delta>) -> bool {
		if node == ROOT || self.parent[node] == ROOT {
			return true;
		}
		let sums = self.effective_sums(node, delta);
		self.fits(node, &sums, self.target[node], || {
			self.effective(node, delta)
		})
	}

	fn outcome(&self, broken: [(usize, Option<Delta>, bool); 2]) -> Option<Option<(usize, Delta)>> {
		let failing = broken.iter().filter(|b| !b.2).collect::<Vec<_>>();
		match failing.len() {
			0 => Some(None),
			1 => failing[0].1.map(|d| Some((failing[0].0, d))),
			_ => None,
		}
	}

	fn chain_transitions(
		&self,
		carrier: usize,
		delta: Option<Delta>,
		out: &mut Vec<(Step, Option<(usize, Delta)>)>,
	) {
		let own = self.effective(carrier, delta);
		let own_sums = self.effective_sums(carrier, delta);
		let is_root = self.parent[carrier] == ROOT;
		let old_parent = self.parent[carrier];
		for t in own_sums.abs_values() {
			if t == 0 {
				if is_root {
					continue;
				}
				let q_ok = self.holds(old_parent, Some(Delta::Remove(carrier)));
				let step = Step {
					moved: carrier,
					new_parent: ROOT,
					distance: 0,
				};
				if let Some(o) = self.outcome([
					(old_parent, Some(Delta::Remove(carrier)), q_ok),
					(ROOT, None, true),
				]) {
					out.push((step, o));
				}
				continue;
			}
			for dir in 0..4 {
				let Some(y) = self.cell_toward(carrier, dir, t) else {
					continue;
				};
				if y == old_parent || self.in_subtree(y, carrier) {
					continue;
				}
				let q_ok = is_root || self.holds(old_parent, Some(Delta::Remove(carrier)));
				let y_ok = self.holds(y, Some(Delta::Add(t)));
				let step = Step {
					moved: carrier,
					new_parent: y,
					distance: t,
				};
				let q = if is_root { ROOT } else { old_parent };
				if let Some(o) = self.outcome([
					(q, Some(Delta::Remove(carrier)), q_ok),
					(y, Some(Delta::Add(t)), y_ok),
				]) {
					out.push((step, o));
				}
			}
		}
		for &c in &self.children[carrier] {
			if matches!(delta, Some(Delta::Remove(r)) if r == c) {
				continue;
			}
			let carrier_ok = is_root || {
				let mut without = own.clone();
				let slot = without.iter().position(|&d| d == self.target[c]).unwrap();
				without.swap_remove(slot);
				let sums = SumSet::of(self.cells[carrier].value, without.iter().copied());
				self.fits(carrier, &sums, self.target[carrier], || without)
			};
			if !carrier_ok {
				continue;
			}
			for t in self.achievable(c).abs_values() {
				for dir in 0..4 {
					let Some(y) = self
						.cell_toward(c, dir, t)
						.filter(|&y| y != carrier && !self.in_subtree(y, c))
					else {
						continue;
					};
					let y_ok = self.holds(y, Some(Delta::Add(t)));
					let step = Step {
						moved: c,
						new_parent: y,
						distance: t,
					};
					out.push((step, if y_ok { None } else { Some((y, Delta::Add(t))) }));
				}
			}
		}
		for &(z, d) in &self.cells[carrier].candidates {
			if self.parent[z] == carrier || !self.achievable(z).contains_abs(d) {
				continue;
			}
			let carrier_ok = is_root || {
				let mut sums = own_sums;
				sums.add(d);
				self.fits(carrier, &sums, self.target[carrier], || {
					let mut with = own.clone();
					with.push(d);
					with
				})
			};
			if !carrier_ok || self.in_subtree(carrier, z) {
				continue;
			}
			let q = self.parent[z];
			let q_ok = self.holds(q, Some(Delta::Remove(z)));
			let step = Step {
				moved: z,
				new_parent: carrier,
				distance: d,
			};
			out.push((
				step,
				if q_ok {
					None
				} else {
					Some((q, Delta::Remove(z)))
				},
			));
		}
	}

	fn apply_steps(&mut self, steps: &[Step]) -> bool {
		let mut touched = Vec::new();
		for step in steps {
			if step.new_parent != ROOT && self.in_subtree(step.new_parent, step.moved) {
				return false;
			}
			if self.parent[step.moved] != ROOT {
				touched.push(self.unlink(step.moved));
			}
			if step.new_parent != ROOT {
				self.link(step.moved, step.new_parent, step.distance);
				touched.push(step.new_parent);
			}
			touched.push(step.moved);
		}
		touched
			.iter()
			.all(|&v| self.parent[v] == ROOT || self.satisfied(v))
	}

	fn chain_repair(&mut self, start: usize, budget: usize, rng: &mut Rng) -> bool {
		let before = self.score();
		let mut arena: Vec<ChainNode> = Vec::new();
		let mut visited = std::collections::HashSet::new();
		visited.insert(start);
		let mut frontier = vec![(start, None, usize::MAX)];
		let mut transitions = Vec::new();
		let mut evaluated = 0;
		while !frontier.is_empty() && arena.len() < budget {
			let mut next = Vec::new();
			for &(carrier, delta, index) in &frontier {
				transitions.clear();
				self.chain_transitions(carrier, delta, &mut transitions);
				let offset = rng.below(transitions.len().max(1));
				for i in 0..transitions.len() {
					let (step, outcome) = transitions[(i + offset) % transitions.len()];
					let terminal = match outcome {
						None => true,
						Some((c, _)) => self.parent[c] == ROOT,
					};
					if !terminal {
						let (c, d) = outcome.unwrap();
						if visited.insert(c) {
							arena.push(ChainNode { prev: index, step });
							next.push((c, Some(d), arena.len() - 1));
						}
						continue;
					}
					let mut steps = vec![step];
					let mut walker = index;
					while walker != usize::MAX {
						steps.push(arena[walker].step);
						walker = arena[walker].prev;
					}
					steps.reverse();
					let mark = self.journal.len();
					let valid = self.apply_steps(&steps);
					evaluated += 1;
					if valid && self.score() < before {
						return true;
					}
					self.rollback(mark);
					if evaluated > budget {
						return false;
					}
				}
			}
			frontier = next;
		}
		false
	}

	fn snapshot(&self) -> Snapshot {
		(self.parent.clone(), self.target.clone())
	}

	fn restore(&mut self, snapshot: &Snapshot) {
		let n = self.cells.len();
		self.children.iter_mut().for_each(Vec::clear);
		self.parent = snapshot.0.clone();
		self.target = snapshot.1.clone();
		for node in 0..n {
			if self.parent[node] != ROOT {
				self.children[self.parent[node]].push(node);
			}
		}
		for node in 0..n {
			self.recompute_sums(node);
		}
		for node in 0..n {
			self.refresh(node);
		}
		self.journal.clear();
	}

	fn root_of(&self, mut node: usize) -> usize {
		while self.parent[node] != ROOT {
			node = self.parent[node];
		}
		node
	}

	fn shape(&self) -> String {
		let mut size = vec![0usize; self.cells.len()];
		for node in 0..self.cells.len() {
			size[self.root_of(node)] += 1;
		}
		let roots = (0..self.cells.len())
			.filter(|&r| self.parent[r] == ROOT)
			.collect::<Vec<_>>();
		let largest = roots.iter().copied().max_by_key(|&r| size[r]).unwrap();
		let mut open = self
			.remaining
			.iter()
			.map(|&r| (size[r], self.residual[r]))
			.collect::<Vec<_>>();
		open.sort_unstable_by(|a, b| b.cmp(a));
		let open_cells = open.iter().map(|&(s, _)| s).sum::<usize>();
		format!(
			"cells {} trees {} largest {} {} open_cells {open_cells} open {open:?}",
			self.cells.len(),
			roots.len(),
			size[largest],
			if self.residual[largest] > 0 {
				"OPEN"
			} else {
				"closed"
			},
		)
	}

	fn solution(&self) -> Result<Vec<Move>, usize> {
		let rooted = Rooted {
			cells: &self.cells,
			parent: self.parent.clone(),
			target: self.target.clone(),
			children: self.children.clone(),
		};
		rooted.solution()
	}
}

struct Knobs {
	depth: usize,
	history: usize,
	detach_rate: usize,
	focus: usize,
	report: u64,
	stagnation: u64,
	kick: usize,
	chain_rate: usize,
	chain_budget: usize,
	neutral_kick: bool,
	weight_step: i64,
	assemble_rate: usize,
	pool_rate: usize,
	pool_rn: usize,
	pool_size: usize,
	pool_threads: usize,
	threads: usize,
}

impl Knobs {
	fn from_env() -> Knobs {
		let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
		let threads = knob("THREADS", cores as f64) as usize;
		Knobs {
			depth: knob("DEPTH", 3.0) as usize,
			history: knob("LFA", 2000.0) as usize,
			detach_rate: knob("DETACH", 5.0) as usize,
			focus: knob("FOCUS", 80.0) as usize,
			report: knob("REPORT", 5_000_000.0) as u64,
			stagnation: knob("STAGNATION", 10000.0) as u64,
			kick: knob("KICK", 6.0) as usize,
			chain_rate: knob("CHAIN", 300.0) as usize,
			chain_budget: knob("CHAIN_BUDGET", 300.0) as usize,
			neutral_kick: knob("NEUTRAL_KICK", 1.0) > 0.0,
			weight_step: knob("WEIGHT_STEP", 1.0) as i64,
			assemble_rate: knob("ASSEMBLE", 50.0) as usize,
			pool_rate: knob("POOL", 500.0) as usize,
			pool_rn: knob("POOL_RN", 12.0) as usize,
			pool_size: knob("POOL_SIZE", 16.0) as usize,
			pool_threads: knob("POOL_THREADS", (threads / 2) as f64) as usize,
			threads,
		}
	}
}

struct Shared {
	stop: AtomicBool,
	pool: Mutex<Pool>,
	start: Instant,
}

fn search(
	level: &Level,
	thread: usize,
	seed: u64,
	knobs: &Knobs,
	shared: &Shared,
) -> Option<(Vec<Move>, u64)> {
	let verbose = thread == 0;
	let mut rng = Rng::new(seed);
	let pool_rate = if thread < knobs.pool_threads {
		knobs.pool_rate
	} else {
		0
	};
	let mut forest = Forest::new(level);
	let n = forest.cells.len();
	let mut lfa = vec![forest.score(); knobs.history];
	let mut best = forest.raw_score();
	let mut best_state = forest.snapshot();
	let mut last_improvement = 0u64;
	let mut chains = 0u64;
	let shape_rn = knob("SHAPE_RN", 0.0) as usize;
	let mut last_shape = 0.0;
	let mut next_report = 0u64;
	for it in 0u64.. {
		if forest.remaining.is_empty() {
			match forest.solution() {
				Ok(moves) => return Some((moves, it)),
				Err(node) => panic!("invariant broken at node {node}"),
			}
		}
		if it % 1024 == 0 && shared.stop.load(Ordering::Relaxed) {
			return None;
		}
		if rng.below(1000) < knobs.chain_rate {
			let start_node = forest.pick_remaining(&mut rng);
			if forest.chain_repair(start_node, knobs.chain_budget, &mut rng) {
				forest.journal.clear();
				chains += 1;
				if forest.raw_score() < best {
					best = forest.raw_score();
					best_state = forest.snapshot();
					last_improvement = it;
					if forest.remaining.len() <= knobs.pool_rn {
						shared.pool.lock().unwrap().offer(best, &best_state);
					}
				}
			}
			continue;
		}
		let before = forest.score();
		let mark = forest.journal.len();
		let moved = if rng.below(1000) < knobs.assemble_rate {
			let node = forest.pick_remaining(&mut rng);
			forest.assemble_move(node, knobs.depth, &mut rng)
		} else if rng.below(100) < knobs.detach_rate {
			let node = rng.below(n);
			forest.detach_move(node, knobs.depth, &mut rng)
		} else {
			let node = if rng.below(100) < knobs.focus {
				forest.pick_remaining(&mut rng)
			} else {
				rng.below(n)
			};
			forest.attach_move(node, knobs.depth, &mut rng)
		};
		if !moved {
			continue;
		}
		let after = forest.score();
		let slot = (it % knobs.history as u64) as usize;
		if after <= before || after <= lfa[slot] {
			lfa[slot] = after;
			forest.journal.clear();
		} else {
			forest.rollback(mark);
			lfa[slot] = before;
		}
		if forest.raw_score() < best {
			best = forest.raw_score();
			best_state = forest.snapshot();
			last_improvement = it;
			if forest.remaining.len() <= knobs.pool_rn {
				shared.pool.lock().unwrap().offer(best, &best_state);
			}
		}
		if it - last_improvement > knobs.stagnation {
			forest.bump_weights(knobs.weight_step);
			let elite = if pool_rate > 0 && rng.below(1000) < pool_rate {
				shared.pool.lock().unwrap().pick(&mut rng)
			} else {
				None
			};
			if let Some((score, state)) = elite
				&& score < best
			{
				best = score;
				best_state = state;
			}
			forest.restore(&best_state);
			let elapsed = shared.start.elapsed().as_secs_f64();
			if verbose && forest.remaining.len() <= shape_rn && elapsed - last_shape > 2.0 {
				last_shape = elapsed;
				eprintln!("SHAPE {elapsed:.1}s {}", forest.shape());
			}
			for _ in 0..knobs.kick {
				if !(knobs.neutral_kick && forest.neutral_detach(&mut rng, 64)) {
					let node = rng.below(n);
					forest.detach_move(node, knobs.depth, &mut rng);
				}
			}
			forest.journal.clear();
			lfa.iter_mut().for_each(|f| *f = forest.score());
			last_improvement = it;
		}
		if verbose && it >= next_report {
			next_report += knobs.report;
			eprintln!(
				"{:.1}s it {it} remaining {} residual {} best {best} chains {chains}",
				shared.start.elapsed().as_secs_f64(),
				forest.remaining.len(),
				forest.residual_sum
			);
		}
	}
	None
}

fn main() {
	let level = Level::read_stdin();
	let knobs = Knobs::from_env();
	let seed = knob("SEED", 1.0) as u64;
	let shared = Shared {
		stop: AtomicBool::new(false),
		pool: Mutex::new(Pool {
			elites: Vec::new(),
			capacity: knobs.pool_size,
		}),
		start: Instant::now(),
	};
	let found = Mutex::new(None);
	std::thread::scope(|scope| {
		for thread in 0..knobs.threads {
			let (level, knobs, shared, found) = (&level, &knobs, &shared, &found);
			scope.spawn(move || {
				if let Some((moves, it)) =
					search(level, thread, seed + thread as u64, knobs, shared)
				{
					let mut slot = found.lock().unwrap();
					if slot.is_none() {
						*slot = Some((moves, it, thread));
						shared.stop.store(true, Ordering::Relaxed);
					}
				}
			});
		}
	});
	let (moves, it, thread) = found
		.into_inner()
		.unwrap()
		.expect("every search thread ended without a solution");
	for m in &moves {
		println!("{m}");
	}
	eprintln!(
		"solved in {:.3}s, {it} iterations, thread {thread}/{}",
		shared.start.elapsed().as_secs_f64(),
		knobs.threads
	);
}
