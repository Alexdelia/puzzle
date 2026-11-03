use engine::{Command, MAX_NODES, Map, Point, State, TriangleSet, bit, nodes_of};
use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::time::{Duration, Instant};

const UNREACHABLE: u8 = u8::MAX;
const WIN: f64 = 1e6;
const IMPROVEMENT_EPSILON: f64 = 1e-9;
const BASE_SEARCH_SHARE: f64 = 0.25;
const COUNTERS_END_SHARE: f64 = 0.55;
const SCREEN_SHARE: f64 = 0.7;
const ROLLOUT_SHARE: f64 = 0.8;
const REPLY_SHARE: f64 = 0.15;

struct Params {
	max_army: i32,
	triangle_weight: f64,
	progress_weight: f64,
	pressure_weight: f64,
	exposure_weight: f64,
	unit_weight: f64,
	score_weight: f64,
	goal_weight: f64,
	idle_weight: f64,
	goal_sum_weight: f64,
	opponent_triangle_bonus: f64,
	prediction_share: f64,
	reply_share: f64,
	reply_rounds: usize,
	predicted_spawns: bool,
	action_margin: f64,
	locked_weight: f64,
	rollout_depth: usize,
	rollout_candidates: usize,
	rollout_floor: f64,
	rollout_margin: f64,
	rollout_holds: bool,
	turn_budget: Duration,
}

fn tuned(name: &str, default: f64) -> f64 {
	std::env::var(format!("ZENITH_{name}"))
		.ok()
		.and_then(|value| value.parse().ok())
		.unwrap_or(default)
}

impl Params {
	fn load() -> Self {
		Params {
			max_army: tuned("MAX_ARMY", 1000.0) as i32,
			triangle_weight: tuned("TRIANGLE_WEIGHT", 10.0),
			progress_weight: tuned("PROGRESS_WEIGHT", 0.2),
			pressure_weight: tuned("PRESSURE_WEIGHT", 1.0),
			exposure_weight: tuned("EXPOSURE_WEIGHT", 6.0),
			unit_weight: tuned("UNIT_WEIGHT", 11.0),
			score_weight: tuned("SCORE_WEIGHT", 0.0),
			goal_weight: tuned("GOAL_WEIGHT", 0.5),
			idle_weight: tuned("IDLE_WEIGHT", 0.3),
			goal_sum_weight: tuned("GOAL_SUM_WEIGHT", 0.2),
			opponent_triangle_bonus: tuned("OPPONENT_TRIANGLE_BONUS", 0.5),
			prediction_share: tuned("PREDICTION_SHARE", 0.75),
			reply_share: tuned("REPLY_SHARE", 1.0),
			reply_rounds: tuned("REPLY_ROUNDS", 2.0) as usize,
			predicted_spawns: tuned("PREDICTED_SPAWNS", 0.0) != 0.0,
			action_margin: tuned("ACTION_MARGIN", 0.0),
			locked_weight: tuned("LOCKED_WEIGHT", 3.0),
			rollout_depth: tuned("ROLLOUT_DEPTH", 0.0) as usize,
			rollout_candidates: tuned("ROLLOUT_CANDIDATES", 12.0) as usize,
			rollout_floor: tuned("ROLLOUT_FLOOR", -40.0),
			rollout_margin: tuned("ROLLOUT_MARGIN", 0.0),
			rollout_holds: tuned("ROLLOUT_HOLDS", 1.0) != 0.0,
			turn_budget: Duration::from_secs_f64(tuned("TURN_MS", 40.0) / 1000.0),
		}
	}
}

type Counts = [i32; MAX_NODES];

struct Pacer {
	deadline: Instant,
	last: Instant,
	longest: Duration,
}

impl Pacer {
	const SAFETY_FACTOR: u32 = 2;

	fn until(deadline: Instant) -> Self {
		Pacer {
			deadline,
			last: Instant::now(),
			longest: Duration::ZERO,
		}
	}

	fn has_time(&mut self) -> bool {
		let now = Instant::now();
		self.longest = self.longest.max(now - self.last);
		self.last = now;
		now + self.longest * Self::SAFETY_FACTOR < self.deadline
	}
}

struct Routes {
	distance: [[u8; MAX_NODES]; MAX_NODES],
}

impl Routes {
	fn new(state: &State, count: usize) -> Self {
		let mut distance = [[UNREACHABLE; MAX_NODES]; MAX_NODES];
		for (source, row) in distance.iter_mut().enumerate().take(count) {
			let mut seen = bit(source);
			let mut frontier = seen;
			let mut depth = 0;
			while frontier != 0 {
				for node in nodes_of(frontier) {
					row[node] = depth;
				}
				let around = nodes_of(frontier).fold(0, |set, n| set | state.adjacency[n]);
				frontier = around & !seen;
				seen |= frontier;
				depth += 1;
			}
		}
		Routes { distance }
	}

	fn between(&self, a: usize, b: usize) -> u8 {
		self.distance[a][b]
	}

	fn first_step(&self, state: &State, from: usize, goal: usize) -> usize {
		let toward = &self.distance[goal];
		nodes_of(state.adjacency[from])
			.find(|&next| toward[next] < toward[from])
			.unwrap_or(from)
	}
}

fn capturable(state: &State, player: usize) -> TriangleSet {
	state.live & !state.owner[player] & state.can_capture[player]
}

fn triangle_of(command: Command) -> Option<usize> {
	match command {
		Command::Move { .. } => None,
		Command::Spawn { triangle, .. }
		| Command::Attack { triangle, .. }
		| Command::AddPath { triangle, .. }
		| Command::RemovePath { triangle, .. } => Some(triangle),
	}
}

fn costs_units(command: Command) -> bool {
	matches!(
		command,
		Command::Attack { .. } | Command::AddPath { .. } | Command::RemovePath { .. }
	)
}

fn counts_of(state: &State, player: usize) -> Counts {
	std::array::from_fn(|node| state.units[node][player])
}

fn nonempty(counts: &Counts) -> u64 {
	counts
		.iter()
		.enumerate()
		.filter(|(_, count)| **count > 0)
		.fold(0, |set, (node, _)| set | bit(node))
}

struct Assignment {
	goals: Vec<(usize, usize)>,
	idle: Counts,
}

struct Policy<'a> {
	map: &'a Map,
	params: &'a Params,
}

impl Policy<'_> {
	fn assign_goals(
		&self,
		routes: &Routes,
		state: &State,
		player: usize,
		mut free: Counts,
	) -> Assignment {
		let opponent = 1 - player;
		let mut will_own = state.node_owners()[player];
		let mut remaining: Vec<usize> = capturable(state, player).iter().collect();
		let mut goals = Vec::new();
		while nonempty(&free) != 0 && !remaining.is_empty() {
			let mut best: Option<(f64, usize, Vec<usize>)> = None;
			'triangles: for (index, &t) in remaining.iter().enumerate() {
				let corners = self.map.triangles[t];
				let mut available = free;
				for &corner in &corners {
					if will_own & bit(corner) != 0 && available[corner] > 0 {
						available[corner] -= 1;
					}
				}
				let mut taken = Vec::with_capacity(3);
				let (mut cost_sum, mut cost_max) = (0u32, 0u32);
				for &corner in &corners {
					if will_own & bit(corner) != 0 {
						continue;
					}
					let nearest =
						nodes_of(nonempty(&available)).min_by_key(|&n| routes.between(n, corner));
					let Some(node) = nearest else {
						continue 'triangles;
					};
					available[node] -= 1;
					taken.push(node);
					let distance = u32::from(routes.between(node, corner));
					cost_sum += distance;
					cost_max = cost_max.max(distance);
				}
				let bonus = if state.owner[opponent].has(t) {
					self.params.opponent_triangle_bonus
				} else {
					0.0
				};
				let cost =
					f64::from(cost_max) + self.params.goal_sum_weight * f64::from(cost_sum) - bonus;
				if best.as_ref().is_none_or(|(lowest, ..)| cost < *lowest) {
					best = Some((cost, index, taken));
				}
			}
			let Some((_, index, mut pool)) = best else {
				break;
			};
			let corners = self.map.triangles[remaining.remove(index)];
			let mut needed: Vec<usize> = corners
				.into_iter()
				.filter(|&c| will_own & bit(c) == 0)
				.collect();
			needed.sort_by_key(|&c| pool.iter().map(|&n| routes.between(n, c)).min());
			for corner in needed {
				let closest = (0..pool.len())
					.min_by_key(|&i| routes.between(pool[i], corner))
					.expect("one unit was taken per needed corner");
				let node = pool.remove(closest);
				goals.push((node, corner));
				free[node] -= 1;
				will_own |= bit(corner);
			}
		}
		Assignment { goals, idle: free }
	}

	fn target_distance(&self, routes: &Routes, state: &State, player: usize) -> [u8; MAX_NODES] {
		let targets = capturable(state, player)
			.iter()
			.fold(0, |set, t| set | self.map.corners(t));
		if targets == 0 {
			return [0; MAX_NODES];
		}
		std::array::from_fn(|node| {
			nodes_of(targets)
				.map(|t| routes.between(node, t))
				.min()
				.unwrap_or(0)
		})
	}

	fn plan_spawns(&self, routes: &Routes, state: &State, player: usize) -> Vec<Command> {
		let army = state.units_of(player);
		let owned = state.live & state.owner[player];
		if army >= self.params.max_army || owned.is_empty() || capturable(state, player).is_empty()
		{
			return Vec::new();
		}
		let frontier = self.target_distance(routes, state, player);
		let mut ranked: Vec<(u8, Command)> = owned
			.iter()
			.map(|triangle| {
				let node = *self.map.triangles[triangle]
					.iter()
					.min_by_key(|&&c| frontier[c])
					.expect("a triangle has corners");
				(frontier[node], Command::Spawn { triangle, node })
			})
			.collect();
		ranked.sort_by_key(|&(distance, _)| distance);
		ranked.truncate((self.params.max_army - army).max(0) as usize);
		ranked.into_iter().map(|(_, command)| command).collect()
	}

	fn goal_moves(&self, routes: &Routes, state: &State, player: usize) -> Vec<Command> {
		let assignment = self.assign_goals(routes, state, player, counts_of(state, player));
		let mut moves: Vec<Command> = assignment
			.goals
			.iter()
			.filter_map(|&(origin, goal)| {
				let next = routes.first_step(state, origin, goal);
				(next != origin).then_some(Command::Move {
					from: origin,
					to: next,
					amount: 1,
				})
			})
			.collect();
		let idle_distance = self.target_distance(routes, state, player);
		for origin in nodes_of(nonempty(&assignment.idle)) {
			let next = nodes_of(state.adjacency[origin])
				.find(|&n| idle_distance[n] < idle_distance[origin]);
			if let Some(next) = next {
				moves.push(Command::Move {
					from: origin,
					to: next,
					amount: assignment.idle[origin],
				});
			}
		}
		moves
	}

	fn default_commands(&self, routes: &Routes, state: &State, player: usize) -> Vec<Command> {
		let mut commands = self.plan_spawns(routes, state, player);
		commands.extend(self.goal_moves(routes, state, player));
		commands
	}

	fn evaluate(&self, state: &State, me: usize) -> f64 {
		let opponent = 1 - me;
		let eliminated = state.eliminated();
		if eliminated[me] || eliminated[opponent] {
			let margin = if eliminated[me] { -WIN } else { WIN };
			if eliminated[me] != eliminated[opponent] {
				return margin;
			}
		}
		let params = self.params;
		let owners = state.node_owners();
		let owned_corners =
			|t: usize, player: usize| (self.map.corners(t) & owners[player]).count_ones() as f64;
		let mut value = 0.0;
		for t in state.live.iter() {
			let mine = state.owner[me].has(t);
			if mine {
				value += params.triangle_weight;
			} else {
				if state.owner[opponent].has(t) {
					value -= params.triangle_weight;
				}
				if state.can_capture[me].has(t) {
					let progress = owned_corners(t, me);
					value += params.progress_weight * progress * progress;
				} else {
					value -= params.locked_weight;
				}
			}
		}
		value += params.unit_weight * f64::from(state.units_of(me) - state.units_of(opponent));
		value + params.pressure_weight * encirclement(state, me)
			- params.exposure_weight * encirclement(state, opponent)
	}
}

fn encirclement(state: &State, attacker: usize) -> f64 {
	let victim = 1 - attacker;
	let ahead = (0..MAX_NODES)
		.filter(|&n| state.units[n][attacker] > state.units[n][victim])
		.fold(0u64, |set, n| set | bit(n));
	(0..MAX_NODES)
		.filter(|&n| state.units[n][victim] > 0 && state.adjacency[n] != 0)
		.map(|n| {
			let around = state.adjacency[n];
			let share = f64::from((around & ahead).count_ones()) / f64::from(around.count_ones());
			f64::from(state.units[n][victim]) * share * share
		})
		.sum()
}

#[derive(Clone)]
struct Plan {
	orders: Vec<Command>,
	units: Vec<usize>,
	destinations: Vec<usize>,
}

impl Plan {
	fn commands(&self) -> Vec<Command> {
		let mut commands = self.orders.clone();
		for (&from, &to) in self.units.iter().zip(&self.destinations) {
			if from != to {
				commands.push(Command::Move {
					from,
					to,
					amount: 1,
				});
			}
		}
		commands
	}
}

#[derive(Clone, Copy)]
enum Change {
	Hold(usize),
	Use(Command),
}

impl Change {
	fn triangle(self) -> usize {
		match self {
			Change::Hold(triangle) => triangle,
			Change::Use(command) => triangle_of(command).expect("changes use triangles"),
		}
	}

	fn order(self) -> Option<Command> {
		match self {
			Change::Hold(_) => None,
			Change::Use(command) => Some(command),
		}
	}

	fn clashes(self, other: Change) -> bool {
		let same_target = matches!(
			(self.order(), other.order()),
			(Some(Command::Attack { target: x, .. }), Some(Command::Attack { target: y, .. })) if x == y
		);
		self.triangle() == other.triangle() || same_target
	}
}

fn expand(counts: &Counts) -> Vec<usize> {
	counts
		.iter()
		.enumerate()
		.flat_map(|(node, &count)| std::iter::repeat_n(node, count.max(0) as usize))
		.collect()
}

type Scenario = (Vec<Command>, f64);

struct Turn<'a> {
	policy: Policy<'a>,
	state: State,
	me: usize,
	routes: Routes,
	scenarios: Vec<Scenario>,
	naive: Vec<Scenario>,
	replies: Vec<Vec<Command>>,
	opponent_spawns: Vec<Command>,
	predicted: Vec<Command>,
}

impl<'a> Turn<'a> {
	fn new(map: &'a Map, params: &'a Params, state: State, me: usize, started: Instant) -> Self {
		let policy = Policy { map, params };
		let routes = Routes::new(&state, map.len());
		let opponent = 1 - me;
		let moves = policy.goal_moves(&routes, &state, opponent);
		let spawns = policy.plan_spawns(&routes, &state, opponent);
		let mut stepping = moves.clone();
		let mut standing = Vec::new();
		if params.predicted_spawns {
			stepping.extend(spawns.iter().copied());
			standing.extend(spawns.iter().copied());
		}
		let mut predicted = moves;
		predicted.extend(spawns.iter().copied());
		let naive = vec![
			(standing, 1.0 - params.prediction_share),
			(stepping, params.prediction_share),
		];
		let mut turn = Turn {
			policy,
			state,
			me,
			routes,
			scenarios: naive.clone(),
			naive,
			replies: Vec::new(),
			opponent_spawns: spawns,
			predicted,
		};
		if params.reply_share > 0.0 {
			let mut stepping = turn.policy.goal_moves(&turn.routes, &turn.state, me);
			let mut standing = Vec::new();
			if params.predicted_spawns {
				let spawns = turn.policy.plan_spawns(&turn.routes, &turn.state, me);
				stepping.extend(spawns.iter().copied());
				standing.extend(spawns);
			}
			let mine = [
				(standing, 1.0 - params.prediction_share),
				(stepping, params.prediction_share),
			];
			turn.add_reply(&mine, started + params.turn_budget.mul_f64(REPLY_SHARE));
		}
		turn
	}

	fn add_reply(&mut self, mine: &[Scenario], deadline: Instant) {
		let opponent = 1 - self.me;
		let spawns = self.opponent_spawns.clone();
		let units = self
			.movable_units_of(opponent, &spawns)
			.expect("spawns cost no units");
		let reply = self
			.climb(opponent, mine, spawns, units, deadline)
			.commands();
		self.replies.push(reply);
		let reply_share = self.params().reply_share;
		let each = reply_share / self.replies.len() as f64;
		self.scenarios = self
			.naive
			.iter()
			.map(|(commands, share)| (commands.clone(), share * (1.0 - reply_share)))
			.chain(self.replies.iter().map(|reply| (reply.clone(), each)))
			.collect();
	}

	fn map(&self) -> &Map {
		self.policy.map
	}

	fn params(&self) -> &Params {
		self.policy.params
	}

	fn step(&self, state: &mut State, mine: &[Command], theirs: &[Command]) {
		self.step_as(self.me, state, mine, theirs);
	}

	fn step_as(&self, player: usize, state: &mut State, own: &[Command], other: &[Command]) {
		let mut commands: [&[Command]; 2] = [own, other];
		if player == 1 {
			commands.swap(0, 1);
		}
		state.step(self.map(), commands);
	}

	fn outcome(&self, plan: &Plan) -> f64 {
		self.outcome_as(self.me, &self.scenarios, plan)
	}

	fn outcome_as(&self, player: usize, scenarios: &[Scenario], plan: &Plan) -> f64 {
		let own = plan.commands();
		scenarios
			.iter()
			.filter(|(_, share)| *share > 0.0)
			.map(|(other, share)| {
				let mut next = self.state;
				self.step_as(player, &mut next, &own, other);
				share * self.policy.evaluate(&next, player)
			})
			.sum()
	}

	fn rollout(&self, plan: &Plan) -> f64 {
		let mut state = self.state;
		self.step(&mut state, &plan.commands(), &self.predicted);
		let count = self.map().len();
		let mut rebuilt: Option<Routes> = None;
		for _ in 1..self.params().rollout_depth {
			if state.eliminated().contains(&true) {
				break;
			}
			if state.adjacency != self.state.adjacency {
				rebuilt = Some(Routes::new(&state, count));
			}
			let routes = rebuilt.as_ref().unwrap_or(&self.routes);
			let mine = self.policy.default_commands(routes, &state, self.me);
			let theirs = self.policy.default_commands(routes, &state, 1 - self.me);
			self.step(&mut state, &mine, &theirs);
			rebuilt = None;
		}
		let gained = |player: usize| f64::from(state.score[player] - self.state.score[player]);
		self.policy.evaluate(&state, self.me)
			+ self.params().score_weight * (gained(self.me) - gained(1 - self.me))
	}

	fn search_moves(&self, orders: Vec<Command>, units: Vec<usize>, deadline: Instant) -> Plan {
		self.climb(self.me, &self.scenarios, orders, units, deadline)
	}

	fn climb(
		&self,
		player: usize,
		scenarios: &[Scenario],
		orders: Vec<Command>,
		units: Vec<usize>,
		deadline: Instant,
	) -> Plan {
		let routes = &self.routes;
		let params = self.params();
		let mut free = [0; MAX_NODES];
		for &node in &units {
			free[node] += 1;
		}
		let assignment = self.policy.assign_goals(routes, &self.state, player, free);
		let mut goals: Vec<Option<usize>> = vec![None; units.len()];
		for &(origin, goal) in &assignment.goals {
			let unit = (0..units.len())
				.find(|&u| units[u] == origin && goals[u].is_none())
				.expect("every goal belongs to a unit");
			goals[unit] = Some(goal);
		}
		let idle_distance = self.policy.target_distance(routes, &self.state, player);
		let unit_cost = |unit: usize, node: usize| match goals[unit] {
			Some(goal) => params.goal_weight * f64::from(routes.between(node, goal)),
			None => params.idle_weight * f64::from(idle_distance[node]),
		};
		let destinations: Vec<usize> = units
			.iter()
			.zip(&goals)
			.map(|(&origin, goal)| {
				goal.map_or(origin, |goal| routes.first_step(&self.state, origin, goal))
			})
			.collect();
		let mut plan = Plan {
			orders,
			units,
			destinations,
		};
		let mut goal_total: f64 = (0..plan.units.len())
			.map(|u| unit_cost(u, plan.destinations[u]))
			.sum();
		let mut best = self.outcome_as(player, scenarios, &plan) - goal_total;
		let mut improved = true;
		let mut pacer = Pacer::until(deadline);
		while improved && pacer.has_time() {
			improved = false;
			for unit in 0..plan.units.len() {
				if !pacer.has_time() {
					break;
				}
				let origin = plan.units[unit];
				let candidates =
					std::iter::once(origin).chain(nodes_of(self.state.adjacency[origin]));
				for candidate in candidates {
					let current = plan.destinations[unit];
					if candidate == current {
						continue;
					}
					plan.destinations[unit] = candidate;
					let candidate_total =
						goal_total - unit_cost(unit, current) + unit_cost(unit, candidate);
					let value = self.outcome_as(player, scenarios, &plan) - candidate_total;
					if value > best + IMPROVEMENT_EPSILON {
						best = value;
						goal_total = candidate_total;
						improved = true;
					} else {
						plan.destinations[unit] = current;
					}
				}
			}
		}
		plan
	}

	fn alternatives(&self) -> Vec<Command> {
		let state = &self.state;
		let map = self.map();
		let (me, opponent) = (self.me, 1 - self.me);
		let usable = state.live & state.owner[me] & state.initial_owner[me];
		let linkable: Vec<[usize; 2]> = map
			.pairs
			.iter()
			.copied()
			.filter(|&[a, b]| !state.connected(a, b) && map.can_connect(&state.edges, a, b))
			.collect();
		let mut found = Vec::new();
		for triangle in usable.iter() {
			let corners = map.corners(triangle);
			if nodes_of(corners).any(|c| state.units[c][me] < 1) {
				continue;
			}
			let around = nodes_of(corners).fold(0, |set, c| set | state.adjacency[c]);
			for target in nodes_of(around).filter(|&n| state.units[n][opponent] > 0) {
				found.push(Command::Attack { triangle, target });
			}
			for &[a, b] in &linkable {
				if corners & bit(a) != 0 {
					found.push(Command::AddPath {
						triangle,
						from: a,
						to: b,
					});
				} else if corners & bit(b) != 0 {
					found.push(Command::AddPath {
						triangle,
						from: b,
						to: a,
					});
				}
			}
			for corner in nodes_of(corners) {
				for other in nodes_of(state.adjacency[corner] & !corners) {
					found.push(Command::RemovePath {
						triangle,
						from: corner,
						to: other,
					});
				}
			}
		}
		found
	}

	fn without_payers(&self, plan: &Plan, orders: &[Command]) -> Option<(Vec<usize>, Vec<usize>)> {
		let mut units = plan.units.clone();
		let mut destinations = plan.destinations.clone();
		for &order in orders.iter().filter(|&&o| costs_units(o)) {
			let triangle = triangle_of(order).expect("paid orders use triangles");
			for corner in nodes_of(self.map().corners(triangle)) {
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
		let mut orders: Vec<Command> = base
			.orders
			.iter()
			.copied()
			.filter(|&o| triangle_of(o) != Some(change.triangle()))
			.collect();
		orders.extend(change.order());
		let (units, destinations) = self.without_payers(base, change.order().as_slice())?;
		Some(Plan {
			orders,
			units,
			destinations,
		})
	}

	fn ranked_changes(&self, base: &Plan, started: Instant) -> Vec<(f64, Change)> {
		let params = self.params();
		let at = |share: f64| started + params.turn_budget.mul_f64(share);
		let rolling = params.rollout_depth > 0;
		let mut changes: Vec<Change> = self.alternatives().into_iter().map(Change::Use).collect();
		if rolling && params.rollout_holds {
			changes.extend(
				base.orders
					.iter()
					.filter_map(|&o| triangle_of(o))
					.map(Change::Hold),
			);
		}
		let floor = if rolling {
			params.rollout_floor
		} else {
			params.action_margin
		};
		let base_value = self.outcome(base);
		let mut screened: Vec<(f64, Change, Plan)> = Vec::new();
		let mut pacer = Pacer::until(at(SCREEN_SHARE));
		for change in changes {
			if !pacer.has_time() {
				break;
			}
			let Some(variant) = self.variant(base, change) else {
				continue;
			};
			let gain = self.outcome(&variant) - base_value;
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
		let mut pacer = Pacer::until(at(ROLLOUT_SHARE));
		for (_, change, variant) in &screened {
			if !pacer.has_time() {
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

	fn movable_units(&self, orders: &[Command]) -> Option<Vec<usize>> {
		self.movable_units_of(self.me, orders)
	}

	fn movable_units_of(&self, player: usize, orders: &[Command]) -> Option<Vec<usize>> {
		let mut counts = counts_of(&self.state, player);
		for &order in orders.iter().filter(|&&o| costs_units(o)) {
			let triangle = triangle_of(order).expect("paid orders use triangles");
			for corner in nodes_of(self.map().corners(triangle)) {
				counts[corner] -= 1;
				if counts[corner] < 0 {
					return None;
				}
			}
		}
		Some(expand(&counts))
	}

	fn decide(&mut self, started: Instant) -> Plan {
		let budget = self.params().turn_budget;
		let at = |share: f64| started + budget.mul_f64(share);
		let spawns = self.policy.plan_spawns(&self.routes, &self.state, self.me);
		let units = self.movable_units(&spawns).expect("spawns cost no units");
		let mut base = self.search_moves(spawns.clone(), units.clone(), at(BASE_SEARCH_SHARE));
		let rounds = self.params().reply_rounds.max(1);
		let slice = (COUNTERS_END_SHARE - BASE_SEARCH_SHARE) / (rounds - 1).max(1) as f64;
		for round in 1..rounds {
			let start = BASE_SEARCH_SHARE + slice * (round - 1) as f64;
			self.add_reply(&[(base.commands(), 1.0)], at(start + slice / 2.0));
			base = self.search_moves(spawns.clone(), units.clone(), at(start + slice));
		}
		let params = self.params();
		let gains = self.ranked_changes(&base, started);
		if gains.is_empty() {
			return base;
		}
		let mut chosen: Vec<Change> = Vec::new();
		for (_, change) in gains {
			if chosen.iter().any(|c| c.clashes(change)) {
				continue;
			}
			chosen.push(change);
			let uses: Vec<Command> = chosen.iter().filter_map(|c| c.order()).collect();
			if self.without_payers(&base, &uses).is_none() {
				chosen.pop();
			}
		}
		let mut orders: Vec<Command> = base
			.orders
			.iter()
			.copied()
			.filter(|&o| chosen.iter().all(|c| triangle_of(o) != Some(c.triangle())))
			.collect();
		orders.extend(chosen.iter().filter_map(|c| c.order()));
		let units = self.movable_units(&orders).expect("payers were checked");
		self.search_moves(orders, units, started + params.turn_budget)
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

fn answer(map: &Map, plan: &Plan) -> String {
	let mut moves: BTreeMap<(usize, usize), i32> = BTreeMap::new();
	for (&from, &to) in plan.units.iter().zip(&plan.destinations) {
		if from != to {
			*moves.entry((from, to)).or_default() += 1;
		}
	}
	let commands: Vec<String> = plan
		.orders
		.iter()
		.map(|&order| text(map, order))
		.chain(
			moves
				.into_iter()
				.map(|((from, to), amount)| text(map, Command::Move { from, to, amount })),
		)
		.collect();
	if commands.is_empty() {
		"WAIT".to_string()
	} else {
		commands.join(";")
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

fn read_state<R: BufRead>(
	input: &mut Input<R>,
	map: &Map,
	me: Option<usize>,
) -> (State, usize, Instant) {
	let scores = input.ints();
	let started = Instant::now();
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
	(state, me, started)
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
		let (state, side, started) = read_state(&mut input, &map, me);
		me = Some(side);
		let mut turn = Turn::new(&map, &params, state, side, started);
		let plan = turn.decide(started);
		writeln!(out, "{}", answer(&map, &plan)).expect("stdout is writable");
		out.flush().expect("stdout is writable");
	}
}
