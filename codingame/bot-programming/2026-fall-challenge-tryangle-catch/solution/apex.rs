use engine::{Command, MAX_NODES, MAX_TURNS, Map, Point, State, bit, nodes_of};
use std::io::{BufRead, Write};
use std::time::{Duration, Instant};

const LAST_SCORED_TURN: i32 = MAX_TURNS - 1;
const UNREACHABLE: u8 = u8::MAX;
const WIN: f64 = 1e6;
const TOURNAMENT_SIZE: usize = 3;
const NEARBY_CHOICES: usize = 3;
const ALWAYS: u8 = u8::MAX;
const REACTIVE: u8 = UNREACHABLE;

struct Params {
	horizon: usize,
	value_turns: f64,
	unit_weight: f64,
	threat_weight: f64,
	progress_weight: f64,
	opponent_share: f64,
	population: usize,
	replies: usize,
	worst_case_share: f64,
	use_turns: usize,
	first_turn_budget: Duration,
	turn_budget: Duration,
}

fn tuned(name: &str, default: f64) -> f64 {
	std::env::var(format!("APEX_{name}"))
		.ok()
		.and_then(|value| value.parse().ok())
		.unwrap_or(default)
}

impl Params {
	fn load() -> Self {
		Params {
			horizon: tuned("HORIZON", 8.0) as usize,
			value_turns: tuned("VALUE_TURNS", 30.0),
			unit_weight: tuned("UNIT_WEIGHT", 4.0),
			threat_weight: tuned("THREAT_WEIGHT", 2.0),
			progress_weight: tuned("PROGRESS_WEIGHT", 0.5),
			opponent_share: tuned("OPPONENT_SHARE", 0.35),
			population: tuned("POPULATION", 12.0) as usize,
			replies: tuned("REPLIES", 2.0) as usize,
			worst_case_share: tuned("WORST_CASE_SHARE", 0.5),
			use_turns: tuned("USE_TURNS", 3.0) as usize,
			first_turn_budget: Duration::from_millis(tuned("FIRST_TURN_MS", 800.0) as u64),
			turn_budget: Duration::from_millis(tuned("TURN_MS", 40.0) as u64),
		}
	}
}

struct Rng(u64);

impl Rng {
	fn below(&mut self, bound: usize) -> usize {
		self.0 ^= self.0 << 13;
		self.0 ^= self.0 >> 7;
		self.0 ^= self.0 << 17;
		(self.0 % bound.max(1) as u64) as usize
	}

	fn pick<T: Copy>(&mut self, items: &[T]) -> Option<T> {
		(!items.is_empty()).then(|| items[self.below(items.len())])
	}

	fn chance(&mut self, one_in: usize) -> bool {
		self.below(one_in) == 0
	}
}

struct Routes {
	distance: Vec<[u8; MAX_NODES]>,
	hop: Vec<[u8; MAX_NODES]>,
}

impl Routes {
	fn new(state: &State, count: usize) -> Self {
		let mut distance = vec![[UNREACHABLE; MAX_NODES]; count];
		for (source, row) in distance.iter_mut().enumerate() {
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
		let hop = (0..count)
			.map(|from| {
				std::array::from_fn(|to| {
					if to >= count || from == to || distance[from][to] == UNREACHABLE {
						return from as u8;
					}
					nodes_of(state.adjacency[from])
						.find(|&n| distance[n][to] + 1 == distance[from][to])
						.unwrap_or(from) as u8
				})
			})
			.collect();
		Routes { distance, hop }
	}

	fn between(&self, a: u8, b: u8) -> u8 {
		self.distance[usize::from(a)][usize::from(b)]
	}
}

#[derive(Clone, Copy, Debug)]
struct Unit {
	at: u8,
	goals: [u8; 2],
	stage: u8,
	next: u8,
}

impl Unit {
	fn holding(at: u8, goal: u8) -> Self {
		Unit {
			at,
			goals: [goal; 2],
			stage: 0,
			next: at,
		}
	}

	fn target(&self) -> u8 {
		self.goals[usize::from(self.stage)]
	}

	fn carried_over(&self) -> [u8; 2] {
		if self.stage == 0 {
			self.goals
		} else {
			[self.goals[1]; 2]
		}
	}
}

#[derive(Clone, Copy, Debug)]
struct Use {
	turn: u8,
	command: Command,
	goal: u8,
}

#[derive(Clone, Debug, Default)]
struct Plan {
	goals: Vec<[u8; 2]>,
	uses: Vec<Use>,
	spawn_until: u8,
}

#[derive(Clone)]
struct Scored {
	plan: Plan,
	fitness: f64,
}

struct Assignment {
	cost: u32,
	triangle: usize,
	chosen: Vec<(usize, u8)>,
}

struct Reaction {
	downhill: [u8; MAX_NODES],
	anchors: u64,
	anchored: u64,
}

impl Reaction {
	fn step(&mut self, at: u8) -> u8 {
		let node = usize::from(at);
		if self.anchors & !self.anchored & bit(node) != 0 {
			self.anchored |= bit(node);
			return at;
		}
		self.downhill[node]
	}
}

struct Memory {
	squads: [Vec<Unit>; 2],
	uses: [Vec<Use>; 2],
	spawn_until: [u8; 2],
}

struct Turn<'a> {
	map: &'a Map,
	root: State,
	routes: Routes,
	params: &'a Params,
	steps: usize,
	future: f64,
	units: [Vec<u8>; 2],
	targets: [Vec<usize>; 2],
	nearest_target: [Vec<u8>; 2],
	guarded: [Vec<usize>; 2],
	usable: [Vec<usize>; 2],
}

impl<'a> Turn<'a> {
	fn new(map: &'a Map, root: State, turn: i32, params: &'a Params) -> Self {
		let routes = Routes::new(&root, map.len());
		let steps = params.horizon.min((MAX_TURNS - turn).max(0) as usize);
		let horizon_end = turn + steps as i32 - 1;
		let future = f64::from((LAST_SCORED_TURN - horizon_end).max(0)).min(params.value_turns);
		let units = [0, 1].map(|player| {
			(0..map.len())
				.flat_map(|n| std::iter::repeat_n(n as u8, root.units[n][player].max(0) as usize))
				.collect()
		});
		let live: Vec<usize> = root.live.iter().collect();
		let targets: [Vec<usize>; 2] = [0, 1].map(|player| {
			live.iter()
				.copied()
				.filter(|&t| root.can_capture[player].has(t) && !root.owner[player].has(t))
				.collect()
		});
		let nearest_target = [0, 1].map(|player: usize| {
			let corners: Vec<u8> = targets[player]
				.iter()
				.flat_map(|&t| map.triangles[t].map(|c| c as u8))
				.collect();
			(0..map.len() as u8)
				.map(|from| {
					corners
						.iter()
						.copied()
						.min_by_key(|&corner| routes.between(from, corner))
						.unwrap_or(from)
				})
				.collect()
		});
		let guarded = [0, 1].map(|player| {
			live.iter()
				.copied()
				.filter(|&t| root.owner[player].has(t))
				.collect()
		});
		let usable = [0, 1].map(|player| {
			live.iter()
				.copied()
				.filter(|&t| root.owner[player].has(t) && root.initial_owner[player].has(t))
				.collect()
		});
		Turn {
			map,
			root,
			routes,
			params,
			steps,
			future,
			units,
			targets,
			nearest_target,
			guarded,
			usable,
		}
	}

	fn corners(&self, triangle: usize) -> [u8; 3] {
		self.map.triangles[triangle].map(|c| c as u8)
	}

	fn squad(&self, player: usize, plan: &Plan) -> Vec<Unit> {
		self.units[player]
			.iter()
			.zip(&plan.goals)
			.map(|(&at, &goals)| Unit {
				at,
				goals,
				stage: 0,
				next: at,
			})
			.collect()
	}

	fn reaction(&self, state: &State, player: usize) -> Reaction {
		let targets = state.live & state.can_capture[player] & !state.owner[player];
		let anchors = targets.iter().fold(0, |set, t| set | self.map.corners(t));
		let wanted = anchors & !state.node_owners()[player];
		let mut downhill: [u8; MAX_NODES] = std::array::from_fn(|n| n as u8);
		let mut seen = wanted;
		let mut frontier = wanted;
		while frontier != 0 {
			let around = nodes_of(frontier).fold(0, |set, n| set | state.adjacency[n]);
			let next = around & !seen;
			for node in nodes_of(next) {
				downhill[node] = (state.adjacency[node] & frontier).trailing_zeros() as u8;
			}
			seen |= next;
			frontier = next;
		}
		Reaction {
			downhill,
			anchors,
			anchored: 0,
		}
	}

	fn spawn_corner(&self, player: usize, triangle: usize) -> usize {
		self.map.triangles[triangle]
			.into_iter()
			.min_by_key(|&c| {
				let target = self.nearest_target[player][c];
				self.routes.between(c as u8, target)
			})
			.expect("a triangle has corners")
	}

	fn orders(
		&self,
		state: &State,
		player: usize,
		squad: &mut [Unit],
		plan: &Plan,
		turn: usize,
		out: &mut Vec<Command>,
	) {
		let mut flows: Vec<(u8, u8)> = Vec::with_capacity(squad.len());
		let mut reaction: Option<Reaction> = None;
		for unit in squad.iter_mut() {
			if unit.stage == 0 && unit.at == unit.goals[0] {
				unit.stage = 1;
			}
			let target = unit.target();
			unit.next = if target == REACTIVE {
				reaction
					.get_or_insert_with(|| self.reaction(state, player))
					.step(unit.at)
			} else {
				self.routes.hop[usize::from(unit.at)][usize::from(target)]
			};
			if unit.next != unit.at {
				flows.push((unit.at, unit.next));
			}
		}
		flows.sort_unstable();
		for group in flows.chunk_by(|a, b| a == b) {
			let (from, to) = group[0];
			out.push(Command::Move {
				from: usize::from(from),
				to: usize::from(to),
				amount: group.len() as i32,
			});
		}
		out.extend(
			plan.uses
				.iter()
				.filter(|u| usize::from(u.turn) == turn)
				.map(|u| u.command),
		);
		if turn < usize::from(plan.spawn_until) {
			let usable = state.owner[player] & state.initial_owner[player] & state.live;
			out.extend(usable.iter().map(|triangle| Command::Spawn {
				triangle,
				node: self.spawn_corner(player, triangle),
			}));
		}
	}

	fn reconcile(
		&self,
		state: &State,
		player: usize,
		squad: &mut Vec<Unit>,
		plan: &Plan,
		turn: usize,
	) {
		let mut kept = [0; MAX_NODES];
		squad.retain_mut(|unit| {
			unit.at = unit.next;
			let node = usize::from(unit.at);
			let alive = kept[node] < state.units[node][player];
			kept[node] += i32::from(alive);
			alive
		});
		for (node, &survivors) in kept.iter().enumerate().take(self.map.len()) {
			let extra = state.units[node][player] - survivors;
			if extra <= 0 {
				continue;
			}
			let goal = plan
				.uses
				.iter()
				.filter(|u| usize::from(u.turn) == turn)
				.filter(|u| matches!(u.command, Command::Spawn { node: n, .. } if n == node))
				.map(|u| u.goal)
				.next_back()
				.unwrap_or(REACTIVE);
			for _ in 0..extra {
				squad.push(Unit::holding(node as u8, goal));
			}
		}
	}

	fn rollout(&self, plans: [&Plan; 2]) -> (State, [bool; 2]) {
		let mut state = self.root;
		let mut squads = [0, 1].map(|p| self.squad(p, plans[p]));
		let mut commands: [Vec<Command>; 2] = Default::default();
		for turn in 0..self.steps {
			for player in 0..2 {
				commands[player].clear();
				self.orders(
					&state,
					player,
					&mut squads[player],
					plans[player],
					turn,
					&mut commands[player],
				);
			}
			state.step(self.map, [&commands[0], &commands[1]]);
			for player in 0..2 {
				self.reconcile(&state, player, &mut squads[player], plans[player], turn);
			}
			let eliminated = state.eliminated();
			if eliminated.contains(&true) {
				return (state, eliminated);
			}
		}
		(state, [false; 2])
	}

	fn evaluate(&self, state: &State, eliminated: [bool; 2], player: usize) -> f64 {
		let opponent = 1 - player;
		match (eliminated[player], eliminated[opponent]) {
			(true, false) => return -WIN,
			(false, true) => return WIN,
			(true, true) => return 0.0,
			_ => {}
		}
		let params = self.params;
		let owners = state.node_owners();
		let held = |corners: u64, p: usize| f64::from((corners & owners[p]).count_ones());
		let mut positional = 0.0;
		for t in state.live.iter() {
			let corners = self.map.corners(t);
			let chased = |p: usize| state.can_capture[p].has(t);
			if state.owner[player].has(t) {
				if chased(opponent) {
					positional -= params.threat_weight * held(corners, opponent).powi(2);
				}
			} else if state.owner[opponent].has(t) {
				if chased(player) {
					positional += params.threat_weight * held(corners, player).powi(2);
				}
			} else {
				if chased(player) {
					positional += params.progress_weight * held(corners, player).powi(2);
				}
				if chased(opponent) {
					positional -= params.progress_weight * held(corners, opponent).powi(2);
				}
			}
		}
		let score = f64::from(state.score[player] - state.score[opponent]);
		let triangles = state.triangles_of(player) as f64 - state.triangles_of(opponent) as f64;
		let units = f64::from(state.units_of(player) - state.units_of(opponent));
		score + self.future * triangles + params.unit_weight * units + positional
	}

	fn fitness(&self, player: usize, plan: &Plan, replies: &[Plan]) -> f64 {
		let mut worst = f64::INFINITY;
		let mut total = 0.0;
		for reply in replies {
			let plans = if player == 0 {
				[plan, reply]
			} else {
				[reply, plan]
			};
			let (state, eliminated) = self.rollout(plans);
			let value = self.evaluate(&state, eliminated, player);
			worst = worst.min(value);
			total += value;
		}
		let share = self.params.worst_case_share;
		share * worst + (1.0 - share) * total / replies.len().max(1) as f64
	}

	fn nearest_target_corner(&self, player: usize, from: u8) -> u8 {
		self.nearest_target[player][usize::from(from)]
	}

	fn assign(
		&self,
		player: usize,
		positions: &[u8],
		taken: &mut Vec<usize>,
		held: &mut u64,
	) -> Vec<u8> {
		let mut goals = vec![UNREACHABLE; positions.len()];
		let mut free: Vec<usize> = (0..positions.len()).collect();
		loop {
			let mut best: Option<Assignment> = None;
			for &t in &self.targets[player] {
				if taken.contains(&t) {
					continue;
				}
				let mut pool = free.clone();
				let mut chosen = Vec::with_capacity(3);
				for corner in self.corners(t) {
					if let Some(slot) = pool.iter().position(|&u| positions[u] == corner) {
						chosen.push((pool.swap_remove(slot), corner));
					}
				}
				let mut cost = 0;
				let mut feasible = true;
				for corner in self.corners(t) {
					let anchored = chosen.iter().any(|&(_, c)| c == corner);
					if anchored || *held & bit(usize::from(corner)) != 0 {
						continue;
					}
					let nearest = pool
						.iter()
						.enumerate()
						.min_by_key(|&(_, &u)| self.routes.between(positions[u], corner));
					let Some((slot, &unit)) = nearest else {
						feasible = false;
						break;
					};
					let distance = u32::from(self.routes.between(positions[unit], corner));
					if distance == u32::from(UNREACHABLE) {
						feasible = false;
						break;
					}
					cost = cost.max(distance);
					chosen.push((unit, corner));
					pool.swap_remove(slot);
				}
				if feasible && best.as_ref().is_none_or(|b| cost < b.cost) {
					best = Some(Assignment {
						cost,
						triangle: t,
						chosen,
					});
				}
			}
			let Some(Assignment {
				triangle: t,
				chosen,
				..
			}) = best
			else {
				break;
			};
			taken.push(t);
			for corner in self.corners(t) {
				*held |= bit(usize::from(corner));
			}
			for (unit, corner) in chosen {
				goals[unit] = corner;
				free.retain(|&u| u != unit);
			}
			if free.is_empty() {
				break;
			}
		}
		for unit in free {
			goals[unit] = self.nearest_target_corner(player, positions[unit]);
		}
		goals
	}

	fn greedy(&self, player: usize, spawning: bool) -> Plan {
		let positions = &self.units[player];
		let mut taken = Vec::new();
		let mut held = self.root.node_owners()[player];
		let first = self.assign(player, positions, &mut taken, &mut held);
		let second = self.assign(player, &first, &mut taken, &mut held);
		let goals = first.into_iter().zip(second).map(|(a, b)| [a, b]).collect();
		Plan {
			goals,
			uses: Vec::new(),
			spawn_until: if spawning { ALWAYS } else { 0 },
		}
	}

	fn holding(&self, player: usize) -> Plan {
		Plan {
			goals: self.units[player].iter().map(|&at| [at; 2]).collect(),
			uses: Vec::new(),
			spawn_until: 0,
		}
	}

	fn remembered(&self, player: usize, memory: &Memory) -> Plan {
		let mut pool = memory.squads[player].clone();
		let goals = self.units[player]
			.iter()
			.map(|&at| match pool.iter().position(|u| u.at == at) {
				Some(index) => pool.swap_remove(index).carried_over(),
				None => [self.nearest_target_corner(player, at); 2],
			})
			.collect();
		let uses = memory.uses[player]
			.iter()
			.filter(|u| u.turn > 0)
			.map(|&u| Use {
				turn: u.turn - 1,
				..u
			})
			.collect();
		let spawn_until = match memory.spawn_until[player] {
			ALWAYS => ALWAYS,
			turns => turns.saturating_sub(1),
		};
		Plan {
			goals,
			uses,
			spawn_until,
		}
	}

	fn interesting_corner(&self, player: usize, from: u8, rng: &mut Rng) -> u8 {
		let guarding = rng.chance(4) && !self.guarded[player].is_empty();
		let pool = if guarding {
			&self.guarded[player]
		} else {
			&self.targets[player]
		};
		let closest = (0..NEARBY_CHOICES)
			.filter_map(|_| rng.pick(pool))
			.min_by_key(|&t| {
				self.corners(t)
					.map(|c| self.routes.between(from, c))
					.into_iter()
					.min()
			});
		match closest {
			Some(t) => self.corners(t)[rng.below(3)],
			None => from,
		}
	}

	fn retarget(&self, player: usize, plan: &mut Plan, rng: &mut Rng) {
		let Some(unit) = (!plan.goals.is_empty()).then(|| rng.below(plan.goals.len())) else {
			return;
		};
		let stage = rng.below(2);
		let from = match plan.goals[unit][0] {
			first if stage == 1 && first != REACTIVE => first,
			_ => self.units[player][unit],
		};
		plan.goals[unit][stage] = match rng.below(7) {
			0 => from,
			6 => REACTIVE,
			1 => rng
				.pick(&nodes_of(self.root.adjacency[usize::from(from)]).collect::<Vec<_>>())
				.map_or(from, |n| n as u8),
			_ => self.interesting_corner(player, from, rng),
		};
	}

	fn rally(&self, player: usize, plan: &mut Plan, rng: &mut Rng) {
		let Some(t) = rng.pick(&self.targets[player]) else {
			return;
		};
		let positions = &self.units[player];
		let mut free: Vec<usize> = (0..positions.len()).collect();
		for corner in self.corners(t) {
			let nearest = free
				.iter()
				.enumerate()
				.min_by_key(|&(_, &u)| self.routes.between(positions[u], corner));
			let Some((slot, &unit)) = nearest else {
				return;
			};
			plan.goals[unit][0] = corner;
			free.swap_remove(slot);
		}
	}

	fn random_use(&self, player: usize, rng: &mut Rng) -> Option<Use> {
		let late = rng.chance(5);
		let triangle = if late {
			rng.pick(&self.targets[player])?
		} else {
			rng.pick(&self.usable[player])?
		};
		let window = self.params.use_turns.min(self.steps).max(1);
		let turn = if late {
			1 + rng.below(window)
		} else {
			rng.below(window)
		} as u8;
		let corners = self.map.triangles[triangle];
		let from = corners[rng.below(3)];
		let state = &self.root;
		let command = match rng.below(5) {
			0 | 1 => Command::Spawn {
				triangle,
				node: from,
			},
			2 => {
				let around = corners.iter().fold(0, |set, &c| set | state.adjacency[c]);
				let enemies: Vec<usize> = nodes_of(around)
					.filter(|&n| state.units[n][1 - player] > 0)
					.collect();
				Command::Attack {
					triangle,
					target: rng.pick(&enemies)?,
				}
			}
			3 => {
				let reachable: Vec<usize> = (0..self.map.len())
					.filter(|&to| to != from && !state.connected(from, to))
					.filter(|&to| self.map.can_connect(&state.edges, from, to))
					.collect();
				Command::AddPath {
					triangle,
					from,
					to: rng.pick(&reachable)?,
				}
			}
			_ => {
				let outward: Vec<usize> = nodes_of(state.adjacency[from])
					.filter(|n| !corners.contains(n))
					.collect();
				Command::RemovePath {
					triangle,
					from,
					to: rng.pick(&outward)?,
				}
			}
		};
		Some(Use {
			turn,
			command,
			goal: self.interesting_corner(player, from as u8, rng),
		})
	}

	fn mutate(&self, player: usize, plan: &mut Plan, rng: &mut Rng) {
		for _ in 0..1 + rng.below(3) {
			match rng.below(13) {
				0..=4 => self.retarget(player, plan, rng),
				5 => self.rally(player, plan, rng),
				6 if !plan.goals.is_empty() => {
					let unit = rng.below(plan.goals.len());
					plan.goals[unit][1] = plan.goals[unit][0];
				}
				7..=8 => plan.uses.extend(self.random_use(player, rng)),
				9 if !plan.uses.is_empty() => {
					let index = rng.below(plan.uses.len());
					plan.uses.swap_remove(index);
				}
				10 if !plan.uses.is_empty() => {
					let index = rng.below(plan.uses.len());
					let shifted = plan.uses[index].turn as usize + 1;
					plan.uses[index].turn = (shifted % self.steps.max(1)) as u8;
				}
				11 => {
					plan.spawn_until = match rng.below(3) {
						0 => 0,
						1 => ALWAYS,
						_ => 1 + rng.below(self.steps.max(1)) as u8,
					}
				}
				_ => plan.uses.clear(),
			}
		}
	}

	fn evolve(
		&self,
		player: usize,
		seeds: Vec<Plan>,
		replies: &[Plan],
		deadline: Instant,
		rng: &mut Rng,
		evaluations: &mut usize,
	) -> Vec<Scored> {
		let mut population: Vec<Scored> = seeds
			.into_iter()
			.map(|plan| {
				*evaluations += 1;
				let fitness = self.fitness(player, &plan, replies);
				Scored { plan, fitness }
			})
			.collect();
		while Instant::now() < deadline && !population.is_empty() {
			let parent = (0..TOURNAMENT_SIZE)
				.map(|_| rng.below(population.len()))
				.max_by(|&a, &b| population[a].fitness.total_cmp(&population[b].fitness))
				.expect("the tournament has entrants");
			let mut child = population[parent].plan.clone();
			self.mutate(player, &mut child, rng);
			*evaluations += 1;
			let fitness = self.fitness(player, &child, replies);
			if population.len() < self.params.population {
				population.push(Scored {
					plan: child,
					fitness,
				});
				continue;
			}
			let worst = (0..population.len())
				.min_by(|&a, &b| population[a].fitness.total_cmp(&population[b].fitness))
				.expect("the population is not empty");
			if fitness > population[worst].fitness {
				population[worst] = Scored {
					plan: child,
					fitness,
				};
			}
		}
		population.sort_by(|a, b| b.fitness.total_cmp(&a.fitness));
		population
	}

	fn reactive(&self, player: usize) -> Plan {
		Plan {
			goals: vec![[REACTIVE; 2]; self.units[player].len()],
			uses: Vec::new(),
			spawn_until: ALWAYS,
		}
	}

	fn seeds(&self, player: usize, memory: Option<&Memory>) -> Vec<Plan> {
		let mut seeds = vec![
			self.reactive(player),
			self.greedy(player, true),
			self.greedy(player, false),
			self.holding(player),
		];
		if let Some(memory) = memory {
			seeds.push(self.remembered(player, memory));
		}
		seeds
	}

	fn decide(
		&self,
		me: usize,
		memory: Option<&Memory>,
		started: Instant,
		budget: Duration,
		rng: &mut Rng,
	) -> (Plan, Plan) {
		let opponent = 1 - me;
		let mut evaluations = 0;
		let mut my_guesses = vec![self.greedy(me, true)];
		if let Some(memory) = memory {
			my_guesses.push(self.remembered(me, memory));
		}
		let opponent_deadline = started + budget.mul_f64(self.params.opponent_share);
		let theirs = self.evolve(
			opponent,
			self.seeds(opponent, memory),
			&my_guesses,
			opponent_deadline,
			rng,
			&mut evaluations,
		);
		let replies: Vec<Plan> = theirs
			.iter()
			.take(self.params.replies.max(1))
			.map(|s| s.plan.clone())
			.collect();
		let mine = self.evolve(
			me,
			self.seeds(me, memory),
			&replies,
			started + budget,
			rng,
			&mut evaluations,
		);
		let root = &self.root;
		eprintln!(
			"units {:?} triangles {:?} score {:?} evaluations {evaluations} best {:.1} predicted reply {:.1}",
			[me, opponent].map(|p| root.units_of(p)),
			[me, opponent].map(|p| root.triangles_of(p)),
			[me, opponent].map(|p| root.score[p]),
			mine[0].fitness,
			theirs[0].fitness
		);
		(mine[0].plan.clone(), replies[0].clone())
	}

	fn first_turn(&self, plans: [&Plan; 2]) -> ([Vec<Command>; 2], Memory) {
		let mut squads = [0, 1].map(|p| self.squad(p, plans[p]));
		let mut commands: [Vec<Command>; 2] = Default::default();
		for player in 0..2 {
			self.orders(
				&self.root,
				player,
				&mut squads[player],
				plans[player],
				0,
				&mut commands[player],
			);
		}
		let mut state = self.root;
		state.step(self.map, [&commands[0], &commands[1]]);
		for player in 0..2 {
			self.reconcile(&state, player, &mut squads[player], plans[player], 0);
		}
		let uses = plans.map(|plan| plan.uses.clone());
		let spawn_until = plans.map(|plan| plan.spawn_until);
		(
			commands,
			Memory {
				squads,
				uses,
				spawn_until,
			},
		)
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
		self.lines
			.next()
			.expect("the referee sends a line")
			.expect("stdin is readable")
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
	let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
	let mut me = None;
	let mut memory: Option<Memory> = None;
	for turn in 1.. {
		let (state, side) = read_state(&mut input, &map, me);
		let started = Instant::now();
		me = Some(side);
		let budget = if turn == 1 {
			params.first_turn_budget
		} else {
			params.turn_budget
		};
		let planner = Turn::new(&map, state, turn, &params);
		let (mine, theirs) = planner.decide(side, memory.as_ref(), started, budget, &mut rng);
		let plans = if side == 0 {
			[&mine, &theirs]
		} else {
			[&theirs, &mine]
		};
		let (commands, next) = planner.first_turn(plans);
		memory = Some(next);
		let line: Vec<String> = commands[side].iter().map(|&c| text(&map, c)).collect();
		let line = if line.is_empty() {
			"WAIT".to_string()
		} else {
			line.join(";")
		};
		writeln!(out, "{line}").expect("stdout is writable");
		out.flush().expect("stdout is writable");
	}
}
