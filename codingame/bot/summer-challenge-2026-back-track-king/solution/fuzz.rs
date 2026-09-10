use std::io::{BufRead, Write};

struct Reader {
	lines: std::io::Lines<std::io::StdinLock<'static>>,
	words: std::collections::VecDeque<String>,
	digest: u64,
}

impl Reader {
	fn new() -> Self {
		Reader {
			lines: std::io::stdin().lock().lines(),
			words: Default::default(),
			digest: 0xcbf2_9ce4_8422_2325,
		}
	}

	fn word(&mut self) -> Option<String> {
		while self.words.is_empty() {
			let line = self.lines.next()?.ok()?;
			for byte in line.bytes() {
				self.digest ^= byte as u64;
				self.digest = self.digest.wrapping_mul(0x0000_0100_0000_01b3);
			}
			self.words
				.extend(line.split_ascii_whitespace().map(str::to_string));
		}
		self.words.pop_front()
	}

	fn number(&mut self) -> Option<i64> {
		self.word()?.parse().ok()
	}
}

struct Rng(u64);

impl Rng {
	fn next(&mut self) -> u64 {
		self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
		let mut z = self.0;
		z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
		z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
		z ^ (z >> 31)
	}

	fn below(&mut self, n: usize) -> usize {
		(self.next() % n.max(1) as u64) as usize
	}

	fn chance(&mut self, percent: u64) -> bool {
		self.next() % 100 < percent
	}
}

fn main() {
	let args: Vec<String> = std::env::args().collect();
	let evil = args.iter().any(|arg| arg == "--evil");
	let die_after: Option<u64> = args
		.iter()
		.position(|arg| arg == "--die")
		.and_then(|at| args.get(at + 1))
		.and_then(|turns| turns.parse().ok());
	let mut input = Reader::new();
	let stdout = std::io::stdout();
	let mut stdout = stdout.lock();

	let me = input.number().unwrap() as u64;
	let width = input.number().unwrap() as i32;
	let height = input.number().unwrap() as i32;
	let cells = (width * height) as usize;
	let mut region = vec![0usize; cells];
	for cell in 0..cells {
		region[cell] = input.number().unwrap() as usize;
		input.number().unwrap();
	}
	let regions = region.iter().copied().max().unwrap_or(0) + 1;
	let town_count = input.number().unwrap() as usize;
	let mut towns: Vec<(i32, i32)> = Vec::with_capacity(town_count);
	for _ in 0..town_count {
		input.number().unwrap();
		let x = input.number().unwrap() as i32;
		let y = input.number().unwrap() as i32;
		input.word().unwrap();
		towns.push((x, y));
	}

	let mut track = vec![-1i64; cells];
	let mut inked = vec![false; regions];

	for turn in 0u64.. {
		let Some(_) = input.number() else { return };
		input.number().unwrap();
		for cell in 0..cells {
			track[cell] = input.number().unwrap();
			input.number().unwrap();
			inked[region[cell]] = input.number().unwrap() == 1;
			input.word().unwrap();
		}

		let mut rng = Rng(input.digest ^ (me << 33) ^ turn);
		let mut actions: Vec<String> = Vec::new();

		if town_count >= 2 && rng.chance(45) {
			let (fx, fy) = towns[rng.below(town_count)];
			let (tx, ty) = towns[rng.below(town_count)];
			actions.push(format!("AUTOPLACE {fx} {fy} {tx} {ty}"));
		}
		for _ in 0..rng.below(4) {
			let (near_x, near_y) = towns[rng.below(town_count.max(1))];
			let free = rng.chance(60);
			let mut at = (0, 0);
			for _ in 0..8 {
				at = if rng.chance(70) {
					(
						(near_x + rng.below(7) as i32 - 3).clamp(0, width - 1),
						(near_y + rng.below(7) as i32 - 3).clamp(0, height - 1),
					)
				} else {
					(
						rng.below(width as usize) as i32,
						rng.below(height as usize) as i32,
					)
				};
				let cell = (at.1 * width + at.0) as usize;
				if !free || (track[cell] == -1 && !inked[region[cell]]) {
					break;
				}
			}
			actions.push(format!("PLACE_TRACKS {} {}", at.0, at.1));
		}
		if rng.chance(35) {
			if rng.chance(50) {
				actions.push(format!("DISRUPT {}", rng.below(regions + 1)));
			} else {
				actions.push(format!(
					"DISRUPT {} {}",
					rng.below(width as usize),
					rng.below(height as usize)
				));
			}
			if rng.chance(20) {
				actions.push(format!("DISRUPT {}", rng.below(regions)));
			}
		}
		if rng.chance(6) {
			actions.push(format!(
				"PLACE_TRACKS {} {}",
				width as usize + rng.below(3),
				height as usize + rng.below(3)
			));
		}
		if rng.chance(6) {
			let (x, y) = towns[rng.below(town_count.max(1))];
			actions.push(format!("PLACE_TRACKS {x} {y}"));
		}
		if town_count >= 2 && rng.chance(5) {
			let (fx, fy) = towns[rng.below(town_count)];
			let (tx, ty) = towns[rng.below(town_count)];
			actions.push(format!("AUTOPLACE {fx} {fy} {tx} {ty}"));
		}
		if rng.chance(12) {
			actions.push(format!("MESSAGE turn {turn} h{:x}", input.digest & 0xffff));
		}
		if evil && rng.chance(2) {
			actions.push(match rng.below(5) {
				0 => "PLACE_TRACKS -1 2".into(),
				1 => "PLACE_TRACKS 99999999999 1".into(),
				2 => "TELEPORT 1 2".into(),
				3 => "place_tracks 1 2 3".into(),
				_ => String::new(),
			});
		}
		if actions.is_empty() {
			actions.push("WAIT".into());
		}

		let _ = writeln!(stdout, "{}", actions.join(";"));
		let _ = stdout.flush();
		if die_after == Some(turn) {
			return;
		}
	}
}
