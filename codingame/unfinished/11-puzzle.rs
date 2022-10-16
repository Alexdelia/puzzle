use std::collections::{BinaryHeap, HashSet};
use std::io;

type Token = u16;
type Size = u8;

const SIZE: Size = 4;
const SIZE_H: Size = 3;

#[derive(Eq, PartialEq, Clone)]
pub struct Coord {
    x: Size,
    y: Size,
}

#[derive(Eq, PartialEq)]
pub struct Board {
    pub board: Vec<Token>,
    pub blank: Token,
    pub score: u32,
    pub solution: Vec<Coord>,
}

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/
fn main() {
    let mut b = Vec::<Token>::new();

    for _i in 0..3 as usize {
        let mut inputs = String::new();
        io::stdin().read_line(&mut inputs).unwrap();
        for j in inputs.split_whitespace() {
            b.push(parse_input!(j, i32) as Token);
        }
    }

    let target: [Token; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
    let solution = solve(&b, &target);

    // Write an action using println!("message...");
    // To debug: eprintln!("Debug message...");

    eprintln!("len: {}", solution.len());
    for coord in solution {
        println!("{} {}", coord.y, coord.x);
    }
}

// ### BOARD ###
impl Ord for Board {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.score.cmp(&self.score)
    }
}

impl PartialOrd for Board {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Board {
    pub fn get_blank_index(board: &[Token]) -> Token {
        for (i, t) in board.iter().enumerate() {
            if *t == 0 {
                return i as Token;
            }
        }
        Token::MAX
    }
}

// ### SOLVE ###
fn solve(board: &[Token], target: &[Token]) -> Vec<Coord> {
    let mut open = BinaryHeap::<Board>::new();
    let mut closed = HashSet::<Vec<Token>>::new();
    let mut allowed_move: [AllowedMove; 4] = AllowedMove::new_array();

    open.push(Board {
        board: board.to_vec().clone(),
        blank: Board::get_blank_index(board),
        score: 0,
        solution: Vec::new(),
    });

    while !open.is_empty() {
        let cur = open.pop().unwrap();
        if cur.board == target {
            return cur.solution;
        }
        if closed.contains(&cur.board) {
            continue;
        }

        Move::update_allowed_move(&mut allowed_move, cur.blank);
        for m in allowed_move.iter() {
            if m.a {
                let mut new_board = cur.play_move(m.m);
                if !closed.contains(&new_board.board) {
                    new_board.score = new_board.solution.len() as u32
                        + manathan_distance(&new_board.board, &target);
                    open.push(new_board);
                }
            }
        }

        closed.insert(cur.board.clone());
    }

    [].to_vec()
}

fn manathan_distance(board: &[Token], target: &[Token]) -> u32 {
    let size: Token = SIZE as Token;
    let mut distance = 0;

    for i in 1..(SIZE * SIZE_H) as Token {
        let mut x = 0;
        let mut y = 0;
        let mut target_x = 0;
        let mut target_y = 0;

        for f in 0..(SIZE * SIZE_H) as Token {
            if board[f as usize] == i {
                x = f % size;
                y = f / size;
            }
            if target[f as usize] == i {
                target_x = f % size;
                target_y = f / size;
            }
        }
        distance += (x as i32 - target_x as i32).unsigned_abs()
            + (y as i32 - target_y as i32).unsigned_abs();
    }
    distance
}

fn get_xy(index: Token) -> (Size, Size) {
    (
        (index % SIZE as Token) as Size,
        (index / SIZE as Token) as Size,
    )
}

// ### MOVE ###
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Move {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

pub struct AllowedMove {
    pub m: Move,
    pub a: bool,
}

impl Move {
    pub fn update_allowed_move(m: &mut [AllowedMove; 4], blank: Token) {
        let (x, y) = get_xy(blank);
        m[Move::Up as usize].a = y > 0;
        m[Move::Down as usize].a = y < SIZE_H - 1;
        m[Move::Left as usize].a = x > 0;
        m[Move::Right as usize].a = x < SIZE - 1;
    }
}

impl AllowedMove {
    pub fn new_array() -> [AllowedMove; 4] {
        let mut m: [AllowedMove; 4] = [
            AllowedMove {
                m: Move::Up,
                a: true,
            },
            AllowedMove {
                m: Move::Down,
                a: true,
            },
            AllowedMove {
                m: Move::Left,
                a: true,
            },
            AllowedMove {
                m: Move::Right,
                a: true,
            },
        ];
        m[Move::Up as usize] = AllowedMove {
            m: Move::Up,
            a: true,
        };
        m[Move::Down as usize] = AllowedMove {
            m: Move::Down,
            a: true,
        };
        m[Move::Left as usize] = AllowedMove {
            m: Move::Left,
            a: true,
        };
        m[Move::Right as usize] = AllowedMove {
            m: Move::Right,
            a: true,
        };
        m
    }
}

impl Board {
    pub fn play_move(&self, m: Move) -> Board {
        let (x, y) = get_xy(self.blank);
        let mut new_x = x;
        let mut new_y = y;
        match m {
            Move::Up => new_y -= 1,
            Move::Down => new_y += 1,
            Move::Left => new_x -= 1,
            Move::Right => new_x += 1,
        }
        let mut new_board = Board {
            board: self.board.clone(),
            blank: new_x as Token + new_y as Token * SIZE as Token,
            score: 0,
            solution: self.solution.clone(),
        };
        new_board
            .board
            .swap((x + y * SIZE) as usize, (new_x + new_y * SIZE) as usize);
        new_board.solution.push(Coord { x: new_x, y: new_y });
        new_board
    }
}
