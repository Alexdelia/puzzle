use engine::{Command, Map, Point, State, bit, nodes_of};
use std::collections::BTreeMap;
use std::io::{BufRead, Write};

const UNREACHABLE: i32 = 1000;
const TOTAL_WEIGHT: f64 = 0.2;
const FRONT_WEIGHT: f64 = 0.05;
const ATTACK_MIN_KILLS: i32 = 3;

fn tuned(name: &str, default: f64) -> f64 {
	std::env::var(format!("SPREADER_{name}"))
		.ok()
		.and_then(|value| value.parse().ok())
		.unwrap_or(default)
}

struct Params {
	total_weight: f64,
	front_weight: f64,
	attack_min_kills: i32,
}

impl Params {
	fn load() -> Self {
		Params {
			total_weight: tuned("TOTAL_WEIGHT", TOTAL_WEIGHT),
			front_weight: tuned("FRONT_WEIGHT", FRONT_WEIGHT),
			attack_min_kills: tuned("ATTACK_MIN_KILLS", f64::from(ATTACK_MIN_KILLS)) as i32,
		}
	}
}

fn distances(state: &State, count: usize) -> Vec<Vec<i32>> {
	(0..count)
		.map(|source| {
			let mut distance = vec![UNREACHABLE; count];
			distance[source] = 0;
			let mut frontier = bit(source);
			let mut seen = frontier;
			let mut depth = 0;
			while frontier != 0 {
				depth += 1;
				let next = nodes_of(frontier).fold(0, |set, n| set | state.adjacency[n]) & !seen;
				for n in nodes_of(next) {
					distance[n] = depth;
				}
				seen |= next;
				frontier = next;
			}
			distance
		})
		.collect()
}

struct Turn<'a> {
	map: &'a Map,
	params: &'a Params,
	state: State,
	me: usize,
	distance: Vec<Vec<i32>>,
}

impl Turn<'_> {
	fn opponent(&self) -> usize {
		1 - self.me
	}

	fn enemy_distance(&self, house: usize) -> i32 {
		let enemy = self.state.occupied_by(self.opponent());
		nodes_of(enemy)
			.map(|e| self.distance[house][e])
			.min()
			.unwrap_or(UNREACHABLE)
	}

	fn nearest_free(&self, free: &[usize], taken: &[usize], house: usize) -> Option<(usize, i32)> {
		free.iter()
			.enumerate()
			.filter(|(u, _)| !taken.contains(u))
			.map(|(u, &from)| (u, self.distance[from][house]))
			.filter(|&(_, d)| d < UNREACHABLE)
			.min_by_key(|&(u, d)| (d, u))
	}

	fn plan_triangle(
		&self,
		free: &[usize],
		covered: u64,
		triangle: usize,
	) -> Option<(f64, Vec<(usize, usize)>)> {
		let opponent = self.opponent();
		let mut taken = Vec::new();
		let mut picks = Vec::new();
		let mut worst = 0;
		let mut total = 0;
		for corner in nodes_of(self.map.corners(triangle) & !covered) {
			for _ in 0..=self.state.units[corner][opponent] {
				let (unit, d) = self.nearest_free(free, &taken, corner)?;
				taken.push(unit);
				picks.push((unit, corner));
				worst = worst.max(d);
				total += d;
			}
		}
		if picks.is_empty() {
			return None;
		}
		let front = nodes_of(self.map.corners(triangle))
			.map(|c| self.enemy_distance(c))
			.min()
			.unwrap_or(UNREACHABLE);
		let cost = f64::from(worst)
			+ self.params.total_weight * f64::from(total)
			+ self.params.front_weight * f64::from(front.min(20));
		Some((cost, picks))
	}

	fn assign(&self) -> Vec<(usize, usize)> {
		let mut free: Vec<usize> = (0..self.map.len())
			.flat_map(|h| std::iter::repeat_n(h, self.state.units[h][self.me].max(0) as usize))
			.collect();
		let mut open: Vec<usize> = self
			.state
			.live
			.iter()
			.filter(|&t| {
				self.state.can_capture[self.me].has(t) && !self.state.owner[self.me].has(t)
			})
			.collect();
		let mut covered = 0u64;
		let mut moves = Vec::new();
		while !free.is_empty() {
			let best = open
				.iter()
				.enumerate()
				.filter_map(|(i, &t)| {
					self.plan_triangle(&free, covered, t)
						.map(|(cost, picks)| (cost, i, picks))
				})
				.min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
			let Some((_, i, picks)) = best else { break };
			covered |= self.map.corners(open.swap_remove(i));
			let mut units: Vec<usize> = picks.iter().map(|&(u, _)| u).collect();
			for &(unit, corner) in &picks {
				moves.push((free[unit], corner));
			}
			units.sort_unstable_by(|a, b| b.cmp(a));
			for unit in units {
				free.swap_remove(unit);
			}
		}
		for from in free {
			let target = (0..self.map.len())
				.filter(|&h| self.distance[from][h] < UNREACHABLE && covered & bit(h) == 0)
				.min_by_key(|&h| (self.distance[from][h], self.enemy_distance(h), h));
			let target = target.unwrap_or(from);
			covered |= bit(target);
			moves.push((from, target));
		}
		moves
	}

	fn first_steps(&self, moves: &[(usize, usize)]) -> Vec<(usize, usize)> {
		moves
			.iter()
			.map(|&(from, to)| {
				(
					from,
					if from == to {
						to
					} else {
						self.state.first_step(from, to).unwrap_or(from)
					},
				)
			})
			.collect()
	}

	fn commands(&self, steps: &[(usize, usize)], orders: &[Command]) -> Vec<Command> {
		let mut counts: BTreeMap<(usize, usize), i32> = BTreeMap::new();
		for &(from, to) in steps {
			if from != to {
				*counts.entry((from, to)).or_default() += 1;
			}
		}
		orders
			.iter()
			.copied()
			.chain(
				counts
					.into_iter()
					.map(|((from, to), amount)| Command::Move { from, to, amount }),
			)
			.collect()
	}

	fn units_lost(&self, commands: &[Command]) -> i32 {
		let mut next = self.state;
		let mut both: [&[Command]; 2] = [&[], &[]];
		both[self.me] = commands;
		next.step(self.map, both);
		let spawned = commands
			.iter()
			.filter(|c| matches!(c, Command::Spawn { .. }))
			.count() as i32;
		self.state.units_of(self.me) + spawned - next.units_of(self.me)
	}

	fn safe_steps(
		&self,
		mut steps: Vec<(usize, usize)>,
		orders: &[Command],
	) -> Vec<(usize, usize)> {
		let mut lost = self.units_lost(&self.commands(&steps, orders));
		for i in 0..steps.len() {
			let (from, to) = steps[i];
			if from == to || lost == 0 {
				continue;
			}
			steps[i] = (from, from);
			let held = self.units_lost(&self.commands(&steps, orders));
			if held < lost {
				lost = held;
			} else {
				steps[i] = (from, to);
			}
		}
		steps
	}

	fn attacks(&self) -> Vec<Command> {
		let opponent = self.opponent();
		let mut used = Vec::new();
		let mut orders = Vec::new();
		let mut remaining = self.state.units;
		let mut targets: Vec<usize> = (0..self.map.len())
			.filter(|&h| self.state.units[h][opponent] >= self.params.attack_min_kills)
			.collect();
		targets.sort_by_key(|&h| -self.state.units[h][opponent]);
		for target in targets {
			let triangle = self.state.owner[self.me].iter().find(|&t| {
				!used.contains(&t)
					&& self.state.initial_owner[self.me].has(t)
					&& nodes_of(self.map.corners(t)).all(|c| remaining[c][self.me] >= 1)
					&& nodes_of(self.map.corners(t)).any(|c| self.state.connected(c, target))
			});
			let Some(triangle) = triangle else { continue };
			for c in nodes_of(self.map.corners(triangle)) {
				remaining[c][self.me] -= 1;
			}
			used.push(triangle);
			orders.push(Command::Attack { triangle, target });
		}
		orders
	}

	fn spawns(&self, used: &[usize]) -> Vec<Command> {
		self.state.owner[self.me]
			.iter()
			.filter(|&t| self.state.initial_owner[self.me].has(t) && !used.contains(&t))
			.map(|triangle| {
				let node = nodes_of(self.map.corners(triangle))
					.min_by_key(|&c| (self.enemy_distance(c), c))
					.expect("triangles have corners");
				Command::Spawn { triangle, node }
			})
			.collect()
	}

	fn decide(&self) -> Vec<Command> {
		let attacks = self.attacks();
		let used: Vec<usize> = attacks
			.iter()
			.filter_map(|c| match *c {
				Command::Attack { triangle, .. } => Some(triangle),
				_ => None,
			})
			.collect();
		let mut orders = attacks;
		orders.extend(self.spawns(&used));
		let steps = self.first_steps(&self.assign());
		let steps = self.safe_steps(steps, &orders);
		self.commands(&steps, &orders)
	}
}

fn text(map: &Map, command: Command) -> String {
	let ordered = |triangle: usize, first: usize| {
		let corners = map.triangles[triangle];
		let rest: Vec<usize> = corners.into_iter().filter(|&c| c != first).collect();
		format!("{first} {} {}", rest[0], rest[1])
	};
	match command {
		Command::Move { from, to, amount } => format!("MOVE {from} {to} {amount}"),
		Command::Spawn { triangle, node } => format!("SPAWN {}", ordered(triangle, node)),
		Command::Attack { triangle, target } => {
			let [a, b, c] = map.triangles[triangle];
			format!("ATTACK {a} {b} {c} {target}")
		}
		Command::AddPath { triangle, from, to } => {
			format!("ADD_PATH {} {to}", ordered(triangle, from))
		}
		Command::RemovePath { triangle, from, to } => {
			format!("REMOVE_PATH {} {to}", ordered(triangle, from))
		}
	}
}

struct Input<R: BufRead> {
	lines: std::io::Lines<R>,
}

impl<R: BufRead> Input<R> {
	fn ints(&mut self) -> Vec<i64> {
		let Some(line) = self.lines.next() else {
			std::process::exit(0);
		};
		line.expect("stdin is readable")
			.split_whitespace()
			.map(|field| field.parse().expect("the referee sends integers"))
			.collect()
	}

	fn int(&mut self) -> i64 {
		self.ints()[0]
	}
}

fn read_state<R: BufRead>(input: &mut Input<R>, map: &Map, me: Option<usize>) -> (State, usize) {
	let scores = input.ints();
	let count = map.len();
	let mut seen = vec![[0; 2]; count];
	for _ in 0..count {
		let row = input.ints();
		seen[row[0] as usize] = [row[1] as i32, row[2] as i32];
	}
	let me = me.unwrap_or_else(|| {
		let leftmost = (0..count)
			.min_by_key(|&n| (map.points[n].x, n))
			.expect("the map has nodes");
		usize::from(seen[leftmost][0] == 0)
	});
	let side = |pair: [i32; 2]| if me == 0 { pair } else { [pair[1], pair[0]] };
	let units: Vec<[i32; 2]> = seen.into_iter().map(side).collect();
	let edges: Vec<(usize, usize)> = (0..input.int())
		.map(|_| {
			let row = input.ints();
			(row[0] as usize, row[1] as usize)
		})
		.collect();
	let mut state = State::new(map, &units, &edges);
	for _ in 0..input.int() {
		let row = input.ints();
		let corners = [row[0] as usize, row[1] as usize, row[2] as usize];
		let triangle = map
			.triangle(corners)
			.expect("live triangles are candidates");
		let owner = match row[3] {
			0 => Some(me),
			1 => Some(1 - me),
			_ => None,
		};
		let capture = side([row[4] as i32, row[5] as i32]).map(|flag| flag != 0);
		state.set_triangle(triangle, owner, capture);
	}
	for _ in 0..input.int() {
		input.ints();
	}
	state.score = side([scores[0] as i32, scores[1] as i32]);
	(state, me)
}

fn main() {
	let params = Params::load();
	let mut input = Input {
		lines: std::io::stdin().lock().lines(),
	};
	let mut out = std::io::stdout().lock();
	let count = input.int() as usize;
	let mut points = vec![Point { x: 0, y: 0 }; count];
	for _ in 0..count {
		let row = input.ints();
		points[row[0] as usize] = Point {
			x: row[1] as i32,
			y: row[2] as i32,
		};
	}
	let map = Map::new(points);
	let mut me = None;
	loop {
		let (state, side) = read_state(&mut input, &map, me);
		me = Some(side);
		let distance = distances(&state, map.len());
		let turn = Turn {
			map: &map,
			params: &params,
			state,
			me: side,
			distance,
		};
		let commands = turn.decide();
		let line: Vec<String> = commands.iter().map(|&c| text(&map, c)).collect();
		let line = if line.is_empty() {
			"WAIT".to_string()
		} else {
			line.join(";")
		};
		writeln!(out, "{line}").expect("stdout is writable");
		out.flush().expect("stdout is writable");
	}
}
