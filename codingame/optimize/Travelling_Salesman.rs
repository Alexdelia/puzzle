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
	let n = parse_input!(input_line, i32); // This variables stores how many nodes are given
	for i in 0..n as usize {
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let inputs = input_line.split(" ").collect::<Vec<_>>();
		let x = parse_input!(inputs[0], i32); // The x coordinate of the given node
		let y = parse_input!(inputs[1], i32); // The y coordinate of the given node
	}

	// Write an action using println!("message...");
	// To debug: eprintln!("Debug message...");

	println!("0 2 1 3"); // You have to output a valid path
}
