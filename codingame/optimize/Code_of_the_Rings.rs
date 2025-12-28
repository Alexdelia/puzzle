use std::io;

const STRIP_SIZE: usize = 30;

type Strip = [char; STRIP_SIZE];

const EMPTY_RUNE: char = ('A' as u8 - 1) as char;

type RollDist = i8;
type MoveDist = i8;

fn roll_dist(from: char, to: char) -> RollDist {
	let from = if from == ' ' { EMPTY_RUNE } else { from };
	let to = if to == ' ' { EMPTY_RUNE } else { to };

	let from = from as RollDist;
	let to = to as RollDist;

	(to - from + 27 + 13) % 27 - 13
}

fn move_dist(from: usize, to: usize) -> MoveDist {
	let forward = (to + STRIP_SIZE - from) % STRIP_SIZE;
	let backward = (from + STRIP_SIZE - to) % STRIP_SIZE;

	if forward <= backward {
		forward as MoveDist
	} else {
		-(backward as MoveDist)
	}
}

fn find_best_dist(s: &Strip, index: usize, to: char) -> (usize, MoveDist, RollDist) {
	let mut best = (
		u8::MAX,
		(usize::default(), MoveDist::default(), RollDist::default()),
	);

	for (i, &rune) in s.iter().enumerate() {
		let d_move = move_dist(index, i);
		let d_roll = roll_dist(rune, to);

		let abs_d_move = d_move.abs() as u8;
		let abs_d_roll = d_roll.abs() as u8;
		let dist = abs_d_move + abs_d_roll;

		if dist < best.0 {
			best = (dist, (i, d_move, d_roll));
		} else if dist == best.0 {
			let best_abs_d_roll = best.1.1.abs() as u8;
			if abs_d_roll < best_abs_d_roll {
				best = (dist, (i, d_move, d_roll));
			}
		}
	}

	best.1
}

fn get_move(d: MoveDist) -> String {
	let mut buf = String::new();

	if d < 0 {
		for _ in 0..d.abs() {
			buf.push('<');
		}
	} else if d > 0 {
		for _ in 0..d {
			buf.push('>');
		}
	}

	buf
}

fn get_roll(d: RollDist) -> String {
	let mut buf = String::new();

	if d < 0 {
		for _ in 0..d.abs() {
			buf.push('-');
		}
	} else if d > 0 {
		for _ in 0..d {
			buf.push('+');
		}
	}

	buf
}

fn main() {
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let magic_phrase = input_line.trim_matches('\n').to_string();

	let mut buf = String::new();

	let mut s: Strip = [EMPTY_RUNE; STRIP_SIZE];
	let mut index: usize = 0;

	for c in magic_phrase.chars() {
		let (new_index, d_move, d_roll) = find_best_dist(&s, index, c);

		buf.push_str(&get_move(d_move));
		buf.push_str(&get_roll(d_roll));

		buf.push('.');

		index = new_index;
		s[index] = c;
	}

	eprintln!("{len}", len = buf.len());
	println!("{buf}");
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_roll_dist() {
		assert_eq!(roll_dist('A', 'C'), 2);
		assert_eq!(roll_dist('C', 'A'), -2);
		assert_eq!(roll_dist(' ', 'A'), 1);
		assert_eq!(roll_dist('A', ' '), -1);
		assert_eq!(roll_dist(EMPTY_RUNE, 'A'), 1);
		assert_eq!(roll_dist('Z', 'A'), 2);
		assert_eq!(roll_dist('A', 'Z'), -2);
		assert_eq!(roll_dist(' ', ' '), 0);
		assert_eq!(roll_dist(EMPTY_RUNE, EMPTY_RUNE), 0);
		assert_eq!(roll_dist(EMPTY_RUNE, ' '), 0);
		assert_eq!(roll_dist('Z', 'C'), 4);
	}

	#[test]
	fn test_move_dist() {
		assert_eq!(move_dist(0, 5), 5);
		assert_eq!(move_dist(5, 0), -5);
		assert_eq!(move_dist(0, 29), -1);
		assert_eq!(move_dist(29, 0), 1);
		assert_eq!(move_dist(15, 25), 10);
		assert_eq!(move_dist(25, 15), -10);
		assert_eq!(move_dist(0, 14), 14);
		assert_eq!(move_dist(14, 0), -14);
		assert_eq!(move_dist(0, 16), -14);
		assert_eq!(move_dist(16, 0), 14);
	}
}
