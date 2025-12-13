use std::collections::{HashMap, VecDeque};

use aocd::*;

fn parse(data: &str) -> (Vec<Vec<usize>>, usize, usize) {
	let mut h = HashMap::<String, usize>::new();

	fn ensure_index(h: &mut HashMap<String, usize>, key: &str) -> usize {
		if let Some(&index) = h.get(key) {
			index
		} else {
			let index = h.len();
			h.insert(key.to_string(), index);
			index
		}
	}

	let data = data.trim().lines().collect::<Vec<&str>>();

	let mut graph = vec![Vec::new(); data.len() + 1];

	for line in data {
		let split: Vec<&str> = line.split(':').collect();

		let from = split[0].trim();
		let to: Vec<&str> = split[1].trim().split_whitespace().collect();

		let from_index = ensure_index(&mut h, from);
		assert!(from_index < graph.len());
		for &t in &to {
			let to_index = ensure_index(&mut h, t);
			assert!(to_index < graph.len());
			graph[from_index].push(to_index);
		}
	}

	let start_index = h.get("you").copied().expect("no 'you' node found");
	let end_index = h.get("out").copied().expect("no 'out' node found");

	(graph, start_index, end_index)
}

fn solve(data: &str) -> (usize, usize) {
	let (graph, start, end) = parse(data);

	let mut p1 = 0;

	let mut q = VecDeque::from([start]);
	while let Some(node) = q.pop_front() {
		for &neighbor in &graph[node] {
			if neighbor == end {
				p1 += 1;
			} else {
				q.push_back(neighbor);
			}
		}
	}

	(p1, 0)
}

#[aocd(2025, 11)]
fn main() {
	let (p1, p2) = solve(&input!());
	println!("part 1:\t{p1}\npart 2:\t{p2}");
}

#[cfg(test)]
mod tests {
	use super::*;

	const TEST_DATA: &str = r#"
aaa: you hhh
you: bbb ccc
bbb: ddd eee
ccc: ddd eee fff
ddd: ggg
eee: out
fff: out
ggg: out
hhh: ccc fff iii
iii: out
"#;

	#[test]
	fn test_example() {
		let expected = (5, 0);
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
}
