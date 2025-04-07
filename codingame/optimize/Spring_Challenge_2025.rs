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

const SYMMETRY_COUNT: u8 = 4;
type SymmetryPathCount = [PathCount; SYMMETRY_COUNT as usize];
type RotationIndex = u8;

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

// TODO: make transformers function faster
static TRANSFORMERS: [fn(BoardBitSize) -> BoardBitSize; SYMMETRY_COUNT as usize] = [
	// 0
	|board| board,
	// 90 clockwise
	|board| {
		(((board >> C_TL) & 0b111) << C_TR)
			| (((board >> C_T_) & 0b111) << C_R_)
			| (((board >> C_TR) & 0b111) << C_BR)
			| (((board >> C_L_) & 0b111) << C_T_)
			| (((board >> C_M_) & 0b111) << C_M_)
			| (((board >> C_R_) & 0b111) << C_B_)
			| (((board >> C_BL) & 0b111) << C_TL)
			| (((board >> C_B_) & 0b111) << C_L_)
			| (((board >> C_BR) & 0b111) << C_BL)
	},
	// 180
	|board| {
		(((board >> C_TL) & 0b111) << C_BR)
			| (((board >> C_T_) & 0b111) << C_B_)
			| (((board >> C_TR) & 0b111) << C_BL)
			| (((board >> C_L_) & 0b111) << C_R_)
			| (((board >> C_M_) & 0b111) << C_M_)
			| (((board >> C_R_) & 0b111) << C_L_)
			| (((board >> C_BL) & 0b111) << C_TR)
			| (((board >> C_B_) & 0b111) << C_T_)
			| (((board >> C_BR) & 0b111) << C_TL)
	},
	// 270 clockwise or 90 counter-clockwise
	|board| {
		(((board >> C_TL) & 0b111) << C_BL)
			| (((board >> C_T_) & 0b111) << C_L_)
			| (((board >> C_TR) & 0b111) << C_TL)
			| (((board >> C_L_) & 0b111) << C_B_)
			| (((board >> C_M_) & 0b111) << C_M_)
			| (((board >> C_R_) & 0b111) << C_T_)
			| (((board >> C_BL) & 0b111) << C_BR)
			| (((board >> C_B_) & 0b111) << C_R_)
			| (((board >> C_BR) & 0b111) << C_TR)
	},
];

static REVERSE_TRANSFORMERS: [fn(BoardBitSize) -> BoardBitSize; SYMMETRY_COUNT as usize] = [
	TRANSFORMERS[0],
	TRANSFORMERS[3],
	TRANSFORMERS[2],
	TRANSFORMERS[1],
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

	fn canonical(&self) -> (BoardBitSize, u8) {
		let mut min = (self.0, 0);
		// skipping the first one because it's the original
		for i in 1..SYMMETRY_COUNT {
			let transformed = TRANSFORMERS[i as usize](self.0);
			if transformed < min.0 {
				min = (transformed, i);
			}
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
				*stored_rotation = (*stored_rotation) % 4;
				let new_rotation = ($rotation + canonical.1) % 4;
				let delta_rotation = *stored_rotation + 4 - new_rotation;
				// let delta_rotation = new_rotation + 4 - *stored_rotation;
				let r = (delta_rotation % 4) as usize;
				// (((*stored_rotation % 4) + 4 - (($rotation + canonical.1) % 4)) % 4) as usize;
				count[0] = count[0].wrapping_add($symmetry_path_count[r]);
				count[1] = count[1].wrapping_add($symmetry_path_count[(r + 1) % 4]);
				count[2] = count[2].wrapping_add($symmetry_path_count[(r + 2) % 4]);
				count[3] = count[3].wrapping_add($symmetry_path_count[(r + 3) % 4]);
			})
			.or_insert((($rotation + canonical.1), $symmetry_path_count));
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
		let reversed_canonical = REVERSE_TRANSFORMERS[($rotation % 4) as usize]($board);
		for i in 0..SYMMETRY_COUNT {
			let current_rotation = ($rotation + i) % 4;
			$sum = $sum.wrapping_add(
				Board(REVERSE_TRANSFORMERS[current_rotation as usize](
					reversed_canonical,
				))
				.hash()
				.wrapping_mul($symmetry_path_count[current_rotation as usize]),
			);
		}
	};
}

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
	let mut sum: Sum = 0;
	let mut queue: Queue = HashMap::with_capacity(QUEUE_CAPACITY);
	let mut current_queue: Queue = HashMap::with_capacity(QUEUE_CAPACITY);
	let mut ngb_buf: [(BoardIndex, DiceValue, BoardBitSize); 4] = [(0, 0, 0); 4];
	let mut ngb_len: u8;
	let mut d = 0;

	// no need to compute first canonical
	// there will be no duplicates possible with only 1 board on depth 0
	queue.insert(starting_board.0, (0, [1, 0, 0, 0]));

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
	fn transform() {
		let board = board_from_hash(123_456_024);
		// no transformation
		assert_eq!(Board(TRANSFORMERS[0](board.0)).hash(), board.hash());
		// 90 clockwise
		assert_eq!(Board(TRANSFORMERS[1](board.0)).hash(), 041_252_463);
		// 180
		assert_eq!(Board(TRANSFORMERS[2](board.0)).hash(), 420_654_321);
		// 270 clockwise or 90 counter-clockwise
		assert_eq!(Board(TRANSFORMERS[3](board.0)).hash(), 364_252_140);
		// vertical flip
		// assert_eq!(Board(TRANSFORMERS[1](board.0)).hash(), 321_654_420);
		// horizontal flip
		// assert_eq!(Board(TRANSFORMERS[2](board.0)).hash(), 024_456_123);

		let board = board_from_hash(616_101_616);
		assert_eq!(Board(TRANSFORMERS[0](board.0)).hash(), 616_101_616);
		assert_eq!(Board(TRANSFORMERS[1](board.0)).hash(), 616_101_616);
		assert_eq!(Board(TRANSFORMERS[2](board.0)).hash(), 616_101_616);
		assert_eq!(Board(TRANSFORMERS[3](board.0)).hash(), 616_101_616);
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
		let spc: SymmetryPathCount = [1, 0, 0, 0];
		let rot = 0;

		let b = board.0;
		play_single_move!(b, C_M_, rot, spc, queue, moved, neighbors_buf, 0, 1);

		assert_eq!(queue.len(), 1);
		assert!(moved);
		let first = queue.iter().next().unwrap();
		assert_eq!(Board(*first.0).hash(), 606021616);
		assert_eq!(*first.1, (0, [1, 0, 0, 0]));
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
