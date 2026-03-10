use std::io;

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
type Grid = Vec<Vec<Tile>>;

enum Tile {
	Empty,
	Block,
	Apple,
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
}

fn main() {
	let env = Env::read();

	loop {
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let power_source_count = parse_input!(input_line, i32);
		for i in 0..power_source_count as usize {
			let mut input_line = String::new();
			io::stdin().read_line(&mut input_line).unwrap();
			let inputs = input_line.split(" ").collect::<Vec<_>>();
			let x = parse_input!(inputs[0], i32);
			let y = parse_input!(inputs[1], i32);
		}
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let snakebot_count = parse_input!(input_line, i32);
		for i in 0..snakebot_count as usize {
			let mut input_line = String::new();
			io::stdin().read_line(&mut input_line).unwrap();
			let inputs = input_line.split(" ").collect::<Vec<_>>();
			let snakebot_id = parse_input!(inputs[0], i32);
			let body = inputs[1].trim().to_string();
		}

		// Write an action using println!("message...");
		// To debug: eprintln!("Debug message...");

		println!("WAIT");
	}
}
