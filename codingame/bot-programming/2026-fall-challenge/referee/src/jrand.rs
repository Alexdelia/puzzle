const MULTIPLIER: u64 = 0x5_DEEC_E66D;
const ADDEND: u64 = 0xB;
const MASK: u64 = (1 << 48) - 1;
const DOUBLE_UNIT: f64 = 1.0 / (1u64 << 53) as f64;

#[derive(Clone, Debug)]
pub struct JavaRandom {
	state: u64,
}

impl JavaRandom {
	pub fn new(seed: i64) -> Self {
		JavaRandom {
			state: (seed as u64 ^ MULTIPLIER) & MASK,
		}
	}

	fn next(&mut self, bits: u32) -> i32 {
		self.state = self.state.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND) & MASK;
		(self.state >> (48 - bits)) as i32
	}

	pub fn next_i32(&mut self) -> i32 {
		self.next(32)
	}

	pub fn next_int(&mut self, bound: i32) -> i32 {
		assert!(bound > 0, "bound must be positive");
		let mut r = self.next(31);
		let m = bound - 1;
		if bound & m == 0 {
			return ((i64::from(bound) * i64::from(r)) >> 31) as i32;
		}
		let mut u = r;
		loop {
			r = u % bound;
			if u.wrapping_sub(r).wrapping_add(m) >= 0 {
				return r;
			}
			u = self.next(31);
		}
	}

	pub fn next_double(&mut self) -> f64 {
		let high = i64::from(self.next(26)) << 27;
		let low = i64::from(self.next(27));
		(high + low) as f64 * DOUBLE_UNIT
	}
}
