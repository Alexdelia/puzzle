mod node;
mod parse;
mod state;

use std::{
	collections::{BinaryHeap, HashSet},
	time::SystemTime,
};

use aocd::*;

pub use node::Node;
pub use state::State;

use crate::state::click_button;

type StateButton = State;
type Joltage = u8;
type JoltageButton = Vec<usize>;

type Distance = usize;

fn solve_line_p1(state_goal: State, button_list: &[StateButton]) -> usize {
	let mut cache = HashSet::<State>::new();
	let mut q = BinaryHeap::<Node<State>>::from([Node {
		t: State::default(),
		dist: 0,
		priority: 0,
	}]);

	while let Some(Node {
		t: state,
		dist,
		priority: _,
	}) = q.pop()
	{
		if state == state_goal {
			return dist;
		}

		let dist = dist + 1;
		for button in button_list {
			let next = click_button(state, *button);

			if !cache.insert(next) {
				continue;
			}

			q.push(Node {
				t: next,
				dist,
				priority: 0,
			});
		}
	}

	unreachable!("did not find a solution for part 1 goal='{state_goal:016b}'");
}

fn solve_line_p2(joltage_goal: &[Joltage], button_list: &[JoltageButton]) -> usize {
	let mut cache = HashSet::<Vec<Joltage>>::new();
	let mut q = BinaryHeap::<Node<Vec<Joltage>>>::from([Node {
		t: vec![0; joltage_goal.len()],
		dist: 0,
		priority: 0,
	}]);

	let max_dist = joltage_goal.iter().map(|&j| j as usize).sum::<usize>();
	let mut max_reached_dist = 0;

	let start = SystemTime::now();

	while let Some(Node {
		t: joltage_list,
		dist,
		priority,
	}) = q.pop()
	{
		if dist > max_reached_dist {
			let elapsed = start
				.elapsed()
				.expect("failed to get elapsed time")
				.as_secs_f32();
			println!("{dist} / {max_dist}\t{elapsed}s");
			max_reached_dist = dist;
		}

		if joltage_list == joltage_goal {
			return dist;
		}

		let dist = dist + 1;
		'button: for button in button_list {
			let mut next = joltage_list.clone();
			let mut next_priority = priority;

			for &index in button {
				if next[index] >= joltage_goal[index] || next[index] >= 255 {
					continue 'button;
				}

				next[index] += 1;
				next_priority += 1;
			}

			if !cache.insert(next.clone()) {
				continue;
			}

			q.push(Node {
				t: next,
				dist,
				priority: next_priority,
			});
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
	data.trim()
		.lines()
		.map(solve_line)
		.fold((0, 0), |(acc1, acc2), (p1, p2)| (acc1 + p1, acc2 + p2))
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
