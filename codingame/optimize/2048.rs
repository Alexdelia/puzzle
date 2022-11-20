use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{fmt, io};

const SIZE: usize = 4;
const GAME: usize = 100000;

const INIT_TIME: Duration = Duration::from_secs(1);
const MOVE_TIME: Duration = Duration::from_millis(50);

// linear congruential generator
const R_A: u128 = 1664525;
const R_C: u128 = 1013904223;
const R_M: u128 = 1 << 32;

type Seed = u128;
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
    empty: bool,
    over: bool,
    moves: Vec<Move>,
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "score: {}", self.score)?;
        writeln!(f, "empty: {}", self.empty)?;
        writeln!(f, "over: {}", self.over)?;
        for x in 0..SIZE {
            for y in 0..SIZE {
                write!(f, "{} ", self.board[x][y])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Board {
    fn new() -> Board {
        Board {
            board: [[0; SIZE]; SIZE],
            score: 0,
            empty: true,
            over: false,
            moves: Vec::new(),
        }
    }

    fn spawn_tile(&mut self, seed: Seed) -> Seed {
        let mut empty: Vec<(usize, usize)> = Vec::new();

        for y in 0..SIZE {
            for x in 0..SIZE {
                if self.board[x][y] == 0 {
                    empty.push((x, y));
                }
            }
        }

        let (x, y) = empty[seed as usize % empty.len()];
        self.board[x][y] = if (seed & 0x10) == 0 { 2 } else { 4 };

        self.empty = empty.len() <= 1;
        seed * seed % 50515093
    }

    fn is_over(&self) -> bool {
        self.empty && !self.can_fuse_row() && !self.can_fuse_col()
    }

    fn can_fuse_row(&self) -> bool {
        for x in 0..SIZE {
            for y in 0..SIZE - 1 {
                if self.board[x][y] == self.board[x][y + 1] {
                    return true;
                }
            }
        }
        false
    }

    fn can_fuse_col(&self) -> bool {
        for x in 0..SIZE - 1 {
            for y in 0..SIZE {
                if self.board[x][y] == self.board[x + 1][y] {
                    return true;
                }
            }
        }
        false
    }

    fn play(&mut self, m: Move) -> bool {
        let change: bool = match m {
            Move::Up => self.up(),
            Move::Down => self.down(),
            Move::Left => self.left(),
            Move::Right => self.right(),
        };
        if change {
            self.moves.push(m);
        }
        change
    }

    fn up(&mut self) -> bool {
        let bv = self.move_up();
        let bg = self.merge_up();
        if !bv && !bg {
            return false;
        }
        self.move_up();
        true
    }

    fn down(&mut self) -> bool {
        let bv = self.move_down();
        let bg = self.merge_down();
        if !bv && !bg {
            return false;
        }
        self.move_down();
        true
    }

    fn left(&mut self) -> bool {
        let bv = self.move_left();
        let bg = self.merge_left();
        if !bv && !bg {
            return false;
        }
        self.move_left();
        true
    }

    fn right(&mut self) -> bool {
        let bv = self.move_right();
        let bg = self.merge_right();
        if !bv && !bg {
            return false;
        }
        self.move_right();
        true
    }

    fn move_up(&mut self) -> bool {
        let mut change = false;

        for col in 0..SIZE {
            let mut i = 0;
            while i < SIZE && self.board[i][col] != 0 {
                i += 1;
            }

            let mut row = 1;
            while i < SIZE && row < SIZE {
                if self.board[row][col] != 0 && i < row {
                    self.board[i][col] = self.board[row][col];
                    self.board[row][col] = 0;
                    i += 1;
                    change = true;
                }
                row += 1;
            }
        }

        change
    }

    fn move_down(&mut self) -> bool {
        let mut change = false;

        for col in 0..SIZE {
            let mut i: isize = SIZE as isize - 1;
            while i >= 0 && self.board[i as usize][col] != 0 {
                i -= 1;
            }

            let mut row: isize = SIZE as isize - 2;
            while i >= 0 && row >= 0 {
                if self.board[row as usize][col] != 0 && i > row {
                    self.board[i as usize][col] = self.board[row as usize][col];
                    self.board[row as usize][col] = 0;
                    i -= 1;
                    change = true;
                }
                row -= 1;
            }
        }

        change
    }

    fn move_left(&mut self) -> bool {
        let mut change = false;

        for row in 0..SIZE {
            let mut i = 0;
            while i < SIZE && self.board[row][i] != 0 {
                i += 1;
            }

            let mut col = 1;
            while i < SIZE && col < SIZE {
                if self.board[row][col] != 0 && i < col {
                    self.board[row][i] = self.board[row][col];
                    self.board[row][col] = 0;
                    i += 1;
                    change = true;
                }
                col += 1;
            }
        }

        change
    }

    fn move_right(&mut self) -> bool {
        let mut change = false;

        for row in 0..SIZE {
            let mut i: isize = SIZE as isize - 1;
            while i >= 0 && self.board[row][i as usize] != 0 {
                i -= 1;
            }

            let mut col: isize = SIZE as isize - 2;
            while i >= 0 && col >= 0 {
                if self.board[row][col as usize] != 0 && i > col {
                    self.board[row][i as usize] = self.board[row][col as usize];
                    self.board[row][col as usize] = 0;
                    i -= 1;
                    change = true;
                }
                col -= 1;
            }
        }

        change
    }

    fn merge_up(&mut self) -> bool {
        let mut change = false;

        for col in 0..SIZE {
            for row in 0..SIZE - 1 {
                if self.board[row][col] != 0 && self.board[row][col] == self.board[row + 1][col] {
                    self.board[row][col] *= 2;
                    self.board[row + 1][col] = 0;
                    self.score += self.board[row][col];
                    change = true;
                }
            }
        }

        change
    }

    fn merge_down(&mut self) -> bool {
        let mut change = false;

        for col in 0..SIZE {
            for row in (1..SIZE).rev() {
                if self.board[row][col] != 0 && self.board[row][col] == self.board[row - 1][col] {
                    self.board[row][col] *= 2;
                    self.board[row - 1][col] = 0;
                    self.score += self.board[row][col];
                    change = true;
                }
            }
        }

        change
    }

    fn merge_left(&mut self) -> bool {
        let mut change = false;

        for row in 0..SIZE {
            for col in 0..SIZE - 1 {
                if self.board[row][col] != 0 && self.board[row][col] == self.board[row][col + 1] {
                    self.board[row][col] *= 2;
                    self.board[row][col + 1] = 0;
                    self.score += self.board[row][col];
                    change = true;
                }
            }
        }

        change
    }

    fn merge_right(&mut self) -> bool {
        let mut change = false;

        for row in 0..SIZE {
            for col in (1..SIZE).rev() {
                if self.board[row][col] != 0 && self.board[row][col] == self.board[row][col - 1] {
                    self.board[row][col] *= 2;
                    self.board[row][col - 1] = 0;
                    self.score += self.board[row][col];
                    change = true;
                }
            }
        }

        change
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
            if b.board[x][y] == 0 {
                b.empty = false;
            }
        }
    }

    (b, seed)
}

fn solve(b: &Board, seed: Seed, time: Duration) -> (Vec<Move>, Seed) {
    let mut cur_seed = 0;
    let mut new_seed = seed;
    let mut games: Vec<Board> = vec![b.clone(); GAME];
    let mut am: Vec<Move>;
    let mut sm = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let start = Instant::now();
    let end = time - Duration::from_millis(10);

    while cur_seed != new_seed && start.elapsed() < end {
        cur_seed = new_seed;

        for i in games.iter_mut() {
            if i.over {
                continue;
            } else if i.is_over() {
                i.over = true;
                continue;
            }

            am = vec![Move::Up, Move::Down, Move::Left, Move::Right];
            while !i.play(am.remove(sm as usize % am.len())) {
                sm = (R_A * sm + R_C) % R_M;
                if am.is_empty() {
                    panic!("Board.is_over() has been checked but no move change the board");
                }
            }

            new_seed = i.spawn_tile(cur_seed);

            if start.elapsed() > end {
                break;
            }
        }
    }

    // seed bug because of break in between loop and using a solution with -1 move than the first one

    eprintln!("time break: {:?}", start.elapsed());
    eprintln!("remaining time: {:?}", time - start.elapsed());
    // return game with highest score
    let m = games.into_iter().max_by_key(|x| x.score).unwrap().moves;
    eprintln!("time exit: {:?}", start.elapsed());
    eprintln!("remaining time: {:?}", time - start.elapsed());
    (m, new_seed)
}

fn main() {
    let mut seed: Seed;
    let mut b: Board;
    let mut m: Vec<Move>;

    (b, seed) = get_info();
    (m, seed) = solve(&b, seed, INIT_TIME);
    // print all moves in one print statement
    // Move::Up = 'U', Move::Down = 'D', Move::Left = 'L', Move::Right = 'R'
    println!("{}", m.iter().map(|x| x.to_string()).collect::<String>());

    // game loop
    loop {
        let b_seed = seed;
        (b, seed) = get_info();
        assert_eq!(b_seed, seed);
        (m, seed) = solve(&b, seed, MOVE_TIME);
        println!("{}", m.iter().map(|x| x.to_string()).collect::<String>());
    }
}

// unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board() {
        let mut b = Board::new();
        let mut seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as Seed;

        seed = b.spawn_tile(seed);
        seed = b.spawn_tile(seed);

        assert_eq!(b.score, 0);
        eprintln!("{:?}", b);

        eprintln!("Up");
        b.play(Move::Up);
        seed = b.spawn_tile(seed);
        eprintln!("{:?}", b);

        eprintln!("Down");
        b.play(Move::Down);
        seed = b.spawn_tile(seed);
        eprintln!("{:?}", b);

        eprintln!("Left");
        b.play(Move::Left);
        seed = b.spawn_tile(seed);
        eprintln!("{:?}", b);

        eprintln!("Right");
        b.play(Move::Right);
        seed = b.spawn_tile(seed);
        eprintln!("{:?}", b);

        eprintln!("seed: {}", seed);
    }
}
