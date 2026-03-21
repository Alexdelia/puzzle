use std::{
	collections::{HashSet, VecDeque},
	fmt::Display,
	io,
	time::{Duration, Instant},
};

const MAX_TURN_DURATION: Duration = Duration::from_millis(45);
// const MAX_TURN_COUNT: Turn = 200;
// const MIN_SNAKEBOT_LEN: usize = 3;
// const MAX_SNAKEBOT_PER_PLAYER: usize = 4;

// type Turn = u8;

type SnakebotId = u8;

/// max 45w x 30h (=1350 tile)
/// snakebot can go out of bounds, but we don't expect under -128 or above 127-45=82
type Axis = i8;
type Coord = (Axis, Axis);

struct Env {
	// turn: Turn,
	g: BlockGrid,

	#[allow(dead_code)]
	my_id: SnakebotId,
	my_snakebot_id_list: Vec<SnakebotId>,
	#[allow(dead_code)]
	foe_snakebot_id_list: Vec<SnakebotId>,
}

#[derive(Clone)]
struct BlockGrid {
	w: Axis,
	h: Axis,
	d: Vec<u64>,
}

struct Action {
	snakebot_id: SnakebotId,
	direction: Dir,
}

#[derive(Clone, Copy)]
enum Dir {
	U = 0,
	D = 1,
	L = 2,
	R = 3,
}

impl From<usize> for Dir {
	fn from(value: usize) -> Self {
		match value {
			0 => Dir::U,
			1 => Dir::D,
			2 => Dir::L,
			3 => Dir::R,
			_ => unreachable!(),
		}
	}
}

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

impl Env {
	fn read() -> Self {
		let mut s = String::new();

		io::stdin().read_line(&mut s).unwrap();
		let my_id = parse_input!(s, SnakebotId);

		let grid = BlockGrid::read();

		s.clear();
		io::stdin().read_line(&mut s).unwrap();
		let snakebot_per_player = parse_input!(s, usize);

		let mut my_snakebot_id_list = Vec::with_capacity(snakebot_per_player);
		let mut foe_snakebot_id_list = Vec::with_capacity(snakebot_per_player);

		for _ in 0..snakebot_per_player {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();
			my_snakebot_id_list.push(parse_input!(s, SnakebotId));
		}
		for _ in 0..snakebot_per_player {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();
			foe_snakebot_id_list.push(parse_input!(s, SnakebotId));
		}

		Env {
			// turn: 0,
			g: grid,

			my_id,
			my_snakebot_id_list,
			foe_snakebot_id_list,
		}
	}

	fn read_apple(w: Axis, h: Axis) -> BlockGrid {
		let mut apple_grid = BlockGrid::new(w, h);

		let mut s = String::new();

		io::stdin().read_line(&mut s).unwrap();
		let power_source_count = parse_input!(s, usize);

		for _ in 0..power_source_count {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();
			let mut input = s.split(" ");
			let x = parse_input!(input.next().unwrap(), Axis);
			let y = parse_input!(input.next().unwrap(), Axis);

			apple_grid.set(x, y);
		}

		apple_grid
	}

	fn read_snakebot(
		&self,
		grid: &mut BlockGrid,
		my_snakebot_list: &mut Vec<(SnakebotId, Vec<Coord>)>,
	) {
		my_snakebot_list.clear();

		let mut s = String::new();

		io::stdin().read_line(&mut s).unwrap();
		let snakebot_count = parse_input!(s, usize);

		for _ in 0..snakebot_count {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();
			let mut input = s.split(" ");
			let snakebot_id = parse_input!(input.next().unwrap(), SnakebotId);
			let body = input
				.next()
				.unwrap()
				.trim()
				.split(":")
				.map(|coord| {
					let mut parts = coord.split(",");
					let x = parse_input!(parts.next().unwrap(), Axis);
					let y = parse_input!(parts.next().unwrap(), Axis);

					(x, y)
				})
				.collect::<Vec<_>>();

			for &(x, y) in body[..body.len() - 1].iter() {
				grid.safe_set(x, y);
			}

			if self.my_snakebot_id_list.contains(&snakebot_id) {
				my_snakebot_list.push((snakebot_id, body));
			} /* else {
			// TODO: check if this is pertinent
			let (x, y) = body[0];
			grid.safe_set(x + 1, y);
			grid.safe_set(x - 1, y);
			grid.safe_set(x, y + 1);
			grid.safe_set(x, y - 1);
			}*/
		}
	}
}

impl BlockGrid {
	#[inline]
	fn index(x: Axis, y: Axis, w: Axis) -> (usize, usize) {
		let index = (y as usize * w as usize + x as usize) / 64;
		let bit = (y as usize * w as usize + x as usize) % 64;
		(index, bit)
	}

	fn new(w: Axis, h: Axis) -> Self {
		let d = vec![0; (w as usize * h as usize).div_ceil(64)];
		BlockGrid { w, h, d }
	}

	fn read() -> Self {
		let mut s = String::new();

		io::stdin().read_line(&mut s).unwrap();
		let w = parse_input!(s, Axis);

		s.clear();
		io::stdin().read_line(&mut s).unwrap();
		let h = parse_input!(s, Axis);

		let mut g = Self::new(w, h);

		for y in 0..h {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();
			for (x, c) in s.trim_matches('\n').chars().enumerate() {
				if c == '#' {
					g.set(x as Axis, y);
				}
			}
		}

		g
	}

	#[inline]
	fn is_safe(&self, x: Axis, y: Axis) -> bool {
		y >= 0 && x >= 0 && x < self.w && y < self.h
	}

	#[inline]
	fn is_set(&self, x: Axis, y: Axis) -> bool {
		if !self.is_safe(x, y) {
			return false;
		}

		let (index, bit) = Self::index(x, y, self.w);
		(self.d[index] & (1 << bit)) != 0
	}

	#[inline]
	fn safe_set(&mut self, x: Axis, y: Axis) {
		if !self.is_safe(x, y) {
			return;
		}

		self.set(x, y);
	}

	#[inline]
	fn set(&mut self, x: Axis, y: Axis) {
		let (index, bit) = Self::index(x, y, self.w);
		self.d[index] |= 1 << bit;
	}

	#[inline]
	fn safe_unset(&mut self, x: Axis, y: Axis) {
		if !self.is_safe(x, y) {
			return;
		}

		self.unset(x, y);
	}

	#[inline]
	fn unset(&mut self, x: Axis, y: Axis) {
		let (index, bit) = Self::index(x, y, self.w);
		self.d[index] &= !(1 << bit);
	}
}

impl Display for Action {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} {}", self.snakebot_id, self.direction)
	}
}

impl Display for Dir {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Dir::U => write!(f, "UP"),
			Dir::D => write!(f, "DOWN"),
			Dir::L => write!(f, "LEFT"),
			Dir::R => write!(f, "RIGHT"),
		}
	}
}

fn apply_dir((x, y): Coord, dir: Dir) -> Coord {
	match dir {
		Dir::U => (x, y - 1),
		Dir::D => (x, y + 1),
		Dir::L => (x - 1, y),
		Dir::R => (x + 1, y),
	}
}

fn is_new_head_in_body(body: &[Coord], x: Axis, y: Axis) -> bool {
	if body[1].0 == x && body[1].1 == y {
		return true;
	}

	if body.len() <= 3 {
		return false;
	}

	body[3..body.len() - 1]
		.iter()
		.any(|&(bx, by)| bx == x && by == y)
}

fn is_upright(body: &[Coord]) -> bool {
	let (x, mut y) = body[0];
	for part in body.iter().skip(1) {
		y += 1;
		if part.0 != x || part.1 != y {
			return false;
		}
	}
	true
}

#[allow(clippy::too_many_arguments)]
fn move_and_queue(
	q: &mut VecDeque<(Dir, Vec<Coord>)>,
	visited: &mut HashSet<Vec<Coord>>,
	grid: &BlockGrid,
	apple: &BlockGrid,
	initial_dir: Dir,
	base_body: &[Coord],
	x: Axis,
	y: Axis,
) {
	// TODO: try VecDeque
	let mut body = Vec::with_capacity(base_body.len());
	body.push((x, y));
	body.extend_from_slice(&base_body[..base_body.len() - 1]);

	if !visited.insert(body.clone()) {
		return;
	}

	// gravity
	for _ in 0..=grid.h {
		// TODO: optimize by reverse iterating
		for i in 0..body.len() {
			body[i].1 += 1;

			if grid.is_set(body[i].0, body[i].1) || apple.is_set(body[i].0, body[i].1) {
				for r in 0..=i {
					body[r].1 -= 1;
				}
				q.push_back((initial_dir, body));
				return;
			}
		}
	}
}

#[allow(clippy::too_many_arguments)]
fn try_visit(
	q: &mut VecDeque<(Dir, Vec<Coord>)>,
	visited: &mut HashSet<Vec<Coord>>,
	grid: &BlockGrid,
	apple: &BlockGrid,
	initial_dir: Dir,
	body: &[Coord],
	x: Axis,
	y: Axis,
) -> Option<(Dir, Option<Coord>)> {
	if apple.is_set(x, y) {
		return Some((initial_dir, Some((x, y))));
	}

	if !grid.is_set(x, y) && !is_new_head_in_body(body, x, y) {
		move_and_queue(q, visited, grid, apple, initial_dir, body, x, y);
	}

	None
}

fn visit_neighbor(
	q: &mut VecDeque<(Dir, Vec<Coord>)>,
	visited: &mut HashSet<Vec<Coord>>,
	grid: &BlockGrid,
	apple: &BlockGrid,
	initial_dir: Dir,
	body: &[Coord],
) -> Option<(Dir, Option<Coord>)> {
	let (x, y) = body[0];

	if let Some(solution) = try_visit(q, visited, grid, apple, initial_dir, body, x, y - 1) {
		return Some(solution);
	}

	// TODO: choose left if more towards right and right if more towards left
	if let Some(solution) = try_visit(q, visited, grid, apple, initial_dir, body, x - 1, y) {
		return Some(solution);
	}
	if let Some(solution) = try_visit(q, visited, grid, apple, initial_dir, body, x + 1, y) {
		return Some(solution);
	}

	if let Some(solution) = try_visit(q, visited, grid, apple, initial_dir, body, x, y + 1) {
		return Some(solution);
	}

	None
}

fn initial_visit_neighbor(
	q: &mut VecDeque<(Dir, Vec<Coord>)>,
	visited: &mut HashSet<Vec<Coord>>,
	grid: &BlockGrid,
	apple: &BlockGrid,
	body: &[Coord],
) -> Option<(Dir, Option<Coord>)> {
	let (x, y) = body[0];
	if !is_upright(body)
		&& let Some(solution) = try_visit(q, visited, grid, apple, Dir::U, body, x, y - 1)
	{
		return Some(solution);
	}

	// TODO: choose left if more towards right and right if more towards left
	if let Some(solution) = try_visit(q, visited, grid, apple, Dir::L, body, x - 1, y) {
		return Some(solution);
	}
	if let Some(solution) = try_visit(q, visited, grid, apple, Dir::R, body, x + 1, y) {
		return Some(solution);
	}

	if let Some(solution) = try_visit(q, visited, grid, apple, Dir::D, body, x, y + 1) {
		return Some(solution);
	}

	None
}

fn has_single_depth_move(
	grid: &BlockGrid,
	apple_grid: &BlockGrid,
	snakebot_body: &[Coord],
) -> Option<(Dir, Option<Coord>)> {
	let (x, y) = snakebot_body[0];

	let mut moves = Vec::<(Axis, Axis, Dir)>::with_capacity(4);
	for (nx, ny, dir) in [
		(x, y - 1, Dir::U),
		(x - 1, y, Dir::L),
		(x + 1, y, Dir::R),
		(x, y + 1, Dir::D),
	] {
		if grid.is_set(nx, ny) {
			continue;
		}

		if apple_grid.is_set(nx, ny) {
			return Some((dir, Some((nx, ny))));
		}

		if is_new_head_in_body(snakebot_body, nx, ny) {
			continue;
		}

		moves.push((nx, ny, dir));
	}

	if moves.len() == 1 {
		return Some((moves[0].2, None));
	} else if moves.is_empty() {
		return Some((Dir::U, None));
	}

	// TODO: maybe early exit if no more apple

	None
}

fn find_snakebot_action(
	grid: &BlockGrid,
	apple_grid: &BlockGrid,
	snakebot_body: &[Coord],
	allowed_time: Duration,
) -> (Dir, Option<Coord>) {
	// TODO: store body more efficiently?
	let mut visited = HashSet::<Vec<Coord>>::new();
	visited.insert(snakebot_body.to_vec());

	let mut q = VecDeque::<(Dir, Vec<Coord>)>::new();
	if let Some(solution) =
		initial_visit_neighbor(&mut q, &mut visited, grid, apple_grid, snakebot_body)
	{
		return solution;
	}

	let default_dir: Dir = q.clone().pop_front().map(|(dir, _)| dir).unwrap_or(Dir::U);
	let default_ret: (Dir, Option<Coord>) = (default_dir, None);
	if q.len() <= 1 {
		return default_ret;
	}

	let mut solution = None;

	let start = Instant::now();
	let mut i = 0;
	while let Some((initial_dir, body)) = q.pop_front() {
		if let Some(s) = visit_neighbor(&mut q, &mut visited, grid, apple_grid, initial_dir, &body)
		{
			if solution.is_none() {
				eprintln!(
					"found solution '{dir}'->{apple:?} after visiting {i} states in {elapsed:?}",
					dir = s.0,
					apple = s.1,
					elapsed = start.elapsed()
				);
				solution = Some(s);
			}
		}

		i += 1;
		let elapsed = start.elapsed();
		if i % 100 == 0 && elapsed >= allowed_time {
			eprintln!("timeout: visited {i} states in {elapsed:?}");
			break;
		}
	}

	if q.is_empty() {
		return solution.unwrap_or(default_ret);
	}

	let mut remaining_dir_count = [0; 4];
	for (dir, _) in q {
		remaining_dir_count[dir as usize] += 1;
	}

	let mut max_dir = 0;
	for i in 1..4 {
		if remaining_dir_count[i] > remaining_dir_count[max_dir] {
			max_dir = i;
		}
	}

	if let Some(solution) = solution {
		if remaining_dir_count[solution.0 as usize] > 0 {
			return solution;
		}
	}

	let best_dir = Dir::from(max_dir);

	return (best_dir, None);
}

fn main() {
	let env = Env::read();
	let mut my_snakebot_list = Vec::with_capacity(env.my_snakebot_id_list.len());

	loop {
		let mut grid = env.g.clone();

		let mut apple_grid = Env::read_apple(env.g.w, env.g.h);
		env.read_snakebot(&mut grid, &mut my_snakebot_list);

		let start = Instant::now();

		let mut action_list = Vec::<Action>::with_capacity(my_snakebot_list.len());
		let mut remaining_my_snakebot_count = my_snakebot_list.len() as u32;

		for (id, body) in &my_snakebot_list {
			if let Some((dir, apple)) = has_single_depth_move(&grid, &apple_grid, body) {
				action_list.push(Action {
					snakebot_id: *id,
					direction: dir,
				});

				let new_head = apply_dir(body[0], dir);
				grid.safe_set(new_head.0, new_head.1);
				remaining_my_snakebot_count -= 1;

				if let Some(apple) = apple {
					apple_grid.safe_unset(apple.0, apple.1);
				}
			}
		}

		for (id, body) in my_snakebot_list.iter() {
			if action_list.iter().any(|action| action.snakebot_id == *id) {
				eprintln!("[{id}] already has a single depth move, skipping");
				continue;
			}

			let sub_start = Instant::now();

			let Some(allowed_time) = MAX_TURN_DURATION
				.checked_sub(start.elapsed())
				.and_then(|remaining| remaining.checked_div(remaining_my_snakebot_count))
			else {
				eprintln!("not enough time for snakebot {id}, skipping");
				action_list.push(Action {
					snakebot_id: *id,
					direction: Dir::U,
				});
				continue;
			};
			eprintln!("[{id}] allowed: {allowed_time:?}");

			for &(x, y) in body {
				grid.safe_unset(x, y);
			}

			let (dir, apple) = find_snakebot_action(&grid, &apple_grid, body, allowed_time);
			action_list.push(Action {
				snakebot_id: *id,
				direction: dir,
			});

			let new_head = apply_dir(body[0], dir);
			grid.safe_set(new_head.0, new_head.1);
			for &(x, y) in body[..body.len() - 1].iter() {
				grid.safe_set(x, y);
			}

			if let Some(apple) = apple {
				apple_grid.safe_unset(apple.0, apple.1);
			}

			let elapsed = sub_start.elapsed();
			eprintln!("[{id}] took: {elapsed:?}");

			remaining_my_snakebot_count -= 1;
		}

		let action_list = action_list
			.into_iter()
			.map(|action| format!("{action}"))
			.collect::<Vec<_>>()
			.join(";");

		println!("{action_list}");
	}
}
