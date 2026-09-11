use std::io::{self, BufRead, Write};

fn main() {
	let path = std::env::var("BTK_SCRIPT").expect("BTK_SCRIPT: path to a recorded log");
	let side: usize = std::env::var("BTK_SIDE")
		.ok()
		.and_then(|text| text.parse().ok())
		.unwrap_or(0);
	let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));

	let mut answers = Vec::new();
	let mut take = false;
	for line in text.lines() {
		if line.trim() == "Standard Output Stream:" {
			take = true;
			continue;
		}
		if take {
			answers.push(line.trim().to_string());
			take = false;
		}
	}

	let mut script = answers
		.chunks(2)
		.filter_map(|pair| pair.get(side).cloned())
		.collect::<Vec<_>>()
		.into_iter();

	let input = io::stdin().lock();
	let mut out = io::stdout().lock();
	let mut lines = input.lines().map_while(Result::ok);
	let mut seen = 0;
	let mut cells = 0;
	while let Some(line) = lines.next() {
		if seen == 0 {
			let width: usize = lines.next().unwrap().trim().parse().unwrap();
			let height: usize = lines.next().unwrap().trim().parse().unwrap();
			cells = width * height;
			for _ in 0..cells {
				lines.next();
			}
			let towns: usize = lines.next().unwrap().trim().parse().unwrap();
			for _ in 0..towns {
				lines.next();
			}
			seen = 1;
			let _ = line;
			continue;
		}
		lines.next();
		for _ in 0..cells {
			lines.next();
		}
		let answer = script.next().unwrap_or_else(|| "WAIT".into());
		writeln!(out, "{answer}").unwrap();
		out.flush().unwrap();
	}
}
