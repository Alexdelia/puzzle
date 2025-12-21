use std::io;

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

fn main() {
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let n = parse_input!(input_line, i32);

	let mut input = Vec::with_capacity(n as usize);

	for _ in 0..n as usize {
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let inputs = input_line.split(" ").collect::<Vec<_>>();
		let x = parse_input!(inputs[0], i32);
		let y = parse_input!(inputs[1], i32);

		input.push((x, y));
	}

	let names = input
		.iter()
		.cloned()
		.enumerate()
		.map(|(i, _)| i.to_string())
		.collect::<Vec<_>>();

	let c = elkai_rs::Coordinates2D::new(std::collections::HashMap::from_iter(
		input
			.iter()
			.cloned()
			.enumerate()
			.map(|(i, c)| (names[i].as_str(), c)),
	));

	let mut solution = c.solve(10);

	let index_0 = solution.iter().position(|s| *s == "0").unwrap();
	solution.rotate_left(index_0);
	solution.push("0");

	println!("{s}", s = solution.join(" "));
}
