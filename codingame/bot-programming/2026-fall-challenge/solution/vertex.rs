use std::collections::{BTreeMap, VecDeque};
use std::io::{BufRead, Write};
use std::time::{Duration, Instant};

const ME: usize = 0;
const OPPONENT: usize = 1;
const UNREACHABLE: u32 = 99;
const TURN_BUDGET: Duration = Duration::from_millis(35);
const IMPROVEMENT_EPSILON: f64 = 1e-9;

struct Params {
	max_army: usize,
	triangle_weight: f64,
	progress_weight: f64,
	threat_weight: f64,
	unit_weight: f64,
	goal_weight: f64,
	idle_weight: f64,
	goal_sum_weight: f64,
	opponent_triangle_bonus: f64,
	prediction_share: f64,
}

fn tuned(name: &str, default: f64) -> f64 {
	std::env::var(format!("VERTEX_{name}"))
		.ok()
		.and_then(|value| value.parse().ok())
		.unwrap_or(default)
}

impl Params {
	fn load() -> Self {
		Params {
			max_army: tuned("MAX_ARMY", 1000.0) as usize,
			triangle_weight: tuned("TRIANGLE_WEIGHT", 10.0),
			progress_weight: tuned("PROGRESS_WEIGHT", 0.2),
			threat_weight: tuned("THREAT_WEIGHT", 0.0),
			unit_weight: tuned("UNIT_WEIGHT", 8.0),
			goal_weight: tuned("GOAL_WEIGHT", 1.0),
			idle_weight: tuned("IDLE_WEIGHT", 0.3),
			goal_sum_weight: tuned("GOAL_SUM_WEIGHT", 0.2),
			opponent_triangle_bonus: tuned("OPPONENT_TRIANGLE_BONUS", 0.5),
			prediction_share: tuned("PREDICTION_SHARE", 0.75),
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
			.map(|field| field.parse().expect("every input field is an integer"))
			.collect()
	}

	fn int(&mut self) -> i64 {
		self.ints()[0]
	}
}

struct Graph {
	neighbors: Vec<Vec<usize>>,
	distance: Vec<Vec<u32>>,
}

impl Graph {
	fn new(house_count: usize, paths: &[(usize, usize)]) -> Self {
		let mut neighbors = vec![Vec::new(); house_count];
		for &(a, b) in paths {
			neighbors[a].push(b);
			neighbors[b].push(a);
		}
		for list in &mut neighbors {
			list.sort_unstable();
		}
		let distance = (0..house_count)
			.map(|source| breadth_first(&neighbors, source))
			.collect();
		Graph {
			neighbors,
			distance,
		}
	}

	fn len(&self) -> usize {
		self.neighbors.len()
	}

	fn first_step(&self, from: usize, goal: usize) -> usize {
		let toward = &self.distance[goal];
		self.neighbors[from]
			.iter()
			.copied()
			.find(|&next| toward[next] < toward[from])
			.unwrap_or(from)
	}
}

fn breadth_first(neighbors: &[Vec<usize>], source: usize) -> Vec<u32> {
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

#[derive(Clone)]
struct Triangle {
	corners: [usize; 3],
	owner: Option<usize>,
	can_capture: [bool; 2],
}

#[derive(Clone)]
struct State {
	units: Vec<[i32; 2]>,
	triangles: Vec<Triangle>,
}

#[derive(Clone, Copy)]
struct Spawn {
	triangle: usize,
	house: usize,
}

fn house_owner(graph: &Graph, units: &[[i32; 2]], house: usize) -> Option<usize> {
	let around = &graph.neighbors[house];
	if !around.is_empty() {
		for player in [ME, OPPONENT] {
			if around
				.iter()
				.all(|&n| units[n][player] > units[n][1 - player])
			{
				return Some(player);
			}
		}
	}
	let [mine, theirs] = units[house];
	match mine.cmp(&theirs) {
		std::cmp::Ordering::Greater => Some(ME),
		std::cmp::Ordering::Less => Some(OPPONENT),
		std::cmp::Ordering::Equal => None,
	}
}

fn house_owners(graph: &Graph, units: &[[i32; 2]]) -> Vec<Option<usize>> {
	(0..graph.len())
		.map(|house| house_owner(graph, units, house))
		.collect()
}

fn units_of(state: &State, player: usize) -> i32 {
	state.units.iter().map(|units| units[player]).sum()
}

fn unit_houses(state: &State, player: usize) -> Vec<usize> {
	state
		.units
		.iter()
		.enumerate()
		.flat_map(|(house, units)| std::iter::repeat_n(house, units[player].max(0) as usize))
		.collect()
}

fn capturable(state: &State, player: usize) -> impl Iterator<Item = usize> + '_ {
	(0..state.triangles.len()).filter(move |&t| {
		let triangle = &state.triangles[t];
		triangle.owner != Some(player) && triangle.can_capture[player]
	})
}

fn apply_spawns(state: &mut State, player: usize, spawns: &[Spawn]) {
	for spawn in spawns {
		let triangle = &mut state.triangles[spawn.triangle];
		if triangle.owner != Some(player) {
			continue;
		}
		triangle.owner = None;
		triangle.can_capture[player] = false;
		state.units[spawn.house][player] += 1;
	}
}

fn refresh_capture_rights(state: &mut State) {
	for triangle in &mut state.triangles {
		for player in [ME, OPPONENT] {
			let vacated = triangle
				.corners
				.iter()
				.all(|&c| state.units[c][player] == 0);
			match triangle.owner {
				Some(owner) => triangle.can_capture[player] = owner != player,
				None => triangle.can_capture[player] |= vacated,
			}
		}
	}
}

fn kill_surrounded(graph: &Graph, units: &mut [[i32; 2]]) -> bool {
	let ahead: Vec<[bool; 2]> = units
		.iter()
		.map(|u| [u[ME] > u[OPPONENT], u[OPPONENT] > u[ME]])
		.collect();
	let mut killed = false;
	for (house, around) in graph.neighbors.iter().enumerate() {
		if around.is_empty() {
			continue;
		}
		for player in [ME, OPPONENT] {
			let opponent = 1 - player;
			if units[house][player] > 0 && around.iter().all(|&n| ahead[n][opponent]) {
				units[house][player] = 0;
				killed = true;
			}
		}
	}
	killed
}

fn capture_triangles(graph: &Graph, state: &mut State) -> bool {
	let owners = house_owners(graph, &state.units);
	let mut changed = false;
	for triangle in &mut state.triangles {
		let before = triangle.owner;
		for player in [ME, OPPONENT] {
			if triangle.can_capture[player]
				&& triangle.corners.iter().all(|&c| owners[c] == Some(player))
			{
				triangle.owner = Some(player);
			}
		}
		changed |= triangle.owner != before;
	}
	changed
}

fn finalize(graph: &Graph, state: &mut State) {
	let mut surround_checked = false;
	loop {
		refresh_capture_rights(state);
		if !surround_checked {
			surround_checked = true;
			if kill_surrounded(graph, &mut state.units) {
				continue;
			}
		}
		if !capture_triangles(graph, state) {
			return;
		}
	}
}

fn evaluate(graph: &Graph, state: &State, params: &Params) -> f64 {
	let owners = house_owners(graph, &state.units);
	let corners_owned_by = |triangle: &Triangle, player: usize| {
		triangle
			.corners
			.iter()
			.filter(|&&c| owners[c] == Some(player))
			.count() as f64
	};
	let mut value = 0.0;
	for triangle in &state.triangles {
		match triangle.owner {
			Some(ME) => {
				value += params.triangle_weight;
				if triangle.can_capture[OPPONENT] {
					let threat = corners_owned_by(triangle, OPPONENT);
					value -= params.threat_weight * threat * threat;
				}
			}
			Some(_) => value -= params.triangle_weight,
			None => {}
		}
		if triangle.owner != Some(ME) && triangle.can_capture[ME] {
			let progress = corners_owned_by(triangle, ME);
			value += params.progress_weight * progress * progress;
		}
	}
	value + params.unit_weight * f64::from(units_of(state, ME) - units_of(state, OPPONENT))
}

fn assign_goals(
	graph: &Graph,
	state: &State,
	player: usize,
	params: &Params,
) -> (Vec<usize>, Vec<Option<usize>>) {
	let units = unit_houses(state, player);
	let opponent = 1 - player;
	let mut will_own: Vec<bool> = house_owners(graph, &state.units)
		.iter()
		.map(|&owner| owner == Some(player))
		.collect();
	let mut goals = vec![None; units.len()];
	let mut free = vec![true; units.len()];
	let mut remaining: Vec<usize> = capturable(state, player).collect();
	let distance = |unit: usize, corner: usize| graph.distance[units[unit]][corner];

	while free.contains(&true) && !remaining.is_empty() {
		let mut best: Option<(f64, usize, Vec<usize>, Vec<usize>)> = None;
		'triangles: for (index, &t) in remaining.iter().enumerate() {
			let triangle = &state.triangles[t];
			let anchors: Vec<usize> = triangle
				.corners
				.iter()
				.filter(|&&c| will_own[c])
				.filter_map(|&c| (0..units.len()).find(|&u| free[u] && units[u] == c))
				.collect();
			let mut taken = Vec::with_capacity(3);
			let (mut cost_sum, mut cost_max) = (0, 0);
			for &corner in &triangle.corners {
				if will_own[corner] {
					continue;
				}
				let nearest = (0..units.len())
					.filter(|&u| free[u] && !taken.contains(&u) && !anchors.contains(&u))
					.min_by_key(|&u| distance(u, corner));
				let Some(unit) = nearest else {
					continue 'triangles;
				};
				taken.push(unit);
				cost_sum += distance(unit, corner);
				cost_max = cost_max.max(distance(unit, corner));
			}
			let bonus = if triangle.owner == Some(opponent) {
				params.opponent_triangle_bonus
			} else {
				0.0
			};
			let cost = f64::from(cost_max) + params.goal_sum_weight * f64::from(cost_sum) - bonus;
			if best.as_ref().is_none_or(|(lowest, ..)| cost < *lowest) {
				best = Some((cost, index, anchors, taken));
			}
		}
		let Some((_, index, _, mut pool)) = best else {
			break;
		};
		let corners = state.triangles[remaining.remove(index)].corners;
		let mut needed: Vec<usize> = corners.into_iter().filter(|&c| !will_own[c]).collect();
		needed.sort_by_key(|&c| pool.iter().map(|&u| distance(u, c)).min());
		for corner in needed {
			let closest = (0..pool.len())
				.min_by_key(|&i| distance(pool[i], corner))
				.expect("one unit was taken per needed corner");
			let unit = pool.remove(closest);
			goals[unit] = Some(corner);
			free[unit] = false;
			will_own[corner] = true;
		}
	}
	(units, goals)
}

fn target_distance(graph: &Graph, state: &State, player: usize) -> Vec<u32> {
	let mut is_target = vec![false; graph.len()];
	for t in capturable(state, player) {
		for corner in state.triangles[t].corners {
			is_target[corner] = true;
		}
	}
	if !is_target.contains(&true) {
		return vec![0; graph.len()];
	}
	(0..graph.len())
		.map(|house| {
			(0..graph.len())
				.filter(|&t| is_target[t])
				.map(|t| graph.distance[house][t])
				.min()
				.unwrap_or(0)
		})
		.collect()
}

fn plan_spawns(graph: &Graph, state: &State, player: usize, params: &Params) -> Vec<Spawn> {
	let army = units_of(state, player).max(0) as usize;
	let owned: Vec<usize> = (0..state.triangles.len())
		.filter(|&t| state.triangles[t].owner == Some(player))
		.collect();
	if army >= params.max_army || owned.is_empty() || capturable(state, player).next().is_none() {
		return Vec::new();
	}
	let frontier = target_distance(graph, state, player);
	let spawn_house = |t: usize| {
		*state.triangles[t]
			.corners
			.iter()
			.min_by_key(|&&c| frontier[c])
			.expect("a triangle has corners")
	};
	let mut ranked: Vec<Spawn> = owned
		.into_iter()
		.map(|triangle| Spawn {
			triangle,
			house: spawn_house(triangle),
		})
		.collect();
	ranked.sort_by_key(|spawn| frontier[spawn.house]);
	ranked.truncate(params.max_army - army);
	ranked
}

fn stepped_counts(graph: &Graph, units: &[usize], goals: &[Option<usize>]) -> Vec<i32> {
	let mut counts = vec![0; graph.len()];
	for (&origin, goal) in units.iter().zip(goals) {
		let destination = goal.map_or(origin, |goal| graph.first_step(origin, goal));
		counts[destination] += 1;
	}
	counts
}

struct Scenario {
	opponent_units: Vec<i32>,
	share: f64,
}

struct MoveSearch<'a> {
	graph: &'a Graph,
	state: &'a State,
	params: &'a Params,
	spawns: &'a [Spawn],
	scenarios: Vec<Scenario>,
}

impl MoveSearch<'_> {
	fn outcome(&self, mine: &[i32]) -> f64 {
		self.scenarios
			.iter()
			.filter(|scenario| scenario.share > 0.0)
			.map(|scenario| {
				let mut next = self.state.clone();
				for (house, units) in next.units.iter_mut().enumerate() {
					*units = [mine[house], scenario.opponent_units[house]];
				}
				apply_spawns(&mut next, ME, self.spawns);
				finalize(self.graph, &mut next);
				scenario.share * evaluate(self.graph, &next, self.params)
			})
			.sum()
	}

	fn run(&self, deadline: Instant) -> (Vec<usize>, Vec<usize>) {
		let (units, goals) = assign_goals(self.graph, self.state, ME, self.params);
		let idle_distance = target_distance(self.graph, self.state, ME);
		let unit_cost = |unit: usize, house: usize| match goals[unit] {
			Some(goal) => self.params.goal_weight * f64::from(self.graph.distance[house][goal]),
			None => self.params.idle_weight * f64::from(idle_distance[house]),
		};
		let mut destinations: Vec<usize> = units
			.iter()
			.zip(&goals)
			.map(|(&origin, goal)| goal.map_or(origin, |goal| self.graph.first_step(origin, goal)))
			.collect();
		let mut mine = vec![0; self.graph.len()];
		for &house in &destinations {
			mine[house] += 1;
		}
		let mut goal_total: f64 = (0..units.len())
			.map(|unit| unit_cost(unit, destinations[unit]))
			.sum();
		let mut best = self.outcome(&mine) - goal_total;
		let mut improved = true;
		while improved && Instant::now() < deadline {
			improved = false;
			for unit in 0..units.len() {
				if Instant::now() >= deadline {
					break;
				}
				let origin = units[unit];
				let candidates =
					std::iter::once(origin).chain(self.graph.neighbors[origin].iter().copied());
				for candidate in candidates {
					let current = destinations[unit];
					if candidate == current {
						continue;
					}
					mine[current] -= 1;
					mine[candidate] += 1;
					let candidate_goal_total =
						goal_total - unit_cost(unit, current) + unit_cost(unit, candidate);
					let value = self.outcome(&mine) - candidate_goal_total;
					if value > best + IMPROVEMENT_EPSILON {
						best = value;
						goal_total = candidate_goal_total;
						destinations[unit] = candidate;
						improved = true;
					} else {
						mine[candidate] -= 1;
						mine[current] += 1;
					}
				}
			}
		}
		(units, destinations)
	}
}

fn opponent_scenarios(graph: &Graph, state: &State, params: &Params) -> Vec<Scenario> {
	let standing: Vec<i32> = state.units.iter().map(|u| u[OPPONENT]).collect();
	let (units, goals) = assign_goals(graph, state, OPPONENT, params);
	vec![
		Scenario {
			opponent_units: standing,
			share: 1.0 - params.prediction_share,
		},
		Scenario {
			opponent_units: stepped_counts(graph, &units, &goals),
			share: params.prediction_share,
		},
	]
}

fn decide(graph: &Graph, state: &State, params: &Params, deadline: Instant) -> String {
	let spawns = plan_spawns(graph, state, ME, params);
	let search = MoveSearch {
		graph,
		state,
		params,
		spawns: &spawns,
		scenarios: opponent_scenarios(graph, state, params),
	};
	let (units, destinations) = search.run(deadline);
	let mut moves: BTreeMap<(usize, usize), i32> = BTreeMap::new();
	for (&origin, &target) in units.iter().zip(&destinations) {
		if origin != target {
			*moves.entry((origin, target)).or_default() += 1;
		}
	}
	let mut commands: Vec<String> = moves
		.into_iter()
		.map(|((from, to), amount)| format!("MOVE {from} {to} {amount}"))
		.collect();
	for spawn in spawns {
		let others: Vec<usize> = state.triangles[spawn.triangle]
			.corners
			.into_iter()
			.filter(|&c| c != spawn.house)
			.collect();
		let (house, second, third) = (spawn.house, others[0], others[1]);
		commands.push(format!("SPAWN {house} {second} {third}"));
	}
	if commands.is_empty() {
		"WAIT".to_string()
	} else {
		commands.join(";")
	}
}

fn read_turn<R: BufRead>(input: &mut Input<R>, house_count: usize) -> (Graph, State) {
	let mut units = vec![[0, 0]; house_count];
	for _ in 0..house_count {
		let row = input.ints();
		units[row[0] as usize] = [row[1] as i32, row[2] as i32];
	}
	let path_count = input.int();
	let paths: Vec<(usize, usize)> = (0..path_count)
		.map(|_| {
			let row = input.ints();
			(row[0] as usize, row[1] as usize)
		})
		.collect();
	let triangle_count = input.int();
	let triangles = (0..triangle_count)
		.map(|_| {
			let row = input.ints();
			Triangle {
				corners: [row[0] as usize, row[1] as usize, row[2] as usize],
				owner: usize::try_from(row[3]).ok(),
				can_capture: [row[4] != 0, row[5] != 0],
			}
		})
		.collect();
	for _ in 0..input.int() {
		input.ints();
	}
	(Graph::new(house_count, &paths), State { units, triangles })
}

fn main() {
	let params = Params::load();
	let mut input = Input {
		lines: std::io::stdin().lock().lines(),
	};
	let mut out = std::io::stdout().lock();
	let house_count = input.int() as usize;
	for _ in 0..house_count {
		input.ints();
	}
	loop {
		input.ints();
		let started = Instant::now();
		let (graph, state) = read_turn(&mut input, house_count);
		let answer = decide(&graph, &state, &params, started + TURN_BUDGET);
		writeln!(out, "{answer}").expect("stdout is writable");
		out.flush().expect("stdout is writable");
	}
}
