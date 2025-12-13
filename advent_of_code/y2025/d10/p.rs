mod node;
mod parse;
mod state;

use std::{
	collections::{BinaryHeap, HashSet},
	time::SystemTime,
};

use aocd::*;

pub use node::LightNode;
pub use state::State;

use crate::state::click_button;

type StateButton = State;
type Joltage = u8;
type JoltageButton = Vec<usize>;

type Distance = usize;

fn solve_line_p1(state_goal: State, button_list: &[StateButton]) -> usize {
	let mut cache = HashSet::<State>::new();
	let mut q = BinaryHeap::<LightNode>::from([LightNode {
		state: State::default(),
		dist: 0,
	}]);

	while let Some(LightNode { state, dist }) = q.pop() {
		if state == state_goal {
			return dist;
		}

		let dist = dist + 1;
		for button in button_list {
			let next = click_button(state, *button);

			if !cache.insert(next) {
				continue;
			}

			q.push(LightNode { state: next, dist });
		}
	}

	unreachable!("did not find a solution for part 1 goal='{state_goal:016b}'");
}

// for all joltage/button combinations
//   ex joltage x = 4 with 2 buttons that affect joltage x
//     [0,4]
//     [1,3]
//     [2,2]
//     [3,1]
//     [4,0]

// press each combination of buttons that do not exceed the goal joltage

// increase step by joltage x

// need:
// - Vec<(Joltage, Vec<usize>)>  -- list of (joltage, button indices that affect that joltage)
// - Vec<Vec<usize>> -- list of button with each joltage index affected by that button

// when joltage reaches goal, remove all affectable buttons for all other joltage

fn is_end(joltage: &[(Joltage, Vec<usize>)]) -> Result<bool, ()> {
	for (j, button_indices) in joltage {
		if *j != 0 {
			return if button_indices.is_empty() {
				Err(())
			} else {
				Ok(false)
			};
		}
	}

	Ok(true)
}

fn solve_line_p2(joltage_goal: &[Joltage], button_list: &[JoltageButton]) -> usize {
	let mut remaining_joltage: Vec<(Joltage, Vec<usize>)> =
		joltage_goal.iter().map(|&j| (j, Vec::new())).collect();
	for (button_index, button) in button_list.iter().enumerate() {
		for &joltage_index in button {
			remaining_joltage[joltage_index].1.push(button_index);
		}
	}

	unreachable!("did not find a solution for part 2 goal='{joltage_goal:?}'");
}

fn solve_line(line: &str) -> (usize, usize) {
	let (state_goal, state_button_list, joltage_goal, joltage_button_list) =
		parse::parse_line(line);

	let p1 = solve_line_p1(state_goal, &state_button_list);
	let p2 = solve_line_p2(&joltage_goal, &joltage_button_list);

	(p1, p2)
}

fn solve(data: &str) -> (usize, usize) {
	let mut p1 = 0;
	let mut p2 = 0;

	let start = SystemTime::now();
	let len = data.trim().lines().count();

	for (i, line) in data.trim().lines().enumerate() {
		let (line_p1, line_p2) = solve_line(line);
		p1 += line_p1;
		p2 += line_p2;

		let elapsed = start
			.elapsed()
			.expect("failed to get elapsed time")
			.as_secs_f32();
		let eta = elapsed / (i as f32 + 1.0) * (len as f32 - i as f32 - 1.0);
		println!(
			"{i}/{len} {percent}%\tETA: {eta:.2}s",
			percent = (i as f32 + 1.0) / (len as f32) * 100.0
		);
	}

	(p1, p2)
}

#[aocd(2025, 10)]
fn main() {
	let (p1, p2) = solve(&input!());
	println!("part 1:\t{p1}\npart 2:\t{p2}");
}

#[cfg(test)]
mod tests {
	use super::*;

	const TEST_DATA: &str = r#"
[.##.] (3) (1,3) (2) (2,3) (0,2) (0,1) {3,5,4,7}
[...#.] (0,2,3,4) (2,3) (0,4) (0,1,2) (1,2,3,4) {7,5,12,7,2}
[.###.#] (0,1,2,3,4) (0,3,4) (0,1,2,4,5) (1,2) {10,11,11,5,10,5}
"#;

	#[test]
	fn test_example() {
		let expected = (7, 33);
		let got = solve(TEST_DATA);
		assert_eq!(
			expected.0, got.0,
			"part 1\nexpected {}\ngot {}",
			expected.0, got.0
		);
		assert_eq!(
			expected.1, got.1,
			"part 2\nexpected {}\ngot\n{}",
			expected.1, got.1
		);
	}

	#[test]
	fn test_solve_line() {
		for (index, expected) in [
			(0, (2, 10)), //
			(1, (3, 12)),
			(2, (2, 11)),
		] {
			let line = TEST_DATA.trim().lines().nth(index).unwrap();
			let got = solve_line(line);
			assert_eq!(
				expected.0, got.0,
				"part 1: line[{index}]='{line}'\nexpected {}\ngot {}",
				expected.0, got.0
			);
			assert_eq!(
				expected.1, got.1,
				"part 2: line[{index}]='{line}'\nexpected {}\ngot\n{}",
				expected.1, got.1
			);
		}
	}
}
