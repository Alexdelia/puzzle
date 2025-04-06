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

const DICE_MAX: DiceValue = 6;

const SUM_MOD: Sum = 1 << 30;

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

impl Board {
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
		(self.get(C_TL) as BoardBitSize * 100_000_000)
			+ (self.get(C_T_) as BoardBitSize * 010_000_000)
			+ (self.get(C_TR) as BoardBitSize * 001_000_000)
			+ (self.get(C_L_) as BoardBitSize * 000_100_000)
			+ (self.get(C_M_) as BoardBitSize * 000_010_000)
			+ (self.get(C_R_) as BoardBitSize * 000_001_000)
			+ (self.get(C_BL) as BoardBitSize * 000_000_100)
			+ (self.get(C_B_) as BoardBitSize * 000_000_010)
			+ self.get(C_BR) as BoardBitSize
	}
}

macro_rules! queue_insert {
	($queue:ident, $board:expr, $path_count:ident) => {
		let board_handle: Board = $board;
		if let Some(count) = $queue.get_mut(&board_handle) {
			*count = (*count).wrapping_add($path_count);
		} else {
			$queue.insert(board_handle, $path_count);
		}
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
					& empty_cell_mask($neighbors_buf[$neighbors].0)
				)+,
				$index,
				n as BoardBitSize
			)), $path_count);
		}
	};
}

macro_rules! play_move {
	($board:ident, $index:ident, $path_count:ident, $queue:ident, $moved:ident, $neighbors_buf:ident, $($neighbors:ident),+) => {
		if $board.get($index) == 0 {
            $moved = true;

			$neighbors_buf.clear();
			$(
				let neighbor = $board.get($neighbors);
				if neighbor != 0 && neighbor != DICE_MAX {
					$neighbors_buf.push(($neighbors, neighbor));
				}
			)+

            if $neighbors_buf.len() <= 1 {
                queue_insert!($queue, Board(set($board.0, empty_cell_mask($index), $index, 1)), $path_count);
            } else {
                match $neighbors_buf.len() {
                    2 => {
                        let n = $neighbors_buf[0].1 + $neighbors_buf[1].1;
                        if n <= DICE_MAX {
                            queue_insert!($queue, Board(set(
		    					$board.0,
		    					empty_cell_mask($index)
		    					& empty_cell_mask($neighbors_buf[0].0)
		    					& empty_cell_mask($neighbors_buf[1].0),
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
	($sum:ident, $board:ident, $path_count:ident) => {
		$sum = $sum.wrapping_add($board.hash().wrapping_mul($path_count));
	};
}

fn solve(depth: Depth, starting_board: Board) -> Sum {
	let mut sum: Sum = 0;
	let mut queue: HashMap<Board, PathCount> = HashMap::new();
	let mut current_queue: HashMap<Board, PathCount> = HashMap::new();
	let mut ngb_buf = Vec::<(BoardIndex, DiceValue)>::with_capacity(4);
	let mut d = 0;

	queue.insert(starting_board, 1);

	while d < depth && !queue.is_empty() {
		std::mem::swap(&mut queue, &mut current_queue);
		// current_queue.extend(queue.drain());
		for (board, pc) in current_queue.drain() {
			let mut moved = false;

			play_move!(board, C_BR, pc, queue, moved, ngb_buf, C_B_, C_R_);
			play_move!(board, C_B_, pc, queue, moved, ngb_buf, C_BL, C_M_, C_BR);
			play_move!(board, C_BL, pc, queue, moved, ngb_buf, C_L_, C_B_);
			play_move!(board, C_R_, pc, queue, moved, ngb_buf, C_M_, C_TR, C_BR);
			play_move!(
				board, C_M_, pc, queue, moved, ngb_buf, C_L_, C_T_, C_R_, C_B_
			);
			play_move!(board, C_L_, pc, queue, moved, ngb_buf, C_TL, C_M_, C_BL);
			play_move!(board, C_TR, pc, queue, moved, ngb_buf, C_T_, C_R_);
			play_move!(board, C_T_, pc, queue, moved, ngb_buf, C_TL, C_TR, C_M_);
			play_move!(board, C_TL, pc, queue, moved, ngb_buf, C_T_, C_L_);

			if !moved {
				sum!(sum, board, pc);
			}
		}

		d += 1;
	}

	for (board, path_count) in queue {
		sum!(sum, board, path_count);
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
	fn test_play_single_move() {
		let board = board_from_hash(616101616);
		let neighbors_buf = Vec::<(BoardIndex, DiceValue)>::from([(C_L_, 1), (C_T_, 1)]);
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
