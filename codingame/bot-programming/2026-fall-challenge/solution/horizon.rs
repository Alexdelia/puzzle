use engine::{Command, MAX_NODES, Map, Point, State, TriangleSet, bit, nodes_of};
use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::time::{Duration, Instant};

const UNREACHABLE: u8 = u8::MAX;
const WIN: f64 = 1e6;
const SLOTS: usize = 4;
const MAX_DEPTH: usize = 12;
const SPAWN_USE: u8 = 0;
const ATTACK_CHOICES: usize = 4;
const ATTACK_COST_PER_CORNER: i32 = 1;

type Counts = [i32; MAX_NODES];

struct Params {
	max_army: i32,
	triangle_weight: f64,
	progress_weight: f64,
	pressure_weight: f64,
	exposure_weight: f64,
	unit_weight: f64,
	locked_weight: f64,
	opponent_triangle_bonus: f64,
	goal_sum_weight: f64,
	prediction_share: f64,
	depth: usize,
	discount: f64,
	first_turn_share: f64,
	stack_move_share: f64,
	corner_move_share: f64,
	use_move_share: f64,
	threat_weight: f64,
	surround_weight: f64,
	strangle_weight: f64,
	house_weight: f64,
	factory_weight: f64,
	greedy_opponent: bool,
	greedy_passes: usize,
	pure_greedy: bool,
	pure_default: bool,
	opening_turns: usize,
	opening_depth: usize,
	opening_discount: f64,
	temperature: f64,
	opening_temperature: f64,
	opening_economy: bool,
	iteration_cap: usize,
	turn_budget: Duration,
	first_budget: Duration,
}

fn tuned(name: &str, default: f64) -> f64 {
	std::env::var(format!("HORIZON_{name}"))
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
			locked_weight: tuned("LOCKED_WEIGHT", 3.0),
			opponent_triangle_bonus: tuned("OPPONENT_TRIANGLE_BONUS", 0.5),
			goal_sum_weight: tuned("GOAL_SUM_WEIGHT", 0.2),
			prediction_share: tuned("PREDICTION_SHARE", 0.75),
			depth: (tuned("DEPTH", 3.0) as usize).clamp(1, MAX_DEPTH),
			discount: tuned("DISCOUNT", 0.8),
			first_turn_share: tuned("FIRST_TURN_SHARE", 0.5),
			stack_move_share: tuned("STACK_MOVE_SHARE", 0.3),
			corner_move_share: tuned("CORNER_MOVE_SHARE", 0.1),
			use_move_share: tuned("USE_MOVE_SHARE", 0.05),
			threat_weight: tuned("THREAT_WEIGHT", 0.0),
			surround_weight: tuned("SURROUND_WEIGHT", 0.0),
			strangle_weight: tuned("STRANGLE_WEIGHT", 0.0),
			house_weight: tuned("HOUSE_WEIGHT", 0.0),
			factory_weight: tuned("FACTORY_WEIGHT", 0.0),
			greedy_opponent: tuned("GREEDY_OPPONENT", 0.0) != 0.0,
			greedy_passes: tuned("GREEDY_PASSES", 2.0) as usize,
			pure_greedy: tuned("PURE_GREEDY", 0.0) != 0.0,
			pure_default: tuned("PURE_DEFAULT", 0.0) != 0.0,
			opening_turns: tuned("OPENING_TURNS", 0.0) as usize,
			opening_depth: (tuned("OPENING_DEPTH", 8.0) as usize).clamp(1, MAX_DEPTH),
			opening_discount: tuned("OPENING_DISCOUNT", 1.0),
			temperature: tuned("TEMPERATURE", 0.0),
			opening_temperature: tuned("OPENING_TEMPERATURE", 0.0),
			opening_economy: tuned("OPENING_ECONOMY", 0.0) != 0.0,
			iteration_cap: tuned("ITERATION_CAP", 1e12) as usize,
			turn_budget: Duration::from_secs_f64(tuned("TURN_MS", 35.0) / 1000.0),
			first_budget: Duration::from_secs_f64(tuned("FIRST_MS", 400.0) / 1000.0),
		}
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
		let houses = |player: usize| f64::from(owners[player].count_ones());
		value += params.house_weight * (houses(me) - houses(opponent));
		if params.factory_weight != 0.0 {
			let regions = territory(state);
			let factories = |player: usize| {
				(state.live & (state.owner[player] | state.can_capture[player]))
					.iter()
					.filter(|&t| self.map.corners(t) & !regions[player] == 0)
					.count() as f64
			};
			value += params.factory_weight * (factories(me) - factories(opponent));
		}
		value + params.pressure_weight * encirclement(state, me)
			- params.exposure_weight * encirclement(state, opponent)
			- params.threat_weight * attack_threat(self.map, state, me)
			- params.surround_weight * surround_threat(state, me)
			+ params.strangle_weight * surround_threat(state, opponent)
	}
}

fn attack_threat(map: &Map, state: &State, victim: usize) -> f64 {
	let attacker = 1 - victim;
	let staffed = state.occupied_by(attacker);
	let reach = (state.live & state.owner[attacker])
		.iter()
		.filter(|&t| map.corners(t) & !staffed == 0)
		.fold(0u64, |set, t| set | attack_targets(map, state, t));
	nodes_of(reach & state.occupied_by(victim))
		.map(|node| f64::from((state.units[node][victim] - ATTACK_COST_PER_CORNER * 3).max(0)))
		.sum()
}

fn surround_threat(state: &State, victim: usize) -> f64 {
	let attacker = 1 - victim;
	let mobile = |node: usize| {
		nodes_of(state.adjacency[node] | bit(node))
			.map(|n| state.units[n][attacker])
			.sum::<i32>()
	};
	nodes_of(state.occupied_by(victim))
		.filter(|&node| state.adjacency[node] != 0)
		.filter(|&node| {
			let mut needed = 0;
			for n in nodes_of(state.adjacency[node]) {
				let deficit = state.units[n][victim] - state.units[n][attacker] + 1;
				if deficit <= 0 {
					continue;
				}
				if mobile(n) - state.units[n][attacker] < deficit {
					return false;
				}
				needed += deficit;
			}
			let around = nodes_of(state.adjacency[node])
				.fold(0u64, |set, n| set | state.adjacency[n] | bit(n));
			let pool: i32 = nodes_of(around & !bit(node))
				.map(|n| state.units[n][attacker])
				.sum();
			needed <= pool
		})
		.map(|node| f64::from(state.units[node][victim]))
		.sum()
}

fn territory(state: &State) -> [u64; 2] {
	let around = |set: u64| nodes_of(set).fold(0u64, |acc, n| acc | state.adjacency[n]);
	let mut regions = [state.occupied_by(0), state.occupied_by(1)];
	let contested = regions[0] & regions[1];
	regions = regions.map(|r| r & !contested);
	let mut claimed = regions[0] | regions[1] | contested;
	let mut frontiers = regions;
	while frontiers[0] | frontiers[1] != 0 {
		let reached = frontiers.map(|f| around(f) & !claimed);
		let tied = reached[0] & reached[1];
		frontiers = reached.map(|r| r & !tied);
		regions[0] |= frontiers[0];
		regions[1] |= frontiers[1];
		claimed |= reached[0] | reached[1];
	}
	regions
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

struct Rng(u64);

impl Rng {
	fn next(&mut self) -> u64 {
		self.0 ^= self.0 << 13;
		self.0 ^= self.0 >> 7;
		self.0 ^= self.0 << 17;
		self.0
	}

	fn below(&mut self, bound: usize) -> usize {
		(self.next() % bound as u64) as usize
	}

	fn chance(&mut self, probability: f64) -> bool {
		const MANTISSA: f64 = (1u64 << 53) as f64;
		((self.next() >> 11) as f64 / MANTISSA) < probability
	}

	fn pick(&mut self, set: u64) -> Option<usize> {
		let count = set.count_ones() as usize;
		(count > 0).then(|| {
			nodes_of(set)
				.nth(self.below(count))
				.expect("index below count")
		})
	}
}

#[derive(Clone)]
struct Genome {
	depth: usize,
	triangle_count: usize,
	destinations: Vec<[u8; SLOTS]>,
	corners: Vec<u8>,
	uses: Vec<u8>,
}

impl Genome {
	fn standing(depth: usize, triangle_count: usize) -> Self {
		Genome {
			depth,
			triangle_count,
			destinations: (0..depth * MAX_NODES)
				.map(|index| [(index % MAX_NODES) as u8; SLOTS])
				.collect(),
			corners: vec![0; depth * triangle_count],
			uses: vec![SPAWN_USE; depth * triangle_count],
		}
	}

	fn slots(&self, turn: usize, node: usize) -> &[u8; SLOTS] {
		&self.destinations[turn * MAX_NODES + node]
	}

	fn slots_mut(&mut self, turn: usize, node: usize) -> &mut [u8; SLOTS] {
		&mut self.destinations[turn * MAX_NODES + node]
	}

	fn corner(&self, turn: usize, triangle: usize) -> usize {
		usize::from(self.corners[turn * self.triangle_count + triangle])
	}

	fn set_corner(&mut self, turn: usize, triangle: usize, corner: usize) {
		self.corners[turn * self.triangle_count + triangle] = corner as u8;
	}

	fn use_of(&self, turn: usize, triangle: usize) -> u8 {
		self.uses[turn * self.triangle_count + triangle]
	}

	fn set_use(&mut self, turn: usize, triangle: usize, choice: u8) {
		self.uses[turn * self.triangle_count + triangle] = choice;
	}

	fn resized(&self, depth: usize) -> Self {
		let mut next = Genome::standing(depth, self.triangle_count);
		for turn in 0..depth.min(self.depth) {
			for node in 0..MAX_NODES {
				*next.slots_mut(turn, node) = *self.slots(turn, node);
			}
			for triangle in 0..self.triangle_count {
				next.set_corner(turn, triangle, self.corner(turn, triangle));
				next.set_use(turn, triangle, self.use_of(turn, triangle));
			}
		}
		next
	}

	fn shifted(&self) -> Self {
		let mut next = Genome::standing(self.depth, self.triangle_count);
		for turn in 1..self.depth {
			for node in 0..MAX_NODES {
				*next.slots_mut(turn - 1, node) = *self.slots(turn, node);
			}
			for triangle in 0..self.triangle_count {
				next.set_corner(turn - 1, triangle, self.corner(turn, triangle));
				next.set_use(turn - 1, triangle, self.use_of(turn, triangle));
			}
		}
		next
	}

	fn record(&mut self, map: &Map, turn: usize, commands: &[Command]) {
		let mut filled = [0usize; MAX_NODES];
		for &command in commands {
			match command {
				Command::Move { from, to, amount } => {
					for _ in 0..amount {
						let slot = filled[from];
						if slot < SLOTS {
							self.slots_mut(turn, from)[slot] = to as u8;
						}
						filled[from] += 1;
					}
				}
				Command::Spawn { triangle, node } => {
					let corner = map.triangles[triangle]
						.iter()
						.position(|&c| c == node)
						.expect("spawns land on a corner");
					self.set_corner(turn, triangle, corner);
				}
				_ => {}
			}
		}
	}
}

fn usable(state: &State, player: usize) -> TriangleSet {
	state.live & state.owner[player] & state.initial_owner[player]
}

fn attack_targets(map: &Map, state: &State, triangle: usize) -> u64 {
	nodes_of(map.corners(triangle)).fold(0, |set, corner| set | state.adjacency[corner])
}

fn decode(
	map: &Map,
	genome: &Genome,
	turn: usize,
	state: &State,
	me: usize,
	out: &mut Vec<Command>,
) {
	out.clear();
	let opponent_occupied = state.occupied_by(1 - me);
	for triangle in usable(state, me).iter() {
		let choice = genome.use_of(turn, triangle);
		let targets = attack_targets(map, state, triangle) & opponent_occupied;
		if choice != SPAWN_USE && targets != 0 {
			let index = usize::from(choice - 1) % targets.count_ones() as usize;
			let target = nodes_of(targets).nth(index).expect("index below count");
			out.push(Command::Attack { triangle, target });
			continue;
		}
		let node = map.triangles[triangle][genome.corner(turn, triangle) % 3];
		out.push(Command::Spawn { triangle, node });
	}
	for node in 0..map.len() {
		let count = state.units[node][me];
		if count <= 0 {
			continue;
		}
		let slots = genome.slots(turn, node);
		let mut amounts = [0i32; SLOTS];
		for unit in 0..count as usize {
			amounts[unit.min(SLOTS - 1)] += 1;
		}
		for (slot, &amount) in amounts.iter().enumerate() {
			let to = usize::from(slots[slot]);
			if amount == 0 || to == node || state.adjacency[node] & bit(to) == 0 {
				continue;
			}
			match out.iter_mut().find(
				|c| matches!(c, Command::Move { from, to: t, .. } if *from == node && *t == to),
			) {
				Some(Command::Move { amount: total, .. }) => *total += amount,
				_ => out.push(Command::Move {
					from: node,
					to,
					amount,
				}),
			}
		}
	}
}

#[derive(Clone)]
struct Rollout {
	states: [State; MAX_DEPTH],
	before: [f64; MAX_DEPTH],
	ended: usize,
	total: f64,
}

type Trace = Vec<Rollout>;

struct Scenario {
	theirs: Vec<Vec<Command>>,
	share: f64,
}

struct Search<'a> {
	map: &'a Map,
	params: &'a Params,
	policy: Policy<'a>,
	state: State,
	me: usize,
	scenarios: Vec<Scenario>,
	default_plan: Genome,
	greedy_plan: Genome,
	plan: Planning,
}

#[derive(Clone, Copy)]
struct Planning {
	depth: usize,
	passive: bool,
	discount: f64,
	temperature: f64,
	economy: bool,
}

impl<'a> Search<'a> {
	fn new(map: &'a Map, params: &'a Params, state: State, me: usize, plan: Planning) -> Self {
		let policy = Policy { map, params };
		let routes = Routes::new(&state, map.len());
		let depth = plan.depth;
		let opponent = 1 - me;
		let mut default_plan = Genome::standing(depth, map.triangles.len());
		let mut greedy_plan = Genome::standing(depth, map.triangles.len());
		let mut predicted = Vec::with_capacity(depth);
		let mut rolling = state;
		let mut greedy_rolling = state;
		for turn in 0..depth {
			let mine = policy.default_commands(&routes, &rolling, me);
			let theirs = if plan.passive {
				Vec::new()
			} else if params.greedy_opponent {
				greedy_commands(map, &rolling, opponent, &mine, params.greedy_passes)
			} else {
				policy.default_commands(&routes, &rolling, opponent)
			};
			default_plan.record(map, turn, &mine);
			step_seated(map, &mut rolling, me, &mine, &theirs);
			let greedy = greedy_commands(map, &greedy_rolling, me, &theirs, params.greedy_passes);
			greedy_plan.record(map, turn, &greedy);
			step_seated(map, &mut greedy_rolling, me, &greedy, &theirs);
			predicted.push(theirs);
		}
		let standing = predicted
			.iter()
			.map(|turn| {
				turn.iter()
					.copied()
					.filter(|c| matches!(c, Command::Spawn { .. }))
					.collect()
			})
			.collect();
		let prediction_share = if plan.passive {
			1.0
		} else {
			params.prediction_share
		};
		let scenarios = vec![
			Scenario {
				theirs: predicted,
				share: prediction_share,
			},
			Scenario {
				theirs: standing,
				share: 1.0 - prediction_share,
			},
		]
		.into_iter()
		.filter(|s| s.share > 0.0)
		.collect();
		Search {
			map,
			params,
			policy,
			state,
			me,
			scenarios,
			default_plan,
			greedy_plan,
			plan,
		}
	}

	fn empty_trace(&self) -> Trace {
		let rollout = Rollout {
			states: [self.state; MAX_DEPTH],
			before: [0.0; MAX_DEPTH],
			ended: 0,
			total: 0.0,
		};
		vec![rollout; self.scenarios.len()]
	}

	fn value(
		&self,
		genome: &Genome,
		from: usize,
		base: &Trace,
		out: &mut Trace,
		scratch: &mut Vec<Command>,
	) -> f64 {
		let mut total = 0.0;
		for (index, scenario) in self.scenarios.iter().enumerate() {
			let known = &base[index];
			let rollout = &mut out[index];
			if from > 0 && from >= known.ended {
				rollout.clone_from(known);
				total += scenario.share * known.total;
				continue;
			}
			rollout.states[..from].copy_from_slice(&known.states[..from]);
			rollout.before[..from].copy_from_slice(&known.before[..from]);
			let (mut state, mut sum) = if from == 0 {
				(self.state, 0.0)
			} else {
				(known.states[from], known.before[from])
			};
			let mut weight = self.plan.discount.powi(from as i32);
			rollout.ended = genome.depth;
			for turn in from..genome.depth {
				rollout.states[turn] = state;
				rollout.before[turn] = sum;
				decode(self.map, genome, turn, &state, self.me, scratch);
				step_seated(
					self.map,
					&mut state,
					self.me,
					scratch,
					&scenario.theirs[turn],
				);
				let value = if self.plan.economy {
					economy(&state, self.me)
				} else {
					self.policy.evaluate(&state, self.me)
				};
				sum += weight * value;
				if value.abs() >= WIN {
					rollout.ended = turn + 1;
					break;
				}
				weight *= self.plan.discount;
			}
			rollout.total = sum;
			total += scenario.share * sum;
		}
		total
	}

	fn mutate(&self, genome: &mut Genome, trace: &Trace, rng: &mut Rng) -> Option<usize> {
		let params = self.params;
		let reached = trace[0].ended.max(1);
		let turn = if reached == 1 || rng.chance(params.first_turn_share) {
			0
		} else {
			rng.below(reached)
		};
		let state = &trace[0].states[turn];
		if rng.chance(params.corner_move_share + params.use_move_share) {
			let owned: Vec<usize> = usable(state, self.me).iter().collect();
			if owned.is_empty() {
				return None;
			}
			let triangle = owned[rng.below(owned.len())];
			if rng
				.chance(params.use_move_share / (params.corner_move_share + params.use_move_share))
			{
				let choice = if genome.use_of(turn, triangle) == SPAWN_USE {
					1 + rng.below(ATTACK_CHOICES) as u8
				} else {
					SPAWN_USE
				};
				genome.set_use(turn, triangle, choice);
			} else {
				let corner = (genome.corner(turn, triangle) + 1 + rng.below(2)) % 3;
				genome.set_corner(turn, triangle, corner);
			}
			return Some(turn);
		}
		let node = rng.pick(state.occupied_by(self.me))?;
		let to = rng.pick(state.adjacency[node] | bit(node))?;
		let slots = genome.slots_mut(turn, node);
		if rng.chance(params.stack_move_share) {
			*slots = [to as u8; SLOTS];
		} else {
			let used = (state.units[node][self.me] as usize).min(SLOTS);
			slots[rng.below(used)] = to as u8;
		}
		Some(turn)
	}

	fn improve(&self, start: &[Genome], deadline: Instant, rng: &mut Rng) -> (Genome, usize) {
		let mut scratch = Vec::new();
		let mut trace = self.empty_trace();
		let mut candidate_trace = self.empty_trace();
		let mut current = start[0].clone();
		let mut current_value = f64::NEG_INFINITY;
		for genome in start {
			let value = self.value(genome, 0, &trace, &mut candidate_trace, &mut scratch);
			if value > current_value {
				current_value = value;
				current = genome.clone();
				std::mem::swap(&mut trace, &mut candidate_trace);
			}
		}
		let mut best = current.clone();
		let mut best_value = current_value;
		let began = Instant::now();
		let span = deadline
			.saturating_duration_since(began)
			.as_secs_f64()
			.max(1e-9);
		let mut temperature = self.plan.temperature;
		let mut iterations = 0;
		while iterations < self.params.iteration_cap {
			if iterations % 16 == 0 {
				let now = Instant::now();
				if now >= deadline {
					break;
				}
				let left = deadline.duration_since(now).as_secs_f64() / span;
				temperature = self.plan.temperature * left;
			}
			iterations += 1;
			let mut candidate = current.clone();
			let Some(from) = self.mutate(&mut candidate, &trace, rng) else {
				continue;
			};
			let value = self.value(&candidate, from, &trace, &mut candidate_trace, &mut scratch);
			let accepted = value >= current_value
				|| (temperature > 0.0 && rng.chance(((value - current_value) / temperature).exp()));
			if accepted {
				current_value = value;
				current = candidate;
				std::mem::swap(&mut trace, &mut candidate_trace);
				if current_value > best_value {
					best_value = current_value;
					best = current.clone();
				}
			}
		}
		(best, iterations)
	}
}

fn economy(state: &State, me: usize) -> f64 {
	const UNIT_VALUE: f64 = 11.0;
	const TRIANGLE_VALUE: f64 = 10.0;
	UNIT_VALUE * f64::from(state.units_of(me)) + TRIANGLE_VALUE * state.triangles_of(me) as f64
}

fn greedy_commands(
	map: &Map,
	state: &State,
	player: usize,
	theirs: &[Command],
	passes: usize,
) -> Vec<Command> {
	const TRIANGLE_SHARE: f64 = 10.0;
	let triangles: Vec<usize> = usable(state, player).iter().collect();
	let mut corners = vec![0usize; triangles.len()];
	let origins: Vec<usize> = (0..map.len())
		.flat_map(|n| std::iter::repeat_n(n, state.units[n][player].max(0) as usize))
		.collect();
	let mut destinations = origins.clone();
	let build = |corners: &[usize], destinations: &[usize]| -> Vec<Command> {
		let spawns = triangles
			.iter()
			.zip(corners)
			.map(|(&triangle, &corner)| Command::Spawn {
				triangle,
				node: map.triangles[triangle][corner],
			});
		let moves = origins
			.iter()
			.zip(destinations)
			.filter(|(from, to)| from != to)
			.map(|(&from, &to)| Command::Move {
				from,
				to,
				amount: 1,
			});
		spawns.chain(moves).collect()
	};
	let score = |commands: &[Command]| {
		let mut next = *state;
		step_seated(map, &mut next, player, commands, theirs);
		let owned = (next.owner[player] & next.live).len() as f64;
		let houses = f64::from(next.node_owners()[player].count_ones());
		TRIANGLE_SHARE * owned + houses
	};
	let mut best = score(&build(&corners, &destinations));
	for _ in 0..passes {
		let mut improved = false;
		for unit in 0..origins.len() {
			let origin = origins[unit];
			for candidate in nodes_of(state.adjacency[origin] | bit(origin)) {
				let current = destinations[unit];
				if candidate == current {
					continue;
				}
				destinations[unit] = candidate;
				let value = score(&build(&corners, &destinations));
				if value > best {
					best = value;
					improved = true;
				} else {
					destinations[unit] = current;
				}
			}
		}
		for index in 0..triangles.len() {
			for corner in 0..3 {
				let current = corners[index];
				if corner == current {
					continue;
				}
				corners[index] = corner;
				let value = score(&build(&corners, &destinations));
				if value > best {
					best = value;
					improved = true;
				} else {
					corners[index] = current;
				}
			}
		}
		if !improved {
			break;
		}
	}
	build(&corners, &destinations)
}

fn step_seated(map: &Map, state: &mut State, me: usize, mine: &[Command], theirs: &[Command]) {
	let commands: [&[Command]; 2] = if me == 0 {
		[mine, theirs]
	} else {
		[theirs, mine]
	};
	state.step(map, commands);
}

fn text(map: &Map, command: Command) -> String {
	let ordered = |triangle: usize, first: usize| {
		let corners = map.triangles[triangle];
		let rest: Vec<usize> = corners.into_iter().filter(|&c| c != first).collect();
		let [second, third] = [rest[0], rest[1]];
		format!("{first} {second} {third}")
	};
	match command {
		Command::Move { from, to, amount } => format!("MOVE {from} {to} {amount}"),
		Command::Spawn { triangle, node } => {
			let corners = ordered(triangle, node);
			format!("SPAWN {corners}")
		}
		Command::Attack { triangle, target } => {
			let [a, b, c] = map.triangles[triangle];
			format!("ATTACK {a} {b} {c} {target}")
		}
		Command::AddPath { triangle, from, to } => {
			let corners = ordered(triangle, from);
			format!("ADD_PATH {corners} {to}")
		}
		Command::RemovePath { triangle, from, to } => {
			let corners = ordered(triangle, from);
			format!("REMOVE_PATH {corners} {to}")
		}
	}
}

fn answer(map: &Map, commands: &[Command], note: &str) -> String {
	let mut moves: BTreeMap<(usize, usize), i32> = BTreeMap::new();
	let mut parts = Vec::new();
	for &command in commands {
		match command {
			Command::Move { from, to, amount } => *moves.entry((from, to)).or_default() += amount,
			other => parts.push(text(map, other)),
		}
	}
	parts.extend(
		moves
			.into_iter()
			.map(|((from, to), amount)| text(map, Command::Move { from, to, amount })),
	);
	parts.push(format!("MSG {note}"));
	parts.join(";")
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
	let mut previous: Option<Genome> = None;
	let mut turn = 0usize;
	let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
	loop {
		let (state, side, started) = read_state(&mut input, &map, me);
		let budget = if me.is_none() {
			params.first_budget
		} else {
			params.turn_budget
		};
		me = Some(side);
		let opening = turn < params.opening_turns;
		let plan = if opening {
			Planning {
				depth: params.opening_depth.saturating_sub(turn).max(params.depth),
				passive: true,
				discount: params.opening_discount,
				temperature: params.opening_temperature,
				economy: params.opening_economy,
			}
		} else {
			Planning {
				depth: params.depth,
				passive: false,
				discount: params.discount,
				temperature: params.temperature,
				economy: false,
			}
		};
		turn += 1;
		let search = Search::new(&map, &params, state, side, plan);
		let mut starts = vec![search.default_plan.clone(), search.greedy_plan.clone()];
		starts.extend(previous.as_ref().map(|p| p.shifted().resized(plan.depth)));
		let (best, iterations) = if params.pure_greedy {
			(search.greedy_plan.clone(), 0)
		} else if params.pure_default {
			(search.default_plan.clone(), 0)
		} else {
			search.improve(&starts, started + budget, &mut rng)
		};
		let mut commands = Vec::new();
		decode(&map, &best, 0, &state, side, &mut commands);
		writeln!(
			out,
			"{}",
			answer(&map, &commands, &format!("i{iterations}"))
		)
		.expect("stdout is writable");
		out.flush().expect("stdout is writable");
		previous = Some(best);
	}
}
