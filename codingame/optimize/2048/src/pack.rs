#[allow(dead_code)]
#[path = "answer.rs"]
mod answer;

use answer::{CHAR_BITS, CJK_BASE, HALF, MOVE_CHARS, Model, Plan, QUARTER, TOP, ranking, start};

struct Encoder {
	low: u64,
	high: u64,
	pending: usize,
	bits: Vec<bool>,
}

impl Encoder {
	fn new() -> Self {
		Self {
			low: 0,
			high: TOP,
			pending: 0,
			bits: Vec::new(),
		}
	}

	fn emit(&mut self, bit: bool) {
		self.bits.push(bit);
		self.bits.extend(std::iter::repeat_n(!bit, self.pending));
		self.pending = 0;
	}

	fn encode(&mut self, frequencies: &[u32], symbol: usize) {
		let total: u64 = frequencies.iter().map(|&f| f as u64).sum();
		let cumulative: u64 = frequencies[..symbol].iter().map(|&f| f as u64).sum();
		let range = self.high - self.low + 1;
		self.high = self.low + range * (cumulative + frequencies[symbol] as u64) / total - 1;
		self.low += range * cumulative / total;
		loop {
			let shift = if self.high < HALF {
				self.emit(false);
				0
			} else if self.low >= HALF {
				self.emit(true);
				HALF
			} else if self.low >= QUARTER && self.high < HALF + QUARTER {
				self.pending += 1;
				QUARTER
			} else {
				break;
			};
			self.low = (self.low - shift) << 1;
			self.high = (self.high - shift) << 1 | 1;
		}
	}

	fn finish(mut self) -> String {
		self.pending += 1;
		self.emit(self.low >= QUARTER);
		self.bits
			.chunks(CHAR_BITS as usize)
			.map(|chunk| {
				let value = (0..CHAR_BITS as usize).fold(0, |v, i| {
					v << 1 | chunk.get(i).copied().unwrap_or(false) as u32
				});
				char::from_u32(CJK_BASE + value).unwrap()
			})
			.collect()
	}
}

struct Game {
	seed0: u64,
	score: u32,
	moves: Vec<usize>,
}

fn parse(line: &str) -> Game {
	let fields: Vec<&str> = line.split_whitespace().collect();
	Game {
		seed0: fields[0].parse().unwrap(),
		score: fields[2].parse().unwrap(),
		moves: fields[3]
			.chars()
			.map(|c| MOVE_CHARS.iter().position(|&m| m == c).unwrap())
			.collect(),
	}
}

fn encode(game: &Game) -> String {
	let seed0 = game.seed0;
	let (mut board, mut seed) = start(game.seed0);
	let mut model = Model::default();
	let mut encoder = Encoder::new();
	let mut score = 0;
	for &mv in &game.moves {
		let steps = ranking(&board, seed);
		let rank = steps
			.iter()
			.position(|s| s.mv == mv)
			.unwrap_or_else(|| panic!("seed {seed0}: illegal move"));
		encoder.encode(model.frequencies(steps.len()), rank);
		model.update(steps.len(), rank);
		board = steps[rank].board;
		score += steps[rank].gain;
		seed = answer::next(seed);
	}
	assert_eq!(
		score, game.score,
		"seed {seed0}: replayed score differ from the result file"
	);
	encoder.finish()
}

fn verify(game: &Game, stream: &str) -> u32 {
	let seed0 = game.seed0;
	let mut plan = Plan::replay(seed0, stream, game.moves.len());
	for (i, &expected) in game.moves.iter().enumerate() {
		assert_eq!(
			plan.next_move(),
			Some(expected),
			"seed {seed0}: decode diverge at move {i}"
		);
	}
	assert_eq!(plan.score, game.score, "seed {seed0}: decoded score differ");
	while plan.next_move().is_some() {}
	plan.score
}

fn main() {
	let path = std::env::args()
		.nth(1)
		.unwrap_or(".2048_results.out".into());
	let games: Vec<Game> = std::fs::read_to_string(&path)
		.unwrap()
		.lines()
		.filter(|l| !l.trim().is_empty())
		.map(parse)
		.collect();
	let mut payload = String::new();
	let mut table = Vec::new();
	let (mut stored_total, mut final_total, mut move_total) = (0u64, 0u64, 0usize);
	for game in &games {
		let stream = encode(game);
		let offset = payload.chars().count();
		let final_score = verify(game, &stream);
		let Game { seed0, score, .. } = *game;
		let move_count = game.moves.len();
		let char_count = stream.chars().count();
		let bit_per_move = (char_count * CHAR_BITS as usize) as f64 / move_count as f64;
		eprintln!(
			"{seed0:>9} {score:>8} move {move_count:>6} char {char_count:>5} bit/move {bit_per_move:.3} final {final_score}"
		);
		table.push(format!("({seed0}, {move_count}, {offset})"));
		payload.push_str(&stream);
		stored_total += score as u64;
		final_total += final_score as u64;
		move_total += move_count;
	}
	let game_count = games.len();
	let payload_char = payload.chars().count();
	eprintln!(
		"{game_count} game, {move_total} move, {payload_char} payload char, stored score {stored_total}, with greedy tail {final_total}"
	);
	let table = table.join(", ");
	println!("const GAMES: &[(u64, usize, usize)] = &[{table}];");
	println!("const PAYLOAD: &str = \"{payload}\";");
}
