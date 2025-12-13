mod combination;
mod node;
mod parse;
mod state;

use std::{
	collections::{BinaryHeap, HashSet, VecDeque},
	time::SystemTime,
};

use aocd::*;

pub use node::{JoltageNode, LightNode};
pub use state::State;

use crate::{
	combination::{CombinationCache, get_joltage_button_press_combination},
	state::click_button,
};

type StateButton = State;
type JoltageList = u128;
type JoltageUnit = u8;
type JoltageButton = Vec<usize>;

type Distance = u16;

fn solve_line_p1(state_goal: State, button_list: &[StateButton]) -> Distance {
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

#[inline]
fn get_joltage_unit(joltage_list: JoltageList, index: usize) -> JoltageUnit {
	((joltage_list >> (index * 8)) & 0xFF) as JoltageUnit
}

#[inline]
fn set_joltage_unit(joltage_list: &mut JoltageList, index: usize, value: JoltageUnit) {
	*joltage_list &= !(0xFF << (index * 8));
	*joltage_list |= (value as JoltageList) << (index * 8);
}

fn cleanse_joltage_state(
	active_joltage: &[usize],
	joltage_list: JoltageList,
	joltage_button_list: &[JoltageButton; 16],
) -> Option<(Vec<usize>, [JoltageButton; 16])> {
	let mut unavailable_joltage_indices = [false; 16];
	let mut delete_count = 0;
	for joltage_index in active_joltage.iter().copied() {
		let joltage = get_joltage_unit(joltage_list, joltage_index);

		if joltage == 0 {
			delete_count += 1;

			let button_indices = &joltage_button_list[joltage_index];
			for &button_index in button_indices {
				unavailable_joltage_indices[button_index] = true;
			}
		}
	}

	let mut next_active_joltage = Vec::with_capacity(active_joltage.len() - delete_count);
	let mut next_joltage_button_list = joltage_button_list.clone();

	for joltage_index in active_joltage.iter().copied() {
		let joltage = get_joltage_unit(joltage_list, joltage_index);
		if joltage == 0 {
			continue;
		}

		next_joltage_button_list[joltage_index] = joltage_button_list[joltage_index]
			.iter()
			.copied()
			.filter(|button_index| !unavailable_joltage_indices[*button_index])
			.collect();

		if next_joltage_button_list[joltage_index].is_empty() {
			return None;
		}

		next_active_joltage.push(joltage_index);
	}

	Some((next_active_joltage, next_joltage_button_list))
}

fn solve_line_p2(
	combination_cache: &mut CombinationCache,
	joltage_goal: JoltageList,
	button_list: Vec<JoltageButton>,
) -> Distance {
	let mut initial_active_joltage = Vec::new();
	for i in 0..16 {
		if get_joltage_unit(joltage_goal, i) > 0 {
			initial_active_joltage.push(i);
		}
	}

	let mut initial_joltage_button_list: [JoltageButton; 16] = Default::default();
	for (button_index, button) in button_list.iter().enumerate() {
		for &joltage_index in button {
			initial_joltage_button_list[joltage_index].push(button_index);
		}
	}

	let mut q = VecDeque::<JoltageNode>::from([JoltageNode {
		active_joltage: initial_active_joltage,
		joltage_list: joltage_goal,
		joltage_button_list: initial_joltage_button_list,
		dist: 0,
	}]);

	let mut min = Distance::MAX;

	while let Some(JoltageNode {
		active_joltage,
		joltage_list,
		joltage_button_list,
		dist,
	}) = q.pop_front()
	{
		let current_joltage_index = *active_joltage
			.iter()
			.min_by_key(|joltage_index| {
				let joltage_index = **joltage_index;
				(
					joltage_button_list[joltage_index].len(),
					-(get_joltage_unit(joltage_list, joltage_index) as i16),
				)
			})
			.expect("active_joltage empty");

		let joltage_unit = get_joltage_unit(joltage_list, current_joltage_index);

		let dist = dist + joltage_unit as Distance;
		if dist > min {
			continue;
		}

		let current_joltage_button_list = &joltage_button_list[current_joltage_index];

		let combination_list = get_joltage_button_press_combination(
			combination_cache,
			joltage_unit,
			current_joltage_button_list.len(),
		);

		'combination: for combination in combination_list {
			let mut next_joltage_list = joltage_list;
			for (button_index, &press_count) in combination.iter().enumerate() {
				if press_count == 0 {
					continue;
				}

				let button = &button_list[current_joltage_button_list[button_index]];
				for &joltage_index in button {
					let joltage_at_index = get_joltage_unit(next_joltage_list, joltage_index);

					if joltage_at_index < press_count {
						continue 'combination;
					}

					set_joltage_unit(
						&mut next_joltage_list,
						joltage_index,
						joltage_at_index - press_count,
					);
				}
			}

			if let Some((active_joltage, joltage_button_list)) =
				cleanse_joltage_state(&active_joltage, next_joltage_list, &joltage_button_list)
			{
				if next_joltage_list == 0 {
					if dist < min {
						print!("\x1b[2K\r{dist}");
						min = dist;
					}
				} else {
					q.push_front(JoltageNode {
						active_joltage,
						joltage_list: next_joltage_list,
						joltage_button_list,
						dist,
					});
				}
			}
		}
	}

	assert!(
		min != Distance::MAX,
		"did not find a solution for part 2 joltage_goal='{joltage_goal:b}'"
	);

	println!();

	min
}

fn solve_line(line: &str, combination_cache: &mut CombinationCache) -> (usize, usize) {
	let (state_goal, state_button_list, joltage_goal, joltage_button_list) =
		parse::parse_line(line);

	let p1 = solve_line_p1(state_goal, &state_button_list) as usize;
	let p2 = solve_line_p2(combination_cache, joltage_goal, joltage_button_list) as usize;

	(p1, p2)
}

fn solve(data: &str) -> (usize, usize) {
	let mut combination_cache = CombinationCache::new();

	let mut p1 = 0;
	let mut p2 = 0;

	let start = SystemTime::now();
	let mut lines = data.trim().lines().collect::<Vec<&str>>();
	lines.sort_by_key(|a| a.len());
	let len = lines.len();

	for (i, line) in lines.iter().enumerate() {
		println!("===\n{line}");

		let start_line = SystemTime::now();

		let (line_p1, line_p2) = solve_line(line, &mut combination_cache);
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
