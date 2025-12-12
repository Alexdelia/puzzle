use crate::{Button, ButtonJoltageList, State};

pub fn parse_line(line: &str) -> (State, Vec<Button>, ButtonJoltageList) {
	let split: Vec<&str> = line.split_whitespace().collect();
	assert!(split.len() >= 3, "line '{line}' does not have enough parts");

	let state = parse_state(split[0]);

	let button_list = split[1..split.len() - 1]
		.iter()
		.map(|part| parse_button(part))
		.collect::<Vec<Button>>();

	let joltage_list = parse_joltage_list(split[split.len() - 1]);

	(state, button_list, joltage_list)
}

fn parse_state(s: &str) -> State {
	assert!(
		s.starts_with('[') && s.ends_with(']'),
		"`State` string '{s}' is not enclosed with '[' and ']'"
	);

	let s = &s[1..s.len() - 1];

	assert!(s.len() <= 16, "`State` string '{s}' is too long");

	let mut state: State = 0;

	for (i, c) in s.chars().enumerate() {
		match c {
			'#' => state |= 1 << i,
			'.' => (),
			_ => panic!("found invalid character '{c}' in `State` string '{s}'"),
		}
	}

	state
}

fn parse_button(s: &str) -> Button {
	assert!(
		s.starts_with('(') && s.ends_with(')'),
		"`Button` string '{s}' is not enclosed with '(' and ')'"
	);

	let s = &s[1..s.len() - 1];

	let mut button: Button = 0;

	for part in s.split(',') {
		let index: usize = part
			.trim()
			.parse()
			.expect(&format!("invalid index '{part}' in `Button` string '{s}'"));

		assert!(
			index <= 16,
			"index {index} in `Button` string '{s}' is out of range"
		);

		button |= 1 << index;
	}

	button
}

fn parse_joltage_list(s: &str) -> ButtonJoltageList {
	assert!(
		s.starts_with('{') && s.ends_with('}'),
		"`ButtonJoltageList` string '{s}' is not enclosed with '{{' and '}}'"
	);

	let s = &s[1..s.len() - 1];

	s.split(',')
		.map(|part| {
			part.trim().parse().expect(&format!(
				"invalid joltage '{part}' in `ButtonJoltageList` string '{s}'"
			))
		})
		.collect::<ButtonJoltageList>()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_example() {
		for (line, expected) in [
			(
				"[.##.] (3) (1,3) (2) (2,3) (0,2) (0,1) {3,5,4,7}",
				(
					0b0110,
					vec![0b1000, 0b1010, 0b0100, 0b1100, 0b0101, 0b0011],
					vec![3, 5, 4, 7],
				),
			),
			(
				"[...#.] (0,2,3,4) (2,3) (0,4) (0,1,2) (1,2,3,4) {7,5,12,7,2}",
				(
					0b01000,
					vec![0b11101, 0b01100, 0b10001, 0b00111, 0b11110],
					vec![7, 5, 12, 7, 2],
				),
			),
			(
				"[.###.#] (0,1,2,3,4) (0,3,4) (0,1,2,4,5) (1,2) {10,11,11,5,10,5}",
				(
					0b101110,
					vec![0b011111, 0b011001, 0b110111, 0b000110],
					vec![10, 11, 11, 5, 10, 5],
				),
			),
		] {
			let got = parse_line(line);
			assert_eq!(expected, got, "parsing line '{line}'");
		}
	}

	#[test]
	fn test_parse_state_zero() {
		assert_eq!(parse_state("[.]"), 0);
		assert_eq!(parse_state("[..]"), 0);
		assert_eq!(parse_state("[...]"), 0);
		assert_eq!(parse_state("[..........]"), 0);
	}

	#[test]
	fn test_parse_state_one() {
		assert_eq!(parse_state("[#]"), 1);
		assert_eq!(parse_state("[#.]"), 1);
		assert_eq!(parse_state("[#..]"), 1);
		assert_eq!(parse_state("[#.........]"), 1);
	}

	#[test]
	fn test_parse_state_increasing() {
		assert_eq!(parse_state("[.#]"), 2);
		assert_eq!(parse_state("[..#]"), 4);
		assert_eq!(parse_state("[...#]"), 8);
		assert_eq!(parse_state("[....#]"), 16);
		assert_eq!(parse_state("[.....#]"), 32);
		assert_eq!(parse_state("[......#]"), 64);
		assert_eq!(parse_state("[.......#]"), 128);
		assert_eq!(parse_state("[........#]"), 256);
		assert_eq!(parse_state("[.........#]"), 512);
	}

	#[test]
	fn test_parse_state_mixed() {
		assert_eq!(parse_state("[.#.#.]"), 0b01010);
		assert_eq!(parse_state("[##..#]"), 0b10011);
		assert_eq!(parse_state("[..###.]"), 0b011100);
		assert_eq!(parse_state("[#.#.#.#]"), 0b1010101);
		assert_eq!(parse_state("[####....]"), 0b00001111);
		assert_eq!(parse_state("[..#.##.#.###.#]"), 0b10111010110100);
		assert_eq!(parse_state("[#..#.#.###.#.##.]"), 0b0110101110101001);
	}

	#[test]
	fn test_parse_state_empty() {
		assert_eq!(parse_state("[]"), 0);
	}
}
