#![allow(clippy::zero_prefixed_literal)]

use std::collections::HashMap;
use std::io::Read;

type Depth = u8;

type BoardBitSize = u32;
#[derive(Eq, Hash, PartialEq)]
struct Board(BoardBitSize);
type BoardIndex = u8;
type DiceValue = u8;

type Sum = u32;
type PathCount = u32;

const SYMMETRY_COUNT: usize = 3;
type SymmetryPathCount = [PathCount; SYMMETRY_COUNT];

const DICE_MAX: DiceValue = 6;

const SUM_MOD: Sum = 1 << 30;

const QUEUE_CAPACITY: usize = 90_000;

fn main() {
	let (depth, starting_board) = parse();

	println!("{sum}", sum = solve(depth, starting_board));
}

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

fn parse() -> (Depth, Board) {
	let mut input_line = String::new();
	std::io::stdin().read_line(&mut input_line).unwrap();
	let depth = parse_input!(input_line, u8);

	let starting_board = Board::parse(Board::read());

	(depth, starting_board)
}

const C_BR: DiceValue = 0;
const C_B_: DiceValue = 3;
const C_BL: DiceValue = 6;
const C_R_: DiceValue = 9;
const C_M_: DiceValue = 12;
const C_L_: DiceValue = 15;
const C_TR: DiceValue = 18;
const C_T_: DiceValue = 21;
const C_TL: DiceValue = 24;

static TRANSFORMERS: [fn(BoardBitSize) -> BoardBitSize; SYMMETRY_COUNT] = [
	// identity
	|board| board,
	// vertical flip
	|board| {
		(((board >> C_TL) & 0b111) << C_TR)
			| (((board >> C_T_) & 0b111) << C_T_)
			| (((board >> C_TR) & 0b111) << C_TL)
			| (((board >> C_L_) & 0b111) << C_R_)
			| (((board >> C_M_) & 0b111) << C_M_)
			| (((board >> C_R_) & 0b111) << C_L_)
			| (((board >> C_BL) & 0b111) << C_BR)
			| (((board >> C_B_) & 0b111) << C_B_)
			| (((board >> C_BR) & 0b111) << C_BL)
	},
	// horizontal flip
	|board| {
		(((board >> C_TL) & 0b111) << C_BL)
			| (((board >> C_T_) & 0b111) << C_B_)
			| (((board >> C_TR) & 0b111) << C_BR)
			| (((board >> C_L_) & 0b111) << C_L_)
			| (((board >> C_M_) & 0b111) << C_M_)
			| (((board >> C_R_) & 0b111) << C_R_)
			| (((board >> C_BL) & 0b111) << C_TL)
			| (((board >> C_B_) & 0b111) << C_T_)
			| (((board >> C_BR) & 0b111) << C_TR)
	},
];

#[inline]
fn empty_cell_mask(index: BoardIndex) -> BoardBitSize {
	!(0b111 << index)
}

// (src & !(0b111 << index)) | (value << index)
#[inline]
fn set(
	src: BoardBitSize,
	mask: BoardBitSize,
	index: BoardIndex,
	value: BoardBitSize,
) -> BoardBitSize {
	(src & mask) | (value << index)
}

type HashDiceTable = [BoardBitSize; 7];

const fn generate_table(multiplier: BoardBitSize) -> HashDiceTable {
	let mut table = [0; 7];
	let mut i = 0;
	while i < 7 {
		table[i] = i as u32 * multiplier;
		i += 1;
	}
	table
}

const HASH_TABLE_TL: HashDiceTable = generate_table(100_000_000);
const HASH_TABLE_T_: HashDiceTable = generate_table(010_000_000);
const HASH_TABLE_TR: HashDiceTable = generate_table(001_000_000);
const HASH_TABLE_L_: HashDiceTable = generate_table(000_100_000);
const HASH_TABLE_M_: HashDiceTable = generate_table(000_010_000);
const HASH_TABLE_R_: HashDiceTable = generate_table(000_001_000);
const HASH_TABLE_BL: HashDiceTable = generate_table(000_000_100);
const HASH_TABLE_B_: HashDiceTable = generate_table(000_000_010);
const HASH_TABLE_BR: HashDiceTable = generate_table(000_000_001);

impl Board {
	// const MAX: BoardBitSize = 0b110_110_110_110_110_110_110_110_110;

	fn read() -> [u8; 18] {
		let mut buffer = [0u8; 18];
		std::io::stdin().read_exact(&mut buffer).unwrap();
		buffer
	}

	fn parse(input: [u8; 18]) -> Self {
		let ascii_zero = b'0';

		Self(
			(((input[0] - ascii_zero) as BoardBitSize) << C_TL)
				| (((input[2] - ascii_zero) as BoardBitSize) << C_T_)
				| (((input[4] - ascii_zero) as BoardBitSize) << C_TR)
				| (((input[6] - ascii_zero) as BoardBitSize) << C_L_)
				| (((input[8] - ascii_zero) as BoardBitSize) << C_M_)
				| (((input[10] - ascii_zero) as BoardBitSize) << C_R_)
				| (((input[12] - ascii_zero) as BoardBitSize) << C_BL)
				| (((input[14] - ascii_zero) as BoardBitSize) << C_B_)
				| ((input[16] - ascii_zero) as BoardBitSize),
		)
	}

	#[inline]
	fn get(&self, index: BoardIndex) -> DiceValue {
		(self.0 >> index) as DiceValue & 0b111
	}

	#[inline]
	fn hash(&self) -> BoardBitSize {
		HASH_TABLE_TL[self.get(C_TL) as usize]
			+ HASH_TABLE_T_[self.get(C_T_) as usize]
			+ HASH_TABLE_TR[self.get(C_TR) as usize]
			+ HASH_TABLE_L_[self.get(C_L_) as usize]
			+ HASH_TABLE_M_[self.get(C_M_) as usize]
			+ HASH_TABLE_R_[self.get(C_R_) as usize]
			+ HASH_TABLE_BL[self.get(C_BL) as usize]
			+ HASH_TABLE_B_[self.get(C_B_) as usize]
			+ HASH_TABLE_BR[self.get(C_BR) as usize]
	}

	fn canonical(&self) -> (BoardBitSize, usize) {
		let mut min = (self.0, 0);
		// skipping the first one because it's the original
		for i in 1..SYMMETRY_COUNT {
			let transformed = TRANSFORMERS[i](self.0);
			if transformed < min.0 {
				min = (transformed, i);
			}
		}
		min
	}
}

macro_rules! queue_insert {
	($queue:ident, $board:expr, $symmetry_path_count:ident) => {
		$queue
			.entry($board)
			.and_modify(|count| {
				for i in 0..SYMMETRY_COUNT {
					count.1[i] = count.1[i].wrapping_add($symmetry_path_count[i]);
				}
			})
			.or_insert($path_count);
	};
}

macro_rules! play_single_move {
	($board:ident, $index:ident, $path_count:ident, $queue:ident, $moved:ident, $neighbors_buf:ident, $($neighbors:literal),+) => {
		let n = 0 $(
			+ $neighbors_buf[$neighbors].1
		)+;
		if n <= DICE_MAX {
			$moved = true;
			queue_insert!($queue, Board(set(
				$board.0,
				empty_cell_mask($index)
				$(
					& $neighbors_buf[$neighbors].2
				)+,
				$index,
				n as BoardBitSize
			)), $path_count);
		}
	};
}

macro_rules! play_move {
	($board:ident, $index:ident, $symmetry_path_count:ident, $queue:ident, $moved:ident, $neighbors_buf:ident, $neighbors_len:ident, $($neighbors:ident),+) => {
		if $board.get($index) == 0 {
			$moved = true;

			$neighbors_len = 0;
			$(
				let neighbor = $board.get($neighbors);
				if neighbor != 0 && neighbor != DICE_MAX {
					$neighbors_buf[$neighbors_len as usize] = ($neighbors, neighbor, empty_cell_mask($neighbors));
					$neighbors_len += 1;
				}
			)+

			if $neighbors_len <= 1 {
				queue_insert!($queue, Board(set($board.0, empty_cell_mask($index), $index, 1)), $path_count);
			} else {
				match $neighbors_len {
					2 => {
						let n = $neighbors_buf[0].1 + $neighbors_buf[1].1;
						if n <= DICE_MAX {
							queue_insert!($queue, Board(set(
								$board.0,
								empty_cell_mask($index)
								& $neighbors_buf[0].2
								& $neighbors_buf[1].2,
								$index,
								n as BoardBitSize
							)), $path_count);
						} else {
							queue_insert!($queue, Board(set($board.0, empty_cell_mask($index), $index, 1)), $path_count);
						}
					},
					3 => {
						let mut moved_here = false;

						// 2 of 3
						play_single_move!($board, $index, $path_count, $queue, moved_here, $neighbors_buf, 0, 1);
						play_single_move!($board, $index, $path_count, $queue, moved_here, $neighbors_buf, 0, 2);
						play_single_move!($board, $index, $path_count, $queue, moved_here, $neighbors_buf, 1, 2);
						// 3 of 3
						play_single_move!($board, $index, $path_count, $queue, moved_here, $neighbors_buf, 0, 1, 2);

						if !moved_here {
							queue_insert!($queue, Board(set($board.0, empty_cell_mask($index), $index, 1)), $path_count);
						}
					},
					_ => {
						let mut moved_here = false;

						// 2 of 4
						play_single_move!($board, $index, $path_count, $queue, moved_here, $neighbors_buf, 0, 1);
						play_single_move!($board, $index, $path_count, $queue, moved_here, $neighbors_buf, 0, 2);
						play_single_move!($board, $index, $path_count, $queue, moved_here, $neighbors_buf, 0, 3);
						play_single_move!($board, $index, $path_count, $queue, moved_here, $neighbors_buf, 1, 2);
						play_single_move!($board, $index, $path_count, $queue, moved_here, $neighbors_buf, 1, 3);
						play_single_move!($board, $index, $path_count, $queue, moved_here, $neighbors_buf, 2, 3);
						// 3 of 4
						play_single_move!($board, $index, $path_count, $queue, moved_here, $neighbors_buf, 0, 1, 2);
						play_single_move!($board, $index, $path_count, $queue, moved_here, $neighbors_buf, 0, 1, 3);
						play_single_move!($board, $index, $path_count, $queue, moved_here, $neighbors_buf, 0, 2, 3);
						play_single_move!($board, $index, $path_count, $queue, moved_here, $neighbors_buf, 1, 2, 3);
						// 4 of 4
						play_single_move!($board, $index, $path_count, $queue, moved_here, $neighbors_buf, 0, 1, 2, 3);

						if !moved_here {
							queue_insert!($queue, Board(set($board.0, empty_cell_mask($index), $index, 1)), $path_count);
						}
					},
				}
			}
		}
	};
}

// we can let it overflow because u32 is 4 times 2^30
macro_rules! sum {
	($sum:ident, $board:ident, $symmetry_path_count:ident) => {
		$sum = $sum.wrapping_add($board.hash().wrapping_mul($symmetry_path_count[0]));
		for i in 1..SYMMETRY_COUNT {
			$sum = $sum.wrapping_add(
				Board(TRANSFORMERS[i]($board.0))
					.hash()
					.wrapping_mul($symmetry_path_count[i]),
			);
		}
	};
}

type Queue = HashMap<
	// canonical board
	BoardBitSize,
	(
		// original board
		Board,
		// path count per symmetry
		SymmetryPathCount,
	),
>;

fn solve(depth: Depth, starting_board: Board) -> Sum {
	let mut sum: Sum = 0;
	let mut queue: Queue = HashMap::with_capacity(QUEUE_CAPACITY);
	let mut current_queue: Queue = HashMap::with_capacity(QUEUE_CAPACITY);
	let mut ngb_buf: [(BoardIndex, DiceValue, BoardBitSize); 4] = [(0, 0, 0); 4];
	let mut ngb_len: u8;
	let mut d = 0;

	queue.insert(starting_board.canonical().0, (starting_board, [1, 0, 0]));

	while d < depth && !queue.is_empty() {
		std::mem::swap(&mut queue, &mut current_queue);

		for (_, (board, spc)) in current_queue.drain() {
			let mut moved = false;

			play_move!(board, C_BR, spc, queue, moved, ngb_buf, ngb_len, C_B_, C_R_);
			play_move!(
				board, C_B_, spc, queue, moved, ngb_buf, ngb_len, C_BL, C_M_, C_BR
			);
			play_move!(board, C_BL, spc, queue, moved, ngb_buf, ngb_len, C_L_, C_B_);
			play_move!(
				board, C_R_, spc, queue, moved, ngb_buf, ngb_len, C_M_, C_TR, C_BR
			);
			play_move!(
				board, C_M_, spc, queue, moved, ngb_buf, ngb_len, C_L_, C_T_, C_R_, C_B_
			);
			play_move!(
				board, C_L_, spc, queue, moved, ngb_buf, ngb_len, C_TL, C_M_, C_BL
			);
			play_move!(board, C_TR, spc, queue, moved, ngb_buf, ngb_len, C_T_, C_R_);
			play_move!(
				board, C_T_, spc, queue, moved, ngb_buf, ngb_len, C_TL, C_TR, C_M_
			);
			play_move!(board, C_TL, spc, queue, moved, ngb_buf, ngb_len, C_T_, C_L_);

			if !moved {
				sum!(sum, board, spc);
			}
		}

		d += 1;
	}

	for (_, (board, spc)) in queue {
		sum!(sum, board, spc);
	}

	sum % SUM_MOD
}

#[cfg(test)]
mod tests {
	use super::*;

	fn board_from_hash(hash: BoardBitSize) -> Board {
		Board(
			(((hash / 100_000_000) % 10) << C_TL)
				| (((hash / 010_000_000) % 10) << C_T_)
				| (((hash / 001_000_000) % 10) << C_TR)
				| (((hash / 000_100_000) % 10) << C_L_)
				| (((hash / 000_010_000) % 10) << C_M_)
				| (((hash / 000_001_000) % 10) << C_R_)
				| (((hash / 000_000_100) % 10) << C_BL)
				| (((hash / 000_000_010) % 10) << C_B_)
				| (hash % 10),
		)
	}

	#[test]
	fn test_parse() {
		let input = [
			'1' as u8, ' ' as u8, '2' as u8, ' ' as u8, '3' as u8, '\n' as u8, //
			'4' as u8, ' ' as u8, '5' as u8, ' ' as u8, '6' as u8, '\n' as u8, //
			'0' as u8, ' ' as u8, '2' as u8, ' ' as u8, '4' as u8, '\n' as u8,
		];
		let board = Board::parse(input);
		assert_eq!(board.0, 0b_001_010_011_100_101_110_000_010_100);
		assert_eq!(board.get(C_TL), 1);
		assert_eq!(board.get(C_T_), 2);
		assert_eq!(board.get(C_TR), 3);
		assert_eq!(board.get(C_L_), 4);
		assert_eq!(board.get(C_M_), 5);
		assert_eq!(board.get(C_R_), 6);
		assert_eq!(board.get(C_BL), 0);
		assert_eq!(board.get(C_B_), 2);
		assert_eq!(board.get(C_BR), 4);
		assert_eq!(board.hash(), 123_456_024);
	}

	#[test]
	fn test_set() {
		let src = 0b_001_010_011_100_101_110_000_010_100;
		assert_eq!(
			set(src, empty_cell_mask(C_TL), C_TL, 0),
			0b_000_010_011_100_101_110_000_010_100
		);
		assert_eq!(
			set(src, empty_cell_mask(C_T_), C_T_, 0),
			0b_001_000_011_100_101_110_000_010_100
		);
		assert_eq!(
			set(src, empty_cell_mask(C_TR), C_TR, 0),
			0b_001_010_000_100_101_110_000_010_100
		);
		assert_eq!(
			set(src, empty_cell_mask(C_L_), C_L_, 0),
			0b_001_010_011_000_101_110_000_010_100
		);
		assert_eq!(
			set(src, empty_cell_mask(C_M_), C_M_, 0),
			0b_001_010_011_100_000_110_000_010_100
		);
		assert_eq!(
			set(src, empty_cell_mask(C_R_), C_R_, 0),
			0b_001_010_011_100_101_000_000_010_100
		);
		assert_eq!(
			set(src, empty_cell_mask(C_BL), C_BL, 0),
			0b_001_010_011_100_101_110_000_010_100
		);
		assert_eq!(
			set(src, empty_cell_mask(C_B_), C_B_, 0),
			0b_001_010_011_100_101_110_000_000_100
		);
		assert_eq!(
			set(src, empty_cell_mask(C_BR), C_BR, 0),
			0b_001_010_011_100_101_110_000_010_000
		);
	}

	#[test]
	fn test_get() {
		let board = Board(0b_001_010_011_100_101_110_000_010_100);
		assert_eq!(board.get(C_TL), 1);
		assert_eq!(board.get(C_T_), 2);
		assert_eq!(board.get(C_TR), 3);
		assert_eq!(board.get(C_L_), 4);
		assert_eq!(board.get(C_M_), 5);
		assert_eq!(board.get(C_R_), 6);
		assert_eq!(board.get(C_BL), 0);
		assert_eq!(board.get(C_B_), 2);
		assert_eq!(board.get(C_BR), 4);
	}

	#[test]
	fn test_hash() {
		let board = Board(0b_001_010_011_100_101_110_000_010_100);
		assert_eq!(board.hash(), 123_456_024);

		let board = board_from_hash(123_456_024);
		assert_eq!(board.0, 0b_001_010_011_100_101_110_000_010_100);
		assert_eq!(board.hash(), 123_456_024);
	}

	#[test]
	fn transform() {
		let board = board_from_hash(123_456_024);
		assert_eq!(Board(TRANSFORMERS[0](board.0)).hash(), board.hash());
		assert_eq!(Board(TRANSFORMERS[1](board.0)).hash(), 321_654_420);
		assert_eq!(Board(TRANSFORMERS[2](board.0)).hash(), 024_456_123);
	}

	#[test]
	fn test_play_single_move() {
		let board = board_from_hash(616101616);
		let neighbors_buf = Vec::<(BoardIndex, DiceValue, BoardBitSize)>::from([
			(C_L_, 1, empty_cell_mask(C_L_)),
			(C_T_, 1, empty_cell_mask(C_T_)),
		]);
		let mut queue: HashMap<Board, PathCount> = HashMap::new();
		let mut moved = false;
		let depth = 1;

		play_single_move!(board, C_M_, depth, queue, moved, neighbors_buf, 0, 1);

		assert_eq!(queue.len(), 1);
		assert!(moved);
		let first = queue.iter().next().unwrap();
		assert_eq!(first.0.hash(), 606021616);
		assert_eq!(*first.1, 1);
	}

	#[test]
	fn test_solve() {
		assert_eq!(solve(20, board_from_hash(60222161)), 322444322);
		assert_eq!(solve(20, board_from_hash(506450064)), 951223336);
		assert_eq!(solve(1, board_from_hash(555005555)), 36379286);
		assert_eq!(solve(1, board_from_hash(616101616)), 264239762);
		assert_eq!(solve(24, board_from_hash(300362102)), 661168294);
		assert_eq!(solve(36, board_from_hash(604202400)), 350917228);
		assert_eq!(solve(32, board_from_hash(54105)), 999653138);
		assert_eq!(solve(40, board_from_hash(4024134)), 521112022);
		assert_eq!(solve(40, board_from_hash(54030030)), 667094338);
		assert_eq!(solve(20, board_from_hash(51000401)), 738691369);
		assert_eq!(solve(20, board_from_hash(100352100)), 808014757);
	}
}
