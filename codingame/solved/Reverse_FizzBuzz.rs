use std::io;

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

fn can(t: (u8, u8)) -> bool {
	t.0 == 0 || t.1 == 0
}

fn set(t: (u8, u8), i: u8) -> (u8, u8) {
	if t.0 == 0 {
		(i, t.1)
	} else {
		(t.0, i)
	}
}

fn get(t: (u8, u8), start: Option<u8>) -> u8 {
	if t.1 != 0 {
		t.1 - t.0
	} else {
		t.0 + start.unwrap_or(0)
	}
}

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/
fn main() {
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let n = parse_input!(input_line, u8);
	dbg!(&n);

	let mut fizz: (u8, u8) = (0, 0);
	let mut buzz: (u8, u8) = (0, 0);
	let mut start: Option<u8> = None;

	for i in 0..n as usize {
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let line = input_line.trim_matches('\n').to_string();
		eprintln!("{line}");

		if line == "Fizz" && can(fizz) {
			fizz = set(fizz, i as u8 + 1);
		} else if line == "Buzz" && can(buzz) {
			buzz = set(buzz, i as u8 + 1);
		} else if line == "FizzBuzz" {
			if can(fizz) {
				fizz = set(fizz, i as u8 + 1);
			}
			if can(buzz) {
				buzz = set(buzz, i as u8 + 1);
			}
		} else if start.is_none() {
			if let Ok(n) = line.parse::<u8>() {
				start = Some(n - 1 - i as u8);
			}
		}
	}

	// Write an answer using println!("message...");
	// To debug: eprintln!("Debug message...");

	println!("{} {}", get(fizz, start), get(buzz, start));
}
