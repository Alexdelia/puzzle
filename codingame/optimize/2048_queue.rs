#[path = "2048.rs"]
mod mod_2048;

#[path = "2048_search.rs"]
mod mod_search;

use std::collections::BinaryHeap;
use std::process::ExitCode;

use crate::mod_2048::Board;
use crate::mod_2048::Move;
use crate::mod_2048::Seed;
use crate::mod_2048::SIZE;

macro_rules! err {
	($($arg:tt)*) => {
		eprint!("\x1b[31;1merror\x1b[0m\x1b[1m:\t");
		eprint!($($arg)*);
		eprintln!("\x1b[0m");
	};
}

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

        p * 10_000_000 + board.score
    }
}

fn solve(board: Board, seed: Seed) -> Board {
    let mut q = BinaryHeap::<Game>::new();
    let mut best: Board = Board::new();
    let m: [Move; 4] = [Move::Up, Move::Left, Move::Right, Move::Down];
    let mut c: usize = 0;

    q.push(Game::new(board, seed));

    while !q.is_empty() {
        let g = q.pop().unwrap();
        if g.board.score > best.score {
            best = g.board.clone();
            println!(
                "\x1b[3m(ongoing)\x1b[0m\t\x1b[32;1m{}\t\x1b[35;1m{}\t\x1b[31;1m{}\x1b[0m\t\x1b[33;1m{}\x1b[0m",
                best.score,
                best.moves.len(),
                c,
				q.len()
            );
            dbg!(&best);
        }

        for i in m {
            let mut b = g.board.clone();
            if b.play(i) {
                let s = b.spawn_tile(g.seed);
                if b.is_over() {
                    if b.score > best.score {
                        best = b.clone();
                        println!(
                            "\x1b[32;1m{}\t\x1b[35;1m{}\t\x1b[31;1m{}\x1b[0m\t\x1b[33;1m{}\x1b[0m",
                            best.score,
                            best.moves.len(),
                            c,
                            q.len()
                        );
                        dbg!(&best);
                    }
                    continue;
                }
                q.push(Game::new(b, s));
                c += 1;
            }
        }
    }

    best
}

fn main() -> ExitCode {
    if std::env::args().len() != 2 {
        err!(
            "usage: \x1b[1m{} [\x1b[35;1m<seed>\x1b[0m",
            std::env::args().next().unwrap()
        );
        return ExitCode::FAILURE;
    }

    let mut seed = std::env::args().nth(1).unwrap().parse::<Seed>().unwrap();
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
