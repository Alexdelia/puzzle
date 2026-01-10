use std::{collections::VecDeque, io, str::FromStr, time::Instant};

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

struct Item {
	r#type: ItemType,
	x: Axis,
	y: Axis,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ItemType {
	IncreaseBombCount,
	IncreaseBombRange,
}

impl From<char> for ItemType {
	fn from(c: char) -> Self {
		match c {
			'0' => Self::IncreaseBombCount,
			'1' => Self::IncreaseBombRange,
			_ => unreachable!(),
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

#[derive(Clone, Copy)]
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
								item: ItemType::from(c),
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
							.chars()
							.next()
							.expect("invalid item type")
							.into(),
						x,
						y,
					});
				}
			}
		}
	}
}

#[inline]
fn get_detonating_bomb_list(bomb_list: &[Bomb], turn: TurnIndex) -> Vec<&Bomb> {
	bomb_list.iter().filter(|b| b.explode_at == turn).collect()
}

fn bomb_min_x_range(board: &Board, bomb: &Bomb) -> Axis {
	for x in 0..=bomb.range {
		if bomb.x < x {
			break;
		}
		let nx = bomb.x - x;
		match board[bomb.y][nx] {
			Cell::Wall => return nx + 1,
			_ => {}
		}
	}
	bomb.x.saturating_sub(bomb.range)
}

fn bomb_max_x_range(board: &Board, bomb: &Bomb) -> Axis {
	for x in 1..=bomb.range {
		let nx = bomb.x + x;
		if nx >= BOARD_WIDTH {
			break;
		}
		match board[bomb.y][nx] {
			Cell::Wall => return nx - 1,
			_ => {}
		}
	}
	(bomb.x + bomb.range).min(BOARD_WIDTH - 1)
}

fn bomb_min_y_range(board: &Board, bomb: &Bomb) -> Axis {
	for y in 0..=bomb.range {
		if bomb.y < y {
			break;
		}
		let ny = bomb.y - y;
		match board[ny][bomb.x] {
			Cell::Wall => return ny + 1,
			_ => {}
		}
	}
	bomb.y.saturating_sub(bomb.range)
}

fn bomb_max_y_range(board: &Board, bomb: &Bomb) -> Axis {
	for y in 1..=bomb.range {
		let ny = bomb.y + y;
		if ny >= BOARD_HEIGHT {
			break;
		}
		match board[ny][bomb.x] {
			Cell::Wall => return ny - 1,
			_ => {}
		}
	}
	(bomb.y + bomb.range).min(BOARD_HEIGHT - 1)
}

fn is_in_range_of_detonating_bomb(
	board: &Board,
	detonating_bomb_list: &[&Bomb],
	x: Axis,
	y: Axis,
) -> bool {
	for bomb in detonating_bomb_list {
		if x == bomb.x && (y >= bomb_min_y_range(board, bomb) && y <= bomb_max_y_range(board, bomb))
		{
			return true;
		}
		if y == bomb.y && (x >= bomb_min_x_range(board, bomb) && x <= bomb_max_x_range(board, bomb))
		{
			return true;
		}
	}

	false
}

fn available_direction(board: &Board, x: Axis, y: Axis) -> Vec<(Option<Direction>, (Axis, Axis))> {
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

	dir
}

type ActionEventIndex = usize;

#[derive(Clone, Copy, Default)]
struct ActionEvent {
	death: usize,
	boxes_destroyed: usize,
	item_range_count: usize,
	item_bomb_count: usize,
}

macro_rules! update_event_action {
	($list:expr, $index:expr, $board:expr, $item_list:expr, $x:expr, $y:expr, $is_bombing:expr, $bomb_range:expr) => {{
		for item in $item_list.iter() {
			if item.x == $x && item.y == $y {
				match item.r#type {
					ItemType::IncreaseBombCount => {
						$list[$index].item_bomb_count += 1;
					}
					ItemType::IncreaseBombRange => {
						$list[$index].item_range_count += 1;
					}
				}
			}
		}

		if $is_bombing {
			let mut destroyed = 0;
			for dy in 1..=$bomb_range {
				if dy > $y {
					break;
				}
				match $board[$y - dy][$x] {
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
				match $board[$y][$x - dx] {
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
			$list[$index].boxes_destroyed += destroyed;
		}
	}};
}

fn init_starting_action(e: &Env, queue: &mut Queue) -> (Vec<Action>, Vec<ActionEvent>) {
	let detonating_bomb_list = get_detonating_bomb_list(&e.bomb_list, e.turn);

	let start_with_bomb = e.me.bomb_count > 0;
	let mut available_dir_list = available_direction(&e.board, e.me.x, e.me.y);

	available_dir_list.retain(|&(_, (nx, ny))| {
		!is_in_range_of_detonating_bomb(&e.board, &detonating_bomb_list, nx, ny)
	});

	let mut starting_action_list = Vec::<Action>::with_capacity(if start_with_bomb {
		available_dir_list.len() * 2
	} else {
		available_dir_list.len()
	});
	let mut starting_action_event_list =
		Vec::<ActionEvent>::with_capacity(starting_action_list.len());

	for (dir, (nx, ny)) in available_dir_list.into_iter() {
		starting_action_list.push(Action {
			bomb: false,
			direction: dir,
		});
		let index = starting_action_event_list.len();
		starting_action_event_list.push(ActionEvent::default());
		update_event_action!(
			starting_action_event_list,
			index,
			e.board,
			e.item_list,
			nx,
			ny,
			false,
			e.me.bomb_range
		);
		queue.push_back(QueueItem {
			action_index: index,
			x: nx,
			y: ny,
		});

		if start_with_bomb {
			starting_action_list.push(Action {
				bomb: true,
				direction: dir,
			});
			let index = starting_action_event_list.len();
			starting_action_event_list.push(ActionEvent::default());
			update_event_action!(
				starting_action_event_list,
				index,
				e.board,
				e.item_list,
				nx,
				ny,
				true,
				e.me.bomb_range
			);
			queue.push_back(QueueItem {
				action_index: index,
				x: nx,
				y: ny,
			});
		}
	}

	(starting_action_list, starting_action_event_list)
}

type Queue = VecDeque<QueueItem>;

struct QueueItem {
	action_index: ActionEventIndex,
	x: Axis,
	y: Axis,
}

fn best_action(e: &Env) -> Action {
	let mut queue = Queue::new();
	let mut current_queue = Queue::new();

	let (starting_action_list, mut starting_action_event_list) =
		init_starting_action(e, &mut queue);

	let mut turn = e.turn;
	while turn < e.turn + 8 {
		turn += 1;
		dbg!(turn, e.turn_start_time.elapsed().as_millis());
		std::mem::swap(&mut queue, &mut current_queue);

		let detonating_bomb_list = get_detonating_bomb_list(&e.bomb_list, turn);

		while let Some(item) = current_queue.pop_front() {
			for (_, (nx, ny)) in available_direction(&e.board, item.x, item.y).into_iter() {
				if is_in_range_of_detonating_bomb(&e.board, &detonating_bomb_list, nx, ny) {
					starting_action_event_list[item.action_index].death += 1;
					continue;
				}

				update_event_action!(
					starting_action_event_list,
					item.action_index,
					e.board,
					e.item_list,
					nx,
					ny,
					true, // TODO: real simulation with bomb count & keep bombed cells
					e.me.bomb_range
				);

				queue.push_back(QueueItem {
					action_index: item.action_index,
					x: nx,
					y: ny,
				});
			}
		}
	}

	let mut action_remaining_count_list = vec![0usize; starting_action_list.len()];
	for item in queue.iter() {
		action_remaining_count_list[item.action_index] += 1;
	}

	let mut best = (0, (usize::MAX, 0));
	for (i, event) in starting_action_event_list.iter().enumerate() {
		let death_ratio = event.death as f32 / (action_remaining_count_list[i].max(1) as f32);
		let death_score = (death_ratio * 10.0).round() as usize;
		if death_score > best.1.0 {
			continue;
		}

		if death_score < best.1.0 {
			best = (i, (death_score, 0));
			continue;
		}

		// TODO: scale to remaining turn
		let score = event.item_bomb_count * 3 + event.item_range_count * 2 + event.boxes_destroyed;

		if score > best.1.1 {
			best = (i, (death_score, score));
		}
	}

	starting_action_list[best.0]
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
