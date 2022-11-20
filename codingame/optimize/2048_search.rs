#[path = "2048.rs"]
mod mod_2048;

use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::mod_2048::Board;
use crate::mod_2048::Move;
use crate::mod_2048::Score;
use crate::mod_2048::Seed;
use crate::mod_2048::R_A;
use crate::mod_2048::R_C;
use crate::mod_2048::R_M;

const FILE: &str = ".2048_results.out";

macro_rules! err {
	($($arg:tt)*) => {
		eprint!("\x1b[31;1merror\x1b[0m\x1b[1m:\t");
		eprint!($($arg)*);
		eprintln!("\x1b[0m");
	};
}

fn play(mut board: Board, mut seed: Seed) -> (Vec<Move>, Score) {
    let mut am: Vec<Move>;
    let mut sm = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    while !board.is_over() {
        am = vec![Move::Up, Move::Down, Move::Left, Move::Right];
        while !board.play(am.remove(sm as usize % am.len())) {
            sm = (R_A * sm + R_C) % R_M;
            if am.is_empty() {
                panic!("Board.is_over() has been checked but no move change the board");
            }
        }

        seed = board.spawn_tile(seed);
    }

    (board.moves, board.score)
}

fn read() -> Option<Score> {
    let lines: Vec<String> = match File::open(FILE) {
        Ok(f) => BufReader::new(f).lines().map(|l| l.unwrap()).collect(),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                err!("\"\x1b[35m{}\x1b[0m\x1b[1m\" not found", FILE);
                return None;
            }
            _ => {
                err!("\"\x1b[35m{}\x1b[0m\x1b[1m\" {}", FILE, e);
                return None;
            }
        },
    };
}

fn write(seed: Seed, score: Score, moves: Vec<Move>) {}

fn main() {
    if std::env::args().len() != 2 {
        eprintln!("usage: {} <seed>", std::env::args().nth(0).unwrap());
        return;
    }
    let mut seed = match std::env::args().nth(1).unwrap().parse::<mod_2048::Seed>() {
        Ok(seed) => seed,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let mut board = Board::new();
    seed = board.spawn_tile(seed);
    seed = board.spawn_tile(seed);

    let mut best_score = match read() {
        Some(score) => score,
        None => return,
    };

    loop {
        let (moves, score) = play(board.clone(), seed);

        if score > best_score {
            best_score = score;
            write(seed, score, moves);
            println!("\x1b[32;1m{} \x1b[35;1m{}", best_score, moves.len());
        }
    }
}
