#![allow(clippy::zero_prefixed_literal)]

use std::collections::VecDeque;

type Depth = u8;
type Board = u32;
type Sum = u32;

const DICE_MAX: Board = 6;

const SUM_MOD: Sum = 2 ^ 30;

fn main() {
	let (depth, starting_board) = parse();
	dbg!(depth, starting_board);

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

	let mut starting_board: u32 = 0;

	for _ in 0..3_usize {
		let mut inputs = String::with_capacity(6);
		std::io::stdin().read_line(&mut inputs).unwrap();
		for x in inputs.split_whitespace() {
			starting_board *= 10;
			starting_board += parse_input!(x, u32);
		}
	}

	(depth, starting_board)
}

// cells
const C_BR: Board = 000_000_001;
const C_B_: Board = 000_000_010;
const C_BL: Board = 000_000_100;
const C_R_: Board = 000_001_000;
const C_M_: Board = 000_010_000;
const C_L_: Board = 000_100_000;
const C_TR: Board = 001_000_000;
const C_T_: Board = 010_000_000;
const C_TL: Board = 100_000_000;

macro_rules! cell {
	($b:expr, $c:expr) => {
		(($b / $c) % 10)
	};
}

macro_rules! raw_cell {
	($b:expr, $c:expr) => {
		((($b / $c) % 10) * $c)
	};
}

macro_rules! play_move {
    ($board:ident, $cell:ident, $depth:ident, $queue:ident, $($neighbors:ident),+) => {
        if cell!($board, $cell) == 0 {
            let n = 0 $(+ cell!($board, $neighbors))+;
            if n == 0 || n > DICE_MAX {
                $queue.push_back(($board + $cell, $depth));
            } else {
                $queue.push_back((
                    $board
                        $( - raw_cell!($board, $neighbors) )+
                        + $cell,
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

		dbg!(d, queue.len());
	}

	sum
}
