pub type State = u16;

pub fn parse_state(s: &str) -> State {
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

#[cfg(test)]
mod tests {
	use super::*;

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
