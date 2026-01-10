use std::{
	collections::{HashSet, VecDeque},
	io,
	str::FromStr,
	time::Instant,
};

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
	Up,
	Down,
	Left,
	Right,
}

type TurnIndex = u8;
type Id = u8;

type Axis = usize;

const BOARD_WIDTH: Axis = 13;
const BOARD_HEIGHT: Axis = 11;

type Board = [[Cell; BOARD_WIDTH]; BOARD_HEIGHT];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cell {
	Empty,
	Wall,
	Box(Box),
}

impl Cell {
	#[inline]
	fn is_walkable(&self) -> bool {
		matches!(self, Cell::Empty)
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Box {
	item: ItemType,
	explode_at: Option<TurnIndex>,
}

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
	x: Axis,
	y: Axis,
	bomb_count: usize,
	bomb_range: usize,
}

struct Bomb {
	x: Axis,
	y: Axis,
	range: Axis,
	explode_at: TurnIndex,
}

impl Bomb {
	#[inline]
	fn min_x_range(&self) -> Axis {
		self.x.saturating_sub(self.range)
	}

	#[inline]
	fn max_x_range(&self) -> Axis {
		(self.x + self.range).min(BOARD_WIDTH - 1)
	}

	#[inline]
	fn min_y_range(&self) -> Axis {
		self.y.saturating_sub(self.range)
	}

	#[inline]
	fn max_y_range(&self) -> Axis {
		(self.y + self.range).min(BOARD_HEIGHT - 1)
	}
}

struct Item {
	r#type: ItemType,
	x: Axis,
	y: Axis,
}

enum ItemType {
	IncreaseBombCount,
	IncreaseBombRange,
}

impl FromStr for ItemType {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"0" => Ok(ItemType::IncreaseBombCount),
			"1" => Ok(ItemType::IncreaseBombRange),
			_ => Err(()),
		}
	}
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
			let x = parse_input!(inputs[2], Axis);
			let y = parse_input!(inputs[3], Axis);

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
					let range = parse_input!(inputs[5], Axis);
					self.bomb_list.push(Bomb {
						x,
						y,
						explode_at,
						range,
					});
				}
				EntityType::Item => {
					self.item_list.push(Item {
						r#type: inputs[4]
							.trim()
							.parse::<ItemType>()
							.expect("invalid item type"),
						x,
						y,
					});
				}
			}
		}
	}
}

fn available_direction(
	board: &Board,
	x: Axis,
	y: Axis,
	turn: TurnIndex,
	bomb_list: &[Bomb],
) -> Vec<(Option<Direction>, (Axis, Axis))> {
	let mut dir = Vec::<(Option<Direction>, (Axis, Axis))>::with_capacity(5);

	dir.push((None, (x, y)));

	if y > 0 {
		let ny = y - 1;
		if board[ny][x].is_walkable() {
			dir.push((Some(Direction::Up), (x, ny)));
		}
	}
	let ny = y + 1;
	if ny < BOARD_HEIGHT {
		if board[ny][x].is_walkable() {
			dir.push((Some(Direction::Down), (x, ny)));
		}
	}
	if x > 0 {
		let nx = x - 1;
		if board[y][nx].is_walkable() {
			dir.push((Some(Direction::Left), (nx, y)));
		}
	}
	let nx = x + 1;
	if nx < BOARD_WIDTH {
		if board[y][nx].is_walkable() {
			dir.push((Some(Direction::Right), (nx, y)));
		}
	}

	let detonating_bomb_list: Vec<&Bomb> =
		bomb_list.iter().filter(|b| b.explode_at == turn).collect();
	for bomb in detonating_bomb_list {
		let bx = bomb.x;
		let by = bomb.y;
		let min_x = bomb.min_x_range();
		let max_x = bomb.max_x_range();
		let min_y = bomb.min_y_range();
		let max_y = bomb.max_y_range();
		dir.retain(|&(_, (x, y))| {
			if x == bx && (y >= min_y && y <= max_y) {
				return false;
			}
			if y == by && (x >= min_x && x <= max_x) {
				return false;
			}
			true
		});
	}

	dir
}

fn flood_fill(e: &Env) -> HashSet<(Axis, Axis)> {
	let mut queue = VecDeque::<(Axis, Axis)>::new();
	let mut visited = HashSet::<(Axis, Axis)>::new();

	queue.push_back((e.me.x, e.me.y));
	visited.insert((e.me.x, e.me.y));
	while let Some((x, y)) = queue.pop_front() {
		if x > 0 {
			let nx = x - 1;
			let ny = y;
			if e.board[ny][nx].is_walkable() && !visited.contains(&(nx, ny)) {
				visited.insert((nx, ny));
				queue.push_back((nx, ny));
			}
		}
		if x + 1 < BOARD_WIDTH {
			let nx = x + 1;
			let ny = y;
			if e.board[ny][nx].is_walkable() && !visited.contains(&(nx, ny)) {
				visited.insert((nx, ny));
				queue.push_back((nx, ny));
			}
		}
		if y > 0 {
			let nx = x;
			let ny = y - 1;
			if e.board[ny][nx].is_walkable() && !visited.contains(&(nx, ny)) {
				visited.insert((nx, ny));
				queue.push_back((nx, ny));
			}
		}
		if y + 1 < BOARD_HEIGHT {
			let nx = x;
			let ny = y + 1;
			if e.board[ny][nx].is_walkable() && !visited.contains(&(nx, ny)) {
				visited.insert((nx, ny));
				queue.push_back((nx, ny));
			}
		}
	}

	visited
}

struct QueueItem {
	action_index: u8,
	x: Axis,
	y: Axis,
}

#[derive(Clone, Copy)]
struct ActionEvent {
	death: usize,
	boxes_destroyed: usize,
	item_rannge_count: usize,
	item_bomb_count: usize,
}

macro_rules! update_event_action {
	($list:expr, $index:expr, $board:expr, $x:expr, $y:expr, $is_bombing:expr, $bomb_range:expr) => {{
		let event = &mut $list[$index];
		if $is_bombing {
			let mut destroyed = 0;
			for dy in 1..=$bomb_range {
				if dy > $y {
					break;
				}
				match $board[y - dy][$x] {
					Cell::Wall => break,
					Cell::Box(_) => {
						destroyed += 1;
						break;
					}
					Cell::Empty => {}
				}
			}
			for dy in 1..=$bomb_range {
				let ny = $y + dy;
				if ny >= BOARD_HEIGHT {
					break;
				}
				match $board[ny][$x] {
					Cell::Wall => break,
					Cell::Box(_) => {
						destroyed += 1;
						break;
					}
					Cell::Empty => {}
				}
			}
			for dx in 1..=$bomb_range {
				if dx > $x {
					break;
				}
				match $board[$y][x - dx] {
					Cell::Wall => break,
					Cell::Box(_) => {
						destroyed += 1;
						break;
					}
					Cell::Empty => {}
				}
			}
			for dx in 1..=$bomb_range {
				let nx = $x + dx;
				if nx >= BOARD_WIDTH {
					break;
				}
				match $board[$y][nx] {
					Cell::Wall => break,
					Cell::Box(_) => {
						destroyed += 1;
						break;
					}
					Cell::Empty => {}
				}
			}
			event.boxes_destroyed += destroyed;
		}
	}};
}

fn best_action(e: &Env) -> Action {
	let mut queue = VecDeque::<QueueItem>::new();
	let mut current_queue = VecDeque::<QueueItem>::new();

	let start_with_bomb = e.me.bomb_count > 0;
	let available_dir_list = available_direction(&e.board, e.me.x, e.me.y, e.turn, &e.bomb_list);

	let mut starting_action_list = Vec::<Action>::with_capacity(if start_with_bomb {
		available_dir_list.len() * 2
	} else {
		available_dir_list.len()
	});
	let mut starting_action_event_list =
		Vec::<ActionEvent>::with_capacity(starting_action_list.len());

	for (i, (dir, (nx, ny))) in available_direction(&e.board, e.me.x, e.me.y, e.turn, &e.bomb_list)
		.into_iter()
		.enumerate()
	{
		starting_action_list.push(Action {
			bomb: false,
			direction: Some(dir),
		});
		queue.push_back(QueueItem {
			action_index: i as u8,
			x: nx,
			y: ny,
		});
	}

	let mut turn = e.turn;
	while turn < e.turn + 8 {
		turn += 1;
		std::mem::swap(&mut queue, &mut current_queue);

		while let Some(item) = current_queue.pop_front() {}
	}

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
	assert_eq!(parse_input!(inputs[0], Axis), BOARD_WIDTH);
	assert_eq!(parse_input!(inputs[1], Axis), BOARD_HEIGHT);
	e.me.id = parse_input!(inputs[2], Id);

	loop {
		e.read_board();
		e.read_entities();

		e.turn += 1;
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
	}
}
