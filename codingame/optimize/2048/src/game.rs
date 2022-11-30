use std::fmt;

use enum_like_derive::EnumLike;
use enum_vec::EnumVec;

pub const SIZE: usize = 4;

// const INIT_TIME: Duration = Duration::from_secs(1);
// const MOVE_TIME: Duration = Duration::from_millis(50);

pub type Seed = u128;
pub type Cell = u8;
pub type Score = u32;

pub fn next(seed: Seed) -> Seed {
    seed * seed % 50515093
}

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

#[derive(Clone, Copy, EnumLike)]
pub enum Move {
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

impl Move {
    pub fn from(c: char) -> Option<Move> {
        match c {
            'U' => Some(Move::Up),
            'D' => Some(Move::Down),
            'L' => Some(Move::Left),
            'R' => Some(Move::Right),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct Board {
    pub board: [[Cell; SIZE]; SIZE],
    pub score: Score,
    empty: bool,
    over: bool,
    pub moves: EnumVec<Move>,
    // moves: Vec<Move>,
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "score: {}", self.score)?;
        writeln!(f, "empty: {}", self.empty)?;
        writeln!(f, "over: {}", self.over)?;
        for x in 0..SIZE {
            for y in 0..SIZE {
                let n = match 1 << self.board[x][y] {
                    1 => 0,
                    n => n,
                };
                write!(f, "{} ", n)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Board {
    pub fn new() -> Board {
        Board {
            board: [[0; SIZE]; SIZE],
            score: 0,
            empty: true,
            over: false,
            moves: EnumVec::new(),
            // moves: Vec::new(),
        }
    }

    pub fn spawn_tile(&mut self, seed: Seed) -> Seed {
        let mut empty: Vec<(usize, usize)> = Vec::new();

        for y in 0..SIZE {
            for x in 0..SIZE {
                if self.board[x][y] == 0 {
                    empty.push((x, y));
                }
            }
        }

        let (x, y) = empty[seed as usize % empty.len()];
        self.board[x][y] = if (seed & 0x10) == 0 { 1 } else { 2 };

        self.empty = empty.len() <= 1;
        next(seed)
    }

    pub fn is_over(&self) -> bool {
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

    pub fn play(&mut self, m: Move) -> bool {
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
                    self.board[row][col] += 1;
                    self.board[row + 1][col] = 0;
                    self.score += 1 << self.board[row][col];
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
                    self.board[row][col] += 1;
                    self.board[row - 1][col] = 0;
                    self.score += 1 << self.board[row][col];
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
                    self.board[row][col] += 1;
                    self.board[row][col + 1] = 0;
                    self.score += 1 << self.board[row][col];
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
                    self.board[row][col] += 1;
                    self.board[row][col - 1] = 0;
                    self.score += 1 << self.board[row][col];
                    change = true;
                }
            }
        }

        change
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

fn b10to2(n: u32) -> u8 {
    if n == 0 {
        return 0;
    }
    n.trailing_zeros() as u8
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

    #[test]
    fn test_b10to2() {
        assert_eq!(b10to2(0), 0);
        assert_eq!(b10to2(2), 1);
        assert_eq!(b10to2(4), 2);
        assert_eq!(b10to2(8), 3);
        assert_eq!(b10to2(16), 4);
        assert_eq!(b10to2(32), 5);
        assert_eq!(b10to2(64), 6);
        assert_eq!(b10to2(128), 7);
        assert_eq!(b10to2(256), 8);
        assert_eq!(b10to2(512), 9);
        assert_eq!(b10to2(1024), 10);
        assert_eq!(b10to2(2048), 11);
        assert_eq!(b10to2(4096), 12);
        assert_eq!(b10to2(8192), 13);
        assert_eq!(b10to2(16384), 14);
        assert_eq!(b10to2(32768), 15);
        assert_eq!(b10to2(65536), 16);
        assert_eq!(b10to2(131072), 17);
    }

    #[test]
    fn test_b10to2_shift() {
        for n in 0..=17 {
            assert_eq!(b10to2(1 << n), n);
            assert_eq!(b10to2(2 << n), n + 1);
        }
    }
}
