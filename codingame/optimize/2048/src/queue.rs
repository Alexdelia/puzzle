use std::collections::BinaryHeap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::ExitCode;

use lib2048::err;
use lib2048::game::{Board, Cell, Move, Score, Seed, SIZE};
use lib2048::io::{read::read, write::write, FILE_RESULT};
use lib2048::priority::{priority, Priority};

const MIN_SIZE: usize = 1_000;
const MAX_SIZE: usize = 1_000_000;
const FILE: &str = ".2048_queue.mem";

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
            priority: priority(&board),
            board,
            seed,
        }
    }
}

fn ouput(board: &Board, counter: usize, q_size: usize, over: bool) {
    println!(
        "\x1b[1m{}\t\x1b[32;1m{}\t\x1b[35;1m{}\t\x1b[31;1m{}\x1b[0m\t\x1b[33;1m{}\x1b[0m",
        over,
        board.score,
        board.moves.len(),
        counter,
        q_size,
    );
    dbg!(board);
}

fn q_in(q: &mut BinaryHeap<Game>) -> usize {
    eprint!("q_in\t{}\t", q.len());
    let mut lines: Vec<String> = match File::open(FILE) {
        Ok(f) => BufReader::new(f).lines().map(|l| l.unwrap()).collect(),
        Err(e) => {
            err!("\"\x1b[35m{}\x1b[0m\x1b[1m\" {}", FILE, e);
            return 0;
        }
    };

    while q.len() < MAX_SIZE && !lines.is_empty() {
        let l = lines.pop().unwrap();
        let mut s = l.split(' ');
        let _priority = s.next().unwrap().parse::<Priority>().unwrap();
        let seed = s.next().unwrap().parse::<Seed>().unwrap();
        let mut board = Board::new();
        board.score = s.next().unwrap().parse::<Score>().unwrap();
        let b = s.next().unwrap().split(',');
        for (i, c) in b.enumerate() {
            board.board[i / SIZE][i % SIZE] = c.parse::<Cell>().unwrap();
        }
        board.moves = s
            .next()
            .unwrap()
            .chars()
            .map(|c| Move::from(c).unwrap())
            .collect();
        q.push(Game::new(board, seed));
    }

    let mut f = File::create(FILE).unwrap();
    f.write_all(lines.join("\n").as_bytes()).unwrap();

    eprintln!("{}", q.len());
    q.len()
}

fn q_out(mut q: BinaryHeap<Game>) -> BinaryHeap<Game> {
    eprint!("q_out\t");
    let mut ret: BinaryHeap<Game> = BinaryHeap::new();

    while !q.is_empty() && ret.len() < MIN_SIZE {
        ret.push(q.pop().unwrap());
    }

    /*
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(FILE)
        .unwrap();

    let mut lines: Vec<String> = Vec::with_capacity(q.len());

    while !q.is_empty() {
        let g = q.pop().unwrap();
        let mut l = String::new();
        l.push_str(&g.priority.to_string());
        l.push(' ');
        l.push_str(&g.seed.to_string());
        l.push(' ');
        l.push_str(&g.board.score.to_string());
        l.push(' ');
        for i in g.board.board.iter().flatten() {
            l.push_str(&i.to_string());
            l.push(',');
        }
        l.pop();
        l.push(' ');
        for i in g.board.moves.iter() {
            l.push_str(&i.to_string());
        }
        lines.push(l);
    }

    file.write_all(lines.join("\n").as_bytes()).unwrap();
    */

    eprintln!("{}", ret.len());
    ret
}

fn upout(
    best: &mut Board,
    saved: &mut (Seed, Score),
    board: &Board,
    counter: usize,
    q_size: usize,
    over: bool,
) {
    if board.score > best.score {
        *best = board.clone();
        ouput(best, counter, q_size, over);
        if best.score > saved.1 {
            let m: Vec<Move> = best.moves.clone().into();
            write(FILE_RESULT, saved.0, best.score, &m);
        }
    }
}

fn solve(board: Board, seed: Seed, mut saved: (Seed, Score)) -> Board {
    let mut q = BinaryHeap::<Game>::new();
    let mut best: Board = Board::new();
    let m: [Move; 4] = [Move::Up, Move::Left, Move::Right, Move::Down];
    let mut c: usize = 0;

    q.push(Game::new(board, seed));

    while !q.is_empty() || q_in(&mut q) > 0 {
        // print!("{}\r", q.len());
        let g = q.pop().unwrap();
        upout(&mut best, &mut saved, &g.board, c, q.len(), false);

        for i in m {
            let mut b = g.board.clone();
            if b.play(i) {
                let s = b.spawn_tile(g.seed);
                if b.is_over() {
                    upout(&mut best, &mut saved, &b, c, q.len(), true);
                    continue;
                }
                q.push(Game::new(b, s));
                c += 1;
            }
        }

        if q.len() > MAX_SIZE {
            ouput(&q.peek().unwrap().board, c, q.len(), false);
            q = q_out(q);
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
    let mut saved_score: (Seed, Score) = (seed, 0);
    {
        let bs: Vec<(Seed, Score)> = read(FILE_RESULT).unwrap();

        for s in bs {
            if s.0 == seed {
                saved_score.1 = s.1;
                break;
            }
        }
    }
    dbg!(saved_score);

    let mut board = Board::new();

    seed = board.spawn_tile(seed);
    seed = board.spawn_tile(seed);

    board = solve(board, seed, saved_score);
    println!("{:?}", &board);
    println!("moves.len(): {}", board.moves.len());

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

    #[test]
    fn q_in_out() {
        let mut q = BinaryHeap::<Game>::new();
        let mut b = Board::new();
        b.spawn_tile(42);
        b.spawn_tile(81);
        b.score = 42;
        dbg!(&b);
        q.push(Game::new(b.clone(), 0));
        q = q_out(q);
        q_in(&mut q);
        assert_eq!(q.len(), 1);
    }
}
