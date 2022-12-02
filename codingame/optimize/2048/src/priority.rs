use crate::game::{Board, Cell, Move, SIZE};

type Priority = u32;

pub fn priority(board: &Board) -> Priority {
    let mut r = radix(&board);
    // let mut p: Priority = r[0] as Priority;
    // for i in r.iter().take(18).skip(1) {
    //     if i == &1 {
    //         p += 1;
    //     }
    // }
    let mut p: Priority = 0;

    // +1 priority for each cell forming a snake (the snake can be in any direction)
    // snake:
    // 15 14 13 12
    // 8 9 10 11
    // 7 6 5 4
    // 0 1 2 3
    // other snake:
    // 12 13 14 15
    // 11 10 9 8
    // 4 5 6 7
    // 3 2 1 0
    // other snake:
    // 3 2 1 0
    // 4 5 6 7
    // 11 10 9 8
    // 12 13 14 15
    let mut x: usize = 0;
    let mut y: usize = 0;
    let mut dir: Move = Move::Up;
    let mut drift: Move = Move::Up;
    let mut b: bool = true;

    for val in (0..18).rev() {
        while r[val] > 0 {
            if p == 0 {
                (b, x, y) = is_corner(&board, val as Cell);
            } else if p == 1 {
                (b, x, y, dir, drift) = find_dir(&board, val as Cell, x, y);
            } else {
                (b, x, y, dir) = apply_dir(&board, val as Cell, x, y, dir, drift);
            }

            if !b {
                break;
            }

            p += 1;
            r[val] -= 1;
        }

        if !b {
            break;
        }
    }

    p * 10_000_000 + board.score
}

fn radix(board: &Board) -> Vec<u8> {
    let mut r = vec![0; 18];
    for x in 0..SIZE {
        for y in 0..SIZE {
            r[board.board[x][y] as usize] += 1;
        }
    }
    r
}

fn is_corner(board: &Board, val: Cell) -> (bool, usize, usize) {
    match val {
        c if board.board[0][0] == c => (true, 0, 0),
        c if board.board[0][SIZE - 1] == c => (true, 0, SIZE - 1),
        c if board.board[SIZE - 1][0] == c => (true, SIZE - 1, 0),
        c if board.board[SIZE - 1][SIZE - 1] == c => (true, SIZE - 1, SIZE - 1),
        _ => (false, 0, 0),
    }
}

fn find_dir(board: &Board, val: Cell, x: usize, y: usize) -> (bool, usize, usize, Move, Move) {
    if x > 0 && board.board[x - 1][y] == val {
        (true, x - 1, y, Move::Up, Move::Left)
    } else if x < SIZE - 1 && board.board[x + 1][y] == val {
        (true, x + 1, y, Move::Down, Move::Right)
    } else if y > 0 && board.board[x][y - 1] == val {
        (true, x, y - 1, Move::Left, Move::Up)
    } else if y < SIZE - 1 && board.board[x][y + 1] == val {
        (true, x, y + 1, Move::Right, Move::Down)
    } else {
        (false, 0, 0, Move::Up, Move::Up)
    }
}

fn apply_dir(
    board: &Board,
    val: Cell,
    x: usize,
    y: usize,
    dir: Move,
    drift: Move,
) -> (bool, usize, usize, Move) {
    let (tx, ty) = get_xy_dir(drift);
    let dx = x as i8 + tx;
    let dy = y as i8 + ty;

    match dir {
        Move::Up => {
            if x == 0 {
                if board.board[dx as usize][dx as usize] == val {
                    return (true, dx as usize, dy as usize, Move::Down);
                }
            } else if board.board[x - 1][y] == val {
                return (true, x - 1, y, Move::Up);
            }
            (false, 0, 0, Move::Up)
        }
        Move::Down => {
            if x == SIZE - 1 {
                if board.board[dx as usize][dx as usize] == val {
                    return (true, dx as usize, dy as usize, Move::Up);
                }
            } else if board.board[x + 1][y] == val {
                return (true, x + 1, y, Move::Down);
            }
            (false, 0, 0, Move::Up)
        }
        Move::Left => {
            if y == 0 {
                if board.board[dx as usize][dx as usize] == val {
                    return (true, dx as usize, dy as usize, Move::Right);
                }
            } else if board.board[x][y - 1] == val {
                return (true, x, y - 1, Move::Left);
            }
            (false, 0, 0, Move::Up)
        }
        Move::Right => {
            if y == SIZE - 1 {
                if board.board[dx as usize][dx as usize] == val {
                    return (true, dx as usize, dy as usize, Move::Left);
                }
            } else if board.board[x][y + 1] == val {
                return (true, x, y + 1, Move::Right);
            }
            (false, 0, 0, Move::Up)
        }
    }
}

fn get_xy_dir(dir: Move) -> (i8, i8) {
    match dir {
        Move::Up => (-1, 0),
        Move::Down => (1, 0),
        Move::Left => (0, -1),
        Move::Right => (0, 1),
    }
}
