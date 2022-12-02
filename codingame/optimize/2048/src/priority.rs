use crate::game::{Board, Cell, Move, SIZE};

type Priority = u32;

pub fn priority(board: &Board) -> Priority {
    let mut r = radix(board);
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
                (b, x, y) = is_corner(board, val as Cell);
            } else if p == 1 {
                (b, x, y, dir, drift) = find_dir(board, val as Cell, x, y);
            } else {
                (b, x, y, dir) = apply_dir(board, val as Cell, x, y, dir, drift);
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
    if x == 0 && y == 0 {
        if board.board[x][y + 1] == val {
            return (true, x, y + 1, Move::Right, Move::Down);
        } else if board.board[x + 1][y] == val {
            return (true, x + 1, y, Move::Down, Move::Right);
        }
    } else if x == 0 && y == SIZE - 1 {
        if board.board[x][y - 1] == val {
            return (true, x, y - 1, Move::Left, Move::Down);
        } else if board.board[x + 1][y] == val {
            return (true, x + 1, y, Move::Down, Move::Left);
        }
    } else if x == SIZE - 1 && y == 0 {
        if board.board[x][y + 1] == val {
            return (true, x, y + 1, Move::Right, Move::Up);
        } else if board.board[x - 1][y] == val {
            return (true, x - 1, y, Move::Up, Move::Right);
        }
    } else if x == SIZE - 1 && y == SIZE - 1 {
        if board.board[x][y - 1] == val {
            return (true, x, y - 1, Move::Left, Move::Up);
        } else if board.board[x - 1][y] == val {
            return (true, x - 1, y, Move::Up, Move::Left);
        }
    }
    (false, 0, 0, Move::Up, Move::Up)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radix() {
        let mut board = Board::new();
        board.board = [[0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];
        assert_eq!(
            radix(&board),
            vec![16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );

        board.board = [[4, 0, 0, 2], [0, 0, 0, 0], [0, 0, 2, 0], [1, 0, 0, 0]];
        assert_eq!(
            radix(&board),
            vec![12, 1, 2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );

        board.board = [
            [2, 3, 4, 5],
            [6, 7, 8, 9],
            [10, 11, 12, 13],
            [14, 15, 16, 17],
        ];
        dbg!(&board);
        assert_eq!(
            radix(&board),
            vec![0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
        );
    }

    #[test]
    fn test_is_corner() {
        let mut board = Board::new();
        board.board = [[2, 0, 0, 0], [1, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];
        dbg!(&board);
        assert_eq!(is_corner(&board, 2), (true, 0, 0));
        assert_eq!(is_corner(&board, 1), (false, 0, 0));

        board.board = [[0, 0, 0, 2], [0, 0, 0, 1], [0, 0, 0, 0], [0, 0, 0, 0]];
        dbg!(&board);
        assert_eq!(is_corner(&board, 2), (true, 0, 3));
        assert_eq!(is_corner(&board, 1), (false, 0, 0));
    }

    #[test]
    fn test_find_dir() {
        let mut board = Board::new();
        board.board = [[2, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];
        dbg!(&board);
        let (b, x, y, dir, drift) = find_dir(&board, 1, 0, 0);
        assert_eq!(b, false);

        board.board = [[2, 0, 0, 0], [1, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];
        dbg!(&board);
        let (b, x, y, dir, drift) = find_dir(&board, 1, 0, 0);
        dbg!(b, x, y, dir, drift);
        assert_eq!(b, true);
        assert_eq!(x, 1);
        assert_eq!(y, 0);
        assert_eq!(dir, Move::Down);
        assert_eq!(drift, Move::Right);

        board.board = [[0, 0, 0, 2], [0, 0, 0, 1], [0, 0, 0, 0], [0, 0, 0, 0]];
        dbg!(&board);
        let (b, x, y, dir, drift) = find_dir(&board, 1, 0, 3);
        dbg!(b, x, y, dir, drift);
        assert_eq!(b, true);
        assert_eq!(x, 1);
        assert_eq!(y, 3);
        assert_eq!(dir, Move::Down);
        assert_eq!(drift, Move::Left);

        board.board = [[0, 0, 0, 0], [0, 0, 0, 0], [1, 0, 0, 0], [2, 0, 0, 0]];
        dbg!(&board);
        let (b, x, y, dir, drift) = find_dir(&board, 1, 3, 0);
        dbg!(b, x, y, dir, drift);
        assert_eq!(b, true);
        assert_eq!(x, 2);
        assert_eq!(y, 0);
        assert_eq!(dir, Move::Up);
        assert_eq!(drift, Move::Right);

        board.board = [[0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 1], [0, 0, 0, 2]];
        dbg!(&board);
        let (b, x, y, dir, drift) = find_dir(&board, 1, 3, 3);
        dbg!(b, x, y, dir, drift);
        assert_eq!(b, true);
        assert_eq!(x, 2);
        assert_eq!(y, 3);
        assert_eq!(dir, Move::Up);
        assert_eq!(drift, Move::Left);

        board.board = [[2, 1, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];
        dbg!(&board);
        let (b, x, y, dir, drift) = find_dir(&board, 1, 0, 0);
        dbg!(b, x, y, dir, drift);
        assert_eq!(b, true);
        assert_eq!(x, 0);
        assert_eq!(y, 1);
        assert_eq!(dir, Move::Right);
        assert_eq!(drift, Move::Down);

        board.board = [[0, 0, 1, 2], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];
        dbg!(&board);
        let (b, x, y, dir, drift) = find_dir(&board, 1, 0, 3);
        dbg!(b, x, y, dir, drift);
        assert_eq!(b, true);
        assert_eq!(x, 0);
        assert_eq!(y, 2);
        assert_eq!(dir, Move::Left);
        assert_eq!(drift, Move::Down);

        board.board = [[0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 1, 2]];
        dbg!(&board);
        let (b, x, y, dir, drift) = find_dir(&board, 1, 3, 3);
        dbg!(b, x, y, dir, drift);
        assert_eq!(b, true);
        assert_eq!(x, 3);
        assert_eq!(y, 2);
        assert_eq!(dir, Move::Left);
        assert_eq!(drift, Move::Up);

        board.board = [[0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [2, 1, 0, 0]];
        dbg!(&board);
        let (b, x, y, dir, drift) = find_dir(&board, 1, 3, 0);
        dbg!(b, x, y, dir, drift);
        assert_eq!(b, true);
        assert_eq!(x, 3);
        assert_eq!(y, 1);
        assert_eq!(dir, Move::Right);
        assert_eq!(drift, Move::Up);
    }

    #[test]
    fn test_apply_dir() {
        let mut board = Board::new();
        board.board = [[2, 1, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];
        dbg!(&board);
        assert!(false);
    }
}
