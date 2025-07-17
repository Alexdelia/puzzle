use std::io;

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

fn main() {
	let mut enemy_ids = Vec::<i32>::new();

	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let my_id = parse_input!(input_line, i32); // Your player id (0 or 1)
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let agent_data_count = parse_input!(input_line, i32); // Total number of agents in the game
	for i in 0..agent_data_count as usize {
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let inputs = input_line.split(" ").collect::<Vec<_>>();
		let agent_id = parse_input!(inputs[0], i32); // Unique identifier for this agent
		let player = parse_input!(inputs[1], i32); // Player id of this agent
		if player != my_id {
			enemy_ids.push(agent_id);
		}
		let shoot_cooldown = parse_input!(inputs[3], i32); // Number of turns between each of this agent's shots
		let optimal_range = parse_input!(inputs[3], i32); // Maximum manhattan distance for greatest damage output
		let soaking_power = parse_input!(inputs[4], i32); // Damage output within optimal conditions
		let splash_bombs = parse_input!(inputs[5], i32); // Number of splash bombs this can throw this game
	}
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let inputs = input_line.split(" ").collect::<Vec<_>>();
	let width = parse_input!(inputs[0], i32); // Width of the game map
	let height = parse_input!(inputs[1], i32); // Height of the game map
	for i in 0..height as usize {
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let inputs = input_line.split_whitespace().collect::<Vec<_>>();
		for j in 0..width as usize {
			let x = parse_input!(inputs[3 * j], i32); // X coordinate, 0 is left edge
			let y = parse_input!(inputs[3 * j + 1], i32); // Y coordinate, 0 is top edge
			let tile_type = parse_input!(inputs[3 * j + 2], i32);
		}
	}

	// game loop
	loop {
		let mut wettest_id = 0;
		let mut wettest_wetness = -1;

		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let agent_count = parse_input!(input_line, i32); // Total number of agents still in the game
		for i in 0..agent_count as usize {
			let mut input_line = String::new();
			io::stdin().read_line(&mut input_line).unwrap();
			let inputs = input_line.split(" ").collect::<Vec<_>>();
			let agent_id = parse_input!(inputs[0], i32);
			let x = parse_input!(inputs[1], i32);
			let y = parse_input!(inputs[2], i32);
			let cooldown = parse_input!(inputs[3], i32); // Number of turns before this agent can shoot
			let splash_bombs = parse_input!(inputs[4], i32);
			let wetness = parse_input!(inputs[5], i32); // Damage (0-100) this agent has taken

			if enemy_ids.contains(&agent_id) && wetness > wettest_wetness {
				wettest_id = agent_id;
				wettest_wetness = wetness;
			}
		}
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let my_agent_count = parse_input!(input_line, i32); // Number of alive agents controlled by you
		for i in 0..my_agent_count as usize {
			// Write an action using println!("message...");
			// To debug: eprintln!("Debug message...");

			// One line per agent: <agentId>;<action1;action2;...> actions are "MOVE x y | SHOOT id | THROW x y | HUNKER_DOWN | MESSAGE text"

			println!("SHOOT {wettest_id}");
		}
	}
}
