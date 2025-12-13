mod combination;
mod node;
mod parse;
mod state;

use std::{
	collections::{BinaryHeap, HashMap, HashSet},
	time::SystemTime,
};

use aocd::*;

pub use node::LightNode;
pub use state::State;

use crate::{
	combination::{first_joltage_button_press_combination, next_joltage_button_press_combination},
	node::JoltageNode,
	state::click_button,
};

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

fn cleanse_joltage_state(
	state: HashMap<usize, (Joltage, Vec<usize>)>,
) -> Option<HashMap<usize, (Joltage, Vec<usize>)>> {
	let mut unavailable_button_indices = HashSet::new();
	for (_, (joltage, button_indices)) in state.iter() {
		if *joltage == 0 {
			for &button_index in button_indices {
				unavailable_button_indices.insert(button_index);
			}
		}
	}

	let mut next_state = HashMap::with_capacity(state.len());
	for (joltage_index, (joltage, button_indices)) in state.into_iter() {
		if joltage == 0 {
			continue;
		}

		let filtered_button_indices: Vec<usize> = button_indices
			.into_iter()
			.filter(|button_index| !unavailable_button_indices.contains(button_index))
			.collect();

		if filtered_button_indices.is_empty() {
			return None;
		}

		next_state.insert(joltage_index, (joltage, filtered_button_indices));
	}

	Some(next_state)
}

fn solve_line_p2(joltage_goal: &[Joltage], button_list: &[JoltageButton]) -> usize {
	let mut remaining_joltage: Vec<(Joltage, Vec<usize>)> =
		joltage_goal.iter().map(|&j| (j, Vec::new())).collect();
	for (button_index, button) in button_list.iter().enumerate() {
		for &joltage_index in button {
			remaining_joltage[joltage_index].1.push(button_index);
		}
	}
	let remaining_joltage = HashMap::<usize, (Joltage, Vec<usize>)>::from_iter(
		remaining_joltage.into_iter().enumerate(),
	);
	let remaining_joltage =
		cleanse_joltage_state(remaining_joltage).expect("initial joltage state invalid");

	// let mut cache = HashMap::<Vec<(usize, (Joltage, Vec<usize>))>, Distance>::new();
	let mut q = BinaryHeap::<JoltageNode>::from([JoltageNode {
		state: remaining_joltage,
		dist: 0,
	}]);

	let mut min = usize::MAX;

	while let Some(JoltageNode { state, dist }) = q.pop() {
		for i in state.keys() {
			let dist = dist + state[i].0 as usize;
			if dist > min {
				continue;
			}

			let mut combination = first_joltage_button_press_combination(&state[i]);
			loop {
				let mut next_state = state.clone();
				let mut possible = true;
				'button: for (button_index, &press_count) in combination.iter().enumerate() {
					if press_count == 0 {
						continue;
					}

					let button = &button_list[state[i].1[button_index]];
					for &joltage_index in button {
						let joltage_at_index = next_state
							.get_mut(&joltage_index)
							.expect("joltage index missing");
						if joltage_at_index.0 < press_count as Joltage {
							possible = false;
							break 'button;
						}
						joltage_at_index.0 -= press_count as Joltage;
					}
				}

				if !possible {
					if !next_joltage_button_press_combination(&mut combination) {
						break;
					}
					continue;
				}

				if let Some(next_state) = cleanse_joltage_state(next_state) {
					if next_state.is_empty() {
						if dist < min {
							print!("\x1b[2K\r{dist}");
						}
						min = min.min(dist);
					}

					/*
					let mut cache_key =
						Vec::from_iter(next_state.iter().map(|(k, v)| (*k, v.clone())));
					cache_key.sort_by(|a, b| a.0.cmp(&b.0));
					if let Some(&cached_dist) = cache.get(&cache_key) {
						if dist > cached_dist {
							if !next_joltage_button_press_combination(&mut combination) {
								break;
							}
							continue;
						}
					} else {
						cache.insert(cache_key, dist);
					}
					*/

					q.push(JoltageNode {
						state: next_state,
						dist,
					});
				}

				if !next_joltage_button_press_combination(&mut combination) {
					break;
				}
			}
		}
	}

	println!();

	min
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
	let mut lines = data.trim().lines().collect::<Vec<&str>>();
	lines.sort_by(|a, b| a.len().cmp(&b.len()));
	let len = lines.len();

	for (i, line) in lines.iter().enumerate() {
		println!("===\n{line}");

		let start_line = SystemTime::now();

		let (line_p1, line_p2) = solve_line(line);
		p1 += line_p1;
		p2 += line_p2;

		let elapsed = start
			.elapsed()
			.expect("failed to get elapsed time")
			.as_secs_f32();
		let eta = elapsed / (i as f32 + 1.0) * (len as f32 - i as f32 - 1.0);
		let elapsed_line = start_line
			.elapsed()
			.expect("failed to get elapsed time for line")
			.as_secs_f32();
		println!(
			"{i}/{len} {percent:.2}%\t{elapsed_line:.2}s\t{elapsed:.2}s\tETA: {eta:.2}s",
			i = i + 1,
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
