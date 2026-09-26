#[allow(dead_code)]
#[path = "answer.rs"]
mod answer;

use std::collections::HashSet;
use std::fs::File;
use std::hash::{BuildHasherDefault, Hasher};
use std::io::Write;
use std::os::unix::fs::FileExt;

use answer::{Board, MOVE_CHARS, Snake, next, oriented, play, spawn, start};

const RESULT: &str = ".2048_results.out";
const LEVELS: usize = 18;
const MAX_TILE: usize = 17;
const SPAWN_HORIZON: usize = 300_000;

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

type KeySet = HashSet<u128, BuildHasherDefault<MixHasher>>;
type Multiset = [u8; LEVELS];

#[derive(PartialEq)]
enum Order {
	Hash,
	Adjacency,
}

struct Config {
	lead: usize,
	finish: usize,
	after: usize,
	cap: usize,
	floor: u8,
	gate: usize,
	salt: u64,
	order: Order,
	threads: usize,
	result: String,
	out: Option<String>,
	spill: String,
	seeds: Vec<u64>,
}

fn parse() -> Config {
	let mut config = Config {
		lead: 300,
		finish: 0,
		after: 40,
		cap: 25_000_000,
		floor: 32,
		gate: 0,
		salt: 0,
		order: Order::Hash,
		threads: 8,
		result: RESULT.to_string(),
		out: None,
		spill: std::env::temp_dir().to_string_lossy().into_owned(),
		seeds: Vec::new(),
	};
	let mut args = std::env::args().skip(1);
	while let Some(arg) = args.next() {
		let mut value = || args.next().expect("flag without value");
		match arg.as_str() {
			"--lead" => config.lead = value().parse().unwrap(),
			"--finish" => config.finish = value().parse().unwrap(),
			"--after" => config.after = value().parse().unwrap(),
			"--cap" => config.cap = value().parse().unwrap(),
			"--floor" => config.floor = value().parse().unwrap(),
			"--gate" => config.gate = value().parse().unwrap(),
			"--salt" => config.salt = value().parse().unwrap(),
			"--order" => {
				config.order = match value().as_str() {
					"hash" => Order::Hash,
					"adjacency" => Order::Adjacency,
					other => panic!("unknown order {other}"),
				}
			}
			"--threads" => config.threads = value().parse().unwrap(),
			"--result" => config.result = value(),
			"--out" => config.out = Some(value()),
			"--spill" => config.spill = value(),
			seed => config.seeds.push(seed.parse().unwrap()),
		}
	}
	config
}

fn pack(multiset: &Multiset) -> u128 {
	multiset.iter().fold(0, |k, &c| k << 5 | c as u128)
}

fn unpack(mut key: u128) -> Multiset {
	let mut multiset = [0; LEVELS];
	for count in multiset.iter_mut().rev() {
		*count = (key & 31) as u8;
		key >>= 5;
	}
	multiset
}

fn tile_count(multiset: &Multiset) -> u32 {
	multiset.iter().map(|&c| c as u32).sum()
}

fn multiset_of(board: &Board) -> Multiset {
	let mut multiset = [0; LEVELS];
	for &v in board.iter().filter(|&&v| v > 0) {
		multiset[v as usize] += 1;
	}
	multiset
}

fn merges(multiset: &Multiset, out: &mut Vec<Multiset>) {
	fn walk(level: usize, source: &Multiset, current: &mut Multiset, out: &mut Vec<Multiset>) {
		if level == MAX_TILE {
			out.push(*current);
			return;
		}
		for pairs in 0..=source[level] / 2 {
			current[level] -= 2 * pairs;
			current[level + 1] += pairs;
			walk(level + 1, source, current, out);
			current[level] += 2 * pairs;
			current[level + 1] -= pairs;
		}
	}
	let mut current = *multiset;
	walk(1, multiset, &mut current, out);
}

fn successors(multiset: &Multiset, spawn_level: usize, buffer: &mut Vec<Multiset>) -> Vec<u128> {
	buffer.clear();
	merges(multiset, buffer);
	buffer
		.iter()
		.filter(|merged| tile_count(merged) <= 15)
		.map(|merged| {
			let mut after = *merged;
			after[spawn_level] += 1;
			pack(&after)
		})
		.collect()
}

fn key(board: &Board) -> u128 {
	board.iter().fold(0, |k, &v| k << 5 | v as u128)
}

fn board_of(key: u128) -> Board {
	std::array::from_fn(|c| (key >> (5 * (15 - c)) & 31) as u8)
}

fn adjacency(board: &Board) -> i64 {
	let mut total = 0;
	for x in 0..4 {
		for y in 0..4 {
			let a = board[x * 4 + y];
			for (nx, ny) in [(x + 1, y), (x, y + 1)] {
				if a == 0 || nx == 4 || ny == 4 {
					continue;
				}
				let b = board[nx * 4 + ny];
				if b != 0 && a.abs_diff(b) <= 1 {
					total += 1i64 << a.max(b);
				}
			}
		}
	}
	total
}

fn chained(board: &Board, snake: &Snake, floor: u8) -> bool {
	let big = board.iter().filter(|&&v| v >= floor).count();
	snake[..big]
		.iter()
		.map(|&i| board[i])
		.try_fold(u8::MAX, |above, v| (v >= floor && v <= above).then_some(v))
		.is_some()
}

struct Timeline {
	spawn_levels: Vec<usize>,
	gates: Vec<(usize, usize)>,
	horizon: usize,
}

fn timeline(seed0: u64) -> Timeline {
	let mut seed = seed0;
	let mut total = 0u64;
	let mut spawn_levels = Vec::with_capacity(SPAWN_HORIZON);
	let mut sums = Vec::with_capacity(SPAWN_HORIZON);
	for _ in 0..SPAWN_HORIZON {
		let level = if seed & 0x10 == 0 { 1 } else { 2 };
		total += 1 << level;
		spawn_levels.push(level);
		sums.push(total);
		seed = next(seed);
	}
	let mut fixed = 0u64;
	let mut gates = Vec::new();
	let mut horizon = SPAWN_HORIZON;
	for tile in (2..=MAX_TILE).rev() {
		let target = fixed + (1 << tile) - 4;
		match sums.iter().position(|&s| s == target) {
			Some(i) if sums[i + 1] == target + 4 => {
				gates.push((i - 1, tile));
				fixed += 1 << tile;
			}
			_ => {
				let ceiling = fixed + (1 << tile) - 2;
				horizon = sums.partition_point(|&s| s <= ceiling) - 2;
				break;
			}
		}
	}
	Timeline {
		spawn_levels,
		gates,
		horizon,
	}
}

fn reachable(start: &Multiset, first: usize, end: usize, spawn_levels: &[usize]) -> Vec<KeySet> {
	let mut buffer = Vec::new();
	let mut forward = vec![KeySet::default(); end - first + 2];
	forward[0].insert(pack(start));
	for depth in first..=end {
		let i = depth - first;
		let reached: KeySet = forward[i]
			.iter()
			.flat_map(|&k| successors(&unpack(k), spawn_levels[depth + 2], &mut buffer))
			.collect();
		forward[i + 1] = reached;
	}
	forward
}

fn alive(
	forward: &[KeySet],
	first: usize,
	spawn_levels: &[usize],
	goal: impl Fn(u128) -> bool,
) -> Vec<KeySet> {
	let mut buffer = Vec::new();
	let mut good = vec![KeySet::default(); forward.len()];
	let last = forward.len() - 1;
	let end = first + last - 1;
	good[last] = forward[last].iter().copied().filter(|&k| goal(k)).collect();
	for depth in (first..=end).rev() {
		let i = depth - first;
		let alive: KeySet = forward[i]
			.iter()
			.copied()
			.filter(|&k| {
				successors(&unpack(k), spawn_levels[depth + 2], &mut buffer)
					.iter()
					.any(|s| good[i + 1].contains(s))
			})
			.collect();
		good[i] = alive;
	}
	good
}

const LINK_BITS: u32 = 48;

fn entry_key(entry: u128) -> u128 {
	entry >> LINK_BITS
}

fn entry_link(entry: u128) -> u32 {
	entry as u32
}

fn expand(
	beam: &[Board],
	seed: u64,
	keep: &(dyn Fn(&Board) -> bool + Sync),
	threads: usize,
) -> Vec<u128> {
	let chunk = beam.len().div_ceil(threads).max(1);
	let mut buckets: Vec<Vec<Vec<u128>>> = std::thread::scope(|scope| {
		let handles: Vec<_> = beam
			.chunks(chunk)
			.enumerate()
			.map(|(c, slice)| {
				scope.spawn(move || {
					let mut out = vec![Vec::new(); threads];
					for (offset, board) in slice.iter().enumerate() {
						let parent = c * chunk + offset;
						for mv in 0..4 {
							let Some((mut after, _)) = play(board, mv) else {
								continue;
							};
							spawn(&mut after, seed);
							if keep(&after) {
								let k = key(&after);
								let spread = ((k as u64) ^ ((k >> 64) as u64))
									.wrapping_mul(0x9E37_79B9_7F4A_7C15);
								out[(spread >> 58) as usize % threads]
									.push(k << LINK_BITS | (parent << 2 | mv) as u128);
							}
						}
					}
					out
				})
			})
			.collect();
		handles.into_iter().map(|h| h.join().unwrap()).collect()
	});
	let mut parts: Vec<Vec<u128>> = (0..threads)
		.map(|t| {
			let mut part = Vec::with_capacity(buckets.iter().map(|b| b[t].len()).sum());
			for bucket in buckets.iter_mut() {
				part.append(&mut bucket[t]);
			}
			part
		})
		.collect();
	drop(buckets);
	std::thread::scope(|scope| {
		for part in parts.iter_mut() {
			scope.spawn(move || {
				part.sort_unstable();
				part.dedup_by_key(|e| entry_key(*e));
			});
		}
	});
	let mut entries = Vec::with_capacity(parts.iter().map(Vec::len).sum());
	for part in parts.iter_mut() {
		entries.append(part);
	}
	entries
}

fn truncate(entries: &mut Vec<u128>, config: &Config) {
	if entries.len() <= config.cap {
		return;
	}
	let rate = |k: u128| match config.order {
		Order::Hash => {
			let mixed =
				((k as u64) ^ ((k >> 64) as u64) ^ config.salt).wrapping_mul(0xFF51_AFD7_ED55_8CCD);
			mixed ^ mixed >> 29
		}
		Order::Adjacency => u64::MAX - adjacency(&board_of(k)) as u64,
	};
	let mut rated: Vec<(u64, u32)> = entries
		.iter()
		.enumerate()
		.map(|(n, &e)| (rate(entry_key(e)), n as u32))
		.collect();
	rated.select_nth_unstable(config.cap);
	rated.truncate(config.cap);
	rated.sort_unstable_by_key(|r| r.1);
	*entries = rated.iter().map(|r| entries[r.1 as usize]).collect();
}

struct Stored {
	moves: Vec<usize>,
	symmetry: usize,
}

fn stored(seed0: u64, result: &str) -> Stored {
	let text = std::fs::read_to_string(result).unwrap();
	let line = text
		.lines()
		.find(|l| l.split_whitespace().next() == Some(&seed0.to_string()))
		.unwrap_or_else(|| panic!("seed {seed0}: no stored game in {result}"));
	let fields: Vec<&str> = line.split_whitespace().collect();
	Stored {
		moves: fields[3]
			.chars()
			.map(|c| MOVE_CHARS.iter().position(|&m| m == c).unwrap())
			.collect(),
		symmetry: fields.get(4).map_or(0, |f| f.parse().unwrap()),
	}
}

fn solve(seed0: u64, config: &Config) -> Option<(Vec<usize>, usize)> {
	let Timeline {
		spawn_levels,
		gates,
		..
	} = timeline(seed0);
	let &(last, tile) = gates.get(config.gate)?;
	let target = 1u32 << tile;
	let first = last - config.lead;
	let end = last + config.after;
	let game = stored(seed0, &config.result);
	let snake = oriented(game.symmetry);
	let (mut board, mut seed) = start(seed0);
	for &mv in &game.moves[..first] {
		board = play(&board, mv).unwrap().0;
		spawn(&mut board, seed);
		seed = next(seed);
	}
	let forward = reachable(&multiset_of(&board), first, end, &spawn_levels);
	let good = alive(&forward, first, &spawn_levels, |k| unpack(k)[tile] > 0);
	drop(forward);
	let good_peak = good.iter().map(KeySet::len).max().unwrap_or(0);
	eprintln!("seed {seed0} gate {target} at move {last}: good multiset peak {good_peak}");
	let mut beam = vec![board];
	let mut trail = Trail::new(&config.spill, seed0);
	for depth in first..=end {
		let good_next = &good[depth - first + 1];
		let keep = |b: &Board| {
			(depth > last || chained(b, &snake, config.floor))
				&& good_next.contains(&pack(&multiset_of(b)))
		};
		let mut entries = expand(&beam, seed, &keep, config.threads);
		let state = entries.len();
		truncate(&mut entries, config);
		seed = next(seed);
		if (depth - first).is_multiple_of(20) {
			let remaining = last as i64 - depth as i64;
			eprintln!("seed {seed0} move {depth} ({remaining} to gate): {state} board");
		}
		beam = entries.iter().map(|&e| board_of(entry_key(e))).collect();
		trail.push(&entries);
		drop(entries);
		if let Some(winner) = beam.iter().position(|b| b.contains(&(tile as u8))) {
			let mut moves = game.moves[..first].to_vec();
			moves.extend(trail.backtrack(winner));
			return Some((moves, game.symmetry));
		}
		if beam.is_empty() {
			break;
		}
	}
	None
}

struct Trail {
	file: File,
	offsets: Vec<u64>,
	length: u64,
}

impl Trail {
	fn new(dir: &str, seed0: u64) -> Self {
		let path = format!("{dir}/gate-{seed0}-{}.links", std::process::id());
		let file = File::options()
			.read(true)
			.write(true)
			.create(true)
			.truncate(true)
			.open(&path)
			.unwrap();
		std::fs::remove_file(&path).unwrap();
		Self {
			file,
			offsets: Vec::new(),
			length: 0,
		}
	}

	fn push(&mut self, entries: &[u128]) {
		let bytes: Vec<u8> = entries
			.iter()
			.flat_map(|&e| entry_link(e).to_le_bytes())
			.collect();
		self.file.write_all(&bytes).unwrap();
		self.offsets.push(self.length);
		self.length += bytes.len() as u64;
	}

	fn link(&self, step: usize, index: usize) -> usize {
		let mut bytes = [0u8; 4];
		self.file
			.read_exact_at(&mut bytes, self.offsets[step] + 4 * index as u64)
			.unwrap();
		u32::from_le_bytes(bytes) as usize
	}

	fn backtrack(&self, mut index: usize) -> Vec<usize> {
		let mut path = vec![0; self.offsets.len()];
		for step in (0..self.offsets.len()).rev() {
			let link = self.link(step, index);
			path[step] = link & 3;
			index = link >> 2;
		}
		path
	}
}

fn multiset_value(multiset: &Multiset) -> i64 {
	multiset
		.iter()
		.enumerate()
		.map(|(v, &c)| c as i64 * (((v as i64) - 1) << v))
		.sum()
}

fn finish(seed0: u64, config: &Config) -> Option<(Vec<usize>, usize)> {
	let Timeline {
		spawn_levels,
		horizon,
		..
	} = timeline(seed0);
	let game = stored(seed0, &config.result);
	let first = game.moves.len().saturating_sub(config.finish);
	let end = horizon - 1;
	let (mut board, mut seed) = start(seed0);
	for &mv in &game.moves[..first] {
		board = play(&board, mv).unwrap().0;
		spawn(&mut board, seed);
		seed = next(seed);
	}
	let forward = reachable(&multiset_of(&board), first, end, &spawn_levels);
	let top = forward
		.last()?
		.iter()
		.map(|&k| multiset_value(&unpack(k)))
		.max()?;
	let good = alive(&forward, first, &spawn_levels, |k| {
		multiset_value(&unpack(k)) == top
	});
	drop(forward);
	if good[0].is_empty() {
		eprintln!("seed {seed0}: horizon {horizon} unreachable from move {first}");
		return None;
	}
	let mut beam = vec![board];
	let mut trail = Trail::new(&config.spill, seed0);
	for depth in first..=end {
		let good_next = &good[depth - first + 1];
		let keep = |b: &Board| good_next.contains(&pack(&multiset_of(b)));
		let mut entries = expand(&beam, seed, &keep, config.threads);
		let state = entries.len();
		if (depth - first).is_multiple_of(50) || state > config.cap {
			eprintln!("seed {seed0} move {depth}: {state} board");
		}
		if entries.is_empty() {
			eprintln!("seed {seed0}: no board left at move {depth}");
			break;
		}
		truncate(&mut entries, config);
		seed = next(seed);
		beam = entries.iter().map(|&e| board_of(entry_key(e))).collect();
		trail.push(&entries);
		drop(entries);
	}
	let best = beam
		.iter()
		.map(multiset_of)
		.enumerate()
		.max_by_key(|(_, m)| multiset_value(m))?
		.0;
	let mut moves = game.moves[..first].to_vec();
	moves.extend(trail.backtrack(best));
	Some((moves, game.symmetry))
}

fn replay_score(seed0: u64, moves: &[usize]) -> u32 {
	let (mut board, mut seed) = start(seed0);
	let mut score = 0;
	for &mv in moves {
		let (mut after, gain) = play(&board, mv).unwrap();
		spawn(&mut after, seed);
		seed = next(seed);
		score += gain;
		board = after;
	}
	score
}

fn write(path: &str, seed0: u64, line: String) {
	let text = std::fs::read_to_string(path).unwrap_or_default();
	let mut lines: Vec<String> = text
		.lines()
		.filter(|l| l.split_whitespace().next() != Some(&seed0.to_string()))
		.map(str::to_string)
		.collect();
	lines.push(line);
	std::fs::write(path, lines.join("\n")).unwrap();
}

fn main() {
	let config = parse();
	for &seed0 in &config.seeds {
		let found = if config.finish > 0 {
			finish(seed0, &config)
		} else {
			solve(seed0, &config)
		};
		let Some((moves, symmetry)) = found else {
			println!("seed {seed0}: no pass");
			continue;
		};
		let score = replay_score(seed0, &moves);
		let move_count = moves.len();
		println!("seed {seed0}: pass at move {move_count}, score {score}");
		if let Some(out) = &config.out {
			let text: String = moves.iter().map(|&m| MOVE_CHARS[m]).collect();
			write(
				out,
				seed0,
				format!("{seed0} {} {score} {text} {symmetry}", next(next(seed0))),
			);
		}
	}
}
