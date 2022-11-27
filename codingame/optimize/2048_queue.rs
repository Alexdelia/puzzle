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

#[derive(Eq, PartialEq)]
struct Game {
    board: Board,
    seed: Seed,
    priority: Priority,
}

impl Ord for Game {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.priority.cmp(&self.priority)
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
            board,
            seed,
            priority: Game::priority(&board),
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
        for i in 1..18 {
            if r[i] == 1 {
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
    let mut best: Board;

    q.push(Game::new(board, seed));

    while !q.is_empty() {}

    best
}

fn main() -> ExitCode {
    let mut games: Games = parse().unwrap_or_else(|| {
        std::process::exit(1);
    });

    {
        let bs = read().unwrap_or_else(|| {
            std::process::exit(2);
        });

        for s in bs {
            if let Some((_, _, score)) = games.get_mut(&s.0) {
                *score = s.1;
            }
        }
    }

    let mut c: usize = 0;

    loop {
        for (seed, (board, cur_seed, score)) in games.iter_mut() {
            let (moves, new_score) = play(board.clone(), *cur_seed);
            if new_score > *score {
                *score = new_score;
                print!("\x1b[33;1m{}\x1b[0m:\t", seed);
                write(*seed, *score, &moves);
                println!(
                    "\x1b[32;1m{} \x1b[35;1m{} \x1b[31;1m{}\x1b[0m",
                    *score,
                    moves.len(),
                    c
                );
            }
        }

        c += 1;
    }
}

mod test {
	
