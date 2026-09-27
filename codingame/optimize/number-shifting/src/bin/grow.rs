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
		Forest {
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
		self.remaining.len() as i64 * REMAINING_WEIGHT + self.residual_sum
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
			Some(Delta::Remove(_)) => {
				SumSet::of(self.cells[node].value, self.effective(node, delta))
			}
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

	fn snapshot(&self) -> (Vec<usize>, Vec<i32>) {
		(self.parent.clone(), self.target.clone())
	}

	fn restore(&mut self, snapshot: &(Vec<usize>, Vec<i32>)) {
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
}

impl Knobs {
	fn from_env() -> Knobs {
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
		}
	}
}

fn search(
	level: &Level,
	seed: u64,
	knobs: &Knobs,
	stop: &AtomicBool,
	start: Instant,
	verbose: bool,
) -> Option<(Vec<Move>, u64)> {
	let mut rng = Rng::new(seed);
	let mut forest = Forest::new(level);
	let n = forest.cells.len();
	let mut lfa = vec![forest.score(); knobs.history];
	let mut best = forest.score();
	let mut best_state = forest.snapshot();
	let mut last_improvement = 0u64;
	let mut chains = 0u64;
	for it in 0u64.. {
		if forest.remaining.is_empty() {
			match forest.solution() {
				Ok(moves) => return Some((moves, it)),
				Err(node) => panic!("invariant broken at node {node}"),
			}
		}
		if it % 1024 == 0 && stop.load(Ordering::Relaxed) {
			return None;
		}
		if rng.below(1000) < knobs.chain_rate {
			let start_node = forest.remaining[rng.below(forest.remaining.len())];
			if forest.chain_repair(start_node, knobs.chain_budget, &mut rng) {
				forest.journal.clear();
				chains += 1;
				if forest.score() < best {
					best = forest.score();
					best_state = forest.snapshot();
					last_improvement = it;
				}
			}
			continue;
		}
		let before = forest.score();
		let mark = forest.journal.len();
		let moved = if rng.below(100) < knobs.detach_rate {
			let node = rng.below(n);
			forest.detach_move(node, knobs.depth, &mut rng)
		} else {
			let node = if rng.below(100) < knobs.focus {
				forest.remaining[rng.below(forest.remaining.len())]
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
		if after < best {
			best = after;
			best_state = forest.snapshot();
			last_improvement = it;
		}
		if it - last_improvement > knobs.stagnation {
			forest.restore(&best_state);
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
		if verbose && it % knobs.report == 0 {
			eprintln!(
				"{:.1}s it {it} remaining {} residual {} best {best} chains {chains}",
				start.elapsed().as_secs_f64(),
				forest.remaining.len(),
				forest.residual_sum
			);
		}
	}
	None
}

fn main() {
	let level = Level::read_stdin();
	let start = Instant::now();
	let knobs = Knobs::from_env();
	let seed = knob("SEED", 1.0) as u64;
	let default_threads = std::thread::available_parallelism().map_or(1, |n| n.get());
	let threads = knob("THREADS", default_threads as f64) as usize;
	let stop = AtomicBool::new(false);
	let found = Mutex::new(None);
	std::thread::scope(|scope| {
		for thread in 0..threads {
			let (level, knobs, stop, found) = (&level, &knobs, &stop, &found);
			scope.spawn(move || {
				if let Some((moves, it)) =
					search(level, seed + thread as u64, knobs, stop, start, thread == 0)
				{
					let mut slot = found.lock().unwrap();
					if slot.is_none() {
						*slot = Some((moves, it, thread));
						stop.store(true, Ordering::Relaxed);
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
		"solved in {:.3}s, {it} iterations, thread {thread}/{threads}",
		start.elapsed().as_secs_f64()
	);
}
