use std::io;

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

#[derive(Default, Debug, Clone, Copy)]
struct GroupedCoin {
	count: usize,
	value: usize,
}

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/
fn main() {
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let value_to_reach = parse_input!(input_line, usize);

	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let n = parse_input!(input_line, usize);

	let mut total = 0;
	let mut pocket = vec![GroupedCoin::default(); n];

	let mut inputs = String::new();
	io::stdin().read_line(&mut inputs).unwrap();
	for (i, count) in inputs.split_whitespace().enumerate() {
		pocket[i].count = parse_input!(count, usize);
	}

	let mut inputs = String::new();
	io::stdin().read_line(&mut inputs).unwrap();
	for (i, value) in inputs.split_whitespace().enumerate() {
		pocket[i].value = parse_input!(value, usize);

		total += pocket[i].count * pocket[i].value;
	}

	if total < value_to_reach {
		println!("-1");
		return;
	}

	pocket.sort_by(|a, b| a.value.cmp(&b.value));

	let mut needed = 0;
	let mut remaining = value_to_reach;

	for coin in pocket {
		let count = (remaining as f64 / coin.value as f64).ceil() as usize;

		if count > coin.count {
			needed += coin.count;
			remaining -= coin.count * coin.value;
		} else {
			needed += count;
			break;
		}
	}

	println!("{needed}");
}
