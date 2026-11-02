use std::collections::{BTreeMap, VecDeque};
use std::io::{BufRead, Write};
use std::time::{Duration, Instant};

const ME: usize = 0;
const OPPONENT: usize = 1;
const UNREACHABLE: u32 = 99;
const TURN_BUDGET: Duration = Duration::from_millis(35);
const BASE_SEARCH_SHARE: f64 = 0.3;
const SCREEN_SHARE: f64 = 0.45;
const ROLLOUT_SHARE: f64 = 0.8;
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
	action_margin: f64,
	locked_weight: f64,
	rollout_depth: usize,
	rollout_candidates: usize,
	rollout_floor: f64,
	rollout_margin: f64,
	rollout_holds: bool,
}

fn tuned(name: &str, default: f64) -> f64 {
	std::env::var(format!("SUMMIT_{name}"))
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
			action_margin: tuned("ACTION_MARGIN", 0.0),
			locked_weight: tuned("LOCKED_WEIGHT", 3.0),
			rollout_depth: tuned("ROLLOUT_DEPTH", 0.0) as usize,
			rollout_candidates: tuned("ROLLOUT_CANDIDATES", 12.0) as usize,
			rollout_floor: tuned("ROLLOUT_FLOOR", -40.0),
			rollout_margin: tuned("ROLLOUT_MARGIN", 0.0),
			rollout_holds: tuned("ROLLOUT_HOLDS", 1.0) != 0.0,
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

fn bit(house: usize) -> u64 {
	1 << house
}

fn houses(mut set: u64) -> impl Iterator<Item = usize> {
	std::iter::from_fn(move || {
		(set != 0).then(|| {
			let house = set.trailing_zeros() as usize;
			set &= set - 1;
			house
		})
	})
}

#[derive(Clone, Copy)]
struct Point {
	x: i32,
	y: i32,
}

fn ccw(a: Point, b: Point, c: Point) -> bool {
	(c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x)
}

fn is_inside(points: &[Point], house: usize, corners: [usize; 3]) -> bool {
	if corners.contains(&house) {
		return false;
	}
	let p = points[house];
	let [a, b, c] = corners.map(|n| points[n]);
	let turns = [ccw(p, a, b), ccw(p, b, c), ccw(p, c, a)];
	!(turns.contains(&false) && turns.contains(&true))
}

fn has_house_inside(points: &[Point], corners: [usize; 3]) -> bool {
	let [a, b, c] = corners.map(|n| points[n]);
	let box_bounds_inside = ccw(a, b, c) || ccw(a, c, b);
	let (left, right) = (a.x.min(b.x).min(c.x), a.x.max(b.x).max(c.x));
	let (top, bottom) = (a.y.min(b.y).min(c.y), a.y.max(b.y).max(c.y));
	(0..points.len()).any(|n| {
		let p = points[n];
		let in_box = (left..=right).contains(&p.x) && (top..=bottom).contains(&p.y);
		(in_box || !box_bounds_inside) && is_inside(points, n, corners)
	})
}

fn find_triangles(points: &[Point], adjacency: &[u64]) -> Vec<[usize; 3]> {
	let mut found = Vec::new();
	for a in 0..adjacency.len() {
		let above_a = (u64::MAX << a) << 1;
		for b in houses(adjacency[a] & above_a) {
			let above_b = (u64::MAX << b) << 1;
			for c in houses(adjacency[a] & adjacency[b] & above_b) {
				if !has_house_inside(points, [a, b, c]) {
					found.push([a, b, c]);
				}
			}
		}
	}
	found
}

struct Graph {
	neighbors: Vec<Vec<usize>>,
	distance: Vec<Vec<u32>>,
}

impl Graph {
	fn new(adjacency: &[u64]) -> Self {
		let neighbors: Vec<Vec<usize>> =
			adjacency.iter().map(|&set| houses(set).collect()).collect();
		let distance = (0..neighbors.len())
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

impl Triangle {
	fn fresh(corners: [usize; 3]) -> Self {
		Triangle {
			corners,
			owner: None,
			can_capture: [true, true],
		}
	}

	fn corner_set(&self) -> u64 {
		self.corners.iter().fold(0, |set, &c| set | bit(c))
	}
}

#[derive(Clone)]
struct State {
	adjacency: Vec<u64>,
	units: Vec<[i32; 2]>,
	triangles: Vec<Triangle>,
}

impl State {
	fn connect(&mut self, a: usize, b: usize) {
		self.adjacency[a] |= bit(b);
		self.adjacency[b] |= bit(a);
	}

	fn disconnect(&mut self, a: usize, b: usize) {
		self.adjacency[a] &= !bit(b);
		self.adjacency[b] &= !bit(a);
	}

	fn rebuild_triangles(&mut self, points: &[Point]) {
		let previous = std::mem::take(&mut self.triangles);
		self.triangles = find_triangles(points, &self.adjacency)
			.into_iter()
			.map(|corners| {
				previous
					.iter()
					.find(|t| t.corners == corners)
					.cloned()
					.unwrap_or_else(|| Triangle::fresh(corners))
			})
			.collect();
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Action {
	Attack(usize),
	Spawn(usize),
	AddPath(usize, usize),
	RemovePath(usize, usize),
}

impl Action {
	fn costs_units(self) -> bool {
		!matches!(self, Action::Spawn(_))
	}

	fn phase(self) -> u8 {
		match self {
			Action::Attack(_) => 0,
			Action::Spawn(_) => 1,
			Action::AddPath(..) => 2,
			Action::RemovePath(..) => 3,
		}
	}
}

#[derive(Clone, Copy, Debug)]
struct Order {
	triangle: usize,
	action: Action,
}

impl Order {
	fn command(self, state: &State) -> String {
		let corners = state.triangles[self.triangle].corners;
		let led_by = |lead: usize| {
			let others: Vec<usize> = corners.into_iter().filter(|&c| c != lead).collect();
			let (second, third) = (others[0], others[1]);
			format!("{lead} {second} {third}")
		};
		match self.action {
			Action::Spawn(house) => {
				let triangle = led_by(house);
				format!("SPAWN {triangle}")
			}
			Action::Attack(target) => {
				let [a, b, c] = corners;
				format!("ATTACK {a} {b} {c} {target}")
			}
			Action::AddPath(from, to) => {
				let triangle = led_by(from);
				format!("ADD_PATH {triangle} {to}")
			}
			Action::RemovePath(from, to) => {
				let triangle = led_by(from);
				format!("REMOVE_PATH {triangle} {to}")
			}
		}
	}
}

fn house_owner(state: &State, house: usize) -> Option<usize> {
	let around = state.adjacency[house];
	let units = &state.units;
	if around != 0 {
		for player in [ME, OPPONENT] {
			if houses(around).all(|n| units[n][player] > units[n][1 - player]) {
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

fn house_owners(state: &State) -> Vec<Option<usize>> {
	(0..state.units.len())
		.map(|house| house_owner(state, house))
		.collect()
}

fn units_of(state: &State, player: usize) -> i32 {
	state.units.iter().map(|units| units[player]).sum()
}

fn expand(counts: impl Iterator<Item = i32>) -> Vec<usize> {
	counts
		.enumerate()
		.flat_map(|(house, count)| std::iter::repeat_n(house, count.max(0) as usize))
		.collect()
}

fn capturable(state: &State, player: usize) -> impl Iterator<Item = usize> + '_ {
	(0..state.triangles.len()).filter(move |&t| {
		let triangle = &state.triangles[t];
		triangle.owner != Some(player) && triangle.can_capture[player]
	})
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

fn kill_surrounded(state: &mut State) -> bool {
	let ahead: Vec<[bool; 2]> = state
		.units
		.iter()
		.map(|u| [u[ME] > u[OPPONENT], u[OPPONENT] > u[ME]])
		.collect();
	let mut killed = false;
	for (house, &around) in state.adjacency.iter().enumerate() {
		if around == 0 {
			continue;
		}
		for player in [ME, OPPONENT] {
			let opponent = 1 - player;
			if state.units[house][player] > 0 && houses(around).all(|n| ahead[n][opponent]) {
				state.units[house][player] = 0;
				killed = true;
			}
		}
	}
	killed
}

fn capture_triangles(state: &mut State) -> bool {
	let owners = house_owners(state);
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

fn finalize(state: &mut State) {
	let mut surround_checked = false;
	loop {
		refresh_capture_rights(state);
		if !surround_checked {
			surround_checked = true;
			if kill_surrounded(state) {
				continue;
			}
		}
		if !capture_triangles(state) {
			return;
		}
	}
}

fn simulate(
	points: &[Point],
	state: &State,
	moves: [&[(usize, usize)]; 2],
	orders: [&[Order]; 2],
) -> State {
	let attacked = orders.map(|list| {
		list.iter().fold(0, |set, order| match order.action {
			Action::Attack(target) => set | bit(target),
			_ => set,
		})
	});
	let mut next = state.clone();
	for units in &mut next.units {
		*units = [0, 0];
	}
	for player in [ME, OPPONENT] {
		let killed = attacked[1 - player];
		for &(origin, destination) in moves[player] {
			if killed & bit(origin) == 0 {
				next.units[destination][player] += 1;
			}
		}
	}
	let mut ordered: Vec<(usize, Order)> = [ME, OPPONENT]
		.into_iter()
		.flat_map(|player| orders[player].iter().map(move |&order| (player, order)))
		.collect();
	ordered.sort_by_key(|&(player, order)| match order.action {
		Action::AddPath(a, b) => (order.action.phase(), squared_length(points, a, b), player),
		action => (action.phase(), 0, player),
	});
	let mut reshaped = false;
	for (player, order) in ordered {
		let triangle = &mut next.triangles[order.triangle];
		if triangle.owner != Some(player) {
			continue;
		}
		triangle.owner = None;
		triangle.can_capture[player] = false;
		match order.action {
			Action::Attack(_) => {}
			Action::Spawn(house) => next.units[house][player] += 1,
			Action::AddPath(a, b) => {
				next.connect(a, b);
				reshaped = true;
			}
			Action::RemovePath(a, b) => {
				next.disconnect(a, b);
				reshaped = true;
			}
		}
	}
	if reshaped {
		next.rebuild_triangles(points);
	}
	finalize(&mut next);
	next
}

fn squared_length(points: &[Point], a: usize, b: usize) -> i32 {
	let (dx, dy) = (points[a].x - points[b].x, points[a].y - points[b].y);
	dx * dx + dy * dy
}

fn evaluate(state: &State, params: &Params) -> f64 {
	let owners = house_owners(state);
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
		if triangle.owner != Some(ME) {
			if triangle.can_capture[ME] {
				let progress = corners_owned_by(triangle, ME);
				value += params.progress_weight * progress * progress;
			} else {
				value -= params.locked_weight;
			}
		}
	}
	value + params.unit_weight * f64::from(units_of(state, ME) - units_of(state, OPPONENT))
}

fn assign_goals(
	graph: &Graph,
	state: &State,
	player: usize,
	units: &[usize],
	params: &Params,
) -> Vec<Option<usize>> {
	let opponent = 1 - player;
	let mut will_own: Vec<bool> = house_owners(state)
		.iter()
		.map(|&owner| owner == Some(player))
		.collect();
	let mut goals = vec![None; units.len()];
	let mut free = vec![true; units.len()];
	let mut remaining: Vec<usize> = capturable(state, player).collect();
	let distance = |unit: usize, corner: usize| graph.distance[units[unit]][corner];

	while free.contains(&true) && !remaining.is_empty() {
		let mut best: Option<(f64, usize, Vec<usize>)> = None;
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
				best = Some((cost, index, taken));
			}
		}
		let Some((_, index, mut pool)) = best else {
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
	goals
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

fn plan_spawns(graph: &Graph, state: &State, player: usize, params: &Params) -> Vec<Order> {
	let army = units_of(state, player).max(0) as usize;
	let owned: Vec<usize> = (0..state.triangles.len())
		.filter(|&t| state.triangles[t].owner == Some(player))
		.collect();
	if army >= params.max_army || owned.is_empty() || capturable(state, player).next().is_none() {
		return Vec::new();
	}
	let frontier = target_distance(graph, state, player);
	let mut ranked: Vec<Order> = owned
		.into_iter()
		.map(|triangle| {
			let house = *state.triangles[triangle]
				.corners
				.iter()
				.min_by_key(|&&c| frontier[c])
				.expect("a triangle has corners");
			Order {
				triangle,
				action: Action::Spawn(house),
			}
		})
		.collect();
	ranked.sort_by_key(|order| match order.action {
		Action::Spawn(house) => frontier[house],
		_ => UNREACHABLE,
	});
	ranked.truncate(params.max_army - army);
	ranked
}

fn movable_units(state: &State, player: usize, orders: &[Order]) -> Option<Vec<usize>> {
	let mut counts: Vec<i32> = state.units.iter().map(|u| u[player]).collect();
	for order in orders.iter().filter(|o| o.action.costs_units()) {
		for corner in state.triangles[order.triangle].corners {
			counts[corner] -= 1;
			if counts[corner] < 0 {
				return None;
			}
		}
	}
	Some(expand(counts.into_iter()))
}

fn opponent_scenarios(
	graph: &Graph,
	state: &State,
	params: &Params,
) -> Vec<(Vec<(usize, usize)>, f64)> {
	let units = expand(state.units.iter().map(|u| u[OPPONENT]));
	let goals = assign_goals(graph, state, OPPONENT, &units, params);
	let standing = units.iter().map(|&h| (h, h)).collect();
	let stepping = units
		.iter()
		.zip(&goals)
		.map(|(&origin, goal)| {
			(
				origin,
				goal.map_or(origin, |goal| graph.first_step(origin, goal)),
			)
		})
		.collect();
	vec![
		(standing, 1.0 - params.prediction_share),
		(stepping, params.prediction_share),
	]
}

fn default_policy(
	graph: &Graph,
	state: &State,
	player: usize,
	params: &Params,
) -> (Vec<(usize, usize)>, Vec<Order>) {
	let spawns = plan_spawns(graph, state, player, params);
	let units = expand(state.units.iter().map(|u| u[player]));
	let goals = assign_goals(graph, state, player, &units, params);
	let idle_distance = target_distance(graph, state, player);
	let moves = units
		.iter()
		.zip(&goals)
		.map(|(&origin, goal)| {
			let destination = match goal {
				Some(goal) => graph.first_step(origin, *goal),
				None => graph.neighbors[origin]
					.iter()
					.copied()
					.find(|&n| idle_distance[n] < idle_distance[origin])
					.unwrap_or(origin),
			};
			(origin, destination)
		})
		.collect();
	(moves, spawns)
}

struct Turn<'a> {
	points: &'a [Point],
	graph: Graph,
	state: State,
	linkable: Vec<(usize, usize)>,
	params: &'a Params,
	scenarios: Vec<(Vec<(usize, usize)>, f64)>,
	opponent_spawns: Vec<Order>,
}

struct Plan {
	orders: Vec<Order>,
	units: Vec<usize>,
	destinations: Vec<usize>,
}

impl Plan {
	fn moves(&self) -> Vec<(usize, usize)> {
		self.units
			.iter()
			.copied()
			.zip(self.destinations.iter().copied())
			.collect()
	}
}

#[derive(Clone, Copy)]
enum Change {
	Hold(usize),
	Use(Order),
}

impl Change {
	fn triangle(self) -> usize {
		match self {
			Change::Hold(triangle) => triangle,
			Change::Use(order) => order.triangle,
		}
	}

	fn order(self) -> Option<Order> {
		match self {
			Change::Hold(_) => None,
			Change::Use(order) => Some(order),
		}
	}

	fn clashes(self, other: Change) -> bool {
		let same_target = matches!(
			(self.order().map(|o| o.action), other.order().map(|o| o.action)),
			(Some(Action::Attack(x)), Some(Action::Attack(y))) if x == y
		);
		self.triangle() == other.triangle() || same_target
	}
}

impl Turn<'_> {
	fn outcome(&self, orders: &[Order], units: &[usize], destinations: &[usize]) -> f64 {
		let mine: Vec<(usize, usize)> = units
			.iter()
			.copied()
			.zip(destinations.iter().copied())
			.collect();
		self.scenarios
			.iter()
			.filter(|(_, share)| *share > 0.0)
			.map(|(opponent, share)| {
				let next = simulate(self.points, &self.state, [&mine, opponent], [orders, &[]]);
				share * evaluate(&next, self.params)
			})
			.sum()
	}

	fn rollout(&self, plan: &Plan) -> f64 {
		let predicted = &self.scenarios[1].0;
		let orders = [plan.orders.as_slice(), self.opponent_spawns.as_slice()];
		let mut state = simulate(self.points, &self.state, [&plan.moves(), predicted], orders);
		for _ in 1..self.params.rollout_depth {
			let reshaped;
			let graph = if state.adjacency == self.state.adjacency {
				&self.graph
			} else {
				reshaped = Graph::new(&state.adjacency);
				&reshaped
			};
			let (my_moves, my_orders) = default_policy(graph, &state, ME, self.params);
			let (their_moves, their_orders) = default_policy(graph, &state, OPPONENT, self.params);
			state = simulate(
				self.points,
				&state,
				[&my_moves, &their_moves],
				[&my_orders, &their_orders],
			);
		}
		evaluate(&state, self.params)
	}

	fn search_moves(&self, orders: Vec<Order>, units: Vec<usize>, deadline: Instant) -> Plan {
		let graph = &self.graph;
		let params = self.params;
		let goals = assign_goals(graph, &self.state, ME, &units, params);
		let idle_distance = target_distance(graph, &self.state, ME);
		let unit_cost = |unit: usize, house: usize| match goals[unit] {
			Some(goal) => params.goal_weight * f64::from(graph.distance[house][goal]),
			None => params.idle_weight * f64::from(idle_distance[house]),
		};
		let mut destinations: Vec<usize> = units
			.iter()
			.zip(&goals)
			.map(|(&origin, goal)| goal.map_or(origin, |goal| graph.first_step(origin, goal)))
			.collect();
		let mut goal_total: f64 = (0..units.len())
			.map(|unit| unit_cost(unit, destinations[unit]))
			.sum();
		let mut best = self.outcome(&orders, &units, &destinations) - goal_total;
		let mut improved = true;
		while improved && Instant::now() < deadline {
			improved = false;
			for unit in 0..units.len() {
				if Instant::now() >= deadline {
					break;
				}
				let origin = units[unit];
				let candidates =
					std::iter::once(origin).chain(graph.neighbors[origin].iter().copied());
				for candidate in candidates {
					let current = destinations[unit];
					if candidate == current {
						continue;
					}
					destinations[unit] = candidate;
					let candidate_goal_total =
						goal_total - unit_cost(unit, current) + unit_cost(unit, candidate);
					let value = self.outcome(&orders, &units, &destinations) - candidate_goal_total;
					if value > best + IMPROVEMENT_EPSILON {
						best = value;
						goal_total = candidate_goal_total;
						improved = true;
					} else {
						destinations[unit] = current;
					}
				}
			}
		}
		Plan {
			orders,
			units,
			destinations,
		}
	}

	fn alternatives(&self, rules: &Rules) -> Vec<Order> {
		let state = &self.state;
		let mut found = Vec::new();
		for (t, triangle) in state.triangles.iter().enumerate() {
			if triangle.owner != Some(ME)
				|| triangle.corners.iter().any(|&c| state.units[c][ME] < 1)
			{
				continue;
			}
			let corners = triangle.corner_set();
			let mut add = |action| {
				found.push(Order {
					triangle: t,
					action,
				})
			};
			if rules.attack != Some(false) {
				let around = houses(corners).fold(0, |set, c| set | state.adjacency[c]);
				for target in houses(around).filter(|&h| state.units[h][OPPONENT] > 0) {
					add(Action::Attack(target));
				}
			}
			if rules.paths {
				for &(a, b) in &self.linkable {
					if corners & bit(a) != 0 {
						add(Action::AddPath(a, b));
					} else if corners & bit(b) != 0 {
						add(Action::AddPath(b, a));
					}
				}
				for corner in triangle.corners {
					for other in houses(state.adjacency[corner] & !corners) {
						add(Action::RemovePath(corner, other));
					}
				}
			}
		}
		found
	}

	fn without_payers(&self, plan: &Plan, orders: &[Order]) -> Option<(Vec<usize>, Vec<usize>)> {
		let mut units = plan.units.clone();
		let mut destinations = plan.destinations.clone();
		for order in orders.iter().filter(|o| o.action.costs_units()) {
			for corner in self.state.triangles[order.triangle].corners {
				let at_corner = |i: &usize| units[*i] == corner;
				let payer = (0..units.len())
					.filter(at_corner)
					.find(|&i| destinations[i] == corner)
					.or_else(|| (0..units.len()).find(at_corner))?;
				units.remove(payer);
				destinations.remove(payer);
			}
		}
		Some((units, destinations))
	}

	fn variant(&self, base: &Plan, change: Change) -> Option<Plan> {
		let mut orders: Vec<Order> = base
			.orders
			.iter()
			.copied()
			.filter(|o| o.triangle != change.triangle())
			.collect();
		orders.extend(change.order());
		let (units, destinations) = self.without_payers(base, change.order().as_slice())?;
		Some(Plan {
			orders,
			units,
			destinations,
		})
	}

	fn ranked_changes(&self, rules: &Rules, base: &Plan, started: Instant) -> Vec<(f64, Change)> {
		let at = |share: f64| started + TURN_BUDGET.mul_f64(share);
		let params = self.params;
		let rolling = params.rollout_depth > 0;
		let mut changes: Vec<Change> = self
			.alternatives(rules)
			.into_iter()
			.map(Change::Use)
			.collect();
		if rolling && params.rollout_holds {
			changes.extend(base.orders.iter().map(|o| Change::Hold(o.triangle)));
		}
		let floor = if rolling {
			params.rollout_floor
		} else {
			params.action_margin
		};
		let base_value = self.outcome(&base.orders, &base.units, &base.destinations);
		let mut screened: Vec<(f64, Change, Plan)> = Vec::new();
		for change in changes {
			if Instant::now() >= at(SCREEN_SHARE) {
				break;
			}
			let Some(variant) = self.variant(base, change) else {
				continue;
			};
			let gain =
				self.outcome(&variant.orders, &variant.units, &variant.destinations) - base_value;
			if gain > floor {
				screened.push((gain, change, variant));
			}
		}
		screened.sort_by(|a, b| b.0.total_cmp(&a.0));
		if !rolling {
			return screened
				.into_iter()
				.map(|(gain, change, _)| (gain, change))
				.collect();
		}
		screened.truncate(params.rollout_candidates);
		let base_rollout = self.rollout(base);
		let mut gains = Vec::new();
		for (_, change, variant) in &screened {
			if Instant::now() >= at(ROLLOUT_SHARE) {
				break;
			}
			let gain = self.rollout(variant) - base_rollout;
			if gain > params.rollout_margin {
				gains.push((gain, *change));
			}
		}
		gains.sort_by(|a, b| b.0.total_cmp(&a.0));
		gains
	}

	fn decide(&self, rules: &Rules, started: Instant) -> Plan {
		let spawns = plan_spawns(&self.graph, &self.state, ME, self.params);
		let units = movable_units(&self.state, ME, &spawns).expect("spawns cost no units");
		let base_deadline = started + TURN_BUDGET.mul_f64(BASE_SEARCH_SHARE);
		let base = self.search_moves(spawns, units, base_deadline);
		let gains = self.ranked_changes(rules, &base, started);
		if gains.is_empty() {
			return base;
		}
		let mut chosen: Vec<Change> = Vec::new();
		for (_, change) in gains {
			if chosen.iter().any(|c| c.clashes(change)) {
				continue;
			}
			chosen.push(change);
			let uses: Vec<Order> = chosen.iter().filter_map(|c| c.order()).collect();
			if self.without_payers(&base, &uses).is_none() {
				chosen.pop();
			}
		}
		let mut orders: Vec<Order> = base
			.orders
			.iter()
			.copied()
			.filter(|o| chosen.iter().all(|c| c.triangle() != o.triangle))
			.collect();
		orders.extend(chosen.iter().filter_map(|c| c.order()));
		let units = movable_units(&self.state, ME, &orders).expect("payers were checked");
		self.search_moves(orders, units, started + TURN_BUDGET)
	}
}

fn answer(plan: &Plan, state: &State) -> String {
	let mut moves: BTreeMap<(usize, usize), i32> = BTreeMap::new();
	for (&origin, &target) in plan.units.iter().zip(&plan.destinations) {
		if origin != target {
			*moves.entry((origin, target)).or_default() += 1;
		}
	}
	let mut commands: Vec<String> = plan
		.orders
		.iter()
		.filter(|o| o.action.phase() == 0)
		.map(|o| o.command(state))
		.collect();
	commands.extend(
		moves
			.into_iter()
			.map(|((from, to), amount)| format!("MOVE {from} {to} {amount}")),
	);
	commands.extend(
		plan.orders
			.iter()
			.filter(|o| o.action.phase() > 0)
			.map(|o| o.command(state)),
	);
	if commands.is_empty() {
		"WAIT".to_string()
	} else {
		commands.join(";")
	}
}

#[derive(Default)]
struct Rules {
	attack: Option<bool>,
	paths: bool,
	attack_probe: Option<[usize; 3]>,
}

impl Rules {
	fn observe(&mut self, state: &State, linkable: &[(usize, usize)]) {
		if !linkable.is_empty() {
			self.paths = true;
			self.attack = Some(true);
		}
		if let Some(corners) = self.attack_probe.take() {
			let still_mine = state
				.triangles
				.iter()
				.any(|t| t.corners == corners && t.owner == Some(ME));
			self.attack = Some(!still_mine);
		}
	}

	fn record(&mut self, plan: &Plan, state: &State) {
		if self.attack.is_some() {
			return;
		}
		self.attack_probe = plan
			.orders
			.iter()
			.find(|o| matches!(o.action, Action::Attack(_)))
			.map(|o| state.triangles[o.triangle].corners);
	}
}

fn read_turn<R: BufRead>(input: &mut Input<R>, house_count: usize) -> (State, Vec<(usize, usize)>) {
	let mut units = vec![[0, 0]; house_count];
	for _ in 0..house_count {
		let row = input.ints();
		units[row[0] as usize] = [row[1] as i32, row[2] as i32];
	}
	let mut adjacency = vec![0; house_count];
	for _ in 0..input.int() {
		let row = input.ints();
		let (a, b) = (row[0] as usize, row[1] as usize);
		adjacency[a] |= bit(b);
		adjacency[b] |= bit(a);
	}
	let triangles = (0..input.int())
		.map(|_| {
			let row = input.ints();
			Triangle {
				corners: [row[0] as usize, row[1] as usize, row[2] as usize],
				owner: usize::try_from(row[3]).ok(),
				can_capture: [row[4] != 0, row[5] != 0],
			}
		})
		.collect();
	let linkable = (0..input.int())
		.map(|_| {
			let row = input.ints();
			(row[0] as usize, row[1] as usize)
		})
		.collect();
	(
		State {
			adjacency,
			units,
			triangles,
		},
		linkable,
	)
}

fn main() {
	let params = Params::load();
	let mut input = Input {
		lines: std::io::stdin().lock().lines(),
	};
	let mut out = std::io::stdout().lock();
	let house_count = input.int() as usize;
	let mut points = vec![Point { x: 0, y: 0 }; house_count];
	for _ in 0..house_count {
		let row = input.ints();
		points[row[0] as usize] = Point {
			x: row[1] as i32,
			y: row[2] as i32,
		};
	}
	let mut rules = Rules::default();
	loop {
		input.ints();
		let started = Instant::now();
		let (state, linkable) = read_turn(&mut input, house_count);
		rules.observe(&state, &linkable);
		let graph = Graph::new(&state.adjacency);
		let scenarios = opponent_scenarios(&graph, &state, &params);
		let opponent_spawns = plan_spawns(&graph, &state, OPPONENT, &params);
		let turn = Turn {
			points: &points,
			graph,
			state,
			linkable,
			params: &params,
			scenarios,
			opponent_spawns,
		};
		let plan = turn.decide(&rules, started);
		rules.record(&plan, &turn.state);
		let line = answer(&plan, &turn.state);
		writeln!(out, "{line}").expect("stdout is writable");
		out.flush().expect("stdout is writable");
	}
}
