use std::io;

const SIZE: usize = 4;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

enum Move {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

struct Board {
    board: [[u32; SIZE]; SIZE],
    score: u32,
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

    fn allowed_moves(&self, moves: &mut [bool; 4]) -> bool {
        let mut any = false;
        moves[Move::Up as usize] = match self.allowed_up() {
            true => {
                any = true;
                true
            }
            false => false,
        };
        moves[Move::Down as usize] = match self.allowed_down() {
            true => {
                any = true;
                true
            }
            false => false,
        };
        moves[Move::Left as usize] = match self.allowed_left() {
            true => {
                any = true;
                true
            }
            false => false,
        };
        moves[Move::Right as usize] = match self.allowed_right() {
            true => {
                any = true;
                true
            }
            false => false,
        };
        any
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

fn main() {
    // game loop
    loop {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let seed = parse_input!(input_line, i32); // needed to predict the next spawns
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let score = parse_input!(input_line, i32);
        for i in 0..4 as usize {
            let mut inputs = String::new();
            io::stdin().read_line(&mut inputs).unwrap();
            for j in inputs.split_whitespace() {
                let cell = parse_input!(j, i32);
            }
        }

        println!("U");
    }
}
