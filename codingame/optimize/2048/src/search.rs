#[allow(dead_code)]
#[path = "answer.rs"]
mod answer;

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use answer::{Board, MOVE_CHARS, Snake, next, oriented, play, ranking, spawn, start};

const RESULT: &str = ".2048_results.out";
const SEEDS: &str = ".2048_seeds.in";
const RANK_COST: [i64; 4] = [0, 2, 3, 4];
const MAX_WIDTH: usize = 1 << 30;
const AFTER_GATE: usize = 60;

#[derive(Default)]
struct MixHasher(u64);

impl Hasher for MixHasher {
	fn finish(&self) -> u64 {
		self.0
	}

	fn write(&mut self, _: &[u8]) {
		unreachable!()
	}

	fn write_u128(&mut self, key: u128) {
		let folded = (key as u64) ^ ((key >> 64) as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
		self.0 = folded.wrapping_mul(0xFF51_AFD7_ED55_8CCD).rotate_left(29);
	}
}

type BoardSet = HashSet<u128, BuildHasherDefault<MixHasher>>;

fn key(board: &Board) -> u128 {
	board.iter().fold(0, |k, &v| k << 5 | v as u128)
}

struct Config {
	width: usize,
	lambda: i64,
	empty: i64,
	window: usize,
	shift: u32,
	endgame: usize,
	cap: usize,
	floor: u8,
	threads: usize,
	symmetries: Vec<usize>,
	from: usize,
	prefix: usize,
	noise: u64,
	salts: u64,
	result: String,
	seeds: Vec<u64>,
}

fn parse() -> Config {
	let mut config = Config {
		width: 3000,
		lambda: 100_000_000,
		empty: 0,
		window: 0,
		shift: 1,
		endgame: 0,
		cap: 2_000_000,
		floor: 32,
		threads: 6,
		symmetries: vec![0],
		from: 0,
		prefix: 0,
		noise: 0,
		salts: 1,
		result: RESULT.to_string(),
		seeds: Vec::new(),
	};
	let mut args = std::env::args().skip(1);
	while let Some(arg) = args.next() {
		let mut value = || args.next().expect("flag without value");
		match arg.as_str() {
			"--width" => config.width = value().parse().unwrap(),
			"--lambda" => config.lambda = value().parse().unwrap(),
			"--threads" => config.threads = value().parse().unwrap(),
			"--empty" => config.empty = value().parse().unwrap(),
			"--window" => config.window = value().parse().unwrap(),
			"--shift" => config.shift = value().parse().unwrap(),
			"--endgame" => config.endgame = value().parse().unwrap(),
			"--cap" => config.cap = value().parse().unwrap(),
			"--floor" => config.floor = value().parse().unwrap(),
			"--result" => config.result = value(),
			"--prefix" => config.prefix = value().parse().unwrap(),
			"--from" => config.from = value().parse().unwrap(),
			"--noise" => config.noise = value().parse().unwrap(),
			"--salts" => config.salts = value().parse().unwrap(),
			"--symmetry" => {
				config.symmetries = value().split(',').map(|s| s.parse().unwrap()).collect()
			}
			seed => config.seeds.push(seed.parse().unwrap()),
		}
	}
	assert!(
		config.width.max(config.cap) <= MAX_WIDTH,
		"width above {MAX_WIDTH}"
	);
	if config.seeds.is_empty() {
		config.seeds = std::fs::read_to_string(SEEDS)
			.unwrap()
			.split_whitespace()
			.map(|s| s.parse().unwrap())
			.collect();
	}
	config
}

struct Gate {
	depth: usize,
	tile: u8,
}

fn gates(seed0: u64) -> Vec<Gate> {
	let mut seed = seed0;
	let mut sums = Vec::new();
	let mut total = 0u64;
	for _ in 0..400_000 {
		total += if seed & 0x10 == 0 { 2 } else { 4 };
		sums.push(total);
		seed = next(seed);
	}
	let index: HashMap<u64, usize> = sums.iter().enumerate().map(|(i, &s)| (s, i)).collect();
	let mut fixed = 0u64;
	let mut out = Vec::new();
	for k in (2..=17).rev() {
		let gate = fixed + (1 << k) - 4;
		match index.get(&gate) {
			Some(&i) if sums[i + 1] == gate + 4 => {
				out.push(Gate {
					depth: i - 1,
					tile: k as u8,
				});
				fixed += 1 << k;
			}
			_ => break,
		}
	}
	out
}

fn shaped(board: &Board, snake: &Snake, shift: u32) -> i64 {
	snake
		.iter()
		.enumerate()
		.map(|(k, &i)| match board[i] {
			0 => 1,
			v => (1i64 << v) << (shift * (15 - k as u32)),
		})
		.sum()
}

fn chained(board: &Board, snake: &Snake, floor: u8) -> bool {
	let big = board.iter().filter(|&&v| v >= floor).count();
	snake[..big]
		.iter()
		.map(|&i| board[i])
		.try_fold(u8::MAX, |above, v| (v >= floor && v <= above).then_some(v))
		.is_some()
}

fn empty_count(board: &Board) -> u32 {
	board.iter().filter(|&&v| v == 0).count() as u32
}

#[derive(Clone, Copy)]
struct Node {
	board: Board,
	score: u32,
	penalty: i64,
}

struct Candidate {
	value: i64,
	node: Node,
	link: u32,
}

struct Outcome {
	score: u32,
	moves: Vec<usize>,
	symmetry: usize,
}

fn scatter(board: &Board, salt: u64, noise: u64) -> i64 {
	if noise == 0 {
		return 0;
	}
	let mixed = ((key(board) as u64)
		^ ((key(board) >> 64) as u64)
		^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15))
	.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
	((mixed ^ mixed >> 31) % (noise + 1)) as i64
}

fn stored(seed0: u64, result: &str) -> Option<(Vec<usize>, usize)> {
	let text = std::fs::read_to_string(result).ok()?;
	let line = text
		.lines()
		.find(|l| l.split_whitespace().next() == Some(&seed0.to_string()))?;
	let fields: Vec<&str> = line.split_whitespace().collect();
	let moves = fields[3]
		.chars()
		.map(|c| MOVE_CHARS.iter().position(|&m| m == c).unwrap())
		.collect();
	Some((moves, fields.get(4).map_or(0, |f| f.parse().unwrap())))
}

fn search(seed0: u64, symmetry: usize, salt: u64, config: &Config) -> Outcome {
	let gate_list = gates(seed0);
	let (board, mut seed) = start(seed0);
	let mut root = Node {
		board,
		score: 0,
		penalty: 0,
	};
	let mut trail: Vec<Vec<u32>> = Vec::new();
	let mut symmetry = symmetry;
	if config.from > 0 || config.prefix > 0 {
		let (moves, stored_symmetry) =
			stored(seed0, &config.result).expect("no stored game to restart from");
		symmetry = stored_symmetry;
		let restart = if config.prefix > 0 {
			config.prefix.min(moves.len())
		} else {
			gate_list
				.first()
				.map_or(0, |g| g.depth.saturating_sub(config.from))
		};
		for &mv in &moves[..restart] {
			let (mut after, gain) = play(&root.board, mv).unwrap();
			spawn(&mut after, seed);
			seed = next(seed);
			root.board = after;
			root.score += gain;
			trail.push(vec![mv as u32]);
		}
	}
	let snake = oriented(symmetry);
	let mut beam = vec![root];
	let mut best = (0u32, 0usize, 0usize);
	while !beam.is_empty() {
		let depth = trail.len() + 1;
		let bonus = if gate_list
			.iter()
			.any(|g| depth <= g.depth && depth + config.window >= g.depth)
		{
			config.empty
		} else {
			0
		};
		let width = if gate_list
			.iter()
			.any(|g| depth + config.endgame >= g.depth && depth <= g.depth + AFTER_GATE)
		{
			config.cap
		} else {
			config.width
		};
		let mut candidates = Vec::with_capacity(beam.len() * 4);
		for (i, node) in beam.iter().enumerate() {
			for (rank, step) in ranking(&node.board, seed, &snake).into_iter().enumerate() {
				let penalty = node.penalty + RANK_COST[rank];
				candidates.push(Candidate {
					value: shaped(&step.board, &snake, config.shift)
						+ scatter(&step.board, salt, config.noise)
						+ bonus * empty_count(&step.board) as i64
						- config.lambda * penalty,
					node: Node {
						board: step.board,
						score: node.score + step.gain,
						penalty,
					},
					link: (i << 2 | step.mv) as u32,
				});
			}
		}
		candidates.sort_unstable_by(|a, b| b.value.cmp(&a.value));
		let mut seen = BoardSet::default();
		let mut links = Vec::with_capacity(width);
		beam.clear();
		for c in candidates {
			if beam.len() == width {
				break;
			}
			if width == config.cap && !chained(&c.node.board, &snake, config.floor) {
				continue;
			}
			if seen.insert(key(&c.node.board)) {
				beam.push(c.node);
				links.push(c.link);
			}
		}
		trail.push(links);
		seed = next(seed);
		if width == config.cap && depth.is_multiple_of(20) {
			let state = beam.len();
			eprintln!("seed {seed0} symmetry {symmetry} depth {depth}: {state} state");
		}
		if let Some(gate) = gate_list.iter().find(|g| g.depth + AFTER_GATE == depth) {
			let tile = gate.tile;
			let state = beam.len();
			let passed = beam
				.iter()
				.filter(|n| n.board.iter().any(|&v| v >= tile))
				.count();
			let target = 1u32 << tile;
			eprintln!(
				"seed {seed0} symmetry {symmetry} depth {depth} after gate {target}: {passed} of {state} state pass"
			);
		}
		if let Some((i, node)) = beam.iter().enumerate().max_by_key(|(_, n)| n.score) {
			if node.score > best.0 {
				best = (node.score, depth, i);
			}
		}
	}
	let (score, len, mut index) = best;
	let mut moves = vec![0; len];
	for depth in (0..len).rev() {
		let link = trail[depth][index] as usize;
		moves[depth] = link & 3;
		index = link >> 2;
	}
	Outcome {
		score,
		moves,
		symmetry,
	}
}

fn record(seed0: u64, outcome: &Outcome, result: &str) -> bool {
	let text = std::fs::read_to_string(result).unwrap_or_default();
	let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
	let score = outcome.score;
	let line = format!(
		"{seed0} {} {score} {} {}",
		next(next(seed0)),
		outcome
			.moves
			.iter()
			.map(|&m| MOVE_CHARS[m])
			.collect::<String>(),
		outcome.symmetry
	);
	let field = |l: &str, i: usize| {
		l.split_whitespace()
			.nth(i)
			.and_then(|f| f.parse::<u64>().ok())
	};
	match lines.iter().position(|l| field(l, 0) == Some(seed0)) {
		Some(i) if field(&lines[i], 2) >= Some(score as u64) => return false,
		Some(i) => lines[i] = line,
		None => lines.push(line),
	}
	std::fs::write(result, lines.join("\n")).unwrap();
	true
}

fn main() {
	let config = parse();
	let jobs: Vec<(u64, usize, u64)> = config
		.seeds
		.iter()
		.flat_map(|&seed0| {
			config
				.symmetries
				.iter()
				.map(move |&symmetry| (seed0, symmetry))
		})
		.flat_map(|(seed0, symmetry)| (0..config.salts).map(move |salt| (seed0, symmetry, salt)))
		.collect();
	let cursor = AtomicUsize::new(0);
	let file = Mutex::new(());
	std::thread::scope(|scope| {
		for _ in 0..config.threads {
			scope.spawn(|| {
				while let Some(&(seed0, symmetry, salt)) =
					jobs.get(cursor.fetch_add(1, Ordering::Relaxed))
				{
					let outcome = search(seed0, symmetry, salt, &config);
					let _guard = file.lock().unwrap();
					let saved = record(seed0, &outcome, &config.result);
					let score = outcome.score;
					let move_count = outcome.moves.len();
					let symmetry = outcome.symmetry;
					eprintln!(
						"seed {seed0} symmetry {symmetry} salt {salt} score {score} move {move_count} saved {saved}"
					);
				}
			});
		}
	});
}
