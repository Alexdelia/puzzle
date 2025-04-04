use std::io;

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

fn solve(a: f64) -> u32 {
	let mut i: f64 = 0.0;
	let mut s: f64 = 0.0;
	let log_a = a.ln();

	while s <= i * log_a {
		i += 1.0;
		s += i.ln();
	}

	i as u32
}

fn main() {
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let _ = parse_input!(input_line, i32);

	let mut inputs = String::new();
	io::stdin().read_line(&mut inputs).unwrap();

	let mut output: Vec<_> = Vec::new();

	for i in inputs.split_whitespace() {
		let a = parse_input!(i, f64);

		output.push(solve(a));
	}

	println!(
		"{}",
		output
			.iter()
			.map(ToString::to_string)
			.collect::<Vec<_>>()
			.join(" ")
	);
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_solve() {
		assert_eq!(solve(2.0), 4);
		assert_eq!(solve(3.0), 7);
	}
}
