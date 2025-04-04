use std::io;

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

type Coord = (u8, u8);

enum Direction {
	Left,
	Right,
}

fn main() {
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let n = parse_input!(input_line, u8);

	let mut aircraft: Vec<(Coord, Direction)> = Vec::new();
	let mut launcher: u8 = 0;

	for h in 0..n {
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let line = input_line.trim().to_string();

		for (w, c) in line.chars().enumerate() {
			match c {
				'<' => aircraft.push(((n - h, w as u8), Direction::Left)),
				'>' => aircraft.push(((n - h, w as u8), Direction::Right)),
				'^' => launcher = w as u8,
				_ => (),
			}
		}
	}

	while !aircraft.is_empty() {
		let mut shoot = false;

		aircraft = aircraft
			.into_iter()
			.filter_map(|a| {
				let ((h, w), dir) = a;

				match dir {
					Direction::Left => {
						if w - h == launcher {
							shoot = true;
							None
						} else {
							Some(((h, w - 1), dir))
						}
					}
					Direction::Right => {
						if w + h == launcher {
							shoot = true;
							None
						} else {
							Some(((h, w + 1), dir))
						}
					}
				}
			})
			.collect();

		if shoot {
			println!("SHOOT");
		} else {
			println!("WAIT");
		}
	}
}
