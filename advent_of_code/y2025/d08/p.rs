mod connection;
mod point;

use std::collections::BinaryHeap;

use aocd::*;

use crate::{connection::Connection, point::Point};

fn parse(data: String) -> Result<Vec<Point>, String> {
	data.trim()
		.lines()
		.map(|line| line.parse::<Point>())
		.collect()
}

fn solve(data: String) -> Result<(i32, i32), String> {
	let point_list = parse(data)?;

	let mut heap = {
		let mut h = BinaryHeap::new();
		for (i, a) in point_list.iter().enumerate() {
			for b in point_list.iter().skip(i + 1) {
				h.push(Connection::new(*a, *b));
			}
		}
		h
	};

	dbg!(&heap.len());
	dbg!(heap.pop());
	dbg!(heap.pop());
	dbg!(heap.pop());

	Ok((0, 0))
}

#[aocd(2025, 8)]
fn main() -> Result<(), String> {
	let (p1, p2) = solve(input!())?;
	println!("part 1:\t{p1}\npart 2:\t{p2}");
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_example() {
		let data = r#"
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
		let expected = (40, 0);
		let got = solve(data.to_string()).unwrap();
		assert_eq!(
			got.0, expected.0,
			"part 1\nexpected {}\ngot {}",
			expected.0, got.0
		);
		assert_eq!(
			got.1, expected.1,
			"part 2\nexpected {}\ngot\n{}",
			expected.1, got.1
		);
	}
}
