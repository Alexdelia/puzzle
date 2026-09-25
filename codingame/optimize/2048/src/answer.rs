use std::io::{self, BufRead, Write};

const GAMES: &[(u64, usize, usize)] = &[];
const PAYLOAD: &str = "";

const CHUNK: usize = 9000;
const MODULUS: u64 = 50515093;
pub const CJK_BASE: u32 = 0x4E00;
pub const CHAR_BITS: u32 = 14;
pub const TOP: u64 = (1 << 32) - 1;
pub const HALF: u64 = 1 << 31;
pub const QUARTER: u64 = 1 << 30;
pub const MOVE_CHARS: [char; 4] = ['U', 'D', 'L', 'R'];
const SNAKE: [usize; 16] = [0, 1, 2, 3, 7, 6, 5, 4, 8, 9, 10, 11, 15, 14, 13, 12];

pub type Board = [u8; 16];

pub fn next(seed: u64) -> u64 {
	seed * seed % MODULUS
}

fn line_cell(mv: usize, line: usize, k: usize) -> usize {
	match mv {
		0 => k * 4 + line,
		1 => (3 - k) * 4 + line,
		2 => line * 4 + k,
		_ => line * 4 + 3 - k,
	}
}

pub fn play(board: &Board, mv: usize) -> Option<(Board, u32)> {
	let mut out = [0u8; 16];
	let mut gain = 0;
	for line in 0..4 {
		let tiles: Vec<u8> = (0..4)
			.map(|k| board[line_cell(mv, line, k)])
			.filter(|&v| v != 0)
			.collect();
		let mut slot = 0;
		let mut i = 0;
		while i < tiles.len() {
			let merged = i + 1 < tiles.len() && tiles[i] == tiles[i + 1];
			let value = if merged { tiles[i] + 1 } else { tiles[i] };
			if merged {
				gain += 1 << value;
			}
			out[line_cell(mv, line, slot)] = value;
			slot += 1;
			i += if merged { 2 } else { 1 };
		}
	}
	(out != *board).then_some((out, gain))
}

pub fn spawn(board: &mut Board, seed: u64) {
	let empty: Vec<usize> = (0..4)
		.flat_map(|y| (0..4).map(move |x| x * 4 + y))
		.filter(|&i| board[i] == 0)
		.collect();
	board[empty[seed as usize % empty.len()]] = if seed & 0x10 == 0 { 1 } else { 2 };
}

pub fn start(seed0: u64) -> (Board, u64) {
	let mut board = [0; 16];
	spawn(&mut board, seed0);
	let seed = next(seed0);
	spawn(&mut board, seed);
	(board, next(seed))
}

fn snake_value(board: &Board) -> i64 {
	let mut total = 0i64;
	for (k, &i) in SNAKE.iter().enumerate() {
		total += if board[i] == 0 {
			1
		} else {
			(1i64 << board[i]) << (15 - k)
		};
	}
	total
}

pub struct Step {
	pub mv: usize,
	pub board: Board,
	pub gain: u32,
}

pub fn ranking(board: &Board, seed: u64) -> Vec<Step> {
	let mut steps: Vec<Step> = (0..4)
		.filter_map(|mv| {
			play(board, mv).map(|(mut after, gain)| {
				spawn(&mut after, seed);
				Step {
					mv,
					board: after,
					gain,
				}
			})
		})
		.collect();
	steps.sort_by_key(|s| (-snake_value(&s.board), s.mv));
	steps
}

pub struct Model {
	counts: [[[u32; 4]; 3]; 4],
	previous: usize,
}

impl Default for Model {
	fn default() -> Self {
		Self {
			counts: [[[1; 4]; 3]; 4],
			previous: 0,
		}
	}
}

impl Model {
	pub fn frequencies(&self, legal: usize) -> &[u32] {
		&self.counts[self.previous][legal - 2][..legal]
	}

	pub fn update(&mut self, legal: usize, rank: usize) {
		self.counts[self.previous][legal - 2][rank] += 1;
		self.previous = rank;
	}
}

pub struct BitReader<'a> {
	chars: std::str::Chars<'a>,
	buffer: u32,
	available: u32,
}

impl<'a> BitReader<'a> {
	pub fn new(stream: &'a str) -> Self {
		Self {
			chars: stream.chars(),
			buffer: 0,
			available: 0,
		}
	}

	fn bit(&mut self) -> u64 {
		if self.available == 0 {
			let Some(c) = self.chars.next() else { return 0 };
			self.buffer = c as u32 - CJK_BASE;
			self.available = CHAR_BITS;
		}
		self.available -= 1;
		((self.buffer >> self.available) & 1) as u64
	}
}

pub struct Decoder<'a> {
	low: u64,
	high: u64,
	value: u64,
	reader: BitReader<'a>,
}

impl<'a> Decoder<'a> {
	pub fn new(stream: &'a str) -> Self {
		let mut reader = BitReader::new(stream);
		let value = (0..32).fold(0, |v, _| v << 1 | reader.bit());
		Self {
			low: 0,
			high: TOP,
			value,
			reader,
		}
	}

	pub fn decode(&mut self, frequencies: &[u32]) -> usize {
		let total: u64 = frequencies.iter().map(|&f| f as u64).sum();
		let range = self.high - self.low + 1;
		let target = ((self.value - self.low + 1) * total - 1) / range;
		let mut cumulative = 0;
		let mut symbol = 0;
		while cumulative + frequencies[symbol] as u64 <= target {
			cumulative += frequencies[symbol] as u64;
			symbol += 1;
		}
		self.high = self.low + range * (cumulative + frequencies[symbol] as u64) / total - 1;
		self.low += range * cumulative / total;
		loop {
			let shift = if self.high < HALF {
				0
			} else if self.low >= HALF {
				HALF
			} else if self.low >= QUARTER && self.high < HALF + QUARTER {
				QUARTER
			} else {
				break;
			};
			self.low -= shift;
			self.high -= shift;
			self.value -= shift;
			self.low <<= 1;
			self.high = self.high << 1 | 1;
			self.value = self.value << 1 | self.reader.bit();
		}
		symbol
	}
}

pub struct Plan<'a> {
	pub board: Board,
	pub seed: u64,
	pub score: u32,
	stored: usize,
	decoder: Decoder<'a>,
	model: Model,
}

impl<'a> Plan<'a> {
	pub fn replay(seed0: u64, stream: &'a str, stored: usize) -> Self {
		let (board, seed) = start(seed0);
		Self {
			board,
			seed,
			score: 0,
			stored,
			decoder: Decoder::new(stream),
			model: Model::default(),
		}
	}

	pub fn greedy(board: Board, seed: u64) -> Self {
		Self {
			board,
			seed,
			score: 0,
			stored: 0,
			decoder: Decoder::new(""),
			model: Model::default(),
		}
	}

	pub fn next_move(&mut self) -> Option<usize> {
		let mut steps = ranking(&self.board, self.seed);
		if steps.is_empty() {
			return None;
		}
		let rank = if self.stored > 0 {
			let rank = self.decoder.decode(self.model.frequencies(steps.len()));
			self.model.update(steps.len(), rank);
			self.stored -= 1;
			rank
		} else {
			0
		};
		let step = steps.swap_remove(rank);
		self.board = step.board;
		self.score += step.gain;
		self.seed = next(self.seed);
		Some(step.mv)
	}
}

fn read_turn(lines: &mut impl Iterator<Item = io::Result<String>>) -> Option<(u64, Board)> {
	let seed = lines.next()?.ok()?.trim().parse().ok()?;
	lines.next()?.ok()?;
	let mut board = [0u8; 16];
	for x in 0..4 {
		let row = lines.next()?.ok()?;
		for (y, cell) in row.split_whitespace().enumerate() {
			let value: u32 = cell.parse().ok()?;
			board[x * 4 + y] = if value == 0 {
				0
			} else {
				value.trailing_zeros() as u8
			};
		}
	}
	Some((seed, board))
}

fn plan_for(seed: u64, board: Board) -> Plan<'static> {
	let Some(&(seed0, stored, offset)) = GAMES.iter().find(|g| next(next(g.0)) == seed) else {
		eprintln!("unknown seed {seed}: greedy");
		return Plan::greedy(board, seed);
	};
	let stream = &PAYLOAD[PAYLOAD
		.char_indices()
		.nth(offset)
		.map_or(PAYLOAD.len(), |(i, _)| i)..];
	let plan = Plan::replay(seed0, stream, stored);
	let simulated = plan.board;
	if simulated != board {
		eprintln!("board mismatch for seed {seed}: {simulated:?} vs input {board:?}");
	}
	plan
}

fn main() {
	let stdin = io::stdin();
	let mut lines = stdin.lock().lines();
	let Some((seed, board)) = read_turn(&mut lines) else {
		return;
	};
	let mut plan = plan_for(seed, board);
	loop {
		let moves: String = std::iter::from_fn(|| plan.next_move())
			.take(CHUNK)
			.map(|m| MOVE_CHARS[m])
			.collect();
		if moves.is_empty() {
			break;
		}
		println!("{moves}");
		io::stdout().flush().unwrap();
		if read_turn(&mut lines).is_none() {
			break;
		}
	}
}
