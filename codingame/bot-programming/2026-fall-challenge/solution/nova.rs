use engine::{Command, MAX_NODES, Map, Point, State, bit, nodes_of};
use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::time::Instant;

pub const WEIGHTS: &str = "";

const NODE_FEATURES: usize = 28;
const GLOBAL_FEATURES: usize = 8;
const TRIANGLE_FEATURES: usize = 12;
const MAX_LIVE: usize = 96;
const HOLD: i32 = 0;
const SPAWN_BASE: i32 = 1;
const ATTACK_BASE: i32 = 4;
const IGNORED: i32 = -2;
const NOT_USABLE: i32 = -1;
const NET_VERSION: u8 = 1;
const PACK_BASE: u32 = 0x4E00;

struct Features {
	nodes: Vec<[f32; NODE_FEATURES]>,
	globals: [f32; GLOBAL_FEATURES],
	live: Vec<usize>,
	triangles: Vec<[f32; TRIANGLE_FEATURES]>,
	usable: Vec<bool>,
	attack_targets: Vec<u64>,
}

fn ln1p(value: i32) -> f32 {
	(value.max(0) as f32).ln_1p()
}

fn distances_from(state: &State, sources: u64, count: usize) -> Vec<f32> {
	let mut distance = vec![f32::INFINITY; count];
	let mut seen = sources;
	let mut frontier = sources;
	let mut depth = 0.0;
	while frontier != 0 {
		for n in nodes_of(frontier) {
			distance[n] = depth;
		}
		let next = nodes_of(frontier).fold(0u64, |acc, n| acc | state.adjacency[n]) & !seen;
		seen |= next;
		frontier = next;
		depth += 1.0;
	}
	distance
}

fn surround_threatened(state: &State, victim: usize, node: usize) -> bool {
	let attacker = 1 - victim;
	let around = state.adjacency[node];
	if state.units[node][victim] == 0 || around == 0 {
		return false;
	}
	let mut needed = 0;
	for n in nodes_of(around) {
		let deficit = state.units[n][victim] - state.units[n][attacker] + 1;
		if deficit <= 0 {
			continue;
		}
		let reach: i32 = nodes_of(state.adjacency[n])
			.map(|m| state.units[m][attacker])
			.sum();
		if reach < deficit {
			return false;
		}
		needed += deficit;
	}
	let ring = nodes_of(around).fold(0u64, |set, n| set | state.adjacency[n] | bit(n)) & !bit(node);
	let pool: i32 = nodes_of(ring).map(|n| state.units[n][attacker]).sum();
	needed <= pool
}

fn attack_targets(map: &Map, state: &State, triangle: usize, opponent: usize) -> u64 {
	let around = nodes_of(map.corners(triangle)).fold(0u64, |acc, c| acc | state.adjacency[c]);
	around & state.occupied_by(opponent)
}

fn features(map: &Map, state: &State, me: usize, turn: i32) -> Features {
	let opponent = 1 - me;
	let count = map.len();
	let owners = state.node_owners();
	let advantage = |player: usize| {
		(0..count).fold(0u64, |set, n| {
			set | (u64::from(state.units[n][player] > state.units[n][1 - player]) << n)
		})
	};
	let ahead = [advantage(0), advantage(1)];
	let (min_x, max_x, min_y, max_y) = map.points.iter().fold(
		(i32::MAX, i32::MIN, i32::MAX, i32::MIN),
		|(a, b, c, d), p| (a.min(p.x), b.max(p.x), c.min(p.y), d.max(p.y)),
	);
	let span = |low: i32, high: i32, value: i32| {
		let relative = (value - low) as f32 / (high - low).max(1) as f32;
		if me == 0 { relative } else { 1.0 - relative }
	};
	let near_opponent = distances_from(state, state.occupied_by(opponent), count);
	let near_me = distances_from(state, state.occupied_by(me), count);
	let live: Vec<usize> = state.live.iter().take(MAX_LIVE).collect();
	let usable_by =
		|player: usize, t: usize| state.owner[player].has(t) && state.initial_owner[player].has(t);
	let corners_owned =
		|player: usize, t: usize| (map.corners(t) & owners[player]).count_ones() as f32;
	let mut nodes = vec![[0.0f32; NODE_FEATURES]; count];
	for (n, row) in nodes.iter_mut().enumerate() {
		let [mine, theirs] = [state.units[n][me], state.units[n][opponent]];
		let around = state.adjacency[n];
		let degree = around.count_ones() as f32;
		let share = |set: u64| {
			if around == 0 {
				0.0
			} else {
				(around & set).count_ones() as f32 / degree
			}
		};
		let near = |d: f32| {
			if d.is_finite() {
				(d / 8.0).min(1.5)
			} else {
				1.5
			}
		};
		let mut incident = [0.0f32; 7];
		let mut progress = 0.0f32;
		for &t in &live {
			if map.corners(t) & bit(n) == 0 {
				continue;
			}
			let [owned_me, owned_them] = [state.owner[me].has(t), state.owner[opponent].has(t)];
			incident[0] += f32::from(u8::from(owned_me));
			incident[1] += f32::from(u8::from(owned_them));
			if !owned_me && !owned_them {
				if state.can_capture[me].has(t) {
					incident[2] += 1.0;
					progress = progress.max(corners_owned(me, t) / 3.0);
				} else {
					incident[3] += 1.0;
				}
			}
			incident[4] += f32::from(u8::from(usable_by(me, t)));
			incident[5] += f32::from(u8::from(usable_by(opponent, t)));
		}
		let around_units = |player: usize| {
			nodes_of(around)
				.map(|m| state.units[m][player])
				.sum::<i32>()
		};
		*row = [
			mine as f32 / 4.0,
			theirs as f32 / 4.0,
			f32::from(u8::from(mine > 0)),
			f32::from(u8::from(theirs > 0)),
			ln1p(mine),
			ln1p(theirs),
			f32::from(u8::from(owners[me] & bit(n) != 0)),
			f32::from(u8::from(owners[opponent] & bit(n) != 0)),
			f32::from(u8::from(mine > theirs)),
			f32::from(u8::from(theirs > mine)),
			degree / 6.0,
			share(ahead[me]),
			share(ahead[opponent]),
			f32::from(u8::from(surround_threatened(state, me, n))),
			f32::from(u8::from(surround_threatened(state, opponent, n))),
			span(min_x, max_x, map.points[n].x),
			span(min_y, max_y, map.points[n].y),
			near(near_opponent[n]),
			near(near_me[n]),
			incident[0] / 4.0,
			incident[1] / 4.0,
			incident[2] / 4.0,
			incident[3] / 4.0,
			incident[4] / 4.0,
			progress,
			incident[5] / 4.0,
			ln1p(around_units(opponent)),
			ln1p(around_units(me)),
		];
	}
	let triangles = live
		.iter()
		.map(|&t| {
			let units_on = |player: usize| {
				nodes_of(map.corners(t))
					.map(|c| state.units[c][player])
					.sum::<i32>()
			};
			let [owned_me, owned_them] = [state.owner[me].has(t), state.owner[opponent].has(t)];
			let flag = |value: bool| f32::from(u8::from(value));
			[
				flag(owned_me),
				flag(owned_them),
				flag(!owned_me && !owned_them),
				flag(state.can_capture[me].has(t)),
				flag(state.can_capture[opponent].has(t)),
				flag(usable_by(me, t)),
				flag(usable_by(opponent, t)),
				corners_owned(me, t) / 3.0,
				corners_owned(opponent, t) / 3.0,
				ln1p(units_on(me)) / 3.0,
				ln1p(units_on(opponent)) / 3.0,
				flag(state.initial_owner[me].has(t)),
			]
		})
		.collect();
	let usable = live.iter().map(|&t| usable_by(me, t)).collect();
	let attack = live
		.iter()
		.map(|&t| attack_targets(map, state, t, opponent))
		.collect();
	let globals = [
		turn as f32 / 200.0,
		ln1p(state.units_of(me)) / 4.0,
		ln1p(state.units_of(opponent)) / 4.0,
		state.triangles_of(me) as f32 / 10.0,
		state.triangles_of(opponent) as f32 / 10.0,
		state.score[me] as f32 / 100.0,
		state.score[opponent] as f32 / 100.0,
		count as f32 / 64.0,
	];
	Features {
		nodes,
		globals,
		live,
		triangles,
		usable,
		attack_targets: attack,
	}
}

struct Layer {
	inputs: usize,
	outputs: usize,
	weights: Vec<f32>,
	bias: Vec<f32>,
}

impl Layer {
	fn apply(&self, input: &[f32], output: &mut [f32]) {
		for (o, out) in output.iter_mut().enumerate().take(self.outputs) {
			let row = &self.weights[o * self.inputs..(o + 1) * self.inputs];
			*out = self.bias[o] + row.iter().zip(input).map(|(w, x)| w * x).sum::<f32>();
		}
	}

	fn run(&self, input: &[f32]) -> Vec<f32> {
		let mut output = vec![0.0; self.outputs];
		self.apply(input, &mut output);
		output
	}
}

struct Round {
	own: Layer,
	neighbors: Layer,
	triangle: Layer,
	gather: Layer,
}

struct Net {
	channels: usize,
	embed: Layer,
	rounds: Vec<Round>,
	query: Layer,
	key: Layer,
	destination: Layer,
	stay: Layer,
	triangle: Layer,
	hold: Layer,
	spawn: Layer,
	attack: Layer,
}

struct Reader<'a> {
	bytes: &'a [u8],
	at: usize,
}

impl Reader<'_> {
	fn take(&mut self, count: usize) -> &[u8] {
		let slice = &self.bytes[self.at..self.at + count];
		self.at += count;
		slice
	}

	fn u8(&mut self) -> u8 {
		self.take(1)[0]
	}

	fn u16(&mut self) -> usize {
		let b = self.take(2);
		usize::from(u16::from_le_bytes([b[0], b[1]]))
	}

	fn f32s(&mut self, count: usize) -> Vec<f32> {
		self.take(4 * count)
			.chunks_exact(4)
			.map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
			.collect()
	}

	fn layer(&mut self) -> Layer {
		let outputs = self.u16();
		let inputs = self.u16();
		let quantised: Vec<i8> = self
			.take(outputs * inputs)
			.iter()
			.map(|&b| b as i8)
			.collect();
		let steps = self.f32s(outputs);
		let bias = self.f32s(outputs);
		let weights = quantised
			.iter()
			.enumerate()
			.map(|(i, &q)| f32::from(q) * steps[i / inputs])
			.collect();
		Layer {
			inputs,
			outputs,
			weights,
			bias,
		}
	}
}

fn unpack(text: &str) -> Vec<u8> {
	let mut out = Vec::new();
	let mut bits: u32 = 0;
	let mut have = 0;
	for c in text.chars() {
		bits = (bits << 14) | (c as u32 - PACK_BASE);
		have += 14;
		while have >= 8 {
			have -= 8;
			out.push(((bits >> have) & 0xFF) as u8);
		}
		bits &= (1 << have) - 1;
	}
	out
}

impl Net {
	fn load(bytes: &[u8]) -> Net {
		let mut reader = Reader { bytes, at: 0 };
		let version = reader.u8();
		assert_eq!(version, NET_VERSION, "weights of another net version");
		let channels = usize::from(reader.u8());
		let rounds_count = usize::from(reader.u8());
		let embed = reader.layer();
		let rounds = (0..rounds_count)
			.map(|_| Round {
				own: reader.layer(),
				neighbors: reader.layer(),
				triangle: reader.layer(),
				gather: reader.layer(),
			})
			.collect();
		Net {
			channels,
			embed,
			rounds,
			query: reader.layer(),
			key: reader.layer(),
			destination: reader.layer(),
			stay: reader.layer(),
			triangle: reader.layer(),
			hold: reader.layer(),
			spawn: reader.layer(),
			attack: reader.layer(),
		}
	}

	fn hidden(&self, map: &Map, state: &State, features: &Features) -> Vec<Vec<f32>> {
		let count = map.len();
		let channels = self.channels;
		let mut h: Vec<Vec<f32>> = features
			.nodes
			.iter()
			.map(|row| {
				let input: Vec<f32> = row.iter().chain(&features.globals).copied().collect();
				self.embed
					.run(&input)
					.into_iter()
					.map(|v| v.max(0.0))
					.collect()
			})
			.collect();
		for round in &self.rounds {
			let mut triangle_sum = vec![vec![0.0f32; channels]; count];
			let mut triangle_count = vec![0.0f32; count];
			for (index, &t) in features.live.iter().enumerate() {
				let corners: Vec<usize> = nodes_of(map.corners(t)).collect();
				let mut input = vec![0.0f32; channels + TRIANGLE_FEATURES];
				for &c in &corners {
					for k in 0..channels {
						input[k] += h[c][k] / 3.0;
					}
				}
				input[channels..].copy_from_slice(&features.triangles[index]);
				let message: Vec<f32> = round
					.triangle
					.run(&input)
					.into_iter()
					.map(|v| v.max(0.0))
					.collect();
				for &c in &corners {
					for k in 0..channels {
						triangle_sum[c][k] += message[k];
					}
					triangle_count[c] += 1.0;
				}
			}
			let mut next = Vec::with_capacity(count);
			for n in 0..count {
				let around = state.adjacency[n];
				let degree = around.count_ones() as f32;
				let mut mean = vec![0.0f32; channels];
				if degree > 0.0 {
					for m in nodes_of(around) {
						for k in 0..channels {
							mean[k] += h[m][k] / degree;
						}
					}
				}
				let gathered: Vec<f32> = if triangle_count[n] > 0.0 {
					triangle_sum[n]
						.iter()
						.map(|v| v / triangle_count[n])
						.collect()
				} else {
					vec![0.0; channels]
				};
				let own = round.own.run(&h[n]);
				let from_neighbors = round.neighbors.run(&mean);
				let from_triangles = round.gather.run(&gathered);
				next.push(
					(0..channels)
						.map(|k| {
							(own[k] + from_neighbors[k] + from_triangles[k]).max(0.0) + h[n][k]
						})
						.collect(),
				);
			}
			h = next;
		}
		h
	}
}

struct Decision {
	moves: Vec<Vec<(usize, f32)>>,
	triangle_actions: Vec<Vec<(i32, f32)>>,
}

fn softmax(logits: &mut [(i32, f32)]) {
	let peak = logits
		.iter()
		.map(|&(_, l)| l)
		.fold(f32::NEG_INFINITY, f32::max);
	let total: f32 = logits.iter().map(|&(_, l)| (l - peak).exp()).sum();
	for entry in logits.iter_mut() {
		entry.1 = (entry.1 - peak).exp() / total;
	}
}

fn decide(net: &Net, map: &Map, state: &State, me: usize, features: &Features) -> Decision {
	let count = map.len();
	let h = net.hidden(map, state, features);
	let query: Vec<Vec<f32>> = h.iter().map(|row| net.query.run(row)).collect();
	let key: Vec<Vec<f32>> = h.iter().map(|row| net.key.run(row)).collect();
	let destination: Vec<f32> = h.iter().map(|row| net.destination.run(row)[0]).collect();
	let dot = |a: &[f32], b: &[f32]| a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>();
	let moves = (0..count)
		.map(|v| {
			if state.units[v][me] == 0 {
				return Vec::new();
			}
			let mut options: Vec<(i32, f32)> =
				vec![(v as i32, dot(&query[v], &key[v]) + net.stay.run(&h[v])[0])];
			options.extend(
				nodes_of(state.adjacency[v])
					.map(|w| (w as i32, dot(&query[v], &key[w]) + destination[w])),
			);
			softmax(&mut options);
			options.into_iter().map(|(w, p)| (w as usize, p)).collect()
		})
		.collect();
	let channels = net.channels;
	let triangle_actions = features
		.live
		.iter()
		.enumerate()
		.map(|(index, &t)| {
			if !features.usable[index] {
				return Vec::new();
			}
			let corners = map.triangles[t];
			let mut input = vec![0.0f32; channels + TRIANGLE_FEATURES];
			for &c in &corners {
				for k in 0..channels {
					input[k] += h[c][k] / 3.0;
				}
			}
			input[channels..].copy_from_slice(&features.triangles[index]);
			let z: Vec<f32> = net
				.triangle
				.run(&input)
				.into_iter()
				.map(|v| v.max(0.0))
				.collect();
			let paired = |node: usize| -> Vec<f32> { z.iter().chain(&h[node]).copied().collect() };
			let mut options = vec![(HOLD, net.hold.run(&z)[0])];
			for (k, &c) in corners.iter().enumerate() {
				options.push((SPAWN_BASE + k as i32, net.spawn.run(&paired(c))[0]));
			}
			for target in nodes_of(features.attack_targets[index]) {
				options.push((
					ATTACK_BASE + target as i32,
					net.attack.run(&paired(target))[0],
				));
			}
			softmax(&mut options);
			options
		})
		.collect();
	Decision {
		moves,
		triangle_actions,
	}
}

struct Rng(u64);

impl Rng {
	fn unit(&mut self) -> f32 {
		self.0 ^= self.0 << 13;
		self.0 ^= self.0 >> 7;
		self.0 ^= self.0 << 17;
		(self.0 >> 40) as f32 / (1u64 << 24) as f32
	}

	fn draw<T: Copy>(&mut self, options: &[(T, f32)]) -> T {
		let mut left = self.unit();
		for &(value, p) in options {
			left -= p;
			if left < 0.0 {
				return value;
			}
		}
		options[options.len() - 1].0
	}
}

fn commands(
	map: &Map,
	state: &State,
	me: usize,
	features: &Features,
	decision: &Decision,
	mut rng: Option<&mut Rng>,
) -> Vec<Command> {
	let mut reserved = [0i32; MAX_NODES];
	let mut orders = Vec::new();
	for (index, options) in decision.triangle_actions.iter().enumerate() {
		if options.is_empty() {
			continue;
		}
		let t = features.live[index];
		let corners = map.triangles[t];
		let mut ranked = options.clone();
		ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
		if let Some(rng) = rng.as_deref_mut() {
			let drawn = rng.draw(options);
			let at = ranked.iter().position(|&(a, _)| a == drawn).unwrap_or(0);
			let first = ranked.remove(at);
			ranked.insert(0, first);
		}
		for &(action, _) in &ranked {
			if action == HOLD {
				break;
			}
			if action < ATTACK_BASE {
				let node = corners[(action - SPAWN_BASE) as usize];
				orders.push(Command::Spawn { triangle: t, node });
				break;
			}
			let affordable = corners
				.iter()
				.all(|&c| state.units[c][me] - reserved[c] >= 1);
			if affordable {
				for &c in &corners {
					reserved[c] += 1;
				}
				orders.push(Command::Attack {
					triangle: t,
					target: (action - ATTACK_BASE) as usize,
				});
				break;
			}
		}
	}
	for (v, options) in decision.moves.iter().enumerate() {
		let units = state.units[v][me] - reserved[v];
		if options.is_empty() || units <= 0 {
			continue;
		}
		if let Some(rng) = rng.as_deref_mut() {
			let mut counts: BTreeMap<usize, i32> = BTreeMap::new();
			for _ in 0..units {
				*counts.entry(rng.draw(options)).or_default() += 1;
			}
			for (w, amount) in counts {
				if w != v {
					orders.push(Command::Move {
						from: v,
						to: w,
						amount,
					});
				}
			}
			continue;
		}
		let mut shares: Vec<(usize, i32, f32)> = options
			.iter()
			.map(|&(w, p)| {
				let exact = p * units as f32;
				(w, exact.floor() as i32, exact - exact.floor())
			})
			.collect();
		let mut left = units - shares.iter().map(|s| s.1).sum::<i32>();
		shares.sort_by(|a, b| b.2.total_cmp(&a.2));
		for share in shares.iter_mut() {
			if left == 0 {
				break;
			}
			share.1 += 1;
			left -= 1;
		}
		for &(w, amount, _) in &shares {
			if w != v && amount > 0 {
				orders.push(Command::Move {
					from: v,
					to: w,
					amount,
				});
			}
		}
	}
	orders
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

fn answer(map: &Map, orders: &[Command]) -> String {
	if orders.is_empty() {
		return "WAIT".to_string();
	}
	orders
		.iter()
		.map(|&c| text(map, c))
		.collect::<Vec<_>>()
		.join(";")
}

struct Labels {
	moves: BTreeMap<(usize, usize), i32>,
	triangle_actions: Vec<i32>,
}

fn labels(map: &Map, state: &State, me: usize, features: &Features, line: &str) -> Labels {
	let mut remaining = [0i32; MAX_NODES];
	for (n, slot) in remaining.iter_mut().enumerate().take(map.len()) {
		*slot = state.units[n][me];
	}
	let mut moves = BTreeMap::new();
	let mut triangle_actions: Vec<i32> = features
		.usable
		.iter()
		.map(|&u| if u { HOLD } else { NOT_USABLE })
		.collect();
	let index_of = |t: usize| features.live.iter().position(|&l| l == t);
	let mut pending_moves = Vec::new();
	for command in line.split(';') {
		let words: Vec<&str> = command.split_whitespace().collect();
		let numbers: Vec<usize> = words
			.iter()
			.skip(1)
			.filter_map(|w| w.parse().ok())
			.collect();
		let triangle_of = |a: usize, b: usize, c: usize| {
			[a, b, c]
				.iter()
				.all(|&n| n < map.len())
				.then(|| map.triangle([a, b, c]))
				.flatten()
		};
		match (words.first().copied(), numbers.as_slice()) {
			(Some("MOVE"), &[from, to, amount, ..]) if from < map.len() && to < map.len() => {
				pending_moves.push((from, to, amount as i32));
			}
			(Some("SPAWN"), &[node, b, c, ..]) => {
				if let Some(index) = triangle_of(node, b, c).and_then(index_of) {
					let t = features.live[index];
					if let Some(k) = map.triangles[t].iter().position(|&x| x == node) {
						if triangle_actions[index] == HOLD {
							triangle_actions[index] = SPAWN_BASE + k as i32;
						}
					}
				}
			}
			(Some("ATTACK"), &[a, b, c, target, ..]) => {
				if let Some(index) = triangle_of(a, b, c).and_then(index_of) {
					if triangle_actions[index] == HOLD
						&& features.attack_targets[index] & bit(target) != 0
					{
						triangle_actions[index] = ATTACK_BASE + target as i32;
						for corner in [a, b, c] {
							remaining[corner] -= 1;
						}
					} else if triangle_actions[index] == HOLD {
						triangle_actions[index] = IGNORED;
					}
				}
			}
			(Some("ADD_PATH" | "REMOVE_PATH"), &[a, b, c, ..]) => {
				if let Some(index) = triangle_of(a, b, c).and_then(index_of) {
					if triangle_actions[index] >= HOLD {
						triangle_actions[index] = IGNORED;
					}
				}
			}
			_ => {}
		}
	}
	for (from, to, amount) in pending_moves {
		let amount = amount.min(remaining[from]);
		if amount <= 0 || from == to {
			continue;
		}
		if let Some(step) = state.first_step(from, to) {
			remaining[from] -= amount;
			*moves.entry((from, step)).or_default() += amount;
		}
	}
	Labels {
		moves,
		triangle_actions,
	}
}

fn write_record(
	out: &mut impl Write,
	map: &Map,
	state: &State,
	me: usize,
	turn: i32,
	features: &Features,
	labels: &Labels,
) {
	let count = map.len();
	let mut bytes: Vec<u8> = Vec::new();
	let put_i32 = |bytes: &mut Vec<u8>, v: i32| bytes.extend_from_slice(&v.to_le_bytes());
	put_i32(&mut bytes, turn);
	put_i32(&mut bytes, count as i32);
	put_i32(&mut bytes, features.live.len() as i32);
	for n in 0..MAX_NODES {
		let row = features
			.nodes
			.get(n)
			.copied()
			.unwrap_or([0.0; NODE_FEATURES]);
		for v in row {
			bytes.extend_from_slice(&v.to_le_bytes());
		}
	}
	for n in 0..MAX_NODES {
		let around = if n < count { state.adjacency[n] } else { 0 };
		bytes.extend_from_slice(&around.to_le_bytes());
	}
	for n in 0..MAX_NODES {
		let units = if n < count { state.units[n][me] } else { 0 };
		put_i32(&mut bytes, units);
	}
	for v in features.globals {
		bytes.extend_from_slice(&v.to_le_bytes());
	}
	for index in 0..MAX_LIVE {
		let corners = features
			.live
			.get(index)
			.map(|&t| map.triangles[t].map(|c| c as i32))
			.unwrap_or([-1; 3]);
		for c in corners {
			put_i32(&mut bytes, c);
		}
	}
	for index in 0..MAX_LIVE {
		let row = features
			.triangles
			.get(index)
			.copied()
			.unwrap_or([0.0; TRIANGLE_FEATURES]);
		for v in row {
			bytes.extend_from_slice(&v.to_le_bytes());
		}
	}
	for index in 0..MAX_LIVE {
		let targets = features.attack_targets.get(index).copied().unwrap_or(0);
		bytes.extend_from_slice(&targets.to_le_bytes());
	}
	for index in 0..MAX_LIVE {
		put_i32(
			&mut bytes,
			labels
				.triangle_actions
				.get(index)
				.copied()
				.unwrap_or(NOT_USABLE),
		);
	}
	let mut flows = vec![0i16; MAX_NODES * MAX_NODES];
	for n in 0..count {
		flows[n * MAX_NODES + n] = state.units[n][me] as i16;
	}
	for (&(from, to), &amount) in &labels.moves {
		flows[from * MAX_NODES + to] += amount as i16;
		flows[from * MAX_NODES + from] -= amount as i16;
	}
	for v in flows {
		bytes.extend_from_slice(&v.to_le_bytes());
	}
	out.write_all(&bytes).expect("the dump file is writable");
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

fn weights() -> Option<Vec<u8>> {
	if let Ok(path) = std::env::var("NOVA_WEIGHTS") {
		return Some(std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}")));
	}
	if WEIGHTS.is_empty() {
		return None;
	}
	Some(unpack(WEIGHTS))
}

fn main() {
	let net = weights().map(|bytes| Net::load(&bytes));
	let mut dump = std::env::var("NOVA_DUMP").ok().map(|path| {
		std::io::BufWriter::new(
			std::fs::File::create(&path).unwrap_or_else(|e| panic!("cannot create {path}: {e}")),
		)
	});
	let mut recorded = std::env::var("NOVA_ANSWERS").ok().map(|path| {
		std::fs::read_to_string(&path)
			.unwrap_or_else(|e| panic!("cannot read {path}: {e}"))
			.lines()
			.map(str::to_string)
			.collect::<Vec<_>>()
			.into_iter()
	});
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
	let mut sampler = std::env::var("NOVA_SAMPLE")
		.ok()
		.map(|seed| Rng(seed.parse::<u64>().expect("NOVA_SAMPLE is a seed") | 1));
	let mut me = None;
	let mut turn = 0;
	loop {
		let (state, side) = read_state(&mut input, &map, me);
		let started = Instant::now();
		me = Some(side);
		turn += 1;
		let features = features(&map, &state, side, turn);
		if let (Some(file), Some(lines)) = (dump.as_mut(), recorded.as_mut()) {
			let line = lines.next().unwrap_or_default();
			let labels = labels(&map, &state, side, &features, &line);
			write_record(file, &map, &state, side, turn, &features, &labels);
			file.flush().expect("the dump file is writable");
		}
		let orders = match &net {
			Some(net) => {
				let decision = decide(net, &map, &state, side, &features);
				if std::env::var("NOVA_DEBUG").is_ok() {
					for (v, options) in decision.moves.iter().enumerate() {
						if !options.is_empty() {
							let shown: Vec<String> =
								options.iter().map(|(w, p)| format!("{w}:{p:.2}")).collect();
							let units = state.units[v][side];
							eprintln!("t{turn} node {v} units {units} -> {}", shown.join(" "));
						}
					}
					for (index, options) in decision.triangle_actions.iter().enumerate() {
						if !options.is_empty() {
							let shown: Vec<String> =
								options.iter().map(|(a, p)| format!("{a}:{p:.2}")).collect();
							let corners = map.triangles[features.live[index]];
							eprintln!("t{turn} triangle {corners:?} -> {}", shown.join(" "));
						}
					}
				}
				commands(&map, &state, side, &features, &decision, sampler.as_mut())
			}
			None => Vec::new(),
		};
		let elapsed = started.elapsed().as_micros();
		writeln!(out, "{};MSG {elapsed}us", answer(&map, &orders)).expect("stdout is writable");
		out.flush().expect("stdout is writable");
	}
}
