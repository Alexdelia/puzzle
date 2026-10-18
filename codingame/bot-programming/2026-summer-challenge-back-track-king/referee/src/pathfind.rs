use crate::grid::{Coord, Direction, Grid};
use crate::jqueue::JPriorityQueue;
use std::collections::{HashMap, HashSet, VecDeque};

pub fn train_path(grid: &Grid, from: Coord, to: Coord) -> Vec<Coord> {
	let mut steps: Vec<(Coord, usize)> = vec![(from, usize::MAX)];
	let mut queue: VecDeque<usize> = VecDeque::from([0]);
	let mut visited = vec![false; grid.cells()];
	visited[grid.index(from)] = true;

	while let Some(step) = queue.pop_front() {
		let (at, _) = steps[step];
		if at == to {
			let mut path = Vec::new();
			let mut walk = step;
			while walk != usize::MAX {
				path.push(steps[walk].0);
				walk = steps[walk].1;
			}
			path.reverse();
			return path;
		}
		for n in grid.neighbours(at) {
			let cell = grid.index(n);
			if grid.can_train_pass(n) && !visited[cell] {
				visited[cell] = true;
				steps.push((n, step));
				queue.push_back(steps.len() - 1);
			}
		}
	}
	Vec::new()
}

pub fn terrain_reachable(grid: &Grid, from: Coord, to: Coord) -> bool {
	if from == to {
		return true;
	}
	let mut queue = VecDeque::from([from]);
	let mut visited = vec![false; grid.cells()];
	visited[grid.index(from)] = true;
	while let Some(at) = queue.pop_front() {
		for n in grid.neighbours(at) {
			let cell = grid.index(n);
			if visited[cell] || grid.zone_of(n).inked {
				continue;
			}
			if n == to {
				return true;
			}
			visited[cell] = true;
			queue.push_back(n);
		}
	}
	false
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Cursor {
	Buildable,
	Built,
	Broken,
}

#[derive(Clone)]
struct Step {
	at: Coord,
	cursor: Cursor,
	planned: Vec<Coord>,
	spent: i32,
	build: Option<Coord>,
}

impl Step {
	fn key(&self) -> (Coord, Cursor) {
		(self.at, self.cursor)
	}
}

struct Node {
	step: usize,
	rank: f64,
	tie: i32,
}

fn cheapest_first(a: &Node, b: &Node) -> std::cmp::Ordering {
	a.rank.total_cmp(&b.rank).then(a.tie.cmp(&b.tie))
}

pub fn autobuild(grid: &Grid, from: Coord, to: Coord) -> Vec<Coord> {
	let mut steps: Vec<Step> = Vec::new();
	steps.push(initial(grid, from));

	let mut open: JPriorityQueue<Node> = JPriorityQueue::new();
	let mut ranks: HashMap<(Coord, Cursor), f64> = HashMap::new();
	let mut came_from: HashMap<(Coord, Cursor), usize> = HashMap::new();
	ranks.insert(steps[0].key(), 0.0);
	open.offer(
		Node {
			step: 0,
			rank: 0.0,
			tie: 0,
		},
		cheapest_first,
	);

	while let Some(node) = open.poll(cheapest_first) {
		if is_goal(grid, &steps[node.step], to) {
			return replay(&steps, &came_from, node.step);
		}
		let reached = ranks[&steps[node.step].key()];
		for next in successors(grid, &steps[node.step], to) {
			let key = next.key();
			let spent = reached + (next.spent - steps[node.step].spent) as f64;
			if spent >= *ranks.get(&key).unwrap_or(&f64::INFINITY) {
				continue;
			}
			let tie = Direction::from_coord(next.at - steps[node.step].at).ordinal();
			came_from.insert(key, node.step);
			ranks.insert(key, spent);
			steps.push(next);
			open.offer(
				Node {
					step: steps.len() - 1,
					rank: spent,
					tie,
				},
				cheapest_first,
			);
		}
	}
	Vec::new()
}

fn initial(grid: &Grid, from: Coord) -> Step {
	let cursor = match grid.get(from) {
		None => Cursor::Broken,
		Some(tile) if tile.is_track_or_town() => Cursor::Built,
		Some(_) => Cursor::Buildable,
	};
	Step {
		at: from,
		cursor,
		planned: Vec::new(),
		spent: 0,
		build: None,
	}
}

fn replay(steps: &[Step], came_from: &HashMap<(Coord, Cursor), usize>, goal: usize) -> Vec<Coord> {
	let mut chain = vec![goal];
	let mut seen: HashSet<(Coord, Cursor)> = HashSet::from([steps[goal].key()]);
	let mut walk = goal;
	while let Some(&parent) = came_from.get(&steps[walk].key()) {
		if !seen.insert(steps[parent].key()) {
			break;
		}
		chain.push(parent);
		walk = parent;
	}
	chain.reverse();
	chain
		.into_iter()
		.filter_map(|step| steps[step].build)
		.collect()
}

fn is_goal(grid: &Grid, step: &Step, to: Coord) -> bool {
	if step.at == to {
		return true;
	}
	grid.get(step.at)
		.is_some_and(|tile| tile.is_track_or_town())
		&& shares_rail_block(grid, step.at, to)
}

fn shares_rail_block(grid: &Grid, at: Coord, goal: Coord) -> bool {
	let mut queue = VecDeque::from([at]);
	let mut visited = HashSet::from([at]);
	while let Some(current) = queue.pop_front() {
		if current == goal {
			return true;
		}
		for n in grid.neighbours(current) {
			if grid.tile(n).is_track_or_town() && visited.insert(n) {
				queue.push_back(n);
			}
		}
	}
	false
}

fn rail_block_fringe(grid: &Grid, at: Coord) -> Vec<Coord> {
	let mut fringe = Vec::new();
	let mut queue = VecDeque::from([at]);
	let mut visited = HashSet::from([at]);
	while let Some(current) = queue.pop_front() {
		if !grid.tile(current).is_track_or_town() {
			fringe.push(current);
			continue;
		}
		for n in grid.neighbours(current) {
			if visited.insert(n) {
				queue.push_back(n);
			}
		}
	}
	fringe
}

fn successors(grid: &Grid, step: &Step, to: Coord) -> Vec<Step> {
	if step.cursor == Cursor::Broken || !grid.holds(to) {
		return Vec::new();
	}

	let standing_on_track =
		step.planned.contains(&step.at) || grid.tile(step.at).is_track_or_town();
	let scope: Vec<Coord> = if !standing_on_track {
		vec![step.at]
	} else if step.planned.contains(&step.at) {
		grid.neighbours(step.at).collect()
	} else {
		rail_block_fringe(grid, step.at)
	};

	let mut out: Vec<Step> = Vec::with_capacity(scope.len());
	for at in scope {
		let tile = grid.tile(at);
		if grid.zone_of(at).inked && !tile.is_town() {
			continue;
		}
		if step.planned.contains(&at) {
			continue;
		}
		let next = if tile.is_track_or_town() {
			if at == step.at {
				continue;
			}
			Step {
				at,
				cursor: Cursor::Built,
				planned: step.planned.clone(),
				spent: step.spent,
				build: None,
			}
		} else {
			let mut planned = step.planned.clone();
			planned.push(at);
			Step {
				at,
				cursor: Cursor::Built,
				planned,
				spent: step.spent + tile.rail_cost(),
				build: Some(at),
			}
		};
		if !out.iter().any(|other| other.key() == next.key()) {
			out.push(next);
		}
	}
	out
}
