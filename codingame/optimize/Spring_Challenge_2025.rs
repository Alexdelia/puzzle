#![allow(clippy::zero_prefixed_literal)]

use std::collections::VecDeque;
use std::io::Read;

type Depth = u8;

type BoardBitSize = u32;
struct Board(BoardBitSize);
type BoardIndex = u8;

type Sum = u32;

const DICE_MAX: BoardBitSize = 6;

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
	dbg!(starting_board.hash());

	(depth, starting_board)
}

const C_BR: BoardIndex = 0;
const C_B_: BoardIndex = 3;
const C_BL: BoardIndex = 6;
const C_R_: BoardIndex = 9;
const C_M_: BoardIndex = 12;
const C_L_: BoardIndex = 15;
const C_TR: BoardIndex = 18;
const C_T_: BoardIndex = 21;
const C_TL: BoardIndex = 24;

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
	fn set_single(&self, index: BoardIndex, value: BoardBitSize) -> BoardBitSize {
		(self.0 & !(0b111 << index)) | (value << index)
	}

	#[inline]
	fn get(&self, index: BoardIndex) -> BoardBitSize {
		(self.0 >> index) & 0b111
	}

	#[inline]
	fn hash(&self) -> BoardBitSize {
		(self.get(C_TL) * 100_000_000)
			+ (self.get(C_T_) * 010_000_000)
			+ (self.get(C_TR) * 001_000_000)
			+ (self.get(C_L_) * 000_100_000)
			+ (self.get(C_M_) * 000_010_000)
			+ (self.get(C_R_) * 000_001_000)
			+ (self.get(C_BL) * 000_000_100)
			+ (self.get(C_B_) * 000_000_010)
			+ self.get(C_BR)
	}
}

macro_rules! play_single_move {
    ($board:ident, $index:ident, $depth:ident, $queue:ident, $neighbors_buf:ident, $($neighbors:literal),+) => {
        let n = 0 $(
            + $neighbors_buf[$neighbors].1
        )+;
        if n <= DICE_MAX {
            $queue.push_back((Board(
                $board.set_single($index, n)
                $(
                    | $board.set_single($neighbors_buf[$neighbors].0, 0)
                )+
            ), $depth));
        }
    };
}

macro_rules! play_move {
    ($board:ident, $index:ident, $depth:ident, $queue:ident, $neighbors_buf:ident, $($neighbors:ident),+) => {
        if $board.get($index) == 0 {
            $neighbors_buf.clear();
            $(
                let neighbor = $board.get($neighbors);
                if neighbor != 0 && neighbor != DICE_MAX {
                    $neighbors_buf.push(($neighbors, neighbor));
                }
            )+

            if $neighbors_buf.len() <= 1 {
                $queue.push_back((Board($board.set_single($index, 1)), $depth));
            } else {
                if $neighbors_buf.len() == 2 {
                    let n = $neighbors_buf[0].1 + $neighbors_buf[1].1;
                    if n <= DICE_MAX {
                        $queue.push_back((Board(
                            $board.set_single($index, n)
                            | $board.set_single($neighbors_buf[0].0, 0)
                            | $board.set_single($neighbors_buf[1].0, 0)
                        ), $depth));
                    } else {
                        $queue.push_back((Board($board.set_single($index, 1)), $depth));
                    }
                } else if $neighbors_buf.len() == 3 {
                    let len = $queue.len();

                    // 2 of 3
                    play_single_move!($board, $index, $depth, $queue, $neighbors_buf, 0, 1);
                    play_single_move!($board, $index, $depth, $queue, $neighbors_buf, 0, 2);
                    play_single_move!($board, $index, $depth, $queue, $neighbors_buf, 1, 2);
                    // 3 of 3
                    play_single_move!($board, $index, $depth, $queue, $neighbors_buf, 0, 1, 2);

                    if $queue.len() == len {
                        $queue.push_back((Board($board.set_single($index, 1)), $depth));
                    }
                } else {
                    let len = $queue.len();

                    // 2 of 4
                    play_single_move!($board, $index, $depth, $queue, $neighbors_buf, 0, 1);
                    play_single_move!($board, $index, $depth, $queue, $neighbors_buf, 0, 2);
                    play_single_move!($board, $index, $depth, $queue, $neighbors_buf, 0, 3);
                    play_single_move!($board, $index, $depth, $queue, $neighbors_buf, 1, 2);
                    play_single_move!($board, $index, $depth, $queue, $neighbors_buf, 1, 3);
                    play_single_move!($board, $index, $depth, $queue, $neighbors_buf, 2, 3);
                    // 3 of 4
                    play_single_move!($board, $index, $depth, $queue, $neighbors_buf, 0, 1, 2);
                    play_single_move!($board, $index, $depth, $queue, $neighbors_buf, 0, 1, 3);
                    play_single_move!($board, $index, $depth, $queue, $neighbors_buf, 0, 2, 3);
                    play_single_move!($board, $index, $depth, $queue, $neighbors_buf, 1, 2, 3);
                    // 4 of 4
                    play_single_move!($board, $index, $depth, $queue, $neighbors_buf, 0, 1, 2, 3);

                    if $queue.len() == len {
                        $queue.push_back((Board($board.set_single($index, 1)), $depth));
                    }
                }
            }
        }
    };
}

fn solve(depth: Depth, starting_board: Board) -> Sum {
	let mut sum: Sum = 0;
	let mut queue = VecDeque::<(Board, Depth)>::new();
	let mut neighbor_buf = Vec::<(BoardIndex, BoardBitSize)>::with_capacity(4);

	queue.push_back((starting_board, 0));

	while let Some((board, d)) = queue.pop_front() {
		if d == depth {
			sum = (sum + board.hash()) % SUM_MOD;
			continue;
		}
		let d = d + 1;

		let queue_len = queue.len();

		play_move!(board, C_BR, d, queue, neighbor_buf, C_B_, C_R_);
		play_move!(board, C_B_, d, queue, neighbor_buf, C_BL, C_M_, C_BR);
		play_move!(board, C_BL, d, queue, neighbor_buf, C_L_, C_B_);
		play_move!(board, C_R_, d, queue, neighbor_buf, C_M_, C_TR, C_BR);
		play_move!(board, C_M_, d, queue, neighbor_buf, C_L_, C_T_, C_R_, C_B_);
		play_move!(board, C_L_, d, queue, neighbor_buf, C_TL, C_M_, C_BL);
		play_move!(board, C_TR, d, queue, neighbor_buf, C_T_, C_R_);
		play_move!(board, C_T_, d, queue, neighbor_buf, C_TL, C_TR, C_M_);
		play_move!(board, C_TL, d, queue, neighbor_buf, C_T_, C_L_);

		if queue.len() == queue_len {
			sum = (sum + board.hash()) % SUM_MOD;
		}
	}

	sum
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
	fn test_single() {
		let board = Board(0b_001_010_011_100_101_110_000_010_100);
		assert_eq!(
			board.set_single(C_TL, 0),
			0b_000_010_011_100_101_110_000_010_100
		);
		assert_eq!(
			board.set_single(C_T_, 0),
			0b_001_000_011_100_101_110_000_010_100
		);
		assert_eq!(
			board.set_single(C_TR, 0),
			0b_001_010_000_100_101_110_000_010_100
		);
		assert_eq!(
			board.set_single(C_L_, 0),
			0b_001_010_011_000_101_110_000_010_100
		);
		assert_eq!(
			board.set_single(C_M_, 0),
			0b_001_010_011_100_000_110_000_010_100
		);
		assert_eq!(
			board.set_single(C_R_, 0),
			0b_001_010_011_100_101_000_000_010_100
		);
		assert_eq!(
			board.set_single(C_BL, 0),
			0b_001_010_011_100_101_110_000_010_100
		);
		assert_eq!(
			board.set_single(C_B_, 0),
			0b_001_010_011_100_101_110_000_000_100
		);
		assert_eq!(
			board.set_single(C_BR, 0),
			0b_001_010_011_100_101_110_000_010_000
		);
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
		let mut neighbors_buf = Vec::<(BoardIndex, BoardBitSize)>::from([(C_L_, 1), (C_T_, 1)]);
		let mut queue = VecDeque::<(Board, Depth)>::new();
		let depth = 1;

		play_single_move!(board, C_M_, depth, queue, neighbors_buf, 0, 1);

		assert_eq!(queue.len(), 1);
		assert_eq!(queue[0].0.hash(), 606021616);
	}

	#[test]
	fn test_solve() {
		assert_eq!(solve(20, board_from_hash(60222161)), 322444322);
		assert_eq!(solve(20, board_from_hash(506450064)), 951223336);
		assert_eq!(solve(1, board_from_hash(555005555)), 36379286);
		// assert_eq!(solve(1, board_from_hash(616101616)), 264239762);
	}
}
