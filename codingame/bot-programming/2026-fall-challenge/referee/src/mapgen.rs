use crate::board::{
	Board, HEIGHT, MAX_NODES, MAX_PATH_LENGTH, NODE_COUNT, NodeId, Point, SIDE_BORDER, TOP_BORDER,
	WIDTH, above, bit, is_blocked, mirror_of, nodes_of,
};
use crate::jrand::JavaRandom;
use std::collections::HashSet;
use std::hash::{BuildHasherDefault, Hasher};

pub const NODE_MIN_DIST: i32 = 140;
pub const SPARSE_MIN: f64 = 0.1;
pub const SPARSE_MAX: f64 = 0.5;
pub const MIN_TRIANGLE_COUNT: usize = 20;
pub const MAX_ANGLE: f64 = 160.0;
const PLACEMENT_TRIES: i32 = 10000;
const START_UNITS: usize = 3;
const CLEARLY_UNDER_MAX_ANGLE_COSINE: f64 = -0.9;
const SPACING_COLUMNS: i32 = (WIDTH as u32).div_ceil(NODE_MIN_DIST as u32) as i32;
const SPACING_ROWS: i32 = (HEIGHT as u32).div_ceil(NODE_MIN_DIST as u32) as i32;
const COVER_CELL: i32 = 10;
const COVER_COLUMNS: i32 = (WIDTH as u32).div_ceil(COVER_CELL as u32) as i32;
const COVER_ROWS: i32 = (HEIGHT as u32).div_ceil(COVER_CELL as u32) as i32;

#[derive(Default)]
struct MultiplyHasher(u64);

impl Hasher for MultiplyHasher {
	fn finish(&self) -> u64 {
		self.0
	}

	fn write(&mut self, bytes: &[u8]) {
		for &byte in bytes {
			self.write_u64(u64::from(byte));
		}
	}

	fn write_u64(&mut self, value: u64) {
		self.0 = (self.0.rotate_left(5) ^ value).wrapping_mul(0x9E37_79B9_7F4A_7C15);
	}

	fn write_i32(&mut self, value: i32) {
		self.write_u64(value as u64);
	}
}

type DistanceSet = HashSet<i32, BuildHasherDefault<MultiplyHasher>>;

struct Spacing {
	cells: Vec<u64>,
	covered: Vec<bool>,
}

impl Spacing {
	fn new() -> Self {
		Spacing {
			cells: vec![0; (SPACING_COLUMNS * SPACING_ROWS) as usize],
			covered: vec![false; (COVER_COLUMNS * COVER_ROWS) as usize],
		}
	}

	fn cover_cell_of(p: Point) -> (i32, i32) {
		(
			(p.x - SIDE_BORDER) / COVER_CELL,
			(p.y - TOP_BORDER) / COVER_CELL,
		)
	}

	fn cover_index(column: i32, row: i32) -> usize {
		(row * COVER_COLUMNS + column) as usize
	}

	fn cover_around(&mut self, center: Point) {
		let min = NODE_MIN_DIST * NODE_MIN_DIST;
		let reach = NODE_MIN_DIST / COVER_CELL + 1;
		let (home_column, home_row) = Self::cover_cell_of(center);
		for row in (home_row - reach).max(0)..=(home_row + reach).min(COVER_ROWS - 1) {
			for column in
				(home_column - reach).max(0)..=(home_column + reach).min(COVER_COLUMNS - 1)
			{
				let left = SIDE_BORDER + column * COVER_CELL;
				let top = TOP_BORDER + row * COVER_CELL;
				let far_x = (left - center.x)
					.abs()
					.max((left + COVER_CELL - 1 - center.x).abs());
				let far_y = (top - center.y)
					.abs()
					.max((top + COVER_CELL - 1 - center.y).abs());
				if far_x * far_x + far_y * far_y < min {
					self.covered[Self::cover_index(column, row)] = true;
				}
			}
		}
	}

	fn cell_of(p: Point) -> (i32, i32) {
		(
			(p.x - SIDE_BORDER) / NODE_MIN_DIST,
			(p.y - TOP_BORDER) / NODE_MIN_DIST,
		)
	}

	fn add(&mut self, p: Point, index: usize) {
		let (column, row) = Self::cell_of(p);
		self.cells[(row * SPACING_COLUMNS + column) as usize] |= bit(index);
		self.cover_around(p);
	}

	fn has_close(&self, points: &[Point], p: Point) -> bool {
		let (cover_column, cover_row) = Self::cover_cell_of(p);
		if self.covered[Self::cover_index(cover_column, cover_row)] {
			return true;
		}
		let (column, row) = Self::cell_of(p);
		let min = NODE_MIN_DIST * NODE_MIN_DIST;
		for r in (row - 1).max(0)..=(row + 1).min(SPACING_ROWS - 1) {
			for c in (column - 1).max(0)..=(column + 1).min(SPACING_COLUMNS - 1) {
				let nearby = self.cells[(r * SPACING_COLUMNS + c) as usize];
				if nodes_of(nearby).any(|n| points[n].dist2(p) < min) {
					return true;
				}
			}
		}
		false
	}
}

fn degrees(cosine: f64) -> f64 {
	cosine.acos() * (180.0 / std::f64::consts::PI)
}

fn within_path_length(a: Point, b: Point) -> bool {
	a.dist2(b) <= MAX_PATH_LENGTH * MAX_PATH_LENGTH
}

fn exceeds_max_angle(cosine: f64) -> bool {
	cosine < CLEARLY_UNDER_MAX_ANGLE_COSINE && degrees(cosine) > MAX_ANGLE
}

fn is_obtuse(a: f64, b: f64, c: f64) -> bool {
	exceeds_max_angle((b * b + c * c - a * a) / (2.0 * b * c))
		|| exceeds_max_angle((a * a + c * c - b * b) / (2.0 * a * c))
		|| exceeds_max_angle((a * a + b * b - c * c) / (2.0 * a * b))
}

fn has_obtuse_angle(points: &[Point]) -> bool {
	let newest = points.len().saturating_sub(2);
	(newest..points.len()).any(|i3| {
		let corner = points[i3];
		let near = (0..i3)
			.filter(|&i| within_path_length(points[i], corner))
			.fold(0, |near, i| near | bit(i));
		nodes_of(near).any(|i1| {
			nodes_of(near & above(i1)).any(|i2| {
				within_path_length(points[i1], points[i2])
					&& is_obtuse(
						points[i1].dist(points[i2]),
						points[i1].dist(corner),
						points[i2].dist(corner),
					)
			})
		})
	})
}

fn place_nodes(random: &mut JavaRandom) -> Vec<Point> {
	let mut points: Vec<Point> = Vec::new();
	let mut distances = DistanceSet::default();
	let mut spacing = Spacing::new();
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
		if spacing.has_close(&points, n1)
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
		for n in [n1, n2] {
			spacing.add(n, points.len());
			points.push(n);
		}
	}
	points
}

fn triangulate(board: &mut Board) -> Vec<(NodeId, NodeId)> {
	let count = board.len();
	let mut candidates = Vec::with_capacity(count * count / 2);
	for i in 0..count {
		for j in i + 1..count {
			if within_path_length(board.points[i], board.points[j]) {
				candidates.push((i, j));
			}
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
	let mut triangle_count = board.triangles.len();
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
		let mut lost = board.triangles_on(edge.0, edge.1);
		if partner != edge {
			lost += board.triangles_on(partner.0, partner.1);
		}
		board.remove_neighbor(edge.0, edge.1);
		board.remove_neighbor(partner.0, partner.1);
		if triangle_count - lost >= MIN_TRIANGLE_COUNT && board.is_strongly_connected() {
			triangle_count -= lost;
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
	board.arena.clear();
	board.triangles.clear();
	board.update_triangles();
	board
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
	if points.len() > MAX_NODES {
		return Err(format!(
			"{} nodes, a board holds at most {MAX_NODES}",
			points.len()
		));
	}
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
