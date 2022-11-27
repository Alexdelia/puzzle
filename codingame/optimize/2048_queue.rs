#[path = "2048.rs"]
mod mod_2048;

#[path = "2048_search.rs"]
mod mod_search;

use std::collections::BinaryHeap;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::mod_2048::Board;
use crate::mod_2048::Move;
use crate::mod_2048::Score;
use crate::mod_2048::Seed;
use crate::mod_2048::R_A;
use crate::mod_2048::R_C;
use crate::mod_2048::R_M;
use crate::mod_2048::SIZE;

use crate::mod_2048::next;

type Priority = u32;

struct Game {
    board: Board,
    seed: Seed,
    priority: Priority,
}

impl Eq for Game {}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Ord for Game {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for Game {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Game {
    fn new(board: Board, seed: Seed) -> Self {
        Self {
            priority: Game::priority(&board),
            board,
            seed,
        }
    }

    fn priority(board: &Board) -> Priority {
        let mut r = vec![0; 18];
        for x in 0..SIZE {
            for y in 0..SIZE {
                r[board.board[x][y] as usize] += 1;
            }
        }

        let mut p = r[0];
        for i in r.iter().take(18).skip(1) {
            if i == &1 {
                p += 1;
            }
        }

        p
    }
}

macro_rules! err {
	($($arg:tt)*) => {
		eprint!("\x1b[31;1merror\x1b[0m\x1b[1m:\t");
		eprint!($($arg)*);
		eprintln!("\x1b[0m");
	};
}

fn solve(board: Board, seed: Seed) -> Board {
    let mut q = BinaryHeap::<Game>::new();
    let mut best: Board = Board::new();

    q.push(Game::new(board, seed));

    while !q.is_empty() {}

    best
}

fn main() -> ExitCode {
    if std::env::args().len() != 2 {
        err!(
            "usage: \x1b[1m{} [\x1b[35;1m<seed>\x1b[0m\x1b[1m]\t\x1b[3m(one seed per arg)\x1b[0m",
            std::env::args().next().unwrap()
        );
        return ExitCode::FAILURE;
    }

    let mut seed = std::env::args().next().unwrap().parse::<Seed>().unwrap();
    let mut board = Board::new();

    seed = board.spawn_tile(seed);
    seed = board.spawn_tile(seed);

    solve(board, seed);

    ExitCode::SUCCESS
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn binheap() {
        let mut q = BinaryHeap::<Game>::new();

        let mut b = Board::new();
        b.spawn_tile(0);
        b.spawn_tile(0);
        q.push(Game::new(b.clone(), 0));
        dbg!(q.peek().unwrap().priority);

        b.spawn_tile(0);
        b.spawn_tile(0);
        b.spawn_tile(0);
        b.spawn_tile(0);
        q.push(Game::new(b.clone(), 0));

        let p1 = q.pop().unwrap().priority;
        let p2 = q.pop().unwrap().priority;
        dbg!(p1);
        dbg!(p2);
        assert!(p1 > p2);
    }
}
