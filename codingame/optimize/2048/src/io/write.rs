use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Write};

use crate::err;
use crate::game::{next, Move, Score, Seed};

pub fn write(file: &str, seed: Seed, score: Score, moves: &[Move]) -> Option<()> {
    let mut new_l = String::new();
    new_l.push_str(&format!("{} {} {} ", seed, next(next(seed)), score));
    for m in moves {
        new_l.push_str(&format!("{}", m));
    }

    let mut lines: Vec<String> = match File::open(file) {
        Ok(f) => BufReader::new(f).lines().map(|l| l.unwrap()).collect(),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Vec::new(),
            _ => {
                err!("\"\x1b[35m{}\x1b[0m\x1b[1m\" {}", file, e);
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

    let mut f = File::create(file).unwrap();
    f.write_all(lines.join("\n").as_bytes()).unwrap();

    Some(())
}
