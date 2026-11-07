use engine::{Command, MAX_NODES, Map, Point, State, bit, nodes_of};
use std::cell::RefCell;
use std::io::{BufRead, Write};
use std::time::{Duration, Instant};

pub const NET_WEIGHTS: &str = "";
pub const DEFAULT_SEARCH_MS: f64 = 48.0;

const ME: usize = 0;
const THEM: usize = 1;
const FAR: u8 = 40;
const LAST_TURN: i32 = 200;
const NODE_FEATURES: usize = 29;
const GLOBAL_FEATURES: usize = 8;
const MAX_DEGREE: usize = 12;
const MOVE_SLOTS: usize = MAX_DEGREE + 1;
const MAX_USABLE: usize = 32;
const RECORD_NODES: usize = 64;
const PACK_BASE: u32 = 0x4E00;
const MAGIC: u32 = 0x3154_4D43;
const HOLD: usize = 0;
const ATTACK: usize = 4;

fn tuned(name: &str, default: f64) -> f64 {
	std::env::var(format!("COMET_{name}"))
		.ok()
		.and_then(|value| value.parse().ok())
		.unwrap_or(default)
}

struct Rng(u64);

impl Rng {
	fn next(&mut self) -> u64 {
		self.0 ^= self.0 << 13;
		self.0 ^= self.0 >> 7;
		self.0 ^= self.0 << 17;
		self.0
	}

	fn unit(&mut self) -> f32 {
		(self.next() >> 40) as f32 / (1u64 << 24) as f32
	}

	fn draw(&mut self, weights: &[f32]) -> usize {
		let total: f32 = weights.iter().sum();
		let mut pick = self.unit() * total;
		for (index, &w) in weights.iter().enumerate() {
			if pick < w {
				return index;
			}
			pick -= w;
		}
		weights.len() - 1
	}
}

#[derive(Default)]
struct Mix(u64);

impl std::hash::Hasher for Mix {
	fn finish(&self) -> u64 {
		self.0
	}

	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.write_u64(u64::from(b));
		}
	}

	fn write_u64(&mut self, value: u64) {
		self.0 = (self.0.rotate_left(5) ^ value).wrapping_mul(0x51_7cc1_b727_220a_95);
	}

	fn write_usize(&mut self, value: usize) {
		self.write_u64(value as u64);
	}

	fn write_i32(&mut self, value: i32) {
		self.write_u64(value as u32 as u64);
	}

	fn write_isize(&mut self, value: isize) {
		self.write_u64(value as u64);
	}
}

type Seen = std::collections::HashSet<u64, std::hash::BuildHasherDefault<Mix>>;

fn fingerprint(commands: &[Command]) -> u64 {
	use std::hash::{Hash, Hasher};
	let mut mix = Mix::default();
	commands.hash(&mut mix);
	mix.finish()
}

fn around(state: &State, nodes: u64) -> u64 {
	nodes_of(nodes).fold(0, |set, n| set | state.adjacency[n])
}

fn distances(state: &State, sources: u64) -> [u8; MAX_NODES] {
	let mut distance = [FAR; MAX_NODES];
	let mut seen = sources;
	let mut frontier = sources;
	let mut depth = 0;
	while frontier != 0 {
		for node in nodes_of(frontier) {
			distance[node] = depth;
		}
		frontier = around(state, frontier) & !seen;
		seen |= frontier;
		depth += 1;
	}
	distance
}

fn occupied(state: &State, player: usize, count: usize) -> u64 {
	(0..count)
		.filter(|&n| state.units[n][player] > 0)
		.fold(0, |set, n| set | bit(n))
}

struct Neighbors {
	flat: Vec<usize>,
	starts: Vec<usize>,
}

impl Neighbors {
	fn of(&self, node: usize) -> &[usize] {
		&self.flat[self.starts[node]..self.starts[node + 1]]
	}
}

struct Obs {
	nodes: Vec<[f32; NODE_FEATURES]>,
	globals: [f32; GLOBAL_FEATURES],
	neighbors: Neighbors,
	usable: Vec<usize>,
	corners: Vec<[usize; 3]>,
	attack_targets: Vec<u64>,
}

fn observe(map: &Map, state: &State, player: usize, turn: i32, flip: bool) -> Obs {
	let count = map.len();
	let opponent = 1 - player;
	let units = |n: usize, p: usize| state.units[n][p];
	let force = |n: usize, p: usize| {
		units(n, p)
			+ nodes_of(state.adjacency[n])
				.map(|m| units(m, p))
				.sum::<i32>()
	};
	let owners = state.node_owners();
	let reach = [player, opponent].map(|p| distances(state, occupied(state, p, count)));
	let mut live_at = [0f32; MAX_NODES];
	let mut owned_at = [[0f32; MAX_NODES]; 2];
	let mut capturable_at = [[0f32; MAX_NODES]; 2];
	let mut locked_at = [[0f32; MAX_NODES]; 2];
	let mut progress_at = [[0f32; MAX_NODES]; 2];
	let mut attackable = [0u64; 2];
	for t in state.live.iter() {
		let corners = map.corners(t);
		let unowned = !state.owner[0].has(t) && !state.owner[1].has(t);
		for (side, p) in [player, opponent].into_iter().enumerate() {
			let owned = state.owner[p].has(t);
			let capturable = !owned && state.can_capture[p].has(t);
			let locked = unowned && !state.can_capture[p].has(t);
			let progress = (corners & owners[p]).count_ones() as f32 / 3.0;
			if owned && nodes_of(corners).all(|c| units(c, p) >= 1) {
				attackable[side] |= around(state, corners);
			}
			for c in nodes_of(corners) {
				if side == 0 {
					live_at[c] += 1.0;
				}
				if owned {
					owned_at[side][c] += 1.0;
				}
				if capturable {
					capturable_at[side][c] += 1.0;
					progress_at[side][c] = progress_at[side][c].max(progress);
				}
				if locked {
					locked_at[side][c] += 1.0;
				}
			}
		}
	}
	let threatened = |n: usize, victim: usize| {
		let attacker = 1 - victim;
		units(n, victim) > 0
			&& state.adjacency[n] != 0
			&& nodes_of(state.adjacency[n]).all(|m| {
				let incoming: i32 = nodes_of(state.adjacency[m] & !bit(n))
					.map(|k| units(k, attacker))
					.sum();
				units(m, attacker) + incoming > units(m, victim)
			})
	};
	let flag = |b: bool| if b { 1.0 } else { 0.0 };
	let nodes = (0..count)
		.map(|n| {
			let point = map.points[n];
			let mut x = (point.x - 80) as f32 / 1760.0;
			let mut y = (point.y - 70) as f32 / 820.0;
			if flip {
				x = 1.0 - x;
				y = 1.0 - y;
			}
			let [mine, theirs] = [units(n, player), units(n, opponent)];
			let [my_force, their_force] = [force(n, player), force(n, opponent)];
			[
				mine as f32 / 4.0,
				theirs as f32 / 4.0,
				flag(mine > 0),
				flag(theirs > 0),
				flag(mine > theirs),
				flag(theirs > mine),
				my_force as f32 / 8.0,
				their_force as f32 / 8.0,
				flag(my_force > their_force),
				state.adjacency[n].count_ones() as f32 / 8.0,
				flag(owners[player] & bit(n) != 0),
				flag(owners[opponent] & bit(n) != 0),
				f32::from(reach[0][n].min(12)) / 8.0,
				f32::from(reach[1][n].min(12)) / 8.0,
				x,
				y,
				live_at[n] / 6.0,
				owned_at[0][n] / 3.0,
				owned_at[1][n] / 3.0,
				capturable_at[0][n] / 3.0,
				locked_at[0][n] / 3.0,
				capturable_at[1][n] / 3.0,
				locked_at[1][n] / 3.0,
				progress_at[0][n],
				progress_at[1][n],
				flag(threatened(n, player)),
				flag(threatened(n, opponent)),
				flag(attackable[1] & bit(n) != 0) * mine as f32 / 4.0,
				flag(attackable[0] & bit(n) != 0) * theirs as f32 / 4.0,
			]
		})
		.collect();
	let score_gap = (state.score[player] - state.score[opponent]) as f32;
	let triangles = [player, opponent].map(|p| (state.owner[p] & state.live).len() as f32);
	let globals = [
		turn as f32 / LAST_TURN as f32,
		(score_gap / 100.0).tanh(),
		state.units_of(player) as f32 / 30.0,
		state.units_of(opponent) as f32 / 30.0,
		triangles[0] / 10.0,
		triangles[1] / 10.0,
		state.live.len() as f32 / 40.0,
		1.0,
	];
	let mut neighbors = Neighbors {
		flat: Vec::with_capacity(count * 4),
		starts: Vec::with_capacity(count + 1),
	};
	neighbors.starts.push(0);
	for n in 0..count {
		neighbors
			.flat
			.extend(nodes_of(state.adjacency[n]).take(MAX_DEGREE));
		neighbors.starts.push(neighbors.flat.len());
	}
	let usable: Vec<usize> = (state.owner[player] & state.initial_owner[player] & state.live)
		.iter()
		.collect();
	let enemy = occupied(state, opponent, count);
	let corners = usable.iter().map(|&t| map.triangles[t]).collect();
	let attack_targets = usable
		.iter()
		.map(|&t| {
			let corners = map.corners(t);
			if nodes_of(corners).all(|c| units(c, player) >= 1) {
				around(state, corners) & enemy
			} else {
				0
			}
		})
		.collect();
	Obs {
		nodes,
		globals,
		neighbors,
		usable,
		corners,
		attack_targets,
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

fn half(bits: u16) -> f32 {
	let sign = if bits >> 15 != 0 { -1.0 } else { 1.0 };
	let exponent = i32::from((bits >> 10) & 0x1F);
	let mantissa = f32::from(bits & 0x3FF);
	match exponent {
		0 => sign * mantissa * 2f32.powi(-24),
		31 => sign * f32::INFINITY,
		_ => sign * (1.0 + mantissa / 1024.0) * 2f32.powi(exponent - 15),
	}
}

struct Reader {
	bytes: Vec<u8>,
	at: usize,
}

impl Reader {
	fn u32(&mut self) -> u32 {
		let value = u32::from_le_bytes(self.bytes[self.at..self.at + 4].try_into().unwrap());
		self.at += 4;
		value
	}

	fn floats(&mut self, count: usize) -> Vec<f32> {
		let out = (0..count)
			.map(|i| {
				let at = self.at + 2 * i;
				half(u16::from_le_bytes([self.bytes[at], self.bytes[at + 1]]))
			})
			.collect();
		self.at += 2 * count;
		out
	}

	fn dense(&mut self, outputs: usize, inputs: usize, bias: bool) -> Dense {
		let weights = self.floats(outputs * inputs);
		let bias = if bias {
			self.floats(outputs)
		} else {
			vec![0.0; outputs]
		};
		Dense::new(inputs, weights, bias)
	}
}

struct Dense {
	inputs: usize,
	weights: Vec<f32>,
	transposed: Vec<f32>,
	bias: Vec<f32>,
}

impl Dense {
	fn new(inputs: usize, weights: Vec<f32>, bias: Vec<f32>) -> Dense {
		let outputs = bias.len();
		let mut transposed = vec![0f32; inputs * outputs];
		for o in 0..outputs {
			for i in 0..inputs {
				transposed[i * outputs + o] = weights[o * inputs + i];
			}
		}
		Dense {
			inputs,
			weights,
			transposed,
			bias,
		}
	}

	fn apply(&self, input: &[f32], out: &mut [f32]) {
		for (o, slot) in out.iter_mut().enumerate() {
			let row = &self.weights[o * self.inputs..(o + 1) * self.inputs];
			*slot = self.bias[o] + dot(row, input);
		}
	}

	fn apply_rows(&self, input: &[f32], out: &mut [f32]) {
		self.apply_rows_with(input, self.inputs, &self.bias, out);
	}

	fn apply_rows_with(&self, input: &[f32], inputs: usize, bias: &[f32], out: &mut [f32]) {
		#[cfg(target_arch = "x86_64")]
		if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
			unsafe { self.apply_rows_avx(input, inputs, bias, out) };
			return;
		}
		self.apply_rows_plain(input, inputs, bias, out);
	}

	#[cfg(target_arch = "x86_64")]
	#[target_feature(enable = "avx2,fma")]
	unsafe fn apply_rows_avx(&self, input: &[f32], inputs: usize, bias: &[f32], out: &mut [f32]) {
		self.apply_rows_plain(input, inputs, bias, out);
	}

	#[inline(always)]
	fn apply_rows_plain(&self, input: &[f32], inputs: usize, bias: &[f32], out: &mut [f32]) {
		match bias.len() {
			32 => self.apply_rows_fixed::<32>(input, inputs, bias, out),
			48 => self.apply_rows_fixed::<48>(input, inputs, bias, out),
			64 => self.apply_rows_fixed::<64>(input, inputs, bias, out),
			_ => self.apply_rows_any(input, inputs, bias, out),
		}
	}

	#[inline(always)]
	fn apply_rows_fixed<const O: usize>(
		&self,
		input: &[f32],
		inputs: usize,
		bias: &[f32],
		out: &mut [f32],
	) {
		let bias: [f32; O] = bias[..O].try_into().unwrap();
		let pairs = input.len() / (2 * inputs);
		for (x, y) in input
			.chunks_exact(2 * inputs)
			.zip(out.chunks_exact_mut(2 * O))
		{
			let (x0, x1) = x.split_at(inputs);
			let mut acc0 = bias;
			let mut acc1 = bias;
			for ((&a, &b), column) in x0.iter().zip(x1).zip(self.transposed.chunks_exact(O)) {
				let column: &[f32; O] = column.try_into().unwrap();
				for k in 0..O {
					acc0[k] = a.mul_add(column[k], acc0[k]);
					acc1[k] = b.mul_add(column[k], acc1[k]);
				}
			}
			y[..O].copy_from_slice(&acc0);
			y[O..].copy_from_slice(&acc1);
		}
		let rest = &input[pairs * 2 * inputs..];
		let rest_out = &mut out[pairs * 2 * O..];
		for (x, y) in rest.chunks_exact(inputs).zip(rest_out.chunks_exact_mut(O)) {
			let mut acc = bias;
			for (&xi, column) in x.iter().zip(self.transposed.chunks_exact(O)) {
				let column: &[f32; O] = column.try_into().unwrap();
				for k in 0..O {
					acc[k] = xi.mul_add(column[k], acc[k]);
				}
			}
			y.copy_from_slice(&acc);
		}
	}

	#[inline(always)]
	fn apply_rows_any(&self, input: &[f32], inputs: usize, bias: &[f32], out: &mut [f32]) {
		let outputs = bias.len();
		for (x, y) in input
			.chunks_exact(inputs)
			.zip(out.chunks_exact_mut(outputs))
		{
			y.copy_from_slice(bias);
			for (&xi, column) in x.iter().zip(self.transposed.chunks_exact(outputs)) {
				if xi == 0.0 {
					continue;
				}
				for (yo, &wo) in y.iter_mut().zip(column) {
					*yo += xi * wo;
				}
			}
		}
	}
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
	let mut lanes = [0f32; 8];
	let chunks = a.len() / 8;
	for i in 0..chunks {
		for k in 0..8 {
			lanes[k] += a[i * 8 + k] * b[i * 8 + k];
		}
	}
	let mut total: f32 = lanes.iter().sum();
	for i in chunks * 8..a.len() {
		total += a[i] * b[i];
	}
	total
}

fn relu(values: &mut [f32]) {
	for v in values {
		*v = v.max(0.0);
	}
}

fn joint(a: &[f32], b: &[f32], extra: Option<&[f32]>, out: &[f32]) -> f32 {
	let mut total = 0.0;
	for k in 0..out.len() {
		let mut sum = a[k] + b[k];
		if let Some(extra) = extra {
			sum += extra[k];
		}
		total += sum.max(0.0) * out[k];
	}
	total
}

struct Round {
	own_and_mean: Dense,
}

impl Round {
	fn new(own: Dense, mean: Dense) -> Round {
		let width = own.bias.len();
		let weights = (0..width)
			.flat_map(|o| {
				let own_row = &own.weights[o * own.inputs..(o + 1) * own.inputs];
				let mean_row = &mean.weights[o * mean.inputs..(o + 1) * mean.inputs];
				own_row
					.iter()
					.chain(mean_row)
					.copied()
					.collect::<Vec<f32>>()
			})
			.collect();
		Round {
			own_and_mean: Dense::new(own.inputs + mean.inputs, weights, own.bias),
		}
	}
}

#[derive(Default)]
struct Scratch {
	hidden: Vec<f32>,
	both: Vec<f32>,
	delta: Vec<f32>,
	bias: Vec<f32>,
}

struct Net {
	width: usize,
	input: Dense,
	rounds: Vec<Round>,
	move_from: Dense,
	move_to: Dense,
	move_stay: Vec<f32>,
	move_out: Vec<f32>,
	triangle: Dense,
	hold: Vec<f32>,
	hold_bias: f32,
	spawn_triangle: Dense,
	spawn_corner: Dense,
	spawn_out: Vec<f32>,
	attack_triangle: Dense,
	attack_target: Dense,
	attack_out: Vec<f32>,
	value_hidden: Dense,
	value_out: Vec<f32>,
	value_bias: f32,
	scratch: RefCell<Scratch>,
}

struct Policy {
	moves: Vec<Vec<(usize, f32)>>,
	actions: Vec<Vec<(usize, f32)>>,
}

impl Net {
	fn load(bytes: Vec<u8>) -> Net {
		let mut r = Reader { bytes, at: 0 };
		assert_eq!(r.u32(), MAGIC, "comet weights start with the magic number");
		let features = r.u32() as usize;
		let globals = r.u32() as usize;
		let width = r.u32() as usize;
		let depth = r.u32() as usize;
		assert_eq!(features, NODE_FEATURES, "node feature count matches");
		assert_eq!(globals, GLOBAL_FEATURES, "global feature count matches");
		let input = r.dense(width, features + globals, true);
		let rounds = (0..depth)
			.map(|_| {
				let own = r.dense(width, width, true);
				let mean = r.dense(width, width, false);
				Round::new(own, mean)
			})
			.collect();
		let net = Net {
			width,
			input,
			rounds,
			move_from: r.dense(width, width, true),
			move_to: r.dense(width, width, false),
			move_stay: r.floats(width),
			move_out: r.floats(width),
			triangle: r.dense(width, 2 * width, true),
			hold: r.floats(width),
			hold_bias: r.floats(1)[0],
			spawn_triangle: r.dense(width, width, true),
			spawn_corner: r.dense(width, width, false),
			spawn_out: r.floats(width),
			attack_triangle: r.dense(width, width, true),
			attack_target: r.dense(width, width, false),
			attack_out: r.floats(width),
			value_hidden: r.dense(width, 2 * width + globals, true),
			value_out: r.floats(width),
			value_bias: r.floats(1)[0],
			scratch: RefCell::new(Scratch::default()),
		};
		assert_eq!(r.at, r.bytes.len(), "comet weights are fully read");
		net
	}

	fn trunk(&self, obs: &Obs, hidden: &mut Vec<f32>) {
		let w = self.width;
		let count = obs.nodes.len();
		let mut scratch = self.scratch.borrow_mut();
		let Scratch {
			both, delta, bias, ..
		} = &mut *scratch;
		bias.clear();
		bias.extend_from_slice(&self.input.bias);
		let global_columns = self.input.transposed[NODE_FEATURES * w..].chunks_exact(w);
		for (&g, column) in obs.globals.iter().zip(global_columns) {
			for (b, &c) in bias.iter_mut().zip(column) {
				*b += g * c;
			}
		}
		hidden.clear();
		hidden.resize(count * w, 0.0);
		self.input
			.apply_rows_with(obs.nodes.as_flattened(), NODE_FEATURES, bias, hidden);
		relu(hidden);
		both.clear();
		both.resize(count * 2 * w, 0.0);
		delta.clear();
		delta.resize(count * w, 0.0);
		for round in &self.rounds {
			for (n, row) in both.chunks_exact_mut(2 * w).enumerate() {
				let (own, pooled) = row.split_at_mut(w);
				own.copy_from_slice(&hidden[n * w..(n + 1) * w]);
				pooled.fill(0.0);
				let around = obs.neighbors.of(n);
				for &m in around {
					for (p, h) in pooled.iter_mut().zip(&hidden[m * w..(m + 1) * w]) {
						*p += h;
					}
				}
				if !around.is_empty() {
					let scale = 1.0 / around.len() as f32;
					pooled.iter_mut().for_each(|p| *p *= scale);
				}
			}
			round.own_and_mean.apply_rows(both, delta);
			for (h, &d) in hidden.iter_mut().zip(delta.iter()) {
				*h += d.max(0.0);
			}
		}
	}

	fn value(&self, obs: &Obs) -> f32 {
		let mut hidden = std::mem::take(&mut self.scratch.borrow_mut().hidden);
		self.trunk(obs, &mut hidden);
		let value = self.value_from(obs, &hidden);
		self.scratch.borrow_mut().hidden = hidden;
		value
	}

	fn value_from(&self, obs: &Obs, hidden: &[f32]) -> f32 {
		let w = self.width;
		let count = obs.nodes.len();
		let mut joined = vec![0f32; 2 * w + GLOBAL_FEATURES];
		for k in w..2 * w {
			joined[k] = f32::NEG_INFINITY;
		}
		for n in 0..count {
			for k in 0..w {
				let h = hidden[n * w + k];
				joined[k] += h / count as f32;
				joined[w + k] = joined[w + k].max(h);
			}
		}
		joined[2 * w..].copy_from_slice(&obs.globals);
		let mut layer = vec![0f32; w];
		self.value_hidden.apply(&joined, &mut layer);
		relu(&mut layer);
		dot(&layer, &self.value_out) + self.value_bias
	}

	fn policy(&self, obs: &Obs, state: &State, player: usize) -> Policy {
		let w = self.width;
		let mut hidden = Vec::new();
		self.trunk(obs, &mut hidden);
		let count = obs.nodes.len();
		let node = |n: usize| &hidden[n * w..(n + 1) * w];
		let mut from = vec![0f32; count * w];
		let mut to = vec![0f32; count * w];
		for n in 0..count {
			if state.units[n][player] > 0 {
				self.move_from.apply(node(n), &mut from[n * w..(n + 1) * w]);
			}
			self.move_to.apply(node(n), &mut to[n * w..(n + 1) * w]);
		}
		let moves = (0..count)
			.map(|v| {
				if state.units[v][player] == 0 {
					return Vec::new();
				}
				let source = &from[v * w..(v + 1) * w];
				let stay = joint(
					source,
					&to[v * w..(v + 1) * w],
					Some(&self.move_stay),
					&self.move_out,
				);
				let mut options: Vec<(usize, f32)> = vec![(v, stay)];
				options.extend(obs.neighbors.of(v).iter().map(|&t| {
					(
						t,
						joint(source, &to[t * w..(t + 1) * w], None, &self.move_out),
					)
				}));
				softmax(&mut options);
				options
			})
			.collect();
		let mut actions = Vec::new();
		let mut mean_max = vec![0f32; 2 * w];
		let mut summary = vec![0f32; w];
		let mut a = vec![0f32; w];
		let mut b = vec![0f32; w];
		for (index, corners) in obs.corners.iter().enumerate() {
			for k in 0..w {
				let values = corners.map(|c| hidden[c * w + k]);
				mean_max[k] = values.iter().sum::<f32>() / 3.0;
				mean_max[w + k] = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
			}
			self.triangle.apply(&mean_max, &mut summary);
			relu(&mut summary);
			let mut options = vec![(HOLD, dot(&summary, &self.hold) + self.hold_bias)];
			self.spawn_triangle.apply(&summary, &mut a);
			for (k, &c) in corners.iter().enumerate() {
				self.spawn_corner.apply(node(c), &mut b);
				options.push((1 + k, joint(&a, &b, None, &self.spawn_out)));
			}
			self.attack_triangle.apply(&summary, &mut a);
			for target in nodes_of(obs.attack_targets[index]) {
				self.attack_target.apply(node(target), &mut b);
				options.push((ATTACK + target, joint(&a, &b, None, &self.attack_out)));
			}
			softmax(&mut options);
			actions.push(options);
		}
		Policy { moves, actions }
	}
}

fn softmax(options: &mut [(usize, f32)]) {
	let top = options
		.iter()
		.map(|o| o.1)
		.fold(f32::NEG_INFINITY, f32::max);
	let mut total = 0.0;
	for o in options.iter_mut() {
		o.1 = (o.1 - top).exp();
		total += o.1;
	}
	for o in options.iter_mut() {
		o.1 /= total;
	}
}

fn tempered(policy: &Policy, temperature: f32) -> Policy {
	let reshape = |options: &Vec<(usize, f32)>| {
		let mut out: Vec<(usize, f32)> = options
			.iter()
			.map(|&(a, p)| (a, p.max(1e-9).powf(1.0 / temperature)))
			.collect();
		let total: f32 = out.iter().map(|o| o.1).sum();
		out.iter_mut().for_each(|o| o.1 /= total);
		out
	};
	Policy {
		moves: policy.moves.iter().map(reshape).collect(),
		actions: policy.actions.iter().map(reshape).collect(),
	}
}

fn plan(
	state: &State,
	player: usize,
	obs: &Obs,
	policy: &Policy,
	mut rng: Option<&mut Rng>,
) -> Vec<Command> {
	let mut reserved = [0i32; MAX_NODES];
	let mut orders = Vec::new();
	for (index, options) in policy.actions.iter().enumerate() {
		let t = obs.usable[index];
		let corners = obs.corners[index];
		let choice = match rng.as_deref_mut() {
			Some(rng) => {
				let weights: Vec<f32> = options.iter().map(|o| o.1).collect();
				options[rng.draw(&weights)].0
			}
			None => {
				options
					.iter()
					.max_by(|a, b| a.1.total_cmp(&b.1))
					.expect("hold is always an option")
					.0
			}
		};
		match choice {
			HOLD => {}
			k if k < ATTACK => orders.push(Command::Spawn {
				triangle: t,
				node: corners[k - 1],
			}),
			target => {
				let affordable = corners
					.iter()
					.all(|&c| state.units[c][player] - reserved[c] >= 1);
				if affordable {
					for &c in &corners {
						reserved[c] += 1;
					}
					orders.push(Command::Attack {
						triangle: t,
						target: target - ATTACK,
					});
				}
			}
		}
	}
	for (v, options) in policy.moves.iter().enumerate() {
		let units = state.units[v][player] - reserved[v];
		if options.is_empty() || units <= 0 {
			continue;
		}
		let mut counts = [0i32; MOVE_SLOTS];
		match rng.as_deref_mut() {
			Some(rng) => {
				let weights: Vec<f32> = options.iter().map(|o| o.1).collect();
				for _ in 0..units {
					counts[rng.draw(&weights)] += 1;
				}
			}
			None => {
				let mut shares: Vec<(usize, i32, f32)> = options
					.iter()
					.enumerate()
					.map(|(slot, &(_, p))| {
						let exact = p * units as f32;
						(slot, exact.floor() as i32, exact - exact.floor())
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
				for (slot, amount, _) in shares {
					counts[slot] += amount;
				}
			}
		}
		for (&(to, _), &amount) in options.iter().zip(&counts) {
			if to != v && amount > 0 {
				orders.push(Command::Move {
					from: v,
					to,
					amount,
				});
			}
		}
	}
	orders
}

struct Params {
	budget: Duration,
	replies: usize,
	worst_share: f32,
	unit_bonus: f32,
	triangle_bonus: f32,
	sampled: bool,
	reply_mode: u8,
	reply_share: f64,
	temperature: f32,
	verify_top: usize,
	verify_replies: usize,
	verify_share: f64,
	verify_keep: usize,
	max_evaluations: usize,
}

impl Params {
	fn load() -> Params {
		Params {
			budget: Duration::from_secs_f64(tuned("SEARCH_MS", DEFAULT_SEARCH_MS) / 1000.0),
			replies: tuned("REPLIES", 1.0) as usize,
			worst_share: tuned("WORST_SHARE", 0.0) as f32,
			unit_bonus: tuned("UNIT_BONUS", 0.0) as f32,
			triangle_bonus: tuned("TRIANGLE_BONUS", 0.0) as f32,
			sampled: tuned("SAMPLE", 0.0) > 0.0,
			reply_mode: tuned("REPLY_MODE", 0.0) as u8,
			reply_share: tuned("REPLY_SHARE", 0.25),
			temperature: tuned("TEMPERATURE", 0.5) as f32,
			verify_top: tuned("VERIFY_TOP", 8.0) as usize,
			verify_replies: tuned("VERIFY_REPLIES", 12.0) as usize,
			verify_share: tuned("VERIFY_SHARE", 0.35),
			verify_keep: tuned("VERIFY_KEEP", 1000.0) as usize,
			max_evaluations: tuned("EVALS", 1e12) as usize,
		}
	}
}

struct Search<'a> {
	map: &'a Map,
	net: &'a Net,
	params: &'a Params,
	root: &'a State,
	turn: i32,
	flip: bool,
	evaluations: usize,
}

impl Search<'_> {
	fn exhausted(&self, until: Instant, evaluations: usize) -> bool {
		self.evaluations >= evaluations || Instant::now() >= until
	}

	fn evaluate(&mut self, mine: &[Command], theirs: &[Command]) -> f32 {
		self.evaluate_as(ME, mine, theirs)
	}

	fn evaluate_as(&mut self, player: usize, mine: &[Command], theirs: &[Command]) -> f32 {
		self.evaluations += 1;
		let opponent = 1 - player;
		let mut next = *self.root;
		next.step(self.map, [mine, theirs]);
		let units = [player, opponent].map(|p| next.units_of(p) as f32);
		let eliminated = next.eliminated();
		match (eliminated[player], eliminated[opponent]) {
			(true, true) => return 0.0,
			(true, false) => return -100.0 + 0.001 * (units[0] - units[1]),
			(false, true) => return 100.0 + 0.001 * (units[0] - units[1]),
			(false, false) => {}
		}
		let flip = self.flip != (player == THEM);
		let obs = observe(self.map, &next, player, self.turn + 1, flip);
		let triangles = [player, opponent].map(|p| (next.owner[p] & next.live).len() as f32);
		self.net.value(&obs)
			+ self.params.unit_bonus * (units[0] - units[1])
			+ self.params.triangle_bonus * (triangles[0] - triangles[1])
	}

	fn best_reply(
		&mut self,
		mine: &[Command],
		theirs: &Obs,
		policy: &Policy,
		deadline: Instant,
		rng: &mut Rng,
	) -> Vec<Command> {
		let mut best = plan(self.root, THEM, theirs, policy, None);
		let mut best_value = self.evaluate_as(THEM, mine, &best);
		let mut seen = Seen::default();
		seen.insert(fingerprint(&best));
		let mut draws = 0;
		while Instant::now() < deadline && draws < 5_000 {
			draws += 1;
			let candidate = plan(self.root, THEM, theirs, policy, Some(rng));
			if !seen.insert(fingerprint(&candidate)) {
				continue;
			}
			let value = self.evaluate_as(THEM, mine, &candidate);
			if value > best_value {
				best_value = value;
				best = candidate;
			}
		}
		best
	}

	fn score(&mut self, mine: &[Command], replies: &[Vec<Command>]) -> f32 {
		let values: Vec<f32> = replies.iter().map(|r| self.evaluate(mine, r)).collect();
		let mean = values.iter().sum::<f32>() / values.len() as f32;
		let worst = values.iter().copied().fold(f32::INFINITY, f32::min);
		mean + self.params.worst_share * (worst - mean)
	}

	fn run(&mut self, started: Instant, rng: &mut Rng) -> Vec<Command> {
		let deadline = started + self.params.budget;
		let mine = observe(self.map, self.root, ME, self.turn, self.flip);
		let theirs = observe(self.map, self.root, THEM, self.turn, !self.flip);
		let my_policy = self.net.policy(&mine, self.root, ME);
		let their_policy = self.net.policy(&theirs, self.root, THEM);
		let greedy = plan(self.root, ME, &mine, &my_policy, None);
		let mut replies = Vec::new();
		if self.params.reply_mode != 1 {
			replies.push(plan(self.root, THEM, &theirs, &their_policy, None));
		}
		if self.params.reply_mode != 0 {
			let reply_deadline =
				Instant::now() + self.params.budget.mul_f64(self.params.reply_share);
			let reply = self.best_reply(&greedy, &theirs, &their_policy, reply_deadline, rng);
			if !replies.contains(&reply) {
				replies.push(reply);
			}
		}
		let their_sampling = tempered(&their_policy, self.params.temperature);
		let mut attempts = 0;
		while replies.len() < self.params.replies.max(1) && attempts < 60 {
			attempts += 1;
			let reply = plan(self.root, THEM, &theirs, &their_sampling, Some(rng));
			if !replies.contains(&reply) {
				replies.push(reply);
			}
		}
		let sampling = tempered(&my_policy, self.params.temperature);
		let verify = self.params.verify_top;
		let sample_end = if verify > 1 {
			started + self.params.budget.mul_f64(1.0 - self.params.verify_share)
		} else {
			deadline
		};
		let cap = self.params.max_evaluations;
		let sample_cap = (cap as f64 * (1.0 - self.params.verify_share)) as usize;
		let first = self.score(&greedy, &replies);
		let mut top: Vec<(f32, Vec<Command>)> = vec![(first, greedy.clone())];
		let mut seen = Seen::default();
		seen.insert(fingerprint(&greedy));
		let mut draws = 0;
		while !self.exhausted(sample_end, sample_cap) && draws < 20_000 {
			draws += 1;
			let candidate = plan(self.root, ME, &mine, &sampling, Some(rng));
			if !seen.insert(fingerprint(&candidate)) {
				continue;
			}
			let value = self.score(&candidate, &replies);
			let keep = verify.max(1);
			if top.len() < keep || value > top[top.len() - 1].0 {
				let at = top.partition_point(|entry| entry.0 >= value);
				top.insert(at, (value, candidate));
				top.truncate(keep);
			}
		}
		let mut field: Vec<(f32, f32, Vec<Command>)> = top
			.into_iter()
			.map(|(value, plan)| (value, 1.0, plan))
			.collect();
		let mut used = replies.clone();
		while field.len() > 1 && !self.exhausted(deadline, cap) {
			let mut extra: Vec<Vec<Command>> = Vec::new();
			let mut attempts = 0;
			while extra.len() < self.params.verify_replies && attempts < 60 {
				attempts += 1;
				let reply = plan(self.root, THEM, &theirs, &their_sampling, Some(rng));
				if !used.contains(&reply) {
					used.push(reply.clone());
					extra.push(reply);
				}
			}
			if extra.is_empty() {
				break;
			}
			let weight = extra.len() as f32;
			let mut scored = 0;
			for entry in field.iter_mut() {
				if self.exhausted(deadline, cap) {
					break;
				}
				let more = self.score(&entry.2, &extra);
				entry.0 = (entry.0 * entry.1 + more * weight) / (entry.1 + weight);
				entry.1 += weight;
				scored += 1;
			}
			field.truncate(scored.max(1));
			field.sort_by(|a, b| b.0.total_cmp(&a.0));
			let keep = (field.len() / self.params.verify_keep.max(1)).max(1);
			field.truncate(keep);
		}
		let best_value = field[0].0;
		let best = field[0].2.clone();
		if std::env::var_os("COMET_DEBUG").is_some() {
			let (turn, evaluations, unique) = (self.turn, self.evaluations, seen.len());
			eprintln!(
				"turn {turn} value {best_value:.3} evaluations {evaluations} unique {unique} draws {draws}"
			);
		}
		best
	}
}

fn text(map: &Map, command: Command) -> String {
	let ordered = |triangle: usize, first: usize| {
		let rest: Vec<usize> = map.triangles[triangle]
			.into_iter()
			.filter(|&c| c != first)
			.collect();
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

fn answer(map: &Map, commands: &[Command]) -> String {
	if commands.is_empty() {
		return "WAIT".to_string();
	}
	commands
		.iter()
		.map(|&command| text(map, command))
		.collect::<Vec<_>>()
		.join(";")
}

struct Labels {
	moves: [[i16; MOVE_SLOTS]; RECORD_NODES],
	actions: Vec<(i8, i8)>,
}

fn labels(map: &Map, state: &State, obs: &Obs, line: &str) -> Labels {
	let mut moves = [[0i16; MOVE_SLOTS]; RECORD_NODES];
	let mut actions: Vec<(i8, i8)> = vec![(HOLD as i8, -1); obs.usable.len()];
	let mut left: Vec<i32> = (0..map.len()).map(|n| state.units[n][ME]).collect();
	for command in line.split(';') {
		let words: Vec<&str> = command.split_whitespace().collect();
		let numbers: Vec<usize> = words
			.iter()
			.skip(1)
			.filter_map(|w| w.parse().ok())
			.collect();
		let kind = words.first().map(|w| w.to_ascii_uppercase());
		match kind.as_deref() {
			Some("MOVE") if numbers.len() == 3 => {
				let [from, to, amount] = [numbers[0], numbers[1], numbers[2]];
				if from >= map.len() || to >= map.len() || from == to {
					continue;
				}
				let Some(step) = state.first_step(from, to) else {
					continue;
				};
				let Some(slot) = obs.neighbors.of(from).iter().position(|&n| n == step) else {
					continue;
				};
				let amount = (amount.min(i32::MAX as usize) as i32).min(left[from]);
				left[from] -= amount;
				moves[from][slot + 1] += amount as i16;
			}
			Some(kind @ ("SPAWN" | "ATTACK")) if numbers.len() >= 3 => {
				if numbers[..3].iter().any(|&n| n >= map.len()) {
					continue;
				}
				let Some(t) = map.triangle([numbers[0], numbers[1], numbers[2]]) else {
					continue;
				};
				let Some(index) = obs.usable.iter().position(|&u| u == t) else {
					continue;
				};
				if kind == "SPAWN" {
					let k = obs.corners[index]
						.iter()
						.position(|&c| c == numbers[0])
						.expect("the spawn house is a corner");
					actions[index] = ((1 + k) as i8, -1);
				} else if numbers.len() == 4
					&& numbers[3] < map.len()
					&& obs.attack_targets[index] & bit(numbers[3]) != 0
				{
					actions[index] = (ATTACK as i8, numbers[3] as i8);
				}
			}
			_ => {}
		}
	}
	for (n, &rest) in left.iter().enumerate() {
		moves[n][0] = rest.max(0) as i16;
	}
	Labels { moves, actions }
}

fn write_record(
	out: &mut impl Write,
	state: &State,
	obs: &Obs,
	turn: i32,
	labels: &Labels,
	source: i32,
) {
	let count = obs.nodes.len();
	let mut bytes: Vec<u8> = Vec::with_capacity(12_000);
	let usable_count = obs.usable.len().min(MAX_USABLE);
	for value in [turn, count as i32, usable_count as i32, source] {
		bytes.extend(value.to_le_bytes());
	}
	for n in 0..RECORD_NODES {
		for k in 0..NODE_FEATURES {
			let value = obs.nodes.get(n).map_or(0.0, |row| row[k]);
			bytes.extend(value.to_le_bytes());
		}
	}
	for value in obs.globals {
		bytes.extend(value.to_le_bytes());
	}
	for n in 0..RECORD_NODES {
		for k in 0..MAX_DEGREE {
			let value = (n < count)
				.then(|| obs.neighbors.of(n))
				.and_then(|list| list.get(k))
				.map_or(-1, |&m| m as i8);
			bytes.push(value as u8);
		}
	}
	for n in 0..RECORD_NODES {
		let units = if n < count { state.units[n][ME] } else { 0 };
		bytes.extend((units as i16).to_le_bytes());
	}
	for row in &labels.moves {
		for value in row {
			bytes.extend(value.to_le_bytes());
		}
	}
	for index in 0..MAX_USABLE {
		if index < usable_count {
			for c in obs.corners[index] {
				bytes.push(c as u8);
			}
			let (label, target) = labels.actions[index];
			bytes.push(label as u8);
			bytes.push(target as u8);
			bytes.extend([0u8; 3]);
			bytes.extend(obs.attack_targets[index].to_le_bytes());
		} else {
			bytes.extend([255u8; 5]);
			bytes.extend([0u8; 3]);
			bytes.extend(0u64.to_le_bytes());
		}
	}
	out.write_all(&bytes).expect("the record file is writable");
}

struct Input<R: BufRead> {
	lines: std::io::Lines<R>,
	transcript: String,
}

impl<R: BufRead> Input<R> {
	fn line(&mut self) -> Option<Vec<i64>> {
		let line = self.lines.next()?.expect("stdin is readable");
		self.transcript.push_str(&line);
		self.transcript.push('\n');
		Some(
			line.split_whitespace()
				.map(|field| field.parse().expect("the referee sends integers"))
				.collect(),
		)
	}

	fn ints(&mut self) -> Vec<i64> {
		self.line().unwrap_or_else(|| std::process::exit(0))
	}

	fn int(&mut self) -> i64 {
		self.ints()[0]
	}

	fn take_transcript(&mut self) -> String {
		std::mem::take(&mut self.transcript)
	}
}

struct Shadow {
	child: std::process::Child,
	answers: std::io::Lines<std::io::BufReader<std::process::ChildStdout>>,
}

impl Shadow {
	fn launch(command: &str) -> Self {
		let mut child = std::process::Command::new("sh")
			.arg("-c")
			.arg(command)
			.stdin(std::process::Stdio::piped())
			.stdout(std::process::Stdio::piped())
			.spawn()
			.expect("the shadowed bot starts");
		let stdout = child.stdout.take().expect("the shadowed bot has a stdout");
		Shadow {
			child,
			answers: std::io::BufReader::new(stdout).lines(),
		}
	}

	fn ask(&mut self, text: &str) -> String {
		let stdin = self
			.child
			.stdin
			.as_mut()
			.expect("the shadowed bot has a stdin");
		stdin
			.write_all(text.as_bytes())
			.expect("the shadowed bot reads");
		stdin.flush().expect("the shadowed bot reads");
		match self.answers.next() {
			Some(Ok(line)) => line,
			_ => std::process::exit(0),
		}
	}
}

fn read_state<R: BufRead>(input: &mut Input<R>, map: &Map) -> Option<(State, Instant)> {
	let scores = input.line()?;
	let started = Instant::now();
	let count = map.len();
	let mut units = vec![[0; 2]; count];
	for _ in 0..count {
		let row = input.ints();
		units[row[0] as usize] = [row[1] as i32, row[2] as i32];
	}
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
			0 => Some(ME),
			1 => Some(THEM),
			_ => None,
		};
		state.set_triangle(triangle, owner, [row[4] != 0, row[5] != 0]);
	}
	for _ in 0..input.int() {
		input.ints();
	}
	state.score = [scores[0] as i32, scores[1] as i32];
	Some((state, started))
}

fn flipped(map: &Map, state: &State) -> bool {
	let mine: Vec<i32> = (0..map.len())
		.filter(|&n| state.units[n][ME] > 0)
		.map(|n| map.points[n].x)
		.collect();
	!mine.is_empty() && mine.iter().sum::<i32>() / mine.len() as i32 > 960
}

fn load_net() -> Option<Net> {
	if let Ok(path) = std::env::var("COMET_NET") {
		return Some(Net::load(std::fs::read(path).expect("the net file reads")));
	}
	(!NET_WEIGHTS.is_empty()).then(|| Net::load(unpack(NET_WEIGHTS)))
}

fn main() {
	let net = load_net();
	let params = Params::load();
	let mut input = Input {
		lines: std::io::stdin().lock().lines(),
		transcript: String::new(),
	};
	let mut shadow = std::env::var("COMET_SHADOW")
		.ok()
		.map(|c| Shadow::launch(&c));
	let recorded: Option<Vec<String>> = std::env::var("COMET_ANSWERS").ok().map(|path| {
		std::fs::read_to_string(path)
			.expect("the answers file reads")
			.lines()
			.map(str::to_string)
			.collect()
	});
	let mut extract = std::env::var("COMET_EXTRACT").ok().map(|path| {
		std::io::BufWriter::new(std::fs::File::create(path).expect("the record file opens"))
	});
	let mut record = std::env::var("COMET_RECORD").ok().map(|path| {
		std::io::BufWriter::new(std::fs::File::create(path).expect("the record file opens"))
	});
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
	let mut turn = 0;
	let mut flip = None;
	let mut rng = Rng(0x2545_F491_4F6C_DD1D);
	while let Some((state, started)) = read_state(&mut input, &map) {
		turn += 1;
		let flip = *flip.get_or_insert_with(|| flipped(&map, &state));
		if let (Some(extract), Some(recorded)) = (extract.as_mut(), recorded.as_ref()) {
			let Some(line) = recorded.get(turn as usize - 1) else {
				break;
			};
			let obs = observe(&map, &state, ME, turn, flip);
			let labels = labels(&map, &state, &obs, line);
			write_record(extract, &state, &obs, turn, &labels, 0);
			continue;
		}
		let transcript = input.take_transcript();
		let line = match (shadow.as_mut(), net.as_ref()) {
			(Some(shadow), _) => shadow.ask(&transcript),
			(None, Some(net)) if params.budget.is_zero() => {
				let obs = observe(&map, &state, ME, turn, flip);
				let policy = net.policy(&obs, &state, ME);
				let orders = plan(
					&state,
					ME,
					&obs,
					&policy,
					params.sampled.then_some(&mut rng),
				);
				answer(&map, &orders)
			}
			(None, Some(net)) => {
				let mut search = Search {
					map: &map,
					net,
					params: &params,
					root: &state,
					turn,
					flip,
					evaluations: 0,
				};
				let orders = search.run(started, &mut rng);
				answer(&map, &orders)
			}
			(None, None) => "WAIT".to_string(),
		};
		if let Some(record) = record.as_mut() {
			let obs = observe(&map, &state, ME, turn, flip);
			let labels = labels(&map, &state, &obs, &line);
			let source = if shadow.is_some() { 2 } else { 1 };
			write_record(record, &state, &obs, turn, &labels, source);
			record.flush().expect("the record file is writable");
		}
		writeln!(out, "{line}").expect("stdout is writable");
		out.flush().expect("stdout is writable");
	}
	if let Some(extract) = extract.as_mut() {
		extract.flush().expect("the record file is writable");
	}
}
