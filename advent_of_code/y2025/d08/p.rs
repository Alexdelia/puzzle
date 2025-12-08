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

fn build_heap(points: &[Point]) -> BinaryHeap<Connection> {
	let mut heap = BinaryHeap::new();
	for (i, a) in points.iter().enumerate() {
		for b in points.iter().skip(i + 1) {
			heap.push(Connection::new(*a, *b));
		}
	}
	heap
}

fn insert_into_circuit(circuit_list: &mut Vec<HashSet<Point>>, connection: &Connection) {
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
				return;
			}

			let b_circuit = circuit_list.remove(b_index);
			circuit_list[a_index].extend(b_circuit);
		}
	}
}

fn build_circuit(mut heap: BinaryHeap<Connection>, iteration_count: usize) -> Vec<HashSet<Point>> {
	let mut circuit_list: Vec<HashSet<Point>> = Vec::new();

	for _ in 0..iteration_count {
		let Some(connection) = heap.pop() else {
			panic!("Heap exhausted before reaching iteration count");
		};

		insert_into_circuit(&mut circuit_list, &connection);
	}

	circuit_list
}

fn solve(data: &str, iteration_count: usize) -> Result<(usize, i32), String> {
	let point_list = parse(data)?;

	let heap = build_heap(&point_list);

	let circuit_list = build_circuit(heap, iteration_count);

	dbg!(&circuit_list);

	let p1 = circuit_list.iter().map(|c| c.len()).product();

	Ok((p1, 0))
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
		let expected = (40, 0);
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
