use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{fmt, io};

const SIZE: usize = 4;
const GAME: usize = 100;

const INIT_TIME: Duration = Duration::from_secs(1);
const MOVE_TIME: Duration = Duration::from_millis(50);

// linear congruential generator
const R_A: u128 = 1664525;
const R_C: u128 = 1013904223;
const R_M: u128 = 1 << 32;

type Seed = u64;
type Cell = u32;
type Score = u32;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

#[derive(Clone, Copy)]
enum Move {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Move::Up => write!(f, "U"),
            Move::Down => write!(f, "D"),
            Move::Left => write!(f, "L"),
            Move::Right => write!(f, "R"),
        }
    }
}

#[derive(Clone)]
struct Board {
    board: [[Cell; SIZE]; SIZE],
    score: Score,
    over: bool,
    moves: Vec<Move>,
}

impl Board {
    fn new() -> Board {
        Board {
            board: [[0; SIZE]; SIZE],
            score: 0,
            over: false,
            moves: Vec::new(),
        }
    }

    fn spawn_tile(&mut self, seed: Seed) -> Seed {
        let mut empty: Vec<(usize, usize)> = Vec::new();
        for x in 0..SIZE {
            for y in 0..SIZE {
                if self.board[x][y] == 0 {
                    empty.push((x, y));
                }
            }
        }

        let (x, y) = empty[(seed as usize % empty.len())];
        self.board[x][y] = if (seed & 0x10) == 0 { 4 } else { 2 };

        seed * seed % 50515093
    }

    fn allowed_moves(&self, moves: &mut Vec<Move>) {
        moves.clear();
        if self.allowed_up() {
            moves.push(Move::Up);
        }
        if self.allowed_down() {
            moves.push(Move::Down);
        }
        if self.allowed_left() {
            moves.push(Move::Left);
        }
        if self.allowed_right() {
            moves.push(Move::Right);
        }
    }

    fn allowed_up(&self) -> bool {
        for x in 1..SIZE {
            for y in 0..SIZE {
                if self.board[x][y] != 0
                    && (self.board[x - 1][y] == 0 || self.board[x - 1][y] == self.board[x][y])
                {
                    return true;
                }
            }
        }
        false
    }

    fn allowed_down(&self) -> bool {
        for x in 0..SIZE - 1 {
            for y in 0..SIZE {
                if self.board[x][y] != 0
                    && (self.board[x + 1][y] == 0 || self.board[x + 1][y] == self.board[x][y])
                {
                    return true;
                }
            }
        }
        false
    }

    fn allowed_left(&self) -> bool {
        for x in 0..SIZE {
            for y in 1..SIZE {
                if self.board[x][y] != 0
                    && (self.board[x][y - 1] == 0 || self.board[x][y - 1] == self.board[x][y])
                {
                    return true;
                }
            }
        }
        false
    }

    fn allowed_right(&self) -> bool {
        for x in 0..SIZE {
            for y in 0..SIZE - 1 {
                if self.board[x][y] != 0
                    && (self.board[x][y + 1] == 0 || self.board[x][y + 1] == self.board[x][y])
                {
                    return true;
                }
            }
        }
        false
    }

    fn play(&mut self, m: Move) {
        match m {
            Move::Up => self.up(),
            Move::Down => self.down(),
            Move::Left => self.left(),
            Move::Right => self.right(),
        }
        self.moves.push(m);
    }

    fn up(&mut self) {
        for x in 0..SIZE {
            let mut y = 0;
            while y < SIZE {
                if self.board[x][y] == 0 {
                    y += 1;
                    continue;
                }
                let mut y2 = y + 1;
                while y2 < SIZE && self.board[x][y2] == 0 {
                    y2 += 1;
                }
                if y2 == SIZE {
                    break;
                }
                if self.board[x][y] == self.board[x][y2] {
                    self.board[x][y] *= 2;
                    self.score += self.board[x][y];
                    self.board[x][y2] = 0;
                    y = y2 + 1;
                } else {
                    y = y2;
                }
            }
        }
        for x in 0..SIZE {
            let mut y = 0;
            while y < SIZE {
                if self.board[x][y] == 0 {
                    let mut y2 = y + 1;
                    while y2 < SIZE && self.board[x][y2] == 0 {
                        y2 += 1;
                    }
                    if y2 == SIZE {
                        break;
                    }
                    self.board[x][y] = self.board[x][y2];
                    self.board[x][y2] = 0;
                }
                y += 1;
            }
        }
    }

    fn down(&mut self) {
        for x in 0..SIZE {
            let mut y = SIZE - 1;
            while y > 0 {
                if self.board[x][y] == 0 {
                    y -= 1;
                    continue;
                }
                let mut y2 = y - 1;
                while y2 > 0 && self.board[x][y2] == 0 {
                    y2 -= 1;
                }
                if y2 == 0 {
                    break;
                }
                if self.board[x][y] == self.board[x][y2] {
                    self.board[x][y] *= 2;
                    self.score += self.board[x][y];
                    self.board[x][y2] = 0;
                    y = y2 - 1;
                } else {
                    y = y2;
                }
            }
        }
        for x in 0..SIZE {
            let mut y = SIZE - 1;
            while y > 0 {
                if self.board[x][y] == 0 {
                    let mut y2 = y - 1;
                    while y2 > 0 && self.board[x][y2] == 0 {
                        y2 -= 1;
                    }
                    if y2 == 0 {
                        break;
                    }
                    self.board[x][y] = self.board[x][y2];
                    self.board[x][y2] = 0;
                }
                y -= 1;
            }
        }
    }

    fn left(&mut self) {
        for y in 0..SIZE {
            let mut x = 0;
            while x < SIZE {
                if self.board[x][y] == 0 {
                    x += 1;
                    continue;
                }
                let mut x2 = x + 1;
                while x2 < SIZE && self.board[x2][y] == 0 {
                    x2 += 1;
                }
                if x2 == SIZE {
                    break;
                }
                if self.board[x][y] == self.board[x2][y] {
                    self.board[x][y] *= 2;
                    self.score += self.board[x][y];
                    self.board[x2][y] = 0;
                    x = x2 + 1;
                } else {
                    x = x2;
                }
            }
        }
        for y in 0..SIZE {
            let mut x = 0;
            while x < SIZE {
                if self.board[x][y] == 0 {
                    let mut x2 = x + 1;
                    while x2 < SIZE && self.board[x2][y] == 0 {
                        x2 += 1;
                    }
                    if x2 == SIZE {
                        break;
                    }
                    self.board[x][y] = self.board[x2][y];
                    self.board[x2][y] = 0;
                }
                x += 1;
            }
        }
    }

    fn right(&mut self) {
        for y in 0..SIZE {
            let mut x = SIZE - 1;
            while x > 0 {
                if self.board[x][y] == 0 {
                    x -= 1;
                    continue;
                }
                let mut x2 = x - 1;
                while x2 > 0 && self.board[x2][y] == 0 {
                    x2 -= 1;
                }
                if x2 == 0 {
                    break;
                }
                if self.board[x][y] == self.board[x2][y] {
                    self.board[x][y] *= 2;
                    self.score += self.board[x][y];
                    self.board[x2][y] = 0;
                    x = x2 - 1;
                } else {
                    x = x2;
                }
            }
        }
        for y in 0..SIZE {
            let mut x = SIZE - 1;
            while x > 0 {
                if self.board[x][y] == 0 {
                    let mut x2 = x - 1;
                    while x2 > 0 && self.board[x2][y] == 0 {
                        x2 -= 1;
                    }
                    if x2 == 0 {
                        break;
                    }
                    self.board[x][y] = self.board[x2][y];
                    self.board[x2][y] = 0;
                }
                x -= 1;
            }
        }
    }
}

fn get_info() -> (Board, Seed) {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let seed = parse_input!(input_line, Seed); // needed to predict the next spawns

    let mut b = Board::new();

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    b.score = parse_input!(input_line, Score);

    for x in 0..SIZE {
        let mut inputs = String::new();
        io::stdin().read_line(&mut inputs).unwrap();

        for (y, cell) in inputs.split_whitespace().enumerate() {
            b.board[x][y] = parse_input!(cell, Cell);
        }
    }

    (b, seed)
}

fn solve(b: &Board, seed: Seed, time: Duration) -> Vec<Move> {
    let mut games: Vec<Board> = vec![b.clone(); GAME];
    let mut am: Vec<Move> = Vec::with_capacity(4);
    let mut over = false;
    let mut sm = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let start = Instant::now();

    while !over && start.elapsed() < time - Duration::from_millis(90) {
        over = true;

        for i in games.iter_mut() {
            if i.over {
                continue;
            }

            i.allowed_moves(&mut am);
            if am.is_empty() {
                i.over = true;
                continue;
            }

            i.play(am[sm as usize % am.len()]);
            sm = (R_A * sm + R_C) % R_M;

            i.spawn_tile(seed);

            over = false;
        }
    }

    eprintln!("remaining time break: {:?}", time - start.elapsed());
    // return game with highest score
    let m = games.into_iter().max_by_key(|x| x.score).unwrap().moves;
    eprintln!("remaining time exit: {:?}", time - start.elapsed());
    m
}

fn main() {
    let (b, seed) = get_info();
    solve(&b, seed, INIT_TIME);
    // print all moves in one print statement
    // Move::Up = 'U', Move::Down = 'D', Move::Left = 'L', Move::Right = 'R'
    println!(
        "{}",
        b.moves.iter().map(|x| x.to_string()).collect::<String>()
    );

    // game loop
    loop {
        let (b, seed) = get_info();
        let m = solve(&b, seed, MOVE_TIME);
        println!("{}", m.iter().map(|x| x.to_string()).collect::<String>());
    }
}
