use std::cmp::Reverse;

use crate::game::{Board, Cell, SIZE};

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
    for val in (0..18).rev() {
        while r[val] > 0 {}
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
