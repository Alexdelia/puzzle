mod parse;
mod state;

use aocd::*;

pub use state::State;

type Button = State;
type ButtonJoltageList = Vec<usize>;

fn solve_line(line: &str) -> Result<(usize, usize), String> {
	Ok((0, 0))
}

fn solve(data: &str) -> Result<(usize, usize), String> {
	Ok((0, 0))
}

#[aocd(2025, 10)]
fn main() -> Result<(), String> {
	let (p1, p2) = solve(&input!())?;
	println!("part 1:\t{p1}\npart 2:\t{p2}");
	Ok(())
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
		let expected = (7, 0);
		let got = solve(TEST_DATA).unwrap();
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
			(0, (2, 0)), //
			(1, (3, 0)),
			(2, (2, 0)),
		] {
			let line = TEST_DATA.trim().lines().nth(index).unwrap();
			let got = solve_line(line).unwrap();
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
