use engine::{Command, MAX_NODES, Map, Point, State, bit, nodes_of};
use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::time::Instant;

pub const POLICY_WEIGHTS: &str = "";
pub const OPPONENT_WEIGHTS: &str = "";
pub const VALUE_WEIGHTS: &str = "";

pub const NODE_FEATURES: usize = 28;
pub const GLOBAL_FEATURES: usize = 8;
pub const TRIANGLE_FEATURES: usize = 12;
pub const MAX_LIVE: usize = 96;
pub const HOLD: i32 = 0;
pub const SPAWN_BASE: i32 = 1;
pub const ATTACK_BASE: i32 = 4;
pub const IGNORED: i32 = -2;
pub const NOT_USABLE: i32 = -1;
pub const NET_VERSION: u8 = 1;
pub const DEFAULT_SEARCH_MS: Option<f64> = None;
pub const DEFAULT_UNIT_BONUS: f32 = 0.15;
pub const DEFAULT_TRIANGLE_BONUS: f32 = 0.08;
pub const DEFAULT_REACH_BONUS: f32 = 0.3;
pub const DEFAULT_REPLIES: f32 = 1.0;
pub const DEFAULT_CLIMB: f32 = 0.0;
pub const DEFAULT_WORST: f32 = 0.0;
pub const DEFAULT_TEMPERATURE: f32 = 1.0;
pub const REACH_CAP: u32 = 12;
pub const STALL_TURN: i32 = 12;
pub const PACK_BASE: u32 = 0x4E00;

pub struct Features {
	pub nodes: Vec<[f32; NODE_FEATURES]>,
	pub globals: [f32; GLOBAL_FEATURES],
	pub live: Vec<usize>,
	pub triangles: Vec<[f32; TRIANGLE_FEATURES]>,
	pub usable: Vec<bool>,
	pub attack_targets: Vec<u64>,
}

pub fn ln1p(value: i32) -> f32 {
	(value.max(0) as f32).ln_1p()
}

pub fn distances_from(state: &State, sources: u64, count: usize) -> Vec<f32> {
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

pub fn surround_threatened(state: &State, victim: usize, node: usize) -> bool {
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

pub fn attack_targets(map: &Map, state: &State, triangle: usize, opponent: usize) -> u64 {
	let around = nodes_of(map.corners(triangle)).fold(0u64, |acc, c| acc | state.adjacency[c]);
	around & state.occupied_by(opponent)
}

pub fn features(map: &Map, state: &State, me: usize, turn: i32) -> Features {
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
		(state.score[me] as f32 / 100.0).min(SCORE_FEATURE_CAP),
		(state.score[opponent] as f32 / 100.0).min(SCORE_FEATURE_CAP),
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

pub const SCORE_FEATURE_CAP: f32 = 8.0;

const LANES: usize = 8;

#[inline(always)]
pub fn dot(a: &[f32], b: &[f32]) -> f32 {
	let mut lanes = [0.0f32; LANES];
	let mut a_chunks = a.chunks_exact(LANES);
	let mut b_chunks = b.chunks_exact(LANES);
	for (x, y) in (&mut a_chunks).zip(&mut b_chunks) {
		for k in 0..LANES {
			lanes[k] += x[k] * y[k];
		}
	}
	let tail: f32 = a_chunks
		.remainder()
		.iter()
		.zip(b_chunks.remainder())
		.map(|(x, y)| x * y)
		.sum();
	lanes.iter().sum::<f32>() + tail
}

pub struct Layer {
	pub inputs: usize,
	pub outputs: usize,
	pub weights: Vec<f32>,
	pub columns: Vec<f32>,
	pub bias: Vec<f32>,
}

impl Layer {
	pub fn new(inputs: usize, outputs: usize, weights: Vec<f32>, bias: Vec<f32>) -> Layer {
		let columns = (0..inputs * outputs)
			.map(|at| weights[(at % outputs) * inputs + at / outputs])
			.collect();
		Layer {
			inputs,
			outputs,
			weights,
			columns,
			bias,
		}
	}

	#[inline(always)]
	pub fn apply(&self, input: &[f32], output: &mut [f32]) {
		let input = &input[..self.inputs];
		if self.outputs % LANES != 0 {
			for (o, out) in output.iter_mut().enumerate().take(self.outputs) {
				*out = self.bias[o]
					+ dot(&self.weights[o * self.inputs..(o + 1) * self.inputs], input);
			}
			return;
		}
		let mut start = 0;
		while start + 4 * LANES <= self.outputs {
			self.block::<{ 4 * LANES }>(input, output, start);
			start += 4 * LANES;
		}
		while start < self.outputs {
			self.block::<LANES>(input, output, start);
			start += LANES;
		}
	}

	#[inline(always)]
	fn block<const WIDTH: usize>(&self, input: &[f32], output: &mut [f32], start: usize) {
		let mut sums = [0.0f32; WIDTH];
		sums.copy_from_slice(&self.bias[start..start + WIDTH]);
		for (x, column) in input.iter().zip(self.columns.chunks_exact(self.outputs)) {
			let weights = &column[start..start + WIDTH];
			for k in 0..WIDTH {
				sums[k] += x * weights[k];
			}
		}
		output[start..start + WIDTH].copy_from_slice(&sums);
	}

	#[inline(always)]
	pub fn apply_rows(&self, inputs: &[f32], outputs: &mut [f32]) {
		for (row_in, row_out) in inputs
			.chunks_exact(self.inputs)
			.zip(outputs.chunks_exact_mut(self.outputs))
		{
			self.apply(row_in, row_out);
		}
	}

	pub fn run(&self, input: &[f32]) -> Vec<f32> {
		let mut output = vec![0.0; self.outputs];
		self.apply(input, &mut output);
		output
	}
}

pub struct Round {
	pub own: Layer,
	pub neighbors: Layer,
	pub triangle: Layer,
	pub gather: Layer,
}

pub struct Net {
	pub channels: usize,
	pub embed: Layer,
	pub rounds: Vec<Round>,
	pub query: Layer,
	pub key: Layer,
	pub destination: Layer,
	pub stay: Layer,
	pub triangle: Layer,
	pub hold: Layer,
	pub spawn: Layer,
	pub attack: Layer,
	pub value: Option<(Layer, Layer)>,
}

pub struct Reader<'a> {
	pub bytes: &'a [u8],
	pub at: usize,
}

impl Reader<'_> {
	pub fn take(&mut self, count: usize) -> &[u8] {
		let slice = &self.bytes[self.at..self.at + count];
		self.at += count;
		slice
	}

	pub fn u8(&mut self) -> u8 {
		self.take(1)[0]
	}

	pub fn u16(&mut self) -> usize {
		let b = self.take(2);
		usize::from(u16::from_le_bytes([b[0], b[1]]))
	}

	pub fn f32s(&mut self, count: usize) -> Vec<f32> {
		self.take(4 * count)
			.chunks_exact(4)
			.map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
			.collect()
	}

	pub fn layer(&mut self) -> Layer {
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
		Layer::new(inputs, outputs, weights, bias)
	}
}

pub fn unpack(text: &str) -> Vec<u8> {
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
	pub fn load(bytes: &[u8]) -> Net {
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
		let mut net = Net {
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
			value: None,
		};
		if reader.bytes.len() > reader.at + 4 {
			net.value = Some((reader.layer(), reader.layer()));
		}
		net
	}

	pub fn hidden(&self, map: &Map, state: &State, features: &Features) -> Vec<Vec<f32>> {
		self.hidden_flat(map, state, features)
			.chunks_exact(self.channels)
			.map(<[f32]>::to_vec)
			.collect()
	}

	pub fn hidden_flat(&self, map: &Map, state: &State, features: &Features) -> Vec<f32> {
		#[cfg(target_arch = "x86_64")]
		if std::is_x86_feature_detected!("avx2") && std::is_x86_feature_detected!("fma") {
			return unsafe { self.hidden_avx(map, state, features) };
		}
		self.hidden_body(map, state, features)
	}

	#[cfg(target_arch = "x86_64")]
	#[target_feature(enable = "avx2,fma")]
	unsafe fn hidden_avx(&self, map: &Map, state: &State, features: &Features) -> Vec<f32> {
		self.hidden_body(map, state, features)
	}

	#[inline(always)]
	fn hidden_body(&self, map: &Map, state: &State, features: &Features) -> Vec<f32> {
		let c = self.channels;
		let count = features.nodes.len();
		let mut h = vec![0.0f32; count * c];
		let mut input = [0.0f32; NODE_FEATURES + GLOBAL_FEATURES];
		input[NODE_FEATURES..].copy_from_slice(&features.globals);
		for (row, out) in features.nodes.iter().zip(h.chunks_exact_mut(c)) {
			input[..NODE_FEATURES].copy_from_slice(row);
			self.embed.apply(&input, out);
			relu(out);
		}
		let corners: Vec<Vec<usize>> = features
			.live
			.iter()
			.map(|&t| nodes_of(map.corners(t)).collect())
			.collect();
		let mut triangle_count = vec![0.0f32; count];
		for &corner in corners.iter().flatten() {
			triangle_count[corner] += 1.0;
		}
		let degrees: Vec<f32> = (0..count)
			.map(|n| state.adjacency[n].count_ones() as f32)
			.collect();
		let mut gathered = vec![0.0f32; count * c];
		let mut next = vec![0.0f32; count * c];
		let mut means = vec![0.0f32; count * c];
		let mut part = vec![0.0f32; count * c];
		let mut message = vec![0.0f32; c];
		let mut triangle_input = vec![0.0f32; c + TRIANGLE_FEATURES];
		for round in &self.rounds {
			gathered.fill(0.0);
			for (corners_of, features_of) in corners.iter().zip(&features.triangles) {
				triangle_input[..c].fill(0.0);
				for &corner in corners_of {
					for (sum, v) in triangle_input[..c]
						.iter_mut()
						.zip(&h[corner * c..(corner + 1) * c])
					{
						*sum += v / 3.0;
					}
				}
				triangle_input[c..].copy_from_slice(features_of);
				round.triangle.apply(&triangle_input, &mut message);
				relu(&mut message);
				for &corner in corners_of {
					for (sum, m) in gathered[corner * c..(corner + 1) * c]
						.iter_mut()
						.zip(&message)
					{
						*sum += m;
					}
				}
			}
			for n in 0..count {
				let mean = &mut means[n * c..(n + 1) * c];
				mean.fill(0.0);
				if degrees[n] > 0.0 {
					for m in nodes_of(state.adjacency[n]) {
						for (sum, v) in mean.iter_mut().zip(&h[m * c..(m + 1) * c]) {
							*sum += v / degrees[n];
						}
					}
				}
				if triangle_count[n] > 0.0 {
					for v in gathered[n * c..(n + 1) * c].iter_mut() {
						*v /= triangle_count[n];
					}
				}
			}
			round.own.apply_rows(&h, &mut next);
			round.neighbors.apply_rows(&means, &mut part);
			for (o, p) in next.iter_mut().zip(&part) {
				*o += p;
			}
			round.gather.apply_rows(&gathered, &mut part);
			for ((o, p), x) in next.iter_mut().zip(&part).zip(&h) {
				*o = (*o + p).max(0.0) + x;
			}
			std::mem::swap(&mut h, &mut next);
		}
		h
	}
}

#[inline(always)]
pub fn relu(values: &mut [f32]) {
	for v in values {
		*v = v.max(0.0);
	}
}

impl Net {
	pub fn value(&self, map: &Map, state: &State, features: &Features) -> Option<f32> {
		let (hidden_layer, out_layer) = self.value.as_ref()?;
		let h = self.hidden_flat(map, state, features);
		let channels = self.channels;
		let count = (h.len() / channels).max(1) as f32;
		let mut pooled = vec![0.0f32; 2 * channels];
		for k in 0..channels {
			let column = h.iter().skip(k).step_by(channels);
			pooled[k] = column.clone().sum::<f32>() / count;
			pooled[channels + k] = column.fold(f32::NEG_INFINITY, |a, &b| a.max(b));
		}
		pooled.extend_from_slice(&features.globals);
		let hidden: Vec<f32> = hidden_layer
			.run(&pooled)
			.into_iter()
			.map(|v| v.max(0.0))
			.collect();
		Some(out_layer.run(&hidden)[0])
	}
}

pub fn hop_distances(map: &Map, state: &State) -> Vec<[u8; MAX_NODES]> {
	(0..map.len())
		.map(|source| {
			let mut distance = [u8::MAX; MAX_NODES];
			distance[source] = 0;
			let mut frontier = vec![source];
			while !frontier.is_empty() {
				let mut next = Vec::new();
				for &v in &frontier {
					for w in nodes_of(state.adjacency[v]) {
						if distance[w] == u8::MAX {
							distance[w] = distance[v] + 1;
							next.push(w);
						}
					}
				}
				frontier = next;
			}
			distance
		})
		.collect()
}

pub fn reach_cost(map: &Map, state: &State, me: usize, hops: &[[u8; MAX_NODES]]) -> u32 {
	let owned = state.node_owners()[me];
	let mine: Vec<usize> = (0..map.len()).filter(|&v| state.units[v][me] > 0).collect();
	state
		.live
		.iter()
		.filter(|&t| !state.owner[me].has(t) && state.can_capture[me].has(t))
		.map(|t| {
			let corners = map.triangles[t];
			corners
				.iter()
				.filter(|&&c| owned & bit(c) == 0)
				.map(|&c| {
					mine.iter()
						.filter(|v| !corners.contains(v))
						.map(|&v| u32::from(hops[v][c]))
						.min()
						.unwrap_or(u32::from(u8::MAX))
				})
				.sum::<u32>()
		})
		.min()
		.unwrap_or(0)
		.min(REACH_CAP)
}

fn weight(name: &str, default: f32) -> f32 {
	std::env::var(name)
		.ok()
		.map(|w| {
			w.parse::<f32>()
				.unwrap_or_else(|_| panic!("{name} is a number"))
		})
		.unwrap_or(default)
}

fn budget(name: &str) -> usize {
	std::env::var(name)
		.ok()
		.map(|n| {
			n.parse::<usize>()
				.unwrap_or_else(|_| panic!("{name} is a count"))
		})
		.unwrap_or(usize::MAX)
}

#[allow(clippy::too_many_arguments)]
pub fn search(
	predictor: &Net,
	valuer: &Net,
	map: &Map,
	state: &State,
	me: usize,
	turn: i32,
	seen: &Features,
	decision: &Decision,
	deadline: Instant,
	rng: &mut Rng,
) -> (Vec<Command>, usize) {
	let opponent = 1 - me;
	let unit_bonus = weight("NOVA_UNIT_BONUS", DEFAULT_UNIT_BONUS);
	let triangle_bonus = weight("NOVA_TRIANGLE_BONUS", DEFAULT_TRIANGLE_BONUS);
	let reach_bonus = weight("NOVA_REACH_BONUS", DEFAULT_REACH_BONUS);
	let candidate_cap = budget("NOVA_SEARCH_CANDIDATES");
	let draw_cap = budget("NOVA_SEARCH_DRAWS");
	let stalled = state.owner[me].len() == 0 && turn >= STALL_TURN;
	let hops = (reach_bonus != 0.0 && stalled).then(|| hop_distances(map, state));
	let their_view = features(map, state, opponent, turn);
	let their_decision = decide(predictor, map, state, opponent, &their_view);
	let reply_count = (weight("NOVA_REPLIES", DEFAULT_REPLIES) as usize).max(1);
	let mut replies = vec![commands(
		map,
		state,
		opponent,
		&their_view,
		&their_decision,
		None,
	)];
	while replies.len() < reply_count {
		replies.push(commands(
			map,
			state,
			opponent,
			&their_view,
			&their_decision,
			Some(rng),
		));
	}
	let against = |mine: &[Command], theirs: &[Command]| {
		let mut next = *state;
		let mut ordered: [&[Command]; 2] = [&[], &[]];
		ordered[me] = mine;
		ordered[opponent] = theirs;
		next.step(map, ordered);
		let view = features(map, &next, me, turn + 1);
		let mut total = valuer.value(map, &next, &view).unwrap_or(0.0);
		total += unit_bonus * (next.units_of(me) - next.units_of(opponent)) as f32;
		total += triangle_bonus * next.owner[me].len() as f32;
		if let Some(hops) = &hops {
			total -= reach_bonus * reach_cost(map, &next, me, hops) as f32;
		}
		total
	};
	let worst_share = weight("NOVA_WORST", DEFAULT_WORST);
	let score = |mine: &[Command]| {
		let values: Vec<f32> = replies.iter().map(|theirs| against(mine, theirs)).collect();
		let mean = values.iter().sum::<f32>() / values.len() as f32;
		let worst = values.iter().copied().fold(f32::INFINITY, f32::min);
		mean + worst_share * (worst - mean)
	};
	let temperature = weight("NOVA_TEMPERATURE", DEFAULT_TEMPERATURE);
	let tempered = (temperature != 1.0).then(|| tempered(decision, temperature));
	let decision_for_draws = tempered.as_ref().unwrap_or(decision);
	let mut best = commands(map, state, me, seen, decision, None);
	let mut best_value = score(&best);
	let mut tried = 1;
	let mut evaluated = std::collections::HashSet::new();
	evaluated.insert(best.clone());
	let mut draws = 0;
	let climb_share = weight("NOVA_CLIMB", DEFAULT_CLIMB);
	let started = Instant::now();
	let sampling_end = started
		+ deadline
			.saturating_duration_since(started)
			.mul_f32(1.0 - climb_share);
	while Instant::now() < sampling_end && tried < candidate_cap && draws < draw_cap {
		draws += 1;
		let candidate = commands(map, state, me, seen, decision_for_draws, Some(rng));
		if !evaluated.insert(candidate.clone()) {
			continue;
		}
		let value = score(&candidate);
		tried += 1;
		if value > best_value {
			best_value = value;
			best = candidate;
		}
	}
	while climb_share > 0.0
		&& Instant::now() < deadline
		&& tried < candidate_cap
		&& draws < draw_cap
	{
		draws += 1;
		let Some(candidate) = mutate(map, state, me, seen, decision, &best, rng) else {
			continue;
		};
		if !evaluated.insert(candidate.clone()) {
			continue;
		}
		let value = score(&candidate);
		tried += 1;
		if value > best_value {
			best_value = value;
			best = candidate;
		}
	}
	(best, tried)
}

fn tempered(decision: &Decision, temperature: f32) -> Decision {
	fn reshape<T: Copy>(options: &[(T, f32)], temperature: f32) -> Vec<(T, f32)> {
		let powered: Vec<(T, f32)> = options
			.iter()
			.map(|&(a, p)| (a, p.max(1e-12).powf(1.0 / temperature)))
			.collect();
		let total: f32 = powered.iter().map(|&(_, p)| p).sum();
		powered.into_iter().map(|(a, p)| (a, p / total)).collect()
	}
	Decision {
		moves: decision
			.moves
			.iter()
			.map(|o| reshape(o, temperature))
			.collect(),
		triangle_actions: decision
			.triangle_actions
			.iter()
			.map(|o| reshape(o, temperature))
			.collect(),
	}
}

fn command_key(command: &Command) -> (u8, usize, usize, i32) {
	match *command {
		Command::Attack { triangle, target } => (0, triangle, target, 0),
		Command::Spawn { triangle, node } => (1, triangle, node, 0),
		Command::Move { from, to, amount } => (2, from, to, amount),
		_ => (3, 0, 0, 0),
	}
}

fn mutate(
	map: &Map,
	state: &State,
	me: usize,
	features: &Features,
	decision: &Decision,
	plan: &[Command],
	rng: &mut Rng,
) -> Option<Vec<Command>> {
	let houses: Vec<usize> = (0..decision.moves.len())
		.filter(|&v| state.units[v][me] > 0 && decision.moves[v].len() > 1)
		.collect();
	let triangles: Vec<usize> = (0..decision.triangle_actions.len())
		.filter(|&i| decision.triangle_actions[i].len() > 1)
		.collect();
	let choices = houses.len() + triangles.len();
	if choices == 0 {
		return None;
	}
	let pick = ((rng.unit() * choices as f32) as usize).min(choices - 1);
	let mut next: Vec<Command> = Vec::with_capacity(plan.len() + 2);
	if pick < houses.len() {
		let v = houses[pick];
		let attacking = plan
			.iter()
			.filter_map(|c| match *c {
				Command::Attack { triangle, .. } if map.triangles[triangle].contains(&v) => {
					Some(())
				}
				_ => None,
			})
			.count() as i32;
		let units = state.units[v][me] - attacking;
		if units <= 0 {
			return None;
		}
		next.extend(
			plan.iter()
				.copied()
				.filter(|c| !matches!(*c, Command::Move { from, .. } if from == v)),
		);
		let options = &decision.moves[v];
		let mut counts: BTreeMap<usize, i32> = BTreeMap::new();
		if rng.unit() < 0.5 {
			*counts.entry(rng.draw(options)).or_default() += units;
		} else {
			for _ in 0..units {
				*counts.entry(rng.draw(options)).or_default() += 1;
			}
		}
		for (w, amount) in counts {
			if w != v {
				next.push(Command::Move {
					from: v,
					to: w,
					amount,
				});
			}
		}
	} else {
		let index = triangles[pick - houses.len()];
		let t = features.live[index];
		let corners = map.triangles[t];
		let action = rng.draw(&decision.triangle_actions[index]);
		if action >= ATTACK_BASE {
			return None;
		}
		if plan
			.iter()
			.any(|c| matches!(*c, Command::Attack { triangle, .. } if triangle == t))
		{
			return None;
		}
		next.extend(
			plan.iter()
				.copied()
				.filter(|c| !matches!(*c, Command::Spawn { triangle, .. } if triangle == t)),
		);
		if action != HOLD {
			next.push(Command::Spawn {
				triangle: t,
				node: corners[(action - SPAWN_BASE) as usize],
			});
		}
	}
	next.sort_by_key(command_key);
	Some(next)
}

pub struct Decision {
	pub moves: Vec<Vec<(usize, f32)>>,
	pub triangle_actions: Vec<Vec<(i32, f32)>>,
}

pub fn softmax(logits: &mut [(i32, f32)]) {
	let peak = logits
		.iter()
		.map(|&(_, l)| l)
		.fold(f32::NEG_INFINITY, f32::max);
	let total: f32 = logits.iter().map(|&(_, l)| (l - peak).exp()).sum();
	for entry in logits.iter_mut() {
		entry.1 = (entry.1 - peak).exp() / total;
	}
}

pub fn decide(net: &Net, map: &Map, state: &State, me: usize, features: &Features) -> Decision {
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

pub struct Rng(pub u64);

impl Rng {
	pub fn unit(&mut self) -> f32 {
		self.0 ^= self.0 << 13;
		self.0 ^= self.0 >> 7;
		self.0 ^= self.0 << 17;
		(self.0 >> 40) as f32 / (1u64 << 24) as f32
	}

	pub fn draw<T: Copy>(&mut self, options: &[(T, f32)]) -> T {
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

pub fn commands(
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

pub fn text(map: &Map, command: Command) -> String {
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

pub fn answer(map: &Map, orders: &[Command]) -> String {
	if orders.is_empty() {
		return "WAIT".to_string();
	}
	orders
		.iter()
		.map(|&c| text(map, c))
		.collect::<Vec<_>>()
		.join(";")
}

pub struct Labels {
	pub moves: BTreeMap<(usize, usize), i32>,
	pub triangle_actions: Vec<i32>,
}

pub fn labels(map: &Map, state: &State, me: usize, features: &Features, line: &str) -> Labels {
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

pub fn write_record(
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

pub struct Input<R: BufRead> {
	pub lines: std::io::Lines<R>,
}

impl<R: BufRead> Input<R> {
	pub fn ints(&mut self) -> Vec<i64> {
		let Some(line) = self.lines.next() else {
			std::process::exit(0);
		};
		line.expect("stdin is readable")
			.split_whitespace()
			.map(|field| field.parse().expect("the referee sends integers"))
			.collect()
	}

	pub fn int(&mut self) -> i64 {
		self.ints()[0]
	}
}

pub fn read_state<R: BufRead>(
	input: &mut Input<R>,
	map: &Map,
	me: Option<usize>,
) -> (State, usize) {
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

pub fn weights() -> Option<Vec<u8>> {
	if let Ok(path) = std::env::var("NOVA_WEIGHTS") {
		return Some(std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}")));
	}
	if POLICY_WEIGHTS.is_empty() {
		return None;
	}
	Some(unpack(POLICY_WEIGHTS))
}

fn opponent_weights() -> Option<Vec<u8>> {
	if let Ok(path) = std::env::var("NOVA_OPPONENT_WEIGHTS") {
		return Some(std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}")));
	}
	if OPPONENT_WEIGHTS.is_empty() {
		return None;
	}
	Some(unpack(OPPONENT_WEIGHTS))
}

fn value_weights() -> Option<Vec<u8>> {
	if let Ok(path) = std::env::var("NOVA_VALUE_WEIGHTS") {
		return Some(std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}")));
	}
	if VALUE_WEIGHTS.is_empty() {
		return None;
	}
	Some(unpack(VALUE_WEIGHTS))
}

fn main() {
	let net = weights().map(|bytes| Net::load(&bytes));
	let predictor = opponent_weights().map(|bytes| Net::load(&bytes));
	let valuer = value_weights().map(|bytes| Net::load(&bytes));
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
	let search_ms = std::env::var("NOVA_SEARCH_MS")
		.ok()
		.map(|ms| ms.parse::<f64>().expect("NOVA_SEARCH_MS is a number"))
		.or(DEFAULT_SEARCH_MS);
	let mut searcher = Rng(
		std::env::var("NOVA_SEARCH_SEED").map_or(0x2545F4914F6CDD1D, |text| {
			text.bytes().fold(0xcbf29ce484222325u64, |h, b| {
				(h ^ u64::from(b)).wrapping_mul(0x100000001b3)
			}) | 1
		}),
	);
	let mut candidates = 0;
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
				let valuer = valuer.as_ref().unwrap_or(net);
				match (search_ms, valuer.value.is_some()) {
					(Some(budget), true) => {
						let deadline =
							started + std::time::Duration::from_secs_f64(budget / 1000.0);
						let (best, tried) = search(
							predictor.as_ref().unwrap_or(net),
							valuer,
							&map,
							&state,
							side,
							turn,
							&features,
							&decision,
							deadline,
							&mut searcher,
						);
						candidates = tried;
						best
					}
					_ => commands(&map, &state, side, &features, &decision, sampler.as_mut()),
				}
			}
			None => Vec::new(),
		};
		let elapsed = started.elapsed().as_micros();
		let line = answer(&map, &orders);
		if let (Some(file), None) = (dump.as_mut(), recorded.as_ref()) {
			let labels = labels(&map, &state, side, &features, &line);
			write_record(file, &map, &state, side, turn, &features, &labels);
			file.flush().expect("the dump file is writable");
		}
		writeln!(out, "{line};MSG {elapsed}us c{candidates}").expect("stdout is writable");
		out.flush().expect("stdout is writable");
	}
}
