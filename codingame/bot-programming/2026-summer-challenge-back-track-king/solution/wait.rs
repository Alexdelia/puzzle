use std::io::{BufRead, Write};

fn main() {
	let stdin = std::io::stdin();
	let mut input = stdin.lock();
	let stdout = std::io::stdout();
	let mut out = stdout.lock();

	let mut line = String::new();
	let mut read = || -> String {
		line.clear();
		input.read_line(&mut line).unwrap();
		line.trim().to_string()
	};

	read();
	let width: usize = read().parse().unwrap();
	let height: usize = read().parse().unwrap();
	for _ in 0..width * height {
		read();
	}
	let town_count: usize = read().parse().unwrap();
	for _ in 0..town_count {
		read();
	}

	loop {
		read();
		read();
		for _ in 0..width * height {
			read();
		}
		writeln!(out, "WAIT").unwrap();
		out.flush().unwrap();
	}
}
