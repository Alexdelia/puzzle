use std::{collections::VecDeque, io, str::FromStr, time::Instant};

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

enum Direction {
	Up,
	Down,
	Left,
	Right,
}

type TurnIndex = u8;
type Id = u8;

const BOARD_WIDTH: usize = 13;
const BOARD_HEIGHT: usize = 11;

type Board = [[Cell; BOARD_WIDTH]; BOARD_HEIGHT];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cell {
	Empty,
	Wall,
	Box(Box),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Box {
	item: ItemId,
	explode_at: Option<TurnIndex>,
}

type ItemId = u8;

enum EntityType {
	Player = 0,
	Bomb = 1,
	Item = 2,
}

impl FromStr for EntityType {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"0" => Ok(EntityType::Player),
			"1" => Ok(EntityType::Bomb),
			"2" => Ok(EntityType::Item),
			_ => Err(()),
		}
	}
}

#[derive(Default)]
struct Player {
	id: Id,
	x: usize,
	y: usize,
	bomb_count: usize,
	bomb_range: usize,
}

struct Bomb {
	x: usize,
	y: usize,
	range: usize,
	explode_at: TurnIndex,
}

struct Item {
	// r#type: u8,
	x: usize,
	y: usize,
}

struct Env {
	turn: TurnIndex,
	board: Board,
	me: Player,
	bomb_list: Vec<Bomb>,
	item_list: Vec<Item>,
	turn_start_time: Instant,
}

struct Action {
	bomb: bool,
	direction: Option<Direction>,
}

impl Env {
	fn read_board(&mut self) {
		for y in 0..BOARD_HEIGHT {
			let mut input_line = String::new();
			io::stdin().read_line(&mut input_line).unwrap();

			for (x, c) in input_line.trim().chars().enumerate() {
				match c {
					'.' => self.board[y][x] = Cell::Empty,
					'X' => self.board[y][x] = Cell::Wall,
					_ => match self.board[y][x] {
						Cell::Box(_) => {}
						_ => {
							self.board[y][x] = Cell::Box(Box {
								item: c.to_digit(10).expect("Invalid box character") as ItemId,
								explode_at: None,
							})
						}
					},
				};
			}
		}
	}

	fn read_entities(&mut self) {
		self.bomb_list.clear();
		self.item_list.clear();

		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let entity_count = parse_input!(input_line, usize);

		for _ in 0..entity_count {
			let mut input_line = String::new();
			io::stdin().read_line(&mut input_line).unwrap();
			let inputs = input_line.split(" ").collect::<Vec<_>>();

			let entity_type = inputs[0]
				.trim()
				.parse::<EntityType>()
				.expect("invalid entity type");
			let x = parse_input!(inputs[2], usize);
			let y = parse_input!(inputs[3], usize);

			match entity_type {
				EntityType::Player => {
					let owner_id = parse_input!(inputs[1], Id);
					let bomb_count = parse_input!(inputs[4], usize);
					let bomb_range = parse_input!(inputs[5], usize);
					if owner_id == self.me.id {
						self.me.x = x;
						self.me.y = y;
						self.me.bomb_count = bomb_count;
						self.me.bomb_range = bomb_range;
					}
				}
				EntityType::Bomb => {
					let explode_at = self.turn + parse_input!(inputs[4], TurnIndex);
					let range = parse_input!(inputs[5], usize);
					self.bomb_list.push(Bomb {
						x,
						y,
						explode_at,
						range,
					});
				}
				EntityType::Item => {
					self.item_list.push(Item { x, y });
				}
			}
		}
	}
}

fn best_action(e: &Env) -> Action {
	let mut queue = VecDeque::new();

	Action {
		bomb: true,
		direction: None,
	}
}

fn main() {
	let mut e = Env {
		turn: 0,
		board: [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
		me: Player::default(),
		bomb_list: Vec::new(),
		item_list: Vec::new(),
		turn_start_time: Instant::now(),
	};

	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let inputs = input_line.split(" ").collect::<Vec<_>>();
	assert_eq!(parse_input!(inputs[0], usize), BOARD_WIDTH);
	assert_eq!(parse_input!(inputs[1], usize), BOARD_HEIGHT);
	e.me.id = parse_input!(inputs[2], Id);

	loop {
		e.read_board();
		e.read_entities();
		e.turn_start_time = Instant::now();

		let action = best_action(&e);
		let target_cell = match action.direction {
			None => (e.me.x, e.me.y),
			Some(dir) => match dir {
				Direction::Up => (e.me.x, e.me.y.saturating_sub(1)),
				Direction::Down => (e.me.x, (e.me.y + 1).min(BOARD_HEIGHT - 1)),
				Direction::Left => (e.me.x.saturating_sub(1), e.me.y),
				Direction::Right => ((e.me.x + 1).min(BOARD_WIDTH - 1), e.me.y),
			},
		};

		println!(
			"{bomb_action} {x} {y}",
			bomb_action = if action.bomb { "BOMB" } else { "MOVE" },
			x = target_cell.0,
			y = target_cell.1
		);

		e.turn += 1;
	}
}
