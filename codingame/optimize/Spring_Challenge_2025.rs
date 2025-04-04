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

impl Board {
	fn parse() -> Self {
		let mut board = Self(0);

		for i in 0..3 {
			let mut inputs = String::with_capacity(6);
			std::io::stdin().read_line(&mut inputs).unwrap();
			let line = inputs.split_whitespace().collect::<Vec<_>>();

			let shift = i * 3;
			board.set(shift, parse_input!(line[0], BoardBitSize));
			board.set(shift + 1, parse_input!(line[1], BoardBitSize));
			board.set(shift + 2, parse_input!(line[2], BoardBitSize));
		}

		board
	}

	pub fn set(&mut self, index: BoardIndex, value: BoardBitSize) {
		let shift = index * 3;
		let mask = !(0b111 << shift);
		self.0 = (self.0 & mask) | (value << shift);
	}

	pub fn get(&self, index: BoardIndex) -> BoardBitSize {
		(self.0 >> (index * 3)) & 0b111
	}

	const C_BR: BoardIndex = 0;
	const C_B_: BoardIndex = 1;
	const C_BL: BoardIndex = 2;
	const C_R_: BoardIndex = 3;
	const C_M_: BoardIndex = 4;
	const C_L_: BoardIndex = 5;
	const C_TR: BoardIndex = 6;
	const C_T_: BoardIndex = 7;
	const C_TL: BoardIndex = 8;
}

macro_rules! play_move {
    ($board:ident, $cell:ident, $depth:ident, $queue:ident, $($neighbors:ident),+) => {
        if cell_value!($board, $cell) == 0 {
            let n = 0 $(+ cell_value!($board, $neighbors))+;
            if n == 0 || n > DICE_MAX {
                $queue.push_back(($board + $cell, $depth));
            } else {
                $queue.push_back((
                    $board
                        $( - raw_cell_value!($board, $neighbors) )+
                        + ($cell * n),
                    $depth,
                ));
            }
        }
    };
}

fn solve(depth: Depth, starting_board: Board) -> Sum {
	let mut sum: Sum = 0;
	let mut queue = VecDeque::<(Board, Depth)>::new();

	queue.push_back((starting_board, 0));

	while let Some((board, d)) = queue.pop_front() {
		if d == depth {
			sum = (sum + board) % SUM_MOD;
			continue;
		}
		let d = d + 1;

		play_move!(board, C_BR, d, queue, C_B_, C_R_);
		play_move!(board, C_B_, d, queue, C_BL, C_M_, C_BR);
		play_move!(board, C_BL, d, queue, C_L_, C_B_);
		play_move!(board, C_R_, d, queue, C_M_, C_TR, C_BR);
		play_move!(board, C_M_, d, queue, C_L_, C_T_, C_R_, C_B_);
		play_move!(board, C_L_, d, queue, C_TL, C_M_, C_BL);
		play_move!(board, C_TR, d, queue, C_T_, C_R_);
		play_move!(board, C_T_, d, queue, C_TL, C_TR, C_M_);
		play_move!(board, C_TL, d, queue, C_T_, C_L_);

		dbg!(d, queue.len(), board);
	}

	sum
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_get() {
		let board = Board(0b_001_010_011_100_101_110_000_010_100);
		assert_eq!(board.get(Board::C_BR), 1);
		assert_eq!(board.get(Board::C_B_), 2);
		assert_eq!(board.get(Board::C_BL), 3);
		assert_eq!(board.get(Board::C_R_), 4);
		assert_eq!(board.get(Board::C_M_), 5);
		assert_eq!(board.get(Board::C_L_), 6);
		assert_eq!(board.get(Board::C_TR), 0);
		assert_eq!(board.get(Board::C_T_), 2);
		assert_eq!(board.get(Board::C_TL), 4);
	}
}
