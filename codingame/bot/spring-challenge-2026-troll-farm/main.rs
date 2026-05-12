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
	let inputs = input_line.split(" ").collect::<Vec<_>>();
	let width = parse_input!(inputs[0], i32);
	let height = parse_input!(inputs[1], i32);
	for i in 0..height as usize {
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let line = input_line.trim_matches('\n').to_string();
	}

	// game loop
	loop {
		for i in 0..2 as usize {
			let mut input_line = String::new();
			io::stdin().read_line(&mut input_line).unwrap();
			let inputs = input_line.split(" ").collect::<Vec<_>>();
			let plum = parse_input!(inputs[0], i32);
			let lemon = parse_input!(inputs[1], i32);
			let apple = parse_input!(inputs[2], i32);
			let banana = parse_input!(inputs[3], i32);
			let iron = parse_input!(inputs[4], i32);
			let wood = parse_input!(inputs[5], i32);
		}
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let trees_count = parse_input!(input_line, i32);
		for i in 0..trees_count as usize {
			let mut input_line = String::new();
			io::stdin().read_line(&mut input_line).unwrap();
			let inputs = input_line.split(" ").collect::<Vec<_>>();
			let _type = inputs[0].trim().to_string();
			let x = parse_input!(inputs[1], i32);
			let y = parse_input!(inputs[2], i32);
			let size = parse_input!(inputs[3], i32);
			let health = parse_input!(inputs[4], i32);
			let fruits = parse_input!(inputs[5], i32);
			let cooldown = parse_input!(inputs[6], i32);
		}
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let trolls_count = parse_input!(input_line, i32);
		for i in 0..trolls_count as usize {
			let mut input_line = String::new();
			io::stdin().read_line(&mut input_line).unwrap();
			let inputs = input_line.split(" ").collect::<Vec<_>>();
			let id = parse_input!(inputs[0], i32);
			let player = parse_input!(inputs[1], i32);
			let x = parse_input!(inputs[2], i32);
			let y = parse_input!(inputs[3], i32);
			let movement_speed = parse_input!(inputs[4], i32);
			let carry_capacity = parse_input!(inputs[5], i32);
			let harvest_power = parse_input!(inputs[6], i32);
			let chop_power = parse_input!(inputs[7], i32);
			let carry_plum = parse_input!(inputs[8], i32);
			let carry_lemon = parse_input!(inputs[9], i32);
			let carry_apple = parse_input!(inputs[10], i32);
			let carry_banana = parse_input!(inputs[11], i32);
			let carry_iron = parse_input!(inputs[12], i32);
			let carry_wood = parse_input!(inputs[13], i32);
		}

		// Write an action using println!("message...");
		// To debug: eprintln!("Debug message...");

		// valid actions:
		// MOVE <id> <x> <y>
		// HARVEST <id> - when you are on the same cell as a tree
		// DROP <id> - when you are next to your shack and carry items
		println!("MOVE 0 7 7");
	}
}
