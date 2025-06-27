use core::fmt::Display;
use std::{collections::BinaryHeap, io};

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

type GridSize = u8;
type Cell = u8;
type Offset = usize;

type Coord = (GridSize, GridSize);

type Move = (Coord, Direction, Operation);

struct Board {
	offset: Offset,
	grid: Vec<Vec<Cell>>,
	active_cell: Vec<Coord>,
	moves: Vec<Move>,
}

#[derive(Clone, Copy)]
enum Direction {
	Up,
	Down,
	Left,
	Right,
}

#[derive(Clone, Copy)]
enum Operation {
	Add,
	Sub,
}

impl Display for Direction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = match self {
			Direction::Up => "U",
			Direction::Down => "D",
			Direction::Left => "L",
			Direction::Right => "R",
		};
		write!(f, "{s}")
	}
}

impl Display for Operation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = match self {
			Operation::Add => "+",
			Operation::Sub => "-",
		};
		write!(f, "{s}")
	}
}

type Queue = BinaryHeap<Board>;

impl Ord for Board {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		other.offset.cmp(&self.offset)
	}
}

impl PartialOrd for Board {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Eq for Board {}
impl PartialEq for Board {
	fn eq(&self, other: &Self) -> bool {
		self.offset == other.offset
	}
}

impl Board {
	fn parse() -> ((GridSize, GridSize), Board) {
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let inputs = input_line.split(" ").collect::<Vec<_>>();
		let w = parse_input!(inputs[0], GridSize);
		let h = parse_input!(inputs[1], GridSize);

		let mut board = Board {
			offset: 0,
			grid: Vec::with_capacity(h as usize),
			active_cell: Vec::new(),
			moves: Vec::new(),
		};

		for y in 0..h as usize {
			board.grid.push(Vec::with_capacity(w as usize));

			let mut inputs = String::new();
			io::stdin().read_line(&mut inputs).unwrap();
			for (x, cell) in inputs.split_whitespace().enumerate() {
				let cell = parse_input!(cell, Cell);
				board.grid[y].push(cell);
				if cell != 0 {
					board.offset += cell as Offset;
					board.active_cell.push((x as GridSize, y as GridSize));
				}
			}
		}

		((w, h), board)
	}

	fn print_moves(&self) {
		for m in &self.moves {
			println!("{} {} {} {}", m.0.0, m.0.1, m.1, m.2);
		}
	}
}

fn main() {
	println!("first_level");

	loop {
		let ((w, h), board) = Board::parse();

		solve(board, w as usize, h as usize);
	}
}

macro_rules! play_shift {
	($w:expr, $h:expr, $q:expr, $b:expr, $active_cell_index:expr, $x:expr, $y:expr, $target_x:expr, $target_y:expr, $value:expr, $d:expr) => {
		let target_value = $b.grid[$target_y][$target_x];

		let new_plus = target_value as usize + $value;
		if new_plus <= Cell::MAX as usize {
			let mut new_board = Board {
				offset: $b.offset + $value as Offset,
				grid: $b.grid.clone(),
				moves: $b.moves.clone(),
				active_cell: $b.active_cell.clone(),
			};
			new_board.grid[$y][$x] = 0;
			new_board.grid[$target_y][$target_x] = new_plus as Cell;
			new_board.active_cell.remove($active_cell_index);
			new_board
				.moves
				.push((($x as GridSize, $y as GridSize), $d, Operation::Add));
		}

		if target_value >= ($value as Cell) {
			let mut new_board = Board {
				offset: $b.offset - ($value as Offset),
				grid: $b.grid.clone(),
				moves: $b.moves.clone(),
				active_cell: $b.active_cell.clone(),
			};
			let target_value = target_value - ($value as Cell);
			new_board.grid[$y][$x] = 0;
			new_board.grid[$target_y][$target_x] = target_value;
			new_board.active_cell.remove($active_cell_index);
			if target_value == 0 {
				new_board.active_cell.retain(|&(tx, ty)| {
					!(tx == (($target_x) as GridSize) && ty == (($target_y) as GridSize))
				});
			}
			new_board
				.moves
				.push((($x as GridSize, $y as GridSize), $d, Operation::Sub));
			$q.push(new_board);
		}
	};
}

macro_rules! play_cell {
	($w:expr, $h:expr, $q:expr, $b:expr, $active_cell_index:expr, $x:expr, $y:expr) => {
		let (x, y) = (*$x as usize, *$y as usize);
		let value = $b.grid[y][x] as usize;

		if value < $h && $b.grid[y + value][x] != 0 {
			play_shift!(
				$w,
				$h,
				$q,
				$b,
				$active_cell_index,
				x,
				y,
				x,
				(y - (value as usize)),
				value,
				Direction::Up
			);
		}
	};
}

fn solve(board: Board, w: usize, h: usize) {
	let mut q = Queue::new();
	// TODO: add visited storage & check optimization

	q.push(board);

	while let Some(current_board) = q.pop() {
		for (i, (x, y)) in current_board.active_cell.iter().enumerate() {
			play_cell!(w, h, q, current_board, i, x, y);
		}

		if current_board.active_cell.is_empty() {
			current_board.print_moves();
			return;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_board_priority() {
		let mut q = Queue::new();

		q.push(Board {
			offset: 10,
			grid: Vec::new(),
			moves: Vec::new(),
		});
		q.push(Board {
			offset: 5,
			grid: Vec::new(),
			moves: Vec::new(),
		});
		q.push(Board {
			offset: 15,
			grid: Vec::new(),
			moves: Vec::new(),
		});

		assert_eq!(q.pop().unwrap().offset, 5);
		assert_eq!(q.pop().unwrap().offset, 10);
		assert_eq!(q.pop().unwrap().offset, 15);
	}
}
