pub const TOP_BORDER: i32 = 70;
pub const BOTTOM_BORDER: i32 = 190;
pub const SIDE_BORDER: i32 = 80;
pub const WIDTH: i32 = 1920 - 2 * SIDE_BORDER;
pub const HEIGHT: i32 = 1080 - TOP_BORDER - BOTTOM_BORDER;
pub const NODE_COUNT: usize = 50;
pub const MAX_PATH_LENGTH: i32 = 400;
pub const MAX_NODES: usize = u64::BITS as usize;

pub type NodeId = usize;
pub type TriangleId = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
	pub x: i32,
	pub y: i32,
}

impl Point {
	pub fn dist2(self, other: Point) -> i32 {
		let dx = self.x - other.x;
		let dy = self.y - other.y;
		dx * dx + dy * dy
	}

	pub fn dist(self, other: Point) -> f64 {
		f64::from(self.dist2(other)).sqrt()
	}

	pub fn mirror(self) -> Point {
		Point {
			x: WIDTH - 1 - (self.x - SIDE_BORDER) + SIDE_BORDER,
			y: HEIGHT - 1 - (self.y - TOP_BORDER) + TOP_BORDER,
		}
	}
}

pub fn ccw(a: Point, b: Point, c: Point) -> bool {
	(c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x)
}

pub fn is_crossing(points: &[Point], edge: (NodeId, NodeId), other: (NodeId, NodeId)) -> bool {
	let (a, b) = edge;
	let (c, d) = other;
	if a == b || a == c || b == c || a == d || b == d {
		return false;
	}
	let [a, b, c, d] = [a, b, c, d].map(|n| points[n]);
	ccw(a, c, d) != ccw(b, c, d) && ccw(a, b, c) != ccw(a, b, d)
}

pub fn is_blocked(points: &[Point], edge: (NodeId, NodeId), existing: &[(NodeId, NodeId)]) -> bool {
	points[edge.0].dist2(points[edge.1]) > MAX_PATH_LENGTH * MAX_PATH_LENGTH
		|| existing
			.iter()
			.any(|&other| is_crossing(points, edge, other))
}

pub fn mirror_of(node: NodeId) -> NodeId {
	node ^ 1
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Triangle {
	pub nodes: [NodeId; 3],
	pub owner: Option<usize>,
	pub initial_owner: Option<usize>,
	pub can_capture: [bool; 2],
}

impl Triangle {
	fn new(nodes: [NodeId; 3]) -> Self {
		Triangle {
			nodes,
			owner: None,
			initial_owner: None,
			can_capture: [true, true],
		}
	}

	pub fn has_node(&self, node: NodeId) -> bool {
		self.nodes.contains(&node)
	}

	fn same_corners(&self, nodes: [NodeId; 3]) -> bool {
		nodes.iter().all(|&n| self.has_node(n)) && self.nodes.iter().all(|n| nodes.contains(n))
	}
}

#[derive(Clone, Debug)]
pub struct Board {
	pub points: Vec<Point>,
	pub neighbors: Vec<Vec<NodeId>>,
	adjacency: Vec<u64>,
	pub units: Vec<[i32; 2]>,
	pub remaining: Vec<[i32; 2]>,
	pub arena: Vec<Triangle>,
	pub triangles: Vec<TriangleId>,
}

impl Board {
	pub fn new(points: Vec<Point>) -> Self {
		let count = points.len();
		assert!(
			count <= MAX_NODES,
			"a board holds at most {MAX_NODES} nodes"
		);
		Board {
			points,
			neighbors: vec![Vec::new(); count],
			adjacency: vec![0; count],
			units: vec![[0, 0]; count],
			remaining: vec![[0, 0]; count],
			arena: Vec::new(),
			triangles: Vec::new(),
		}
	}

	pub fn len(&self) -> usize {
		self.points.len()
	}

	pub fn is_empty(&self) -> bool {
		self.points.is_empty()
	}

	pub fn add_neighbor(&mut self, a: NodeId, b: NodeId) {
		for (from, to) in [(a, b), (b, a)] {
			let list = &mut self.neighbors[from];
			if !list.contains(&to) {
				list.push(to);
				list.sort_unstable();
			}
			self.adjacency[from] |= bit(to);
		}
	}

	pub fn remove_neighbor(&mut self, a: NodeId, b: NodeId) {
		self.neighbors[a].retain(|&n| n != b);
		self.neighbors[b].retain(|&n| n != a);
		self.adjacency[a] &= !bit(b);
		self.adjacency[b] &= !bit(a);
	}

	pub fn connected(&self, a: NodeId, b: NodeId) -> bool {
		self.adjacency[a] & bit(b) != 0
	}

	pub fn edges(&self) -> Vec<(NodeId, NodeId)> {
		let mut edges = Vec::new();
		for (a, list) in self.neighbors.iter().enumerate() {
			for &b in list {
				if b > a {
					edges.push((a, b));
				}
			}
		}
		edges
	}

	pub fn can_connect(&self, from: NodeId, to: NodeId) -> bool {
		!is_blocked(&self.points, (from, to), &self.edges())
	}

	pub fn connect(&mut self, from: NodeId, to: NodeId) {
		self.add_neighbor(from, to);
		self.update_triangles();
	}

	pub fn disconnect(&mut self, from: NodeId, to: NodeId) {
		self.remove_neighbor(from, to);
		self.update_triangles();
	}

	fn is_inside(&self, node: NodeId, corners: [NodeId; 3]) -> bool {
		if corners.contains(&node) {
			return false;
		}
		let p = self.points[node];
		let [a, b, c] = corners.map(|n| self.points[n]);
		let turns = [ccw(p, a, b), ccw(p, b, c), ccw(p, c, a)];
		let has_neg = turns.iter().any(|&t| !t);
		let has_pos = turns.iter().any(|&t| t);
		!(has_neg && has_pos)
	}

	fn has_node_inside(&self, corners: [NodeId; 3]) -> bool {
		let [a, b, c] = corners.map(|n| self.points[n]);
		let box_bounds_inside = ccw(a, b, c) || ccw(a, c, b);
		let [left, right] = [a.x.min(b.x).min(c.x), a.x.max(b.x).max(c.x)];
		let [top, bottom] = [a.y.min(b.y).min(c.y), a.y.max(b.y).max(c.y)];
		(0..self.len()).any(|n| {
			let p = self.points[n];
			let in_box = (left..=right).contains(&p.x) && (top..=bottom).contains(&p.y);
			(in_box || !box_bounds_inside) && self.is_inside(n, corners)
		})
	}

	pub fn triangles_on(&self, a: NodeId, b: NodeId) -> usize {
		nodes_of(self.adjacency[a] & self.adjacency[b])
			.filter(|&c| {
				let mut corners = [a, b, c];
				corners.sort_unstable();
				!self.has_node_inside(corners)
			})
			.count()
	}

	pub fn find_triangles(&self) -> Vec<[NodeId; 3]> {
		let mut found = Vec::new();
		for a in 0..self.len() {
			for b in nodes_of(self.adjacency[a] & above(a)) {
				for c in nodes_of(self.adjacency[a] & self.adjacency[b] & above(b)) {
					if !self.has_node_inside([a, b, c]) {
						found.push([a, b, c]);
					}
				}
			}
		}
		found
	}

	pub fn update_triangles(&mut self) {
		let mut live = Vec::new();
		for corners in self.find_triangles() {
			let kept = self
				.triangles
				.iter()
				.rev()
				.copied()
				.find(|&t| self.arena[t].same_corners(corners));
			match kept {
				Some(t) => live.push(t),
				None => {
					self.arena.push(Triangle::new(corners));
					live.push(self.arena.len() - 1);
				}
			}
		}
		self.triangles = live;
	}

	pub fn triangle(&self, id: TriangleId) -> &Triangle {
		&self.arena[id]
	}

	fn adjacent_to(&self, nodes: u64) -> u64 {
		nodes_of(nodes).fold(0, |around, n| around | self.adjacency[n])
	}

	pub fn reachable(&self, from: NodeId, blocked: u64) -> u64 {
		let mut seen = bit(from);
		let mut frontier = seen;
		while frontier != 0 {
			frontier = self.adjacent_to(frontier) & !seen & !blocked;
			seen |= frontier;
		}
		seen
	}

	pub fn first_step(&self, from: NodeId, to: NodeId) -> Option<NodeId> {
		let mut seen = bit(to);
		let mut frontier = seen;
		while frontier != 0 {
			let next = self.adjacent_to(frontier) & !seen;
			if next & bit(from) != 0 {
				return self.neighbors[from]
					.iter()
					.copied()
					.find(|&n| frontier & bit(n) != 0);
			}
			seen |= next;
			frontier = next;
		}
		None
	}

	pub fn is_strongly_connected(&self) -> bool {
		(0..self.len()).all(|blocked| {
			let start = if blocked == 0 { 1 } else { 0 };
			let reached = self.reachable(start, bit(blocked)).count_ones() as usize;
			self.len() - reached <= 1
		})
	}

	pub fn owner(&self, node: NodeId) -> Option<usize> {
		let around = &self.neighbors[node];
		for player in 0..2 {
			if !around.is_empty()
				&& around
					.iter()
					.all(|&n| self.units[n][player] > self.units[n][1 - player])
			{
				return Some(player);
			}
		}
		let [mine, theirs] = self.units[node];
		match mine.cmp(&theirs) {
			std::cmp::Ordering::Greater => Some(0),
			std::cmp::Ordering::Less => Some(1),
			std::cmp::Ordering::Equal => None,
		}
	}

	pub fn has_neighbor(&self, triangle: TriangleId, node: NodeId) -> bool {
		self.arena[triangle]
			.nodes
			.iter()
			.any(|&corner| self.connected(corner, node))
	}

	pub fn can_use(&self, triangle: TriangleId, player: usize, cost: i32) -> bool {
		let t = &self.arena[triangle];
		t.owner == Some(player)
			&& t.initial_owner == Some(player)
			&& t.nodes.iter().all(|&n| self.remaining[n][player] >= cost)
	}

	pub fn use_triangle(&mut self, triangle: TriangleId, cost: i32) {
		let owner = self.arena[triangle]
			.owner
			.expect("only an owned triangle can be used");
		for node in self.arena[triangle].nodes {
			let take = cost.min(self.units[node][owner]);
			self.units[node][owner] -= take;
			self.remaining[node][owner] -= take;
		}
		let t = &mut self.arena[triangle];
		t.can_capture[owner] = false;
		t.owner = None;
	}

	pub fn owners(&self) -> Vec<Option<usize>> {
		(0..self.len()).map(|node| self.owner(node)).collect()
	}

	pub fn capture(&mut self, triangle: TriangleId, owners: &[Option<usize>]) -> bool {
		let old = self.arena[triangle].owner;
		for player in 0..2 {
			let t = &self.arena[triangle];
			if t.can_capture[player] && t.nodes.iter().all(|&n| owners[n] == Some(player)) {
				self.arena[triangle].owner = Some(player);
			}
		}
		self.arena[triangle].owner != old
	}

	pub fn update_allowed_captures(&mut self, triangle: TriangleId) {
		let nodes = self.arena[triangle].nodes;
		for player in 0..2 {
			let empty = nodes.iter().all(|&n| self.units[n][player] == 0);
			let t = &mut self.arena[triangle];
			match t.owner {
				Some(owner) => t.can_capture[player] = owner != player,
				None => t.can_capture[player] |= empty,
			}
		}
	}

	pub fn units_of(&self, player: usize) -> i32 {
		self.units.iter().map(|u| u[player]).sum()
	}

	pub fn triangles_of(&self, player: usize) -> usize {
		self.triangles
			.iter()
			.filter(|&&t| self.arena[t].owner == Some(player))
			.count()
	}
}

pub fn bit(node: NodeId) -> u64 {
	1 << node
}

pub fn above(node: NodeId) -> u64 {
	(u64::MAX << node) << 1
}

pub fn nodes_of(mut nodes: u64) -> impl Iterator<Item = NodeId> {
	std::iter::from_fn(move || {
		(nodes != 0).then(|| {
			let node = nodes.trailing_zeros() as usize;
			nodes &= nodes - 1;
			node
		})
	})
}
