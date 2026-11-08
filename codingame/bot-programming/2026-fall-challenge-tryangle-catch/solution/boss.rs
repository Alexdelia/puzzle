use std::collections::VecDeque;
use std::io::{BufRead, Write};

const UNREACHABLE: i32 = 1000;
const MAX_PATH_LENGTH: f64 = 400.0;
const ATTACK_ARMY: usize = 20;
const MUTATIONS: usize = 10000;

struct Rng(u64);

impl Rng {
	fn below(&mut self, bound: usize) -> usize {
		self.0 ^= self.0 << 13;
		self.0 ^= self.0 >> 7;
		self.0 ^= self.0 << 17;
		(self.0 % bound as u64) as usize
	}
}

#[derive(Clone, Copy)]
struct Point {
	x: i64,
	y: i64,
}

fn ccw(a: Point, b: Point, c: Point) -> bool {
	(c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x)
}

struct Board {
	points: Vec<Point>,
	neighbors: Vec<Vec<usize>>,
	mine: Vec<i32>,
	theirs: Vec<i32>,
	distance: Vec<Vec<i32>>,
}

impl Board {
	fn step_toward(&self, from: usize, target: usize) -> Option<usize> {
		if from == target {
			return None;
		}
		self.neighbors[from]
			.iter()
			.copied()
			.min_by_key(|&n| self.distance[n][target])
	}

	fn is_crossing(&self, edge: [usize; 4]) -> bool {
		let mut distinct = edge.to_vec();
		distinct.sort_unstable();
		distinct.dedup();
		if distinct.len() < 4 {
			return false;
		}
		let [a, b, c, d] = edge.map(|n| self.points[n]);
		ccw(a, c, d) != ccw(b, c, d) && ccw(a, b, a) != ccw(a, b, d)
	}

	fn can_connect(&self, from: usize, to: usize) -> bool {
		let (p, q) = (self.points[from], self.points[to]);
		let length = (((p.x - q.x).pow(2) + (p.y - q.y).pow(2)) as f64).sqrt();
		if length > MAX_PATH_LENGTH {
			return false;
		}
		(0..self.points.len()).all(|a| {
			self.neighbors[a]
				.iter()
				.all(|&b| !self.is_crossing([from, to, a, b]))
		})
	}
}

#[derive(Clone)]
struct Triangle {
	corners: [usize; 3],
	owner: i64,
	me_can_capture: bool,
}

impl Triangle {
	fn around(&self, board: &Board) -> Vec<usize> {
		self.corners
			.iter()
			.flat_map(|&c| board.neighbors[c].iter().copied())
			.collect()
	}

	fn name(&self) -> String {
		let [a, b, c] = self.corners;
		format!("{a} {b} {c}")
	}
}

fn breadth_first(neighbors: &[Vec<usize>], source: usize) -> Vec<i32> {
	let mut distance = vec![UNREACHABLE; neighbors.len()];
	distance[source] = 0;
	let mut queue = VecDeque::from([source]);
	while let Some(node) = queue.pop_front() {
		for &next in &neighbors[node] {
			if distance[next] == UNREACHABLE {
				distance[next] = distance[node] + 1;
				queue.push_back(next);
			}
		}
	}
	distance
}

fn score(triangles: &[Triangle], targets: &[usize], house_count: usize) -> f64 {
	let mut count = vec![0; house_count];
	for &t in targets {
		count[t] += 1;
	}
	triangles
		.iter()
		.map(|t| {
			let covered = t.corners.iter().all(|&c| count[c] > 0);
			let empty = t.corners.iter().all(|&c| count[c] == 0);
			if t.me_can_capture && covered {
				1.0
			} else if t.owner != 0 && !t.me_can_capture && empty {
				0.5
			} else {
				0.0
			}
		})
		.sum()
}

fn random_neighbor(board: &Board, house: usize, rng: &mut Rng) -> usize {
	let around = &board.neighbors[house];
	if around.is_empty() {
		return house;
	}
	around[rng.below(around.len())]
}

fn mutate(
	board: &Board,
	triangles: &[Triangle],
	units: &[usize],
	planned: &[(usize, usize)],
	rng: &mut Rng,
) -> Vec<usize> {
	let mut targets: Vec<Option<usize>> = vec![None; units.len()];
	for &(from, to) in planned {
		if let Some(slot) = (0..units.len()).find(|&j| targets[j].is_none() && units[j] == from) {
			targets[slot] = Some(to);
		}
	}
	let mut targets: Vec<usize> = targets
		.into_iter()
		.zip(units)
		.map(|(target, &unit)| target.unwrap_or_else(|| random_neighbor(board, unit, rng)))
		.collect();
	if targets.is_empty() {
		return targets;
	}
	let house_count = board.points.len();
	let mut best = score(triangles, &targets, house_count);
	for _ in 0..MUTATIONS {
		let index = rng.below(targets.len());
		let backup = targets[index];
		targets[index] = random_neighbor(board, units[index], rng);
		let value = score(triangles, &targets, house_count);
		if value >= best {
			best = value;
		} else {
			targets[index] = backup;
		}
	}
	targets
}

fn cheapest_capture(board: &Board, triangles: &[Triangle], units: &[usize]) -> Vec<(usize, usize)> {
	let mut best_cost = UNREACHABLE;
	let mut best = Vec::new();
	if units.len() < 3 {
		return best;
	}
	let orders = [
		[0, 1, 2],
		[0, 2, 1],
		[1, 0, 2],
		[1, 2, 0],
		[2, 0, 1],
		[2, 1, 0],
	];
	for triangle in triangles
		.iter()
		.filter(|t| t.owner != 0 && t.me_can_capture)
	{
		for order in orders {
			let mut free = units.to_vec();
			let mut cost = 0;
			let mut plan = Vec::with_capacity(3);
			for corner in order.map(|i| triangle.corners[i]) {
				let mut index = 0;
				for i in 1..free.len() {
					if board.distance[corner][free[i]] < board.distance[corner][free[index]] {
						index = i;
					}
				}
				let source = free.remove(index);
				cost = cost.max(board.distance[source][corner]);
				plan.push((source, corner));
			}
			if cost < best_cost {
				best_cost = cost;
				best = plan;
			}
		}
	}
	best
}

fn use_a_triangle(board: &mut Board, triangles: &mut [Triangle], units: &mut Vec<usize>) -> String {
	let mine: Vec<usize> = (0..triangles.len())
		.filter(|&t| {
			triangles[t].owner == 0 && triangles[t].corners.iter().all(|&c| board.mine[c] > 0)
		})
		.collect();
	let threat = |t: usize| {
		triangles[t]
			.around(board)
			.iter()
			.map(|&n| board.theirs[n])
			.max()
			.unwrap_or(0)
	};
	let Some(source) = mine
		.iter()
		.copied()
		.reduce(|best, t| if threat(t) > threat(best) { t } else { best })
	else {
		return String::new();
	};
	let around = triangles[source].around(board);
	let target = around
		.iter()
		.copied()
		.reduce(|best, n| {
			if board.theirs[n] > board.theirs[best] {
				n
			} else {
				best
			}
		})
		.expect("a triangle has neighbors");
	let name = triangles[source].name();
	if board.theirs[target] > 1 {
		return format!("ATTACK {name} {target};MSG ATTACK!;");
	}
	let corners = triangles[source].corners;
	for corner in corners {
		let partner = (0..board.points.len())
			.find(|&p| board.distance[p][corner] == 2 && board.can_connect(corner, p));
		if let Some(partner) = partner {
			let others: Vec<usize> = corners.into_iter().filter(|&c| c != corner).collect();
			let (second, third) = (others[0], others[1]);
			for c in corners {
				if let Some(at) = units.iter().position(|&u| u == c) {
					units.remove(at);
				}
				board.mine[c] -= 1;
			}
			triangles[source].owner = -1;
			return format!("ADD_PATH {corner} {second} {third} {partner};");
		}
	}
	String::new()
}

fn main() {
	let mut lines = std::io::stdin().lock().lines();
	let mut ints = || -> Vec<i64> {
		let Some(line) = lines.next() else {
			std::process::exit(0);
		};
		line.expect("stdin is readable")
			.split_whitespace()
			.map(|f| f.parse().expect("integer field"))
			.collect()
	};
	let mut out = std::io::stdout().lock();
	let house_count = ints()[0] as usize;
	let mut points = vec![Point { x: 0, y: 0 }; house_count];
	for _ in 0..house_count {
		let row = ints();
		points[row[0] as usize] = Point {
			x: row[1],
			y: row[2],
		};
	}
	let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
	loop {
		ints();
		let mut mine = vec![0; house_count];
		let mut theirs = vec![0; house_count];
		for _ in 0..house_count {
			let row = ints();
			mine[row[0] as usize] = row[1] as i32;
			theirs[row[0] as usize] = row[2] as i32;
		}
		let mut neighbors = vec![Vec::new(); house_count];
		for _ in 0..ints()[0] {
			let row = ints();
			let (a, b) = (row[0] as usize, row[1] as usize);
			neighbors[a].push(b);
			neighbors[b].push(a);
		}
		let mut triangles: Vec<Triangle> = (0..ints()[0])
			.map(|_| {
				let row = ints();
				Triangle {
					corners: [row[0] as usize, row[1] as usize, row[2] as usize],
					owner: row[3],
					me_can_capture: row[4] != 0,
				}
			})
			.collect();
		for _ in 0..ints()[0] {
			ints();
		}
		let distance = (0..house_count)
			.map(|s| breadth_first(&neighbors, s))
			.collect();
		let mut board = Board {
			points: points.clone(),
			neighbors,
			mine,
			theirs,
			distance,
		};
		let mut units: Vec<usize> = (0..house_count)
			.flat_map(|h| std::iter::repeat_n(h, board.mine[h].max(0) as usize))
			.collect();

		let mut solution = String::new();
		if units.len() > ATTACK_ARMY {
			solution = use_a_triangle(&mut board, &mut triangles, &mut units);
		}

		let planned = cheapest_capture(&board, &triangles, &units);
		let mut targets = mutate(&board, &triangles, &units, &planned, &mut rng);
		let moves: Vec<String> = units
			.iter()
			.zip(&targets)
			.map(|(&from, &target)| match board.step_toward(from, target) {
				Some(next) => format!("MOVE {from} {next} 1"),
				None => String::new(),
			})
			.collect();
		solution.push_str(&moves.join(";"));

		let mut unit_count = units.len();
		for triangle in triangles.iter().filter(|t| t.owner == 0) {
			if unit_count >= house_count {
				unit_count += 1;
				continue;
			}
			unit_count += 1;
			let mut corners = triangle.corners.to_vec();
			corners.sort_by_key(|&c| {
				let planned_here = targets.iter().filter(|&&t| t == c).count();
				(planned_here, std::cmp::Reverse(board.neighbors[c].len()))
			});
			let mut best = f64::MIN;
			let mut spawn = corners[0];
			for &corner in &corners {
				targets.push(corner);
				let value = score(&triangles, &targets, house_count);
				targets.pop();
				if value > best {
					spawn = corner;
					best = value;
				}
			}
			let others: Vec<usize> = triangle
				.corners
				.into_iter()
				.filter(|&c| c != spawn)
				.collect();
			let (second, third) = (others[0], others[1]);
			solution.push_str(&format!(";SPAWN {spawn} {second} {third}"));
			targets.push(spawn);
		}
		writeln!(out, "{solution}").expect("stdout is writable");
		out.flush().expect("stdout is writable");
	}
}
