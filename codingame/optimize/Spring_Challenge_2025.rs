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

const SYMMETRY_COUNT: u8 = 8;
type SymmetryPathCount = [PathCount; SYMMETRY_COUNT as usize];
type RotationIndex = u8;

const DICE_MAX: DiceValue = 6;

const SUM_MOD: Sum = 1 << 30;
const DEPTH_MAX: Depth = 40;

const QUEUE_CAPACITY: usize = 1 << 13;

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

static TRANSFORMERS: [fn(BoardBitSize) -> BoardBitSize; SYMMETRY_COUNT as usize] = [
	// 0
	#[inline(always)]
	|board| board,
	// 90 clockwise
	#[inline(always)]
	|board| {
		// ((board & 0b111_000_000_000_000_000_000_000_000) >> 6)
		((board & 0b000_111_000_000_000_000_000_000_000) >> 12)
			| ((board & 0b000_000_111_000_000_000_000_000_000) >> 18)
			// | ((board & 0b000_000_000_111_000_000_000_000_000) << 6)
			| (board & 0b000_000_000_000_111_000_000_000_000)
			// | ((board & 0b000_000_000_000_000_111_000_000_000) >> 6)
			| ((board & 0b000_000_000_000_000_000_111_000_000) << 18)
			| ((board & 0b000_000_000_000_000_000_000_111_000) << 12)
			// | ((board & 0b000_000_000_000_000_000_000_000_111) << 6)
			| ((board & 0b111_000_000_000_000_111_000_000_000) >> 6)
			| ((board & 0b000_000_000_111_000_000_000_000_111) << 6)
	},
	// 180
	#[inline(always)]
	|board| {
		((board & 0b111_000_000_000_000_000_000_000_000) >> 24)
			| ((board & 0b000_111_000_000_000_000_000_000_000) >> 18)
			| ((board & 0b000_000_111_000_000_000_000_000_000) >> 12)
			| ((board & 0b000_000_000_111_000_000_000_000_000) >> 6)
			| (board & 0b000_000_000_000_111_000_000_000_000)
			| ((board & 0b000_000_000_000_000_111_000_000_000) << 6)
			| ((board & 0b000_000_000_000_000_000_111_000_000) << 12)
			| ((board & 0b000_000_000_000_000_000_000_111_000) << 18)
			| ((board & 0b000_000_000_000_000_000_000_000_111) << 24)
	},
	// 270 clockwise or 90 counter-clockwise
	#[inline(always)]
	|board| {
		((board & 0b111_000_000_000_000_000_000_000_000) >> 18)
			// | ((board & 0b000_111_000_000_000_000_000_000_000) >> 6)
			// | ((board & 0b000_000_111_000_000_000_000_000_000) << 6)
			| ((board & 0b000_000_000_111_000_000_000_000_000) >> 12)
			| (board & 0b000_000_000_000_111_000_000_000_000)
			| ((board & 0b000_000_000_000_000_111_000_000_000) << 12)
			// | ((board & 0b000_000_000_000_000_000_111_000_000) >> 6)
			// | ((board & 0b000_000_000_000_000_000_000_111_000) << 6)
			| ((board & 0b000_000_000_000_000_000_000_000_111) << 18)
			| ((board & 0b000_111_000_000_000_000_111_000_000) >> 6)
			| ((board & 0b000_000_111_000_000_000_000_111_000) << 6)
	},
	// vertical flip
	#[inline(always)]
	|board| {
		// ((board & 0b111_000_000_000_000_000_000_000_000) >> 6)
		// | (board & 0b000_111_000_000_000_000_000_000_000)
		// | ((board & 0b000_000_111_000_000_000_000_000_000) << 6)
		// | ((board & 0b000_000_000_111_000_000_000_000_000) >> 6)
		// | (board & 0b000_000_000_000_111_000_000_000_000)
		// | ((board & 0b000_000_000_000_000_111_000_000_000) << 6)
		// | ((board & 0b000_000_000_000_000_000_111_000_000) >> 6)
		// | (board & 0b000_000_000_000_000_000_000_111_000)
		// | ((board & 0b000_000_000_000_000_000_000_000_111) << 6)
		((board & 0b111_000_000_111_000_000_111_000_000) >> 6)
			| (board & 0b000_111_000_000_111_000_000_111_000)
			| ((board & 0b000_000_111_000_000_111_000_000_111) << 6)
	},
	// horizontal flip
	#[inline(always)]
	|board| {
		// ((board & 0b111_000_000_000_000_000_000_000_000) >> 18)
		// | ((board & 0b000_111_000_000_000_000_000_000_000) >> 18)
		// | ((board & 0b000_000_111_000_000_000_000_000_000) >> 18)
		// | (board & 0b000_000_000_111_000_000_000_000_000)
		// | (board & 0b000_000_000_000_111_000_000_000_000)
		// | (board & 0b000_000_000_000_000_111_000_000_000)
		// | ((board & 0b000_000_000_000_000_000_111_000_000) << 18)
		// | ((board & 0b000_000_000_000_000_000_000_111_000) << 18)
		// | ((board & 0b000_000_000_000_000_000_000_000_111) << 18)
		((board & 0b111_111_111_000_000_000_000_000_000) >> 18)
			| (board & 0b000_000_000_111_111_111_000_000_000)
			| ((board & 0b000_000_000_000_000_000_111_111_111) << 18)
	},
	// diagonal /
	#[inline(always)]
	|board| {
		((board & 0b111_000_000_000_000_000_000_000_000) >> 24)
			| ((board & 0b000_111_000_000_000_000_000_000_000) >> 12)
			| (board & 0b000_000_111_000_000_000_000_000_000)
			| ((board & 0b000_000_000_111_000_000_000_000_000) >> 12)
			| (board & 0b000_000_000_000_111_000_000_000_000)
			| ((board & 0b000_000_000_000_000_111_000_000_000) << 12)
			| (board & 0b000_000_000_000_000_000_111_000_000)
			| ((board & 0b000_000_000_000_000_000_000_111_000) << 12)
			| ((board & 0b000_000_000_000_000_000_000_000_111) << 24)
	},
	// diagonal \
	#[inline(always)]
	|board| {
		((board & 0b111_000_000_000_000_000_000_000_000) >> 6)
			| ((board & 0b000_111_000_000_000_000_000_000_000) >> 12)
			| ((board & 0b000_000_111_000_000_000_000_000_000) >> 18)
			| (board & 0b000_000_000_111)
			| ((board & 0b000_000_000_111) << 6)
			| ((board & 0b111) << 12)
			| ((board & 0b111) << 18)
	},
];

static REVERSE_TRANSFORMERS: [fn(BoardBitSize) -> BoardBitSize; SYMMETRY_COUNT as usize] = [
	TRANSFORMERS[0],
	TRANSFORMERS[3],
	TRANSFORMERS[2],
	TRANSFORMERS[1],
	TRANSFORMERS[4],
	TRANSFORMERS[5],
	TRANSFORMERS[6],
	TRANSFORMERS[7],
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

	fn canonical(&self) -> (BoardBitSize, RotationIndex) {
		let mut min = (self.0, 0);

		let transformed = TRANSFORMERS[1](self.0);
		if transformed < min.0 {
			min = (transformed, 1);
		}

		let transformed = TRANSFORMERS[2](self.0);
		if transformed < min.0 {
			min = (transformed, 2);
		}

		let transformed = TRANSFORMERS[3](self.0);
		if transformed < min.0 {
			min = (transformed, 3);
		}

		min
	}
}

macro_rules! queue_insert {
	($queue:ident, $board:expr, $rotation:ident, $symmetry_path_count:ident) => {
		let canonical = $board.canonical();
		$queue
			.entry(canonical.0)
			.and_modify(|(stored_rotation, count)| {
				let r = ((*stored_rotation).wrapping_sub($rotation.wrapping_add(canonical.1)) % 4)
					as usize;
				count[0] = count[0].wrapping_add($symmetry_path_count[r]);
				count[1] = count[1].wrapping_add($symmetry_path_count[(r + 1) % 4]);
				count[2] = count[2].wrapping_add($symmetry_path_count[(r + 2) % 4]);
				count[3] = count[3].wrapping_add($symmetry_path_count[(r + 3) % 4]);
			})
			.or_insert((($rotation.wrapping_add(canonical.1)), $symmetry_path_count));
	};
}

macro_rules! play_single_move {
	($board:ident, $index:ident, $rotation:ident, $symmetry_path_count:ident, $queue:ident, $moved:ident, $neighbors_buf:ident, $($neighbors:literal),+) => {
		let n = 0 $(
			+ $neighbors_buf[$neighbors].1
		)+;
		if n <= DICE_MAX {
			$moved = true;
			queue_insert!($queue, Board(set(
				$board,
				empty_cell_mask($index)
				$(
					& $neighbors_buf[$neighbors].2
				)+,
				$index,
				n as BoardBitSize
			)), $rotation, $symmetry_path_count);
		}
	};
}

macro_rules! play_move {
	($board:ident, $index:ident, $rotation:ident, $symmetry_path_count:ident, $queue:ident, $moved:ident, $neighbors_buf:ident, $neighbors_len:ident, $($neighbors:ident),+) => {
		if Board($board).get($index) == 0 {
			$moved = true;

			$neighbors_len = 0;
			$(
				let neighbor = Board($board).get($neighbors);
				if neighbor != 0 && neighbor != DICE_MAX {
					$neighbors_buf[$neighbors_len as usize] = ($neighbors, neighbor, empty_cell_mask($neighbors));
					$neighbors_len += 1;
				}
			)+

			if $neighbors_len <= 1 {
				queue_insert!($queue, Board(set($board, empty_cell_mask($index), $index, 1)), $rotation, $symmetry_path_count);
			} else {
				match $neighbors_len {
					2 => {
						let n = $neighbors_buf[0].1 + $neighbors_buf[1].1;
						if n <= DICE_MAX {
							queue_insert!($queue, Board(set(
								$board,
								empty_cell_mask($index)
								& $neighbors_buf[0].2
								& $neighbors_buf[1].2,
								$index,
								n as BoardBitSize
							)), $rotation, $symmetry_path_count);
						} else {
							queue_insert!($queue, Board(set($board, empty_cell_mask($index), $index, 1)), $rotation, $symmetry_path_count);
						}
					},
					3 => {
						let mut moved_here = false;

						// 2 of 3
						play_single_move!($board, $index, $rotation, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 1);
						play_single_move!($board, $index, $rotation, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 2);
						play_single_move!($board, $index, $rotation, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 1, 2);
						// 3 of 3
						play_single_move!($board, $index, $rotation, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 1, 2);

						if !moved_here {
							queue_insert!($queue, Board(set($board, empty_cell_mask($index), $index, 1)), $rotation, $symmetry_path_count);
						}
					},
					_ => {
						let mut moved_here = false;

						// 2 of 4
						play_single_move!($board, $index, $rotation, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 1);
						play_single_move!($board, $index, $rotation, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 2);
						play_single_move!($board, $index, $rotation, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 3);
						play_single_move!($board, $index, $rotation, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 1, 2);
						play_single_move!($board, $index, $rotation, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 1, 3);
						play_single_move!($board, $index, $rotation, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 2, 3);
						// 3 of 4
						play_single_move!($board, $index, $rotation, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 1, 2);
						play_single_move!($board, $index, $rotation, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 1, 3);
						play_single_move!($board, $index, $rotation, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 2, 3);
						play_single_move!($board, $index, $rotation, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 1, 2, 3);
						// 4 of 4
						play_single_move!($board, $index, $rotation, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 1, 2, 3);

						if !moved_here {
							queue_insert!($queue, Board(set($board, empty_cell_mask($index), $index, 1)), $rotation, $symmetry_path_count);
						}
					},
				}
			}
		}
	};
}

// we can let it overflow because u32 is 4 times 2^30
macro_rules! sum {
	($sum:ident, $board:ident, $rotation:ident, $symmetry_path_count:ident) => {
		let r0 = ($rotation % 4) as usize;
		let r1 = (($rotation + 1) % 4) as usize;
		let r2 = (($rotation + 2) % 4) as usize;
		let r3 = (($rotation + 3) % 4) as usize;
		let reversed_canonical = REVERSE_TRANSFORMERS[r0]($board);
		$sum = $sum
			.wrapping_add(
				Board(REVERSE_TRANSFORMERS[r0](reversed_canonical))
					.hash()
					.wrapping_mul($symmetry_path_count[r0]),
			)
			.wrapping_add(
				Board(REVERSE_TRANSFORMERS[r1](reversed_canonical))
					.hash()
					.wrapping_mul($symmetry_path_count[r1]),
			)
			.wrapping_add(
				Board(REVERSE_TRANSFORMERS[r2](reversed_canonical))
					.hash()
					.wrapping_mul($symmetry_path_count[r2]),
			)
			.wrapping_add(
				Board(REVERSE_TRANSFORMERS[r3](reversed_canonical))
					.hash()
					.wrapping_mul($symmetry_path_count[r3]),
			);
	};
}

const EMPTY_SOLUTION: [Sum; DEPTH_MAX as usize + 1] = [
	0, 111111111, 704035952, 840352818, 600875666, 50441886, 680243700, 597686656, 584450980,
	55305380, 193520836, 521847116, 1054388152, 518795448, 366207036, 678967952, 476916052,
	1009258340, 592651828, 1063467872, 400415524, 233248832, 230461008, 245411624, 899694236,
	384163740, 888060600, 347933640, 340717612, 73295296, 851289228, 221286388, 375032784,
	723342020, 92414440, 745533092, 331519112, 993643868, 72093236, 422667876, 503115192,
];

type Queue = HashMap<
	// canonical board
	BoardBitSize,
	(
		// current ratation count of 90 clockwise
		RotationIndex,
		// path count per symmetry
		SymmetryPathCount,
	),
>;

fn solve(depth: Depth, starting_board: Board) -> Sum {
	if starting_board.0 == 0 {
		return EMPTY_SOLUTION[depth as usize];
	}

	let mut sum: Sum = 0;
	let mut queue: Queue = HashMap::with_capacity(QUEUE_CAPACITY);
	let mut current_queue: Queue = HashMap::with_capacity(QUEUE_CAPACITY);
	let mut ngb_buf: [(BoardIndex, DiceValue, BoardBitSize); 4] = [(0, 0, 0); 4];
	let mut ngb_len: u8;
	let mut d = 0;

	// no need to compute first canonical
	// there will be no duplicates possible with only 1 board on depth 0
	queue.insert(starting_board.0, (0, [1, 0, 0, 0, 0, 0, 0, 0]));

	while d < depth && !queue.is_empty() {
		std::mem::swap(&mut queue, &mut current_queue);

		for (board, (rot, spc)) in current_queue.drain() {
			let mut moved = false;

			play_move!(
				board, C_BR, rot, spc, queue, moved, ngb_buf, ngb_len, C_B_, C_R_
			);
			play_move!(
				board, C_B_, rot, spc, queue, moved, ngb_buf, ngb_len, C_BL, C_M_, C_BR
			);
			play_move!(
				board, C_BL, rot, spc, queue, moved, ngb_buf, ngb_len, C_L_, C_B_
			);
			play_move!(
				board, C_R_, rot, spc, queue, moved, ngb_buf, ngb_len, C_M_, C_TR, C_BR
			);
			play_move!(
				board, C_M_, rot, spc, queue, moved, ngb_buf, ngb_len, C_L_, C_T_, C_R_, C_B_
			);
			play_move!(
				board, C_L_, rot, spc, queue, moved, ngb_buf, ngb_len, C_TL, C_M_, C_BL
			);
			play_move!(
				board, C_TR, rot, spc, queue, moved, ngb_buf, ngb_len, C_T_, C_R_
			);
			play_move!(
				board, C_T_, rot, spc, queue, moved, ngb_buf, ngb_len, C_TL, C_TR, C_M_
			);
			play_move!(
				board, C_TL, rot, spc, queue, moved, ngb_buf, ngb_len, C_T_, C_L_
			);

			if !moved {
				sum!(sum, board, rot, spc);
			}
		}

		d += 1;
	}

	for (board, (rot, spc)) in queue {
		sum!(sum, board, rot, spc);
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
	fn test_transform() {
		let i = board_from_hash(123_456_024);
		let r90 = board_from_hash(041_252_463);
		let r180 = board_from_hash(420_654_321);
		let r270 = board_from_hash(364_252_140);
		let v = board_from_hash(321_654_420);
		let h = board_from_hash(024_456_123);
		let dz = board_from_hash(463_252_041);

		assert_eq!(Board(TRANSFORMERS[0](i.0)).hash(), i.hash());
		assert_eq!(Board(TRANSFORMERS[1](i.0)).hash(), r90.hash());
		assert_eq!(Board(TRANSFORMERS[2](i.0)).hash(), r180.hash());
		assert_eq!(Board(TRANSFORMERS[3](i.0)).hash(), r270.hash());
		assert_eq!(Board(TRANSFORMERS[4](i.0)).hash(), v.hash());
		assert_eq!(Board(TRANSFORMERS[5](i.0)).hash(), h.hash());
		assert_eq!(Board(TRANSFORMERS[6](i.0)).hash(), dz.hash());

		let i = board_from_hash(616_101_616);
		assert_eq!(Board(TRANSFORMERS[0](i.0)).hash(), 616_101_616);
		assert_eq!(Board(TRANSFORMERS[1](i.0)).hash(), 616_101_616);
		assert_eq!(Board(TRANSFORMERS[2](i.0)).hash(), 616_101_616);
		assert_eq!(Board(TRANSFORMERS[3](i.0)).hash(), 616_101_616);
	}

	#[test]
	fn test_canonical() {
		let board = board_from_hash(616_101_616);
		let c = board.canonical();
		assert_eq!(c.1, 0);
		assert_eq!(Board(c.0).hash(), 616_101_616);
	}

	#[test]
	fn test_play_single_move() {
		let board = board_from_hash(616101616);
		let neighbors_buf = Vec::<(BoardIndex, DiceValue, BoardBitSize)>::from([
			(C_L_, 1, empty_cell_mask(C_L_)),
			(C_T_, 1, empty_cell_mask(C_T_)),
		]);
		let mut queue: Queue = HashMap::new();
		let mut moved = false;
		let mut spc: SymmetryPathCount = [0; SYMMETRY_COUNT as usize];
		spc[0] = 1;
		let rot: RotationIndex = 0;

		let b = board.0;
		play_single_move!(b, C_M_, rot, spc, queue, moved, neighbors_buf, 0, 1);

		assert_eq!(queue.len(), 1);
		assert!(moved);
		let first = queue.iter().next().unwrap();
		assert_eq!(Board(*first.0).hash(), 606021616);
		assert_eq!(*first.1, (0, spc));
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

	#[test]
	fn test_solve_empty() {
		let board = board_from_hash(0);
		assert_eq!(solve(0, Board(board.0)), 0);
		assert_eq!(solve(1, Board(board.0)), 111111111);
		assert_eq!(solve(2, Board(board.0)), 704035952);
		assert_eq!(solve(3, Board(board.0)), 840352818);
		assert_eq!(solve(4, Board(board.0)), 600875666);
		assert_eq!(solve(5, Board(board.0)), 50441886);
		assert_eq!(solve(6, Board(board.0)), 680243700);
		assert_eq!(solve(7, Board(board.0)), 597686656);
		assert_eq!(solve(8, Board(board.0)), 584450980);
		assert_eq!(solve(9, Board(board.0)), 55305380);
		assert_eq!(solve(10, Board(board.0)), 193520836);
		assert_eq!(solve(11, Board(board.0)), 521847116);
		assert_eq!(solve(12, Board(board.0)), 1054388152);
		assert_eq!(solve(13, Board(board.0)), 518795448);
		assert_eq!(solve(14, Board(board.0)), 366207036);
		assert_eq!(solve(15, Board(board.0)), 678967952);
		assert_eq!(solve(16, Board(board.0)), 476916052);
		assert_eq!(solve(17, Board(board.0)), 1009258340);
		assert_eq!(solve(18, Board(board.0)), 592651828);
		assert_eq!(solve(19, Board(board.0)), 1063467872);
		assert_eq!(solve(20, Board(board.0)), 400415524);
		assert_eq!(solve(21, Board(board.0)), 233248832);
		assert_eq!(solve(22, Board(board.0)), 230461008);
		assert_eq!(solve(23, Board(board.0)), 245411624);
		assert_eq!(solve(24, Board(board.0)), 899694236);
		assert_eq!(solve(25, Board(board.0)), 384163740);
		assert_eq!(solve(26, Board(board.0)), 888060600);
		assert_eq!(solve(27, Board(board.0)), 347933640);
		assert_eq!(solve(28, Board(board.0)), 340717612);
		assert_eq!(solve(29, Board(board.0)), 73295296);
		assert_eq!(solve(30, Board(board.0)), 851289228);
		assert_eq!(solve(31, Board(board.0)), 221286388);
		assert_eq!(solve(32, Board(board.0)), 375032784);
		assert_eq!(solve(33, Board(board.0)), 723342020);
		assert_eq!(solve(34, Board(board.0)), 92414440);
		assert_eq!(solve(35, Board(board.0)), 745533092);
		assert_eq!(solve(36, Board(board.0)), 331519112);
		assert_eq!(solve(37, Board(board.0)), 993643868);
		assert_eq!(solve(38, Board(board.0)), 72093236);
		assert_eq!(solve(39, Board(board.0)), 422667876);
		assert_eq!(solve(DEPTH_MAX, Board(board.0)), 503115192);
	}
}
