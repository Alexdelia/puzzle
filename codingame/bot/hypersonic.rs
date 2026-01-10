use std::io;

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

type TurnIndex = u16;
type Id = u8;

const BOARD_WIDTH: usize = 13;
const BOARD_HEIGHT: usize = 11;

type Board = [[Cell; BOARD_WIDTH]; BOARD_HEIGHT];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cell {
	Empty,
	/// explod at turn index
	Box(Option<TurnIndex>),
}

fn read_board(board: &mut Board) {
	for y in 0..BOARD_HEIGHT {
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();

		for (x, c) in input_line.trim().chars().enumerate() {
			match c {
				'.' => board[y][x] = Cell::Empty,
				'0' => match board[y][x] {
					Cell::Box(_) => {}
					_ => board[y][x] = Cell::Box(None),
				},
				_ => panic!("unknown cell character: '{c}'"),
			};
		}
	}
}

const BOMB_RANGE: usize = 3;
fn find_cell_with_most_destructible(board: &Board) -> Option<(usize, usize)> {
	let mut best = (0, None);

	for y in 0..BOARD_HEIGHT {
		for x in 0..BOARD_WIDTH {
			let cell = board[y][x];
			if cell != Cell::Empty {
				continue;
			}

			let mut count = 0;

			for dir in &[(1, 0), (0, 1), (-1, 0), (0, -1)] {
				let (dx, dy) = *dir;
				for dist in 1..=BOMB_RANGE {
					let nx = x as isize + dx * dist as isize;
					let ny = y as isize + dy * dist as isize;
					if nx < 0 || nx >= BOARD_WIDTH as isize || ny < 0 || ny >= BOARD_HEIGHT as isize
					{
						break;
					}
					match board[ny as usize][nx as usize] {
						Cell::Box(None) => {
							count += 1;
						}
						Cell::Box(Some(_)) => {
							break;
						}
						Cell::Empty => {}
					}
				}
			}

			if count > best.0 {
				best = (count, Some((x, y)));
			}
		}
	}

	best.1
}

fn main() {
	let mut board: Board = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];

	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let inputs = input_line.split(" ").collect::<Vec<_>>();
	assert_eq!(parse_input!(inputs[0], usize), BOARD_WIDTH);
	assert_eq!(parse_input!(inputs[1], usize), BOARD_HEIGHT);
	let my_id = parse_input!(inputs[2], Id);

	let mut my_x: usize = 0;
	let mut my_y: usize = 0;

	let mut turn: TurnIndex = 0;
	loop {
		read_board(&mut board);

		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let entities = parse_input!(input_line, usize);
		for i in 0..entities {
			let mut input_line = String::new();
			io::stdin().read_line(&mut input_line).unwrap();
			eprintln!("entity[{i}] = '{input_line}'");
			let inputs = input_line.split(" ").collect::<Vec<_>>();
			let _entity_type = parse_input!(inputs[0], i32);
			let owner_id: Id = parse_input!(inputs[1], Id);
			let x = parse_input!(inputs[2], usize);
			let y = parse_input!(inputs[3], usize);
			let _param1 = parse_input!(inputs[4], i32);
			let _param2 = parse_input!(inputs[5], i32);

			if owner_id == my_id {
				my_x = x;
				my_y = y;
			}
		}

		let best_cell = find_cell_with_most_destructible(&board);
		if let Some((x, y)) = best_cell {
			println!("BOMB {x} {y} {turn}");
			board[y][x] = Cell::Box(Some(turn + 8));
		} else {
			println!("MOVE {my_x} {my_y} WAIT");
		}

		turn += 1;
	}
}
