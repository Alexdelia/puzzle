use std::io::{BufRead, Write};

struct Town {
	x: usize,
	y: usize,
	desired: Vec<usize>,
}

fn main() {
	let stdin = std::io::stdin();
	let mut input = stdin.lock();
	let stdout = std::io::stdout();
	let mut out = stdout.lock();
	let mut line = String::new();
	let mut read = |input: &mut std::io::StdinLock| -> String {
		line.clear();
		input.read_line(&mut line).unwrap();
		line.trim().to_string()
	};

	read(&mut input);
	let width: usize = read(&mut input).parse().unwrap();
	let height: usize = read(&mut input).parse().unwrap();
	for _ in 0..width * height {
		read(&mut input);
	}

	let town_count: usize = read(&mut input).parse().unwrap();
	let mut towns = Vec::with_capacity(town_count);
	for _ in 0..town_count {
		let row = read(&mut input);
		let field: Vec<&str> = row.split_whitespace().collect();
		towns.push(Town {
			x: field[1].parse().unwrap(),
			y: field[2].parse().unwrap(),
			desired: if field[3] == "x" {
				Vec::new()
			} else {
				field[3].split(',').map(|id| id.parse().unwrap()).collect()
			},
		});
	}

	let mut wanted: Vec<(usize, usize)> = Vec::new();
	for (src, town) in towns.iter().enumerate() {
		for &dst in &town.desired {
			wanted.push((src, dst));
		}
	}
	wanted.sort_by_key(|&(src, dst)| {
		let (a, b) = (&towns[src], &towns[dst]);
		a.x.abs_diff(b.x) + a.y.abs_diff(b.y)
	});

	loop {
		read(&mut input);
		read(&mut input);
		let mut live = vec![false; wanted.len()];
		for _ in 0..width * height {
			let row = read(&mut input);
			let Some(joined) = row.split_whitespace().nth(3) else {
				continue;
			};
			if joined == "x" {
				continue;
			}
			for pair in joined.split(',') {
				let Some((src, dst)) = pair.split_once('-') else {
					continue;
				};
				let pair = (src.parse().unwrap(), dst.parse().unwrap());
				if let Some(at) = wanted.iter().position(|&w| w == pair) {
					live[at] = true;
				}
			}
		}

		let next = wanted.iter().zip(&live).find(|(_, live)| !**live);
		match next {
			Some((&(src, dst), _)) => {
				let (a, b) = (&towns[src], &towns[dst]);
				writeln!(out, "AUTOPLACE {} {} {} {}", a.x, a.y, b.x, b.y).unwrap();
			}
			None => writeln!(out, "WAIT").unwrap(),
		}
		out.flush().unwrap();
	}
}
