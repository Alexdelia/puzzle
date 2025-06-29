use core::fmt::Display;
use std::{
	collections::{BinaryHeap, HashMap, HashSet},
	io,
};

const LOCAL: bool = true;

const LEVEL: &str = "first_level";

const MAX_QUEUE_SIZE: usize = 4_000_000;
const QUEUE_DROP_SIZE: usize = 100_000;

#[cfg(feature = "lossless")]
const MAX_SEEN_TOTAL_SIZE: usize = 12_000_000;

#[cfg(not(feature = "lossless"))]
type HashedGrid = u32;
#[cfg(feature = "lossless")]
type HashedGrid = Vec<(Coord, Cell)>;

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

type GridSize = u8;
type Cell = u8;
type Offset = u32;
// type Depth = u16; // max board is 56 * 32 = 1792

type Coord = (GridSize, GridSize);

type Grid = HashMap<Coord, Cell>;
type Move = (Coord, Direction, Operation);

struct Board {
	offset: Offset,
	grid: Grid,
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
type Seen = Vec<HashSet<HashedGrid>>;

impl Ord for Board {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		other.offset.cmp(&self.offset)
		// other.active_cell.len().cmp(&self.active_cell.len())
		// (other.active_cell.len(), other.offset).cmp(&(self.active_cell.len(), self.offset))
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
			grid: Grid::new(),
			moves: Vec::new(),
		};

		for y in 0..h as usize {
			let mut inputs = String::new();
			io::stdin().read_line(&mut inputs).unwrap();
			for (x, cell) in inputs.split_whitespace().enumerate() {
				let cell = parse_input!(cell, Cell);
				if cell != 0 {
					board.grid.insert((x as GridSize, y as GridSize), cell);
					board.offset += cell as Offset;
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

#[cfg(not(feature = "lossless"))]
fn hash_grid(grid: &Grid) -> HashedGrid {
	grid.iter().fold(0, |acc, ((x, y), &value)| {
		acc.wrapping_add(
			((*x as HashedGrid) << 16) | ((*y as HashedGrid) << 8) | (value as HashedGrid),
		)
	})
}
#[cfg(feature = "lossless")]
fn hash_grid(grid: &Grid) -> HashedGrid {
	let mut hashed_grid = Vec::with_capacity(grid.len());
	for (&(x, y), &value) in grid {
		hashed_grid.push(((x, y), value));
	}
	hashed_grid.sort_by_key(|&(coord, _)| coord);
	hashed_grid
}

fn main() {
	if !LOCAL {
		println!("{LEVEL}");
	}

	loop {
		let ((w, h), board) = Board::parse();

		solve(board, w, h);

		if LOCAL {
			break;
		}
	}
}

// TODO: .clone() .push() should be replaced by .into_iter().collect() or .with_capacity()
macro_rules! play_shift {
	($w:expr, $h:expr, $q:expr, $s:expr, $b:expr, $x:expr, $y:expr, $value:expr, $target_x:expr, $target_y:expr, $target_value:expr, $d:expr) => {{
		let (new_minus, new_offset) = if $target_value >= $value {
			($target_value - ($value), $b.offset - ($value as Offset))
		} else {
			(
				($value) - $target_value,
				$b.offset + (($target_value as Offset) - ($value as Offset)),
			)
		};
		let mut new_board = Board {
			offset: new_offset,
			grid: $b.grid.clone(),
			moves: $b.moves.clone(),
		};
		new_board.grid.remove(&($x, $y));
		new_board.moves.push((($x, $y), $d, Operation::Sub));
		if new_minus != 0 {
			*new_board
				.grid
				.get_mut(&($target_x, $target_y))
				.expect("target cell must exist") = new_minus;
		} else {
			new_board.grid.remove(&($target_x, $target_y));
			if new_board.grid.is_empty() {
				new_board.print_moves();
				return;
			}
		}

		if new_board.grid.len() >= 2 {
			let hashed_grid = hash_grid(&new_board.grid);
			if !$s[new_board.grid.len()].contains(&hashed_grid) {
				$s[new_board.grid.len()].insert(hashed_grid);
				$q.push(new_board);
			}
		}
	}

	if $b.grid.len() > 2 {
		let new_plus = $target_value + $value;
		if new_plus <= $w {
			let mut new_board = Board {
				offset: $b.offset + ($value as Offset),
				// TODO: look for shrink_to_fit or other way to avoid capacity carrying after clone
				grid: $b.grid.clone(),
				moves: $b.moves.clone(),
			};
			new_board.grid.remove(&($x, $y));
			*new_board
				.grid
				.get_mut(&($target_x, $target_y))
				.expect("target cell must exist") = new_plus as Cell;
			new_board.moves.push((($x, $y), $d, Operation::Add));

			// TODO: use .entry() + Vacant
			let hashed_grid = hash_grid(&new_board.grid);
			if !$s[new_board.grid.len()].contains(&hashed_grid) {
				$s[new_board.grid.len()].insert(hashed_grid);
				$q.push(new_board);
			}
		}
	}};
}

macro_rules! play_cell {
	($w:expr, $h:expr, $q:expr, $s:expr, $b:expr, $x:expr, $y:expr, $value:expr) => {
		let (x, y) = (*$x, *$y);

		if $value <= y {
			let new_y = y - $value;
			if let Some(&target_value) = $b.grid.get(&(x, new_y)) {
				if target_value != 0 {
					play_shift!(
						$w,
						$h,
						$q,
						$s,
						$b,
						x,
						y,
						$value,
						x,
						new_y,
						target_value,
						Direction::Up
					);
				}
			}
		}
		let new_y = y + $value;
		if new_y < $h {
			if let Some(&target_value) = $b.grid.get(&(x, new_y)) {
				if target_value != 0 {
					play_shift!(
						$w,
						$h,
						$q,
						$s,
						$b,
						x,
						y,
						$value,
						x,
						new_y,
						target_value,
						Direction::Down
					);
				}
			}
		}
		if $value <= x {
			let new_x = x - $value;
			if let Some(&target_value) = $b.grid.get(&(new_x, y)) {
				if target_value != 0 {
					play_shift!(
						$w,
						$h,
						$q,
						$s,
						$b,
						x,
						y,
						$value,
						new_x,
						y,
						target_value,
						Direction::Left
					);
				}
			}
		}
		let new_x = x + $value;
		if new_x < $w as GridSize {
			if let Some(&target_value) = $b.grid.get(&(new_x, y)) {
				if target_value != 0 {
					play_shift!(
						$w,
						$h,
						$q,
						$s,
						$b,
						x,
						y,
						$value,
						new_x,
						y,
						target_value,
						Direction::Right
					);
				}
			}
		}
	};
}

fn solve(board: Board, w: GridSize, h: GridSize) {
	let mut q = Queue::with_capacity(MAX_QUEUE_SIZE + u8::MAX as usize);
	let mut s: Seen = vec![Default::default(); board.grid.len()];

	q.push(board);

	while let Some(current_board) = q.pop() {
		for ((x, y), &value) in &current_board.grid {
			play_cell!(w, h, q, s, current_board, x, y, value);
		}

		if q.len() > MAX_QUEUE_SIZE {
			q = q
				.into_iter()
				.take(MAX_QUEUE_SIZE - QUEUE_DROP_SIZE)
				.collect();
		}

		#[cfg(feature = "lossless")]
		{
			let s_total = s.iter().map(|set| set.len()).sum::<usize>();
			if s_total > MAX_SEEN_TOTAL_SIZE {
				let top_depth = q
					.iter()
					.map(|b| b.grid.len())
					.max()
					.expect("queue should not be empty");
				if s.len() - top_depth > 0 {
					s.truncate(top_depth);
					s.shrink_to_fit();
				}

				let bottom_depth = s.iter().position(|set| !set.is_empty()).unwrap_or(0) + 4;
				#[allow(clippy::needless_range_loop)]
				for i in 0..bottom_depth {
					s[i] = Default::default();
				}
			}
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
			grid: Grid::new(),
			moves: Vec::new(),
		});
		q.push(Board {
			offset: 5,
			grid: Grid::new(),
			moves: Vec::new(),
		});
		q.push(Board {
			offset: 15,
			grid: Grid::new(),
			moves: Vec::new(),
		});

		assert_eq!(q.pop().unwrap().offset, 5);
		assert_eq!(q.pop().unwrap().offset, 10);
		assert_eq!(q.pop().unwrap().offset, 15);
	}
}
