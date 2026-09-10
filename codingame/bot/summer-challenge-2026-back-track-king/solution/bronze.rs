use std::io;

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

fn main() {
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let my_id = parse_input!(input_line, i32); // 0 or 1
	let foe_id = 1 - my_id;
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let width = parse_input!(input_line, i32); // map size
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let height = parse_input!(input_line, i32);
	for i in 0..height as usize {
		for j in 0..width as usize {
			let mut input_line = String::new();
			io::stdin().read_line(&mut input_line).unwrap();
			let inputs = input_line.split(" ").collect::<Vec<_>>();
			let region_id = parse_input!(inputs[0], i32);
			let _type = parse_input!(inputs[1], i32); // 0 (PLAINS), 1 (RIVER), 2 (MOUNTAIN), 3 (POI)
		}
	}

	let mut found = false;
	let mut start = (0, 0);
	let mut desired_connection_index = 0;
	let mut end = (0, 0);

	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let town_count = parse_input!(input_line, i32);
	for i in 0..town_count as usize {
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let inputs = input_line.split(" ").collect::<Vec<_>>();
		let town_id = parse_input!(inputs[0], i32);
		let town_x = parse_input!(inputs[1], i32);
		let town_y = parse_input!(inputs[2], i32);
		let desired_connections = inputs[3].trim().to_string(); // comma-separated town ids e.g. 0,1,2,3
		let desired_connections = if desired_connections != "x" {
			desired_connections
				.split(",")
				.map(|s| s.parse::<i32>().unwrap())
				.collect::<Vec<_>>()
		} else {
			Vec::new()
		};

		if !found && !desired_connections.is_empty() {
			start = (town_x as usize, town_y as usize);
			desired_connection_index = desired_connections[0] as usize;
		} else if found && i == desired_connection_index {
			end = (town_x as usize, town_y as usize);
		}
	}

	// game loop
	loop {
		let mut ink_coord = None;

		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let my_score = parse_input!(input_line, i32);
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		let foe_score = parse_input!(input_line, i32);
		for i in 0..height as usize {
			for j in 0..width as usize {
				let mut input_line = String::new();
				io::stdin().read_line(&mut input_line).unwrap();
				let inputs = input_line.split(" ").collect::<Vec<_>>();
				let tracks_owner = parse_input!(inputs[0], i32);
				let instability = parse_input!(inputs[1], i32); // region inked (destroyed) when this >= 3.
				let inked = parse_input!(inputs[2], i32); // true if region is destroyed.
				let part_of_active_connections = inputs[3].trim().to_string(); // if this cell is part of one or more railway connections, this will be town ids (separated by -) in a list separated by commas. e.g. 0-1,1-2,1-3. "x" otherwise.

				if tracks_owner == foe_id && inked == 0 {
					ink_coord = Some((j, i));
				}
			}
		}

		// Write an action using println!("message...");
		// To debug: eprintln!("Debug message...");

		// AUTOPLACE x1 y1 x2 y2 | PLACE_TRACKS x y | DISRUPT regionId | MESSAGE text
		if let Some((x, y)) = ink_coord {
			println!("DISRUPT {} {}", x, y);
		} else {
			println!("AUTOPLACE {} {} {} {}", start.0, start.1, end.0, end.1);
		}
	}
}
