use super::{
	Cell, Engine, MAP_AREA, MAX_TURNS, NONE, Score, SimOutput, Solution, Tile, Turn, VOID,
};

const VISITED_WORDS: usize = (MAP_AREA * 4).div_ceil(64);

pub fn simulate_one(engine: &Engine, next: &[Cell], solution: &Solution) -> SimOutput {
	let mut total_score: Score = 0;
	let mut game_length: Turn = 0;
	let mut visited = [0u64; VISITED_WORDS];

	for robot in &engine.robot_list {
		visited.iter_mut().for_each(|word| *word = 0);

		let mut cell = robot.cell;
		let start_arrow = solution.arrow[cell as usize];
		let mut heading: Tile = if start_arrow != NONE {
			start_arrow
		} else {
			robot.heading
		};

		let state = cell as usize * 4 + heading as usize;
		visited[state >> 6] |= 1u64 << (state & 63);

		let mut turn: Turn = 0;
		loop {
			turn += 1;
			cell = next[cell as usize * 4 + heading as usize];

			let mut tile = engine.base[cell as usize];
			if tile == NONE {
				tile = solution.arrow[cell as usize];
			}
			if tile == VOID {
				total_score += turn;
				break;
			}
			if tile != NONE {
				heading = tile;
			}

			let state = cell as usize * 4 + heading as usize;
			let bit = 1u64 << (state & 63);
			if visited[state >> 6] & bit != 0 {
				total_score += turn;
				break;
			}
			visited[state >> 6] |= bit;

			if turn >= MAX_TURNS {
				total_score += turn;
				break;
			}
		}

		if turn > game_length {
			game_length = turn;
		}
	}

	SimOutput {
		score: total_score,
		game_length,
	}
}

pub fn simulate_batch(
	engine: &Engine,
	next: &[Cell],
	generation: &[Solution],
	output_list: &mut [SimOutput],
) {
	for (solution, output) in generation.iter().zip(output_list.iter_mut()) {
		*output = simulate_one(engine, next, solution);
	}
}

#[cfg(test)]
mod tests {
	use super::simulate_one;
	use crate::parse::parse_map;
	use crate::simulation::{LEFT, MAP_WIDTH, RIGHT, Solution, build_next_table};

	const SIMPLE: &str = concat!(
		"###################\n",
		"###################\n",
		"###################\n",
		"###################\n",
		"###R...........####\n",
		"###################\n",
		"###################\n",
		"###################\n",
		"###################\n",
		"###################",
	);

	const OPEN: &str = concat!(
		"R..................\n",
		"...................\n",
		"...................\n",
		"...................\n",
		"...................\n",
		"...................\n",
		"...................\n",
		"...................\n",
		"...................\n",
		"...................",
	);

	#[test]
	fn empty_solution_runs_into_void() {
		let engine = parse_map(SIMPLE).unwrap();
		let next = build_next_table();
		let output = simulate_one(&engine, &next, &Solution::empty());
		assert_eq!(output.score, 12);
		assert_eq!(output.game_length, 12);
	}

	#[test]
	fn toroidal_wraparound_loops_after_one_lap() {
		let engine = parse_map(OPEN).unwrap();
		let next = build_next_table();
		let output = simulate_one(&engine, &next, &Solution::empty());
		assert_eq!(output.score, MAP_WIDTH as u16);
	}

	#[test]
	fn bouncing_arrows_extend_lifetime() {
		let engine = parse_map(SIMPLE).unwrap();
		let next = build_next_table();
		let mut solution = Solution::empty();
		solution.arrow[4 * MAP_WIDTH + 14] = LEFT;
		solution.arrow[4 * MAP_WIDTH + 3] = RIGHT;
		let output = simulate_one(&engine, &next, &solution);
		assert!(output.score > 12);
	}
}
