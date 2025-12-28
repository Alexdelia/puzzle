use std::{collections::VecDeque, io};

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

type Elevation = u8;

fn main() {
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let n = parse_input!(input_line, usize);

	let mut m: Vec<Vec<Elevation>> = Vec::with_capacity(n);

	for i in 0..n {
		m.push(Vec::with_capacity(n));

		let mut inputs = String::new();
		io::stdin().read_line(&mut inputs).unwrap();
		for elevation in inputs.split_whitespace() {
			m[i].push(parse_input!(elevation, u8));
		}
	}

	let mut q: VecDeque<(usize, usize)> = VecDeque::new();
	let mut seen: Vec<Vec<bool>> = vec![vec![false; n]; n];

	let center = n / 2;
	q.push_back((center, center));
	seen[center][center] = true;

	while let Some((x, y)) = q.pop_front() {
		if m[x][y] == 0 {
			println!("yes");
			return;
		}

		let neighbors = [
			(x as isize - 1, y as isize),
			(x as isize + 1, y as isize),
			(x as isize, y as isize - 1),
			(x as isize, y as isize + 1),
		];

		let elevation = m[x][y];

		for (nx, ny) in neighbors.iter() {
			if *nx < 0 || *ny < 0 || *nx >= n as isize || *ny >= n as isize {
				continue;
			}

			let nx = *nx as usize;
			let ny = *ny as usize;

			if seen[nx][ny] {
				continue;
			}

			if (m[nx][ny] as isize - elevation as isize).abs() > 1 {
				continue;
			}

			q.push_back((nx, ny));
			seen[nx][ny] = true;
		}
	}

	println!("no");
}
