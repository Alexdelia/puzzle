pub struct Rng(u64);

impl Rng {
	pub fn seeded(seed: u64) -> Rng {
		Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
	}

	fn next_u64(&mut self) -> u64 {
		self.0 ^= self.0 << 13;
		self.0 ^= self.0 >> 7;
		self.0 ^= self.0 << 17;
		self.0
	}

	pub fn below(&mut self, n: usize) -> usize {
		(self.next_u64() % n as u64) as usize
	}

	pub fn unit(&mut self) -> f64 {
		(self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
	}
}
