#[path = "2048.rs"]
mod mod_2048;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::mod_2048::Board;
use crate::mod_2048::Move;
use crate::mod_2048::Score;
use crate::mod_2048::Seed;
use crate::mod_2048::R_A;
use crate::mod_2048::R_C;
use crate::mod_2048::R_M;

type Games = HashMap<Seed, (Board, Seed, Score)>;

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

fn read() -> Option<Vec<(Seed, Score)>> {
    let lines: Vec<String> = match File::open(FILE) {
        Ok(f) => BufReader::new(f).lines().map(|l| l.unwrap()).collect(),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                return Some(Vec::new());
            }
            _ => {
                err!("\"\x1b[35m{}\x1b[0m\x1b[1m\" {}", FILE, e);
                return None;
            }
        },
    };

    let mut ret = Vec::new();
    for l in lines {
        let mut s = l.split_whitespace();
        let seed = s.next().unwrap().parse::<Seed>().unwrap();
        let score = s.next().unwrap().parse::<Score>().unwrap();
        ret.push((seed, score));
    }

    Some(ret)
}

fn write(seed: Seed, score: Score, moves: &[Move]) -> Option<()> {
    let mut new_l = String::new();
    new_l.push_str(&format!("{} {} ", seed, score));
    for m in moves {
        new_l.push_str(&format!("{}", m));
    }

    let mut lines: Vec<String> = match File::open(FILE) {
        Ok(f) => BufReader::new(f).lines().map(|l| l.unwrap()).collect(),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Vec::new(),
            _ => {
                err!("\"\x1b[35m{}\x1b[0m\x1b[1m\" {}", FILE, e);
                return None;
            }
        },
    };

    let mut found = false;
    for l in &mut lines {
        if l.split_whitespace()
            .next()
            .unwrap()
            .parse::<Seed>()
            .unwrap()
            == seed
        {
            *l = new_l.clone();
            found = true;
            break;
        }
    }

    if !found {
        lines.push(new_l);
    }

    let mut f = File::create(FILE).unwrap();
    f.write_all(lines.join("\n").as_bytes()).unwrap();

    Some(())
}

fn parse() -> Option<Games> {
    if std::env::args().len() < 2 {
        eprintln!(
            "usage: \x1b[1m{} [\x1b[35;1m<seed>\x1b[0m\x1b[1m]\t\x1b[3m(one seed per arg)\x1b[0m",
            std::env::args().next().unwrap()
        );
        return None;
    }

    let mut games: Games = HashMap::new();

    for arg in std::env::args().skip(1) {
        match arg.parse::<Seed>() {
            Ok(seed) => {
                let mut b = Board::new();
                let cur_seed = b.spawn_tile(seed);
                let cur_seed = b.spawn_tile(cur_seed);
                games.insert(seed, (b, cur_seed, 0));
            }
            Err(e) => {
                err!("{}", e);
                return None;
            }
        }
    }

    Some(games)
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
