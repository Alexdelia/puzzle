use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind};

use crate::err;
use crate::game::{Score, Seed};

pub fn read(file: &str) -> Option<Vec<(Seed, Score)>> {
    let lines: Vec<String> = match File::open(file) {
        Ok(f) => BufReader::new(f).lines().map(|l| l.unwrap()).collect(),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                return Some(Vec::new());
            }
            _ => {
                err!("\"\x1b[35m{}\x1b[0m\x1b[1m\" {}", file, e);
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
