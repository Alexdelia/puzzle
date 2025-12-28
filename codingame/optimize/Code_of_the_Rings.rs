use std::io;

const EMPTY_RUNE: char = ('A' as u8 - 1) as char;

fn dist(from: char, to: char) -> i8 {
	let from = if from == ' ' { EMPTY_RUNE } else { from };
	let to = if to == ' ' { EMPTY_RUNE } else { to };

	dbg!(from, to);
	let from = from as i8;
	let to = to as i8;
	dbg!(from, to);

	(to - from + 27 + 13) % 27 - 13
}

fn main() {
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let magic_phrase = input_line.trim_matches('\n').to_string();

	let mut buf = String::new();

	let mut rune = EMPTY_RUNE;
	for c in magic_phrase.chars() {
		let d = dist(rune, c);

		if d < 0 {
			for _ in 0..d.abs() {
				buf.push('-');
			}
		} else if d > 0 {
			for _ in 0..d {
				buf.push('+');
			}
		}

		buf.push('.');

		rune = c;
	}

	println!("{buf}");
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_dist() {
		assert_eq!(dist('A', 'C'), 2);
		assert_eq!(dist('C', 'A'), -2);
		assert_eq!(dist(' ', 'A'), 1);
		assert_eq!(dist('A', ' '), -1);
		assert_eq!(dist(EMPTY_RUNE, 'A'), 1);
		assert_eq!(dist('Z', 'A'), 2);
		assert_eq!(dist('A', 'Z'), -2);
		assert_eq!(dist(' ', ' '), 0);
		assert_eq!(dist(EMPTY_RUNE, EMPTY_RUNE), 0);
		assert_eq!(dist(EMPTY_RUNE, ' '), 0);
		assert_eq!(dist('Z', 'C'), 4);
	}
}
