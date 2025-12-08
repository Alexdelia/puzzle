mod connection;
mod point;

use std::collections::{BinaryHeap, HashSet};

use aocd::*;

use crate::{connection::Connection, point::Point};

fn parse(data: &str) -> Result<Vec<Point>, String> {
	data.trim()
		.lines()
		.map(|line| line.parse::<Point>())
		.collect()
}

fn build_heap(point_list: &[Point]) -> BinaryHeap<Connection> {
	let mut heap = BinaryHeap::new();
	for (i, a) in point_list.iter().enumerate() {
		for b in point_list.iter().skip(i + 1) {
			heap.push(Connection::new(*a, *b));
		}
	}
	heap
}

fn insert_into_circuit(circuit_list: &mut Vec<HashSet<Point>>, connection: &Connection) -> bool {
	let mut a_circuit_index: Option<usize> = None;
	let mut b_circuit_index: Option<usize> = None;

	for (index, circuit) in circuit_list.iter().enumerate() {
		if circuit.contains(&connection.point.0) {
			a_circuit_index = Some(index);
			if b_circuit_index.is_some() {
				break;
			}
		}

		if circuit.contains(&connection.point.1) {
			b_circuit_index = Some(index);
			if a_circuit_index.is_some() {
				break;
			}
		}
	}

	match (a_circuit_index, b_circuit_index) {
		(None, None) => {
			circuit_list.push(HashSet::from([connection.point.0, connection.point.1]));
		}
		(Some(a_index), None) => {
			circuit_list[a_index].insert(connection.point.1);
		}
		(None, Some(b_index)) => {
			circuit_list[b_index].insert(connection.point.0);
		}
		(Some(a_index), Some(b_index)) => {
			if a_index == b_index {
				return false;
			}

			let b_circuit = circuit_list[b_index].drain().collect::<HashSet<Point>>();
			circuit_list[a_index].extend(b_circuit);
			circuit_list.remove(b_index);
		}
	}

	true
}

fn build_circuit(
	mut heap: BinaryHeap<Connection>,
	iteration_count: usize,
) -> (Vec<HashSet<Point>>, Connection) {
	let mut circuit_list: Vec<HashSet<Point>> = Vec::new();

	for _ in 0..iteration_count {
		let Some(connection) = heap.pop() else {
			panic!("Heap exhausted before reaching iteration count");
		};

		insert_into_circuit(&mut circuit_list, &connection);
	}
	let p1_circuit_list = circuit_list.clone();

	let mut last_connection = heap.peek().expect("heap exhausted").to_owned();
	while let Some(connection) = heap.pop() {
		if insert_into_circuit(&mut circuit_list, &connection) {
			last_connection = connection;
		}
	}

	(p1_circuit_list, last_connection)
}

fn solve(data: &str, iteration_count: usize) -> Result<(usize, usize), String> {
	let point_list = parse(data)?;

	let heap = build_heap(&point_list);

	let (circuit_list, last_connection) = build_circuit(heap, iteration_count);

	let mut sorted_circuit_len = circuit_list.iter().map(|c| c.len()).collect::<Vec<usize>>();
	sorted_circuit_len.sort();
	let p1 = sorted_circuit_len[sorted_circuit_len.len() - 3..sorted_circuit_len.len()]
		.iter()
		.product();

	let p2 = last_connection.point.0.x as usize * last_connection.point.1.x as usize;

	Ok((p1, p2))
}

const ITERATION_COUNT: usize = 1000;

#[aocd(2025, 8)]
fn main() -> Result<(), String> {
	let (p1, p2) = solve(&input!(), ITERATION_COUNT)?;
	println!("part 1:\t{p1}\npart 2:\t{p2}");
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	const TEST_DATA: &str = r#"
162,817,812
57,618,57
906,360,560
592,479,940
352,342,300
466,668,158
542,29,236
431,825,988
739,650,466
52,470,668
216,146,977
819,987,18
117,168,530
805,96,715
346,949,466
970,615,88
941,993,340
862,61,35
984,92,344
425,690,689
"#;
	const TEST_ITERATION_COUNT: usize = 10;

	#[test]
	fn test_example() {
		let expected = (40, 25272);
		let got = solve(TEST_DATA, TEST_ITERATION_COUNT).unwrap();
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
