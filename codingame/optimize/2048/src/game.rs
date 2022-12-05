use std::fmt;
use std::mem::size_of;

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

#[derive(Clone, Copy, EnumLike, Debug, PartialEq, Eq)]
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
    pub moves: EnumVec<Move>,
    // moves: Vec<Move>,
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "score: {}", self.score)?;
        writeln!(f, "empty: {}", self.empty)?;
        writeln!(f, "over: {}", self.is_over())?;
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
            moves: EnumVec::new(),
            // moves: Vec::new(),
        }
    }

    pub fn heapsize(&self) -> usize {
        size_of::<Board>() + self.moves.capacity() / 4
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

/*
fn b10to2(n: u32) -> u8 {
    if n == 0 {
        return 0;
    }
    n.trailing_zeros() as u8
}
*/
