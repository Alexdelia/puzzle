use crate::board::{
	Board, HEIGHT, MAX_PATH_LENGTH, NODE_COUNT, NodeId, Point, SIDE_BORDER, TOP_BORDER, WIDTH,
	is_blocked, mirror_of,
};
use crate::jrand::JavaRandom;
use std::collections::HashSet;

pub const NODE_MIN_DIST: i32 = 140;
pub const SPARSE_MIN: f64 = 0.1;
pub const SPARSE_MAX: f64 = 0.5;
pub const MIN_TRIANGLE_COUNT: usize = 20;
pub const MAX_ANGLE: f64 = 160.0;
const PLACEMENT_TRIES: i32 = 10000;
const START_UNITS: usize = 3;

fn degrees(cosine: f64) -> f64 {
	cosine.acos() * (180.0 / std::f64::consts::PI)
}

fn has_obtuse_angle(points: &[Point]) -> bool {
	let count = points.len();
	let newest = count.saturating_sub(2);
	for i1 in 0..count {
		for i2 in i1 + 1..count {
			for i3 in (i2 + 1).max(newest)..count {
				let a = points[i1].dist(points[i2]);
				let b = points[i1].dist(points[i3]);
				let c = points[i2].dist(points[i3]);
				let limit = f64::from(MAX_PATH_LENGTH);
				if a > limit || b > limit || c > limit {
					continue;
				}
				let alpha = degrees((b * b + c * c - a * a) / (2.0 * b * c));
				let beta = degrees((a * a + c * c - b * b) / (2.0 * a * c));
				let gamma = degrees((a * a + b * b - c * c) / (2.0 * a * b));
				if alpha > MAX_ANGLE || beta > MAX_ANGLE || gamma > MAX_ANGLE {
					return true;
				}
			}
		}
	}
	false
}

fn place_nodes(random: &mut JavaRandom) -> Vec<Point> {
	let mut points: Vec<Point> = Vec::new();
	let mut distances: HashSet<i32> = HashSet::new();
	let mut tries = 0;
	while points.len() < NODE_COUNT && {
		tries += 1;
		tries - 1 < PLACEMENT_TRIES
	} {
		let x = SIDE_BORDER + random.next_int(WIDTH);
		let y = TOP_BORDER + random.next_int(HEIGHT);
		let n1 = Point { x, y };
		let n2 = n1.mirror();
		let min = NODE_MIN_DIST * NODE_MIN_DIST;
		if points.iter().any(|&n| n.dist2(n1) < min)
			|| n1.dist2(n2) < min
			|| distances.contains(&n1.dist2(n2))
			|| points.iter().any(|&n| distances.contains(&n.dist2(n1)))
		{
			continue;
		}
		points.push(n1);
		points.push(n2);
		let obtuse = has_obtuse_angle(&points);
		points.truncate(points.len() - 2);
		if obtuse {
			continue;
		}
		distances.insert(n1.dist2(n2));
		for &n in &points {
			distances.insert(n.dist2(n1));
		}
		points.push(n1);
		points.push(n2);
	}
	points
}

fn triangulate(board: &mut Board) -> Vec<(NodeId, NodeId)> {
	let count = board.len();
	let mut candidates = Vec::with_capacity(count * count / 2);
	for i in 0..count {
		for j in i + 1..count {
			candidates.push((i, j));
		}
	}
	candidates.sort_by_key(|&(a, b)| board.points[a].dist2(board.points[b]));
	let mut existing = Vec::new();
	for edge in candidates {
		if !is_blocked(&board.points, edge, &existing) {
			board.add_neighbor(edge.0, edge.1);
			existing.push(edge);
		}
	}
	existing
}

fn sparsify(board: &mut Board, existing: &mut Vec<(NodeId, NodeId)>, random: &mut JavaRandom) {
	let mut tested: HashSet<(NodeId, NodeId)> = HashSet::new();
	let share = random.next_double() * (SPARSE_MAX - SPARSE_MIN) + SPARSE_MIN;
	let mut remove_count = (existing.len() as f64 * share) as i32;
	while remove_count > 0 {
		remove_count -= 2;
		let edge = existing[random.next_int(existing.len() as i32) as usize];
		if tested.contains(&edge) {
			continue;
		}
		let (m1, m2) = (mirror_of(edge.0), mirror_of(edge.1));
		let partner = existing
			.iter()
			.rev()
			.copied()
			.find(|&(a, b)| (a, b) == (m1, m2) || (a, b) == (m2, m1))
			.expect("every edge has a mirrored partner");
		tested.insert(edge);
		tested.insert(partner);
		board.remove_neighbor(edge.0, edge.1);
		board.remove_neighbor(partner.0, partner.1);
		board.update_triangles();
		if board.triangles.len() >= MIN_TRIANGLE_COUNT && board.is_strongly_connected() {
			for gone in [edge, partner] {
				if let Some(at) = existing.iter().position(|&e| e == gone) {
					existing.remove(at);
				}
			}
		} else {
			board.add_neighbor(edge.0, edge.1);
			board.add_neighbor(partner.0, partner.1);
		}
	}
}

fn place_units(board: &mut Board) {
	let mut by_x: Vec<NodeId> = (0..board.len()).collect();
	by_x.sort_by_key(|&n| board.points[n].x);
	for &node in by_x.iter().take(START_UNITS) {
		board.units[node][0] = 1;
		board.units[mirror_of(node)][1] = 1;
	}
}

pub fn generate(seed: i64) -> Board {
	let mut random = JavaRandom::new(seed);
	let (mut board, mut existing) = loop {
		let mut board = Board::new(place_nodes(&mut random));
		let existing = triangulate(&mut board);
		board.update_triangles();
		if board.is_strongly_connected() && board.triangles.len() >= MIN_TRIANGLE_COUNT {
			break (board, existing);
		}
	};
	sparsify(&mut board, &mut existing, &mut random);
	place_units(&mut board);
	let mut fresh = Board::new(board.points.clone());
	fresh.neighbors = board.neighbors;
	fresh.units = board.units;
	fresh.update_triangles();
	fresh
}

pub struct Options {
	pub nodes: String,
	pub edges: String,
	pub units: String,
}

pub fn options(board: &Board) -> Options {
	let join = |parts: Vec<String>| parts.join("_");
	Options {
		nodes: join(
			board
				.points
				.iter()
				.map(|p| format!("{},{}", p.x, p.y))
				.collect(),
		),
		edges: join(
			board
				.edges()
				.iter()
				.map(|(a, b)| format!("{a},{b}"))
				.collect(),
		),
		units: join(
			board
				.units
				.iter()
				.map(|u| format!("{},{}", u[0], u[1]))
				.collect(),
		),
	}
}

fn pairs(text: &str, what: &str) -> Result<Vec<(i32, i32)>, String> {
	text.split('_')
		.filter_map(|part| {
			let mut fields = part.split(',');
			let first = fields.next()?;
			let second = fields.next()?;
			Some((first, second))
		})
		.map(|(first, second)| {
			let parse = |field: &str| {
				field
					.trim()
					.parse::<i32>()
					.map_err(|e| format!("{what} `{field}`: {e}"))
			};
			Ok((parse(first)?, parse(second)?))
		})
		.collect()
}

pub fn from_options(nodes: &str, edges: &str, units: &str) -> Result<Board, String> {
	let points: Vec<Point> = pairs(nodes, "node")?
		.into_iter()
		.map(|(x, y)| Point { x, y })
		.collect();
	let mut board = Board::new(points);
	let count = board.len();
	for (a, b) in pairs(edges, "edge")? {
		let (a, b) = (a as usize, b as usize);
		if a >= count || b >= count {
			return Err(format!("edge {a},{b} names a missing node"));
		}
		board.add_neighbor(a, b);
	}
	for (node, (mine, theirs)) in pairs(units, "units")?.into_iter().enumerate() {
		if node >= count {
			return Err(format!("units for missing node {node}"));
		}
		board.units[node] = [mine, theirs];
	}
	board.update_triangles();
	Ok(board)
}
