use std::io;

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

const TIME_LIMIT: usize = 2usize.pow(32);

struct Arrow {
	x: usize,
	y: usize,
	dir: Direction,
	dead: bool,
}

enum Direction {
	Up,
	Down,
	Left,
	Right,
}

impl From<char> for Direction {
	fn from(c: char) -> Self {
		match c {
			'^' => Direction::Up,
			'v' => Direction::Down,
			'<' => Direction::Left,
			'>' => Direction::Right,
			_ => unreachable!(),
		}
	}
}

impl Arrow {
	fn new(x: usize, y: usize, dir: Direction) -> Self {
		Arrow {
			x,
			y,
			dir,
			dead: false,
		}
	}

	fn r#move(&mut self, h: usize, w: usize) {
		let (dx, dy) = self.dir.pos();

		let mut x = self.x as isize + dx;
		let mut y = self.y as isize + dy;

		if x < 0 {
			x = w as isize - 1;
		} else if x >= w as isize {
			x = 0;
		}
		if y < 0 {
			y = h as isize - 1;
		} else if y >= h as isize {
			y = 0;
		}

		self.x = x as usize;
		self.y = y as usize;
	}
}

impl Direction {
	fn pos(&self) -> (isize, isize) {
		match self {
			Direction::Up => (0, -1),
			Direction::Down => (0, 1),
			Direction::Left => (-1, 0),
			Direction::Right => (1, 0),
		}
	}
}

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/
fn main() {
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let inputs = input_line.split(" ").collect::<Vec<_>>();
	let h = parse_input!(inputs[0], usize);
	let w = parse_input!(inputs[1], usize);

	let mut arrows = Vec::new();

	for y in 0..h as usize {
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let line = input_line.trim_matches('\n').to_string();

		for (x, c) in line.chars().enumerate() {
			if c == '.' {
				continue;
			}

			arrows.push(Arrow::new(x, y, c.into()));
		}
	}

	for t in 1..TIME_LIMIT {
		for i in 0..arrows.len() {
			arrows[i].r#move(h, w);

			for o in 0..i {
				if arrows[i].x == arrows[o].x && arrows[i].y == arrows[o].y {
					arrows[i].dead = true;
					arrows[o].dead = true;
				}
			}
		}

		arrows.retain(|a| !a.dead);

		if arrows.is_empty() {
			println!("{t}");
			return;
		}
	}

	println!("more than {TIME_LIMIT} timesteps");
}
