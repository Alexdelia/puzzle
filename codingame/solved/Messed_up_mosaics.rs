use std::io;

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/
fn main() {
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let n = parse_input!(input_line, i32);
	eprintln!("{n}");
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let p = input_line.trim_matches('\n').to_string();
	let ps = " ".repeat(p.len());

	// create an array of all combinations of the pattern, each moved one index to the right
	let mut a = vec![p.clone()];
	for _ in 0..p.len() - 1 {
		let mut s = a.last().unwrap().clone();
		let c = s.remove(0);
		s.push(c);
		a.push(s);
	}

	dbg!(&a);
	for x in 0..n as usize {
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let row = input_line.trim_matches('\n').to_string();
		eprintln!("{row}");

		let mut removed = Vec::new();

		for s in &a {
			removed.push(row.replace(s, &ps));
		}

		// find the only index in all the strings that is not a space
		let mut i = 0;
		let mut found = false;
		while i < removed[0].len() {
			let mut c = removed[0].chars().nth(i).unwrap();
			for s in &removed {
				if s.chars().nth(i).unwrap() != c {
					c = ' ';
					break;
				}
			}
			if c != ' ' {
				found = true;
				break;
			}
			i += 1;
		}

		if found {
			println!("({i},{x})");
			return;
		}
	}

	// Write an answer using println!("message...");
	// To debug: eprintln!("Debug message...");
}
