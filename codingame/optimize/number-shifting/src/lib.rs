use std::fmt;
use std::io::Read;

pub const DX: [i32; 4] = [0, 1, 0, -1];
pub const DY: [i32; 4] = [1, 0, -1, 0];
pub const DIRS: [char; 4] = ['D', 'R', 'U', 'L'];

#[derive(Clone)]
pub struct Level {
	pub w: usize,
	pub h: usize,
	pub grid: Vec<i32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move {
	pub x: usize,
	pub y: usize,
	pub dir: usize,
	pub add: bool,
}

impl fmt::Display for Move {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let op = if self.add { '+' } else { '-' };
		write!(f, "{} {} {} {}", self.x, self.y, DIRS[self.dir], op)
	}
}

impl Move {
	pub fn parse(line: &str) -> Option<Move> {
		let mut it = line.split_whitespace();
		let x = it.next()?.parse().ok()?;
		let y = it.next()?.parse().ok()?;
		let dir_text = it.next()?;
		let dir = DIRS.iter().position(|&d| dir_text.starts_with(d))?;
		let add = it.next()? == "+";
		Some(Move { x, y, dir, add })
	}
}

impl Level {
	pub fn parse(text: &str) -> Level {
		let mut nums = text.split_whitespace().map(|t| t.parse::<i32>().unwrap());
		let w = nums.next().unwrap() as usize;
		let h = nums.next().unwrap() as usize;
		let grid = nums.take(w * h).collect::<Vec<_>>();
		assert_eq!(grid.len(), w * h);
		Level { w, h, grid }
	}

	pub fn read_stdin() -> Level {
		let mut text = String::new();
		std::io::stdin().read_to_string(&mut text).unwrap();
		Level::parse(&text)
	}

	pub fn at(&self, x: usize, y: usize) -> i32 {
		self.grid[y * self.w + x]
	}

	pub fn cell_count(&self) -> usize {
		self.grid.iter().filter(|&&v| v != 0).count()
	}

	pub fn apply(&mut self, m: Move) -> Result<(), String> {
		if m.x >= self.w || m.y >= self.h {
			return Err(format!("source outside grid: {m}"));
		}
		let v = self.at(m.x, m.y);
		if v == 0 {
			return Err(format!("source empty: {m}"));
		}
		let tx = m.x as i32 + DX[m.dir] * v;
		let ty = m.y as i32 + DY[m.dir] * v;
		if tx < 0 || ty < 0 || tx >= self.w as i32 || ty >= self.h as i32 {
			return Err(format!("target outside grid: {m}"));
		}
		let t = ty as usize * self.w + tx as usize;
		if self.grid[t] == 0 {
			return Err(format!("target empty: {m}"));
		}
		self.grid[t] = if m.add {
			self.grid[t] + v
		} else {
			(self.grid[t] - v).abs()
		};
		self.grid[m.y * self.w + m.x] = 0;
		Ok(())
	}

	pub fn check(&self, moves: &[Move]) -> Result<(), String> {
		let mut level = self.clone();
		for &m in moves {
			level.apply(m)?;
		}
		match level.cell_count() {
			0 => Ok(()),
			n => Err(format!("{n} cells remaining")),
		}
	}
}

impl fmt::Display for Level {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		writeln!(f, "{} {}", self.w, self.h)?;
		for y in 0..self.h {
			let row = (0..self.w)
				.map(|x| self.at(x, y).to_string())
				.collect::<Vec<_>>();
			writeln!(f, "{}", row.join(" "))?;
		}
		Ok(())
	}
}

pub struct Rng(u64);

impl Rng {
	pub fn new(seed: u64) -> Rng {
		Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03)
	}

	pub fn next_u64(&mut self) -> u64 {
		self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
		let mut z = self.0;
		z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
		z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
		z ^ (z >> 31)
	}

	pub fn unit(&mut self) -> f64 {
		(self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
	}

	pub fn below(&mut self, n: usize) -> usize {
		(((self.next_u64() >> 32) * n as u64) >> 32) as usize
	}
}

pub fn level_shape(level: usize) -> (usize, usize, usize) {
	let mut spawns = if level > 150 {
		3 + level - 75
	} else {
		3 + level / 2
	};
	let mut h = 5;
	let mut w = h * 16 / 9;
	while w * h < spawns * 2 {
		spawns -= 2;
		h += 1;
		w = h * 16 / 9;
	}
	(w, h, spawns)
}

pub fn generate(level: usize, seed: u64) -> (Level, Vec<Move>) {
	let mut rng = Rng::new(seed ^ ((level as u64) << 40));
	let (w, h, spawns) = level_shape(level);
	let mut grid = vec![0i32; w * h];
	let mut solution = Vec::new();
	for i in 0..spawns {
		let pair = i == 0 || rng.below(5) == 0;
		loop {
			let x1 = rng.below(w) as i32;
			let y1 = rng.below(h) as i32;
			let dir = rng.below(4);
			let length = 1 + rng.below(w) as i32;
			let x2 = x1 - length * DX[dir];
			let y2 = y1 - length * DY[dir];
			let add = !pair && rng.below(2) == 0;
			if x2 < 0 || y2 < 0 || x2 >= w as i32 || y2 >= h as i32 {
				continue;
			}
			let a = y1 as usize * w + x1 as usize;
			let b = y2 as usize * w + x2 as usize;
			if grid[b] != 0 {
				continue;
			}
			let mv = |add| Move {
				x: x2 as usize,
				y: y2 as usize,
				dir,
				add,
			};
			if pair {
				if grid[a] != 0 {
					continue;
				}
				grid[a] = length;
				grid[b] = length;
				solution.push(mv(false));
			} else {
				if grid[a] == 0 || grid[a] == length {
					continue;
				}
				grid[b] = length;
				let mut add = add;
				grid[a] += if add { -length } else { length };
				if grid[a] < 0 {
					grid[a] = -grid[a];
					add = !add;
				}
				solution.push(mv(add));
			}
			break;
		}
	}
	solution.reverse();
	(Level { w, h, grid }, solution)
}
pub mod plan;
