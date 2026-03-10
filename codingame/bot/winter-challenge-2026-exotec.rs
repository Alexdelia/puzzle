use std::{fmt::Display, io};

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

type Id = u8;

struct Env {
	w: usize,
	h: usize,
	base_grid: Grid,

	my_id: Id,
	my_snakebot_id_list: Vec<Id>,
	foe_snakebot_id_list: Vec<Id>,
}

// NOTE: does not handle going out of bounds
// TODO: try flat Vec<Tile> and index with y * w + x
// TODO: try 1 bit per tile and bitwise operations
type Grid = Vec<Vec<Tile>>;
type Coord = (usize, usize);

#[derive(Clone, Copy)]
enum Tile {
	Empty,
	Block,
	Apple,
}

struct Action {
	snakebot_id: Id,
	direction: Dir,
}

enum Dir {
	U,
	D,
	L,
	R,
}

impl Tile {
	fn from_char(c: char) -> Self {
		match c {
			'.' => Tile::Empty,
			'#' => Tile::Block,
			_ => panic!("invalid tile character: {c}"),
		}
	}
}

impl Env {
	fn read() -> Self {
		let mut s = String::new();

		io::stdin().read_line(&mut s).unwrap();
		let my_id = parse_input!(s, Id);

		s.clear();
		io::stdin().read_line(&mut s).unwrap();
		let w = parse_input!(s, usize);

		s.clear();
		io::stdin().read_line(&mut s).unwrap();
		let h = parse_input!(s, usize);

		let mut grid = Vec::with_capacity(h);
		for _ in 0..h {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();
			let row = s.trim_matches('\n').chars().map(Tile::from_char).collect();
			grid.push(row);
		}

		s.clear();
		io::stdin().read_line(&mut s).unwrap();
		let snakebot_per_player = parse_input!(s, usize);

		let mut my_snakebot_id_list = Vec::with_capacity(snakebot_per_player);
		let mut foe_snakebot_id_list = Vec::with_capacity(snakebot_per_player);

		for _ in 0..snakebot_per_player {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();
			my_snakebot_id_list.push(parse_input!(s, Id));
		}
		for _ in 0..snakebot_per_player {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();
			foe_snakebot_id_list.push(parse_input!(s, Id));
		}

		Env {
			w,
			h,
			base_grid: grid,

			my_id,
			my_snakebot_id_list,
			foe_snakebot_id_list,
		}
	}

	fn read_apple(grid: &mut Grid) {
		let mut s = String::new();

		io::stdin().read_line(&mut s).unwrap();
		let power_source_count = parse_input!(s, usize);
		for _ in 0..power_source_count {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();
			let mut input = s.split(" ");
			let x = parse_input!(input.next().unwrap(), usize);
			let y = parse_input!(input.next().unwrap(), usize);

			grid[y][x] = Tile::Apple;
		}
	}

	fn read_snakebot(&self, grid: &mut Grid, my_snakebot_list: &mut Vec<(Id, Vec<Coord>)>) {
		my_snakebot_list.clear();

		let mut s = String::new();

		io::stdin().read_line(&mut s).unwrap();
		let snakebot_count = parse_input!(s, usize);

		for _ in 0..snakebot_count {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();
			let mut input = s.split(" ");
			let snakebot_id = parse_input!(input.next().unwrap(), Id);
			let body = input
				.next()
				.unwrap()
				.trim()
				.split(":")
				.filter_map(|coord| {
					let mut parts = coord.split(",");
					let x = parse_input!(parts.next().unwrap(), isize);
					let y = parse_input!(parts.next().unwrap(), isize);

					if x < 0 || y < 0 {
						return None;
					}

					let (x, y) = (x as usize, y as usize);
					if x >= self.w || y >= self.h {
						return None;
					}

					Some((x, y))
				})
				.collect::<Vec<_>>();
			if body.is_empty() {
				continue;
			}

			if self.my_snakebot_id_list.contains(&snakebot_id) {
				my_snakebot_list.push((snakebot_id, body));
			} else {
				let (x, y) = body[0];
				for_neighbor(x, y, self.w, self.h, |x, y| {
					grid[y][x] = Tile::Block;
				});
				for (x, y) in body {
					grid[y][x] = Tile::Block;
				}
			}
		}
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

#[inline]
fn for_neighbor<F>(x: usize, y: usize, w: usize, h: usize, mut f: F)
where
	F: FnMut(usize, usize),
{
	if x > 0 {
		f(x - 1, y);
	}
	if x + 1 < w {
		f(x + 1, y);
	}
	if y > 0 {
		f(x, y - 1);
	}
	if y + 1 < h {
		f(x, y + 1);
	}
}

fn find_snakebot_action(env: &Env, snakebot_id: Id, snakebot_body: &[Coord]) -> Action {
	Action {
		snakebot_id: snakebot_id,
		direction: Dir::U,
	}
}

fn main() {
	let env = Env::read();
	let mut my_snakebot_list = Vec::with_capacity(env.my_snakebot_id_list.len());

	loop {
		let mut grid = env.base_grid.clone();

		Env::read_apple(&mut grid);
		env.read_snakebot(&mut grid, &mut my_snakebot_list);

		let action_list = my_snakebot_list
			.iter()
			.map(|(id, body)| find_snakebot_action(&env, *id, body).to_string())
			.collect::<Vec<_>>()
			.join(";");

		println!("{action_list}");
	}
}
