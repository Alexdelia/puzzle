use core::fmt::Display;
use std::{
	collections::{BinaryHeap, HashMap, HashSet},
	io,
};

const LOCAL: bool = true;

const LEVEL: &str = "first_level";

const MAX_QUEUE_SIZE: usize = 4_000_000;
const QUEUE_DROP_SIZE: usize = 100_000;

const MAX_SEEN_TOTAL_SIZE: usize = 12_000_000;

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

type GridSize = u8;
type Cell = u8;
type Offset = u32;
type Depth = u16; // max board is 56 * 32 = 1792

type Coord = (GridSize, GridSize);

type Grid = Vec<Vec<Cell>>;
type Move = (Coord, Direction, Operation);

struct Board {
	offset: Offset,
	grid: Grid,
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
		// other.active_cell.len().cmp(&self.active_cell.len())
		// other.offset.cmp(&self.offset)
		(other.active_cell.len(), other.offset).cmp(&(self.active_cell.len(), self.offset))
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
	if !LOCAL {
		println!("{LEVEL}");
	}

	loop {
		let ((w, h), board) = Board::parse();

		solve(board, w as usize, h as usize);

		if LOCAL {
			break;
		}
	}
}

// TODO: .clone() .push() should be replaced by .into_iter().collect() or .with_capacity()
macro_rules! play_shift {
	($w:expr, $h:expr, $q:expr, $s:expr, $b:expr, $active_cell_index:expr, $x:expr, $y:expr, $target_x:expr, $target_y:expr, $value:expr, $d:expr) => {
		let target_value = $b.grid[$target_y][$target_x];

		if $b.active_cell.len() > 2 {
			let new_plus = target_value as usize + $value;
			if new_plus <= $w {
				let mut new_board = Board {
					offset: $b.offset + ($value as Offset),
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

				let entry = $s
					.entry(new_board.active_cell.len() as Depth)
					.or_insert(HashSet::new());
				if !entry.contains(&new_board.grid) {
					entry.insert(new_board.grid.clone());
					$q.push(new_board);
				}
			}
		}

		{
			let (new_minus, new_offset) = if target_value as usize >= $value {
				(
					target_value as Cell - ($value as Cell),
					$b.offset - ($value as Offset),
				)
			} else {
				(
					($value as Cell) - target_value,
					$b.offset + ((target_value as Offset) - ($value as Offset)),
				)
			};
			let mut new_board = Board {
				offset: new_offset,
				grid: $b.grid.clone(),
				moves: $b.moves.clone(),
				active_cell: $b.active_cell.clone(),
			};
			new_board.grid[$y][$x] = 0;
			new_board.grid[$target_y][$target_x] = new_minus;
			new_board.active_cell.remove($active_cell_index);
			if new_minus == 0 {
				new_board.active_cell.retain(|&(tx, ty)| {
					!(tx == (($target_x) as GridSize) && ty == (($target_y) as GridSize))
				});
			}
			new_board
				.moves
				.push((($x as GridSize, $y as GridSize), $d, Operation::Sub));
			if new_board.active_cell.is_empty() {
				new_board.print_moves();
				return;
			}

			if new_board.active_cell.len() >= 2 {
				let entry = $s
					.entry(new_board.active_cell.len() as Depth)
					.or_insert(HashSet::new());
				if !entry.contains(&new_board.grid) {
					entry.insert(new_board.grid.clone());
					$q.push(new_board);
				}
			}
		}
	};
}

macro_rules! play_cell {
	($w:expr, $h:expr, $q:expr, $s:expr, $b:expr, $active_cell_index:expr, $x:expr, $y:expr) => {
		let (x, y) = (*$x as usize, *$y as usize);
		let value = $b.grid[y][x] as usize;

		if value <= y {
			let new_y = y - value;
			if $b.grid[new_y][x] != 0 {
				play_shift!(
					$w,
					$h,
					$q,
					$s,
					$b,
					$active_cell_index,
					x,
					y,
					x,
					new_y,
					value,
					Direction::Up
				);
			}
		}
		let new_y = y + value;
		if new_y < $h && $b.grid[new_y][x] != 0 {
			play_shift!(
				$w,
				$h,
				$q,
				$s,
				$b,
				$active_cell_index,
				x,
				y,
				x,
				new_y,
				value,
				Direction::Down
			);
		}
		if value <= x {
			let new_x = x - value;
			if $b.grid[y][new_x] != 0 {
				play_shift!(
					$w,
					$h,
					$q,
					$s,
					$b,
					$active_cell_index,
					x,
					y,
					new_x,
					y,
					value,
					Direction::Left
				);
			}
		}
		let new_x = x + value;
		if new_x < $w && $b.grid[y][new_x] != 0 {
			play_shift!(
				$w,
				$h,
				$q,
				$s,
				$b,
				$active_cell_index,
				x,
				y,
				new_x,
				y,
				value,
				Direction::Right
			);
		}
	};
}

fn solve(board: Board, w: usize, h: usize) {
	let mut q = Queue::with_capacity(MAX_QUEUE_SIZE + u8::MAX as usize);
	let mut s =
		HashMap::<Depth, HashSet<Grid>>::with_capacity(board.active_cell.len() as usize + 1);

	q.push(board);

	while let Some(current_board) = q.pop() {
		for (i, (x, y)) in current_board.active_cell.iter().enumerate() {
			play_cell!(w, h, q, s, current_board, i, x, y);
		}

		if q.len() > MAX_QUEUE_SIZE {
			q = q
				.into_iter()
				.take(MAX_QUEUE_SIZE - QUEUE_DROP_SIZE)
				.collect();
		}

		let s_total = s.values().map(|set| set.len()).sum::<usize>();
		if s_total > MAX_SEEN_TOTAL_SIZE {
			let top_depth = q
				.iter()
				.map(|b| b.active_cell.len())
				.max()
				.unwrap_or(Depth::MAX as usize) as Depth;
			let bottom_depth = s.keys().min().unwrap_or(&0) + 4;
			s.retain(|&depth, _| depth <= top_depth && depth > bottom_depth);
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
			active_cell: Vec::new(),
		});
		q.push(Board {
			offset: 5,
			grid: Vec::new(),
			moves: Vec::new(),
			active_cell: Vec::new(),
		});
		q.push(Board {
			offset: 15,
			grid: Vec::new(),
			moves: Vec::new(),
			active_cell: Vec::new(),
		});

		assert_eq!(q.pop().unwrap().offset, 5);
		assert_eq!(q.pop().unwrap().offset, 10);
		assert_eq!(q.pop().unwrap().offset, 15);
	}

	#[test]
	fn test_board_priority_with_active_cells() {
		let mut q = Queue::new();

		q.push(Board {
			offset: 10,
			grid: Vec::new(),
			moves: Vec::new(),
			active_cell: vec![(0, 0)],
		});
		q.push(Board {
			offset: 5,
			grid: Vec::new(),
			moves: Vec::new(),
			active_cell: vec![(0, 0), (1, 1)],
		});
		q.push(Board {
			offset: 15,
			grid: Vec::new(),
			moves: Vec::new(),
			active_cell: vec![(0, 0)],
		});

		assert_eq!(q.pop().unwrap().offset, 10);
		assert_eq!(q.pop().unwrap().offset, 15);
		assert_eq!(q.pop().unwrap().offset, 5);
	}
}
