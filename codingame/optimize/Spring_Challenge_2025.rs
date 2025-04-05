#![allow(clippy::zero_prefixed_literal)]

use std::collections::VecDeque;

type Depth = u8;

type BoardBitSize = u32;
struct Board(BoardBitSize);
type BoardIndex = u8;

type Sum = u32;

const DICE_MAX: BoardBitSize = 6;

const SUM_MOD: Sum = 2 ^ 30;

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

	let starting_board = Board::parse();

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
	fn parse() -> Self {
		let ascii_zero = '0' as BoardBitSize;
		let mut line = String::with_capacity(6);

		std::io::stdin().read_line(&mut line).unwrap();
		let mut chars = line.chars();
		let c0 = chars.next().unwrap() as BoardBitSize - ascii_zero;
		chars.next();
		let c1 = chars.next().unwrap() as BoardBitSize - ascii_zero;
		chars.next();
		let c2 = chars.next().unwrap() as BoardBitSize - ascii_zero;

		std::io::stdin().read_line(&mut line).unwrap();
		let mut chars = line.chars();
		let c3 = chars.next().unwrap() as BoardBitSize - ascii_zero;
		chars.next();
		let c4 = chars.next().unwrap() as BoardBitSize - ascii_zero;
		chars.next();
		let c5 = chars.next().unwrap() as BoardBitSize - ascii_zero;

		std::io::stdin().read_line(&mut line).unwrap();
		let mut chars = line.chars();
		let c6 = chars.next().unwrap() as BoardBitSize - ascii_zero;
		chars.next();
		let c7 = chars.next().unwrap() as BoardBitSize - ascii_zero;
		chars.next();
		let c8 = chars.next().unwrap() as BoardBitSize - ascii_zero;

		Self(
			(c0 << C_TL)
				| (c1 << C_T_)
				| (c2 << C_TR)
				| (c3 << C_L_)
				| (c4 << C_M_)
				| (c5 << C_R_)
				| (c6 << C_BL)
				| (c7 << C_B_)
				| (c8 << C_BR),
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

		play_move!(board, C_BR, d, queue, neighbor_buf, C_B_, C_R_);
		play_move!(board, C_B_, d, queue, neighbor_buf, C_BL, C_M_, C_BR);
		play_move!(board, C_BL, d, queue, neighbor_buf, C_L_, C_B_);
		play_move!(board, C_R_, d, queue, neighbor_buf, C_M_, C_TR, C_BR);
		play_move!(board, C_M_, d, queue, neighbor_buf, C_L_, C_T_, C_R_, C_B_);
		play_move!(board, C_L_, d, queue, neighbor_buf, C_TL, C_M_, C_BL);
		play_move!(board, C_TR, d, queue, neighbor_buf, C_T_, C_R_);
		play_move!(board, C_T_, d, queue, neighbor_buf, C_TL, C_TR, C_M_);
		play_move!(board, C_TL, d, queue, neighbor_buf, C_T_, C_L_);

		dbg!(d, queue.len());
	}

	sum
}

#[cfg(test)]
mod tests {
	use super::*;

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
}
