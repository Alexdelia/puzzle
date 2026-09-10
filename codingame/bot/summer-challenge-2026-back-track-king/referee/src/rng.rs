pub struct Rng(u64);

impl Rng {
	pub fn new(seed: u64) -> Self {
		Rng(seed.wrapping_add(0x9E37_79B9_7F4A_7C15))
	}

	pub fn next_u64(&mut self) -> u64 {
		self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
		let mut z = self.0;
		z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
		z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
		z ^ (z >> 31)
	}

	pub fn below(&mut self, n: usize) -> usize {
		debug_assert!(n > 0);
		(((self.next_u64() >> 11) as u128 * n as u128) >> 53) as usize
	}

	pub fn between(&mut self, lo: usize, hi: usize) -> usize {
		lo + self.below(hi - lo + 1)
	}

	pub fn chance(&mut self, num: usize, den: usize) -> bool {
		self.below(den) < num
	}

	pub fn shuffle<T>(&mut self, slice: &mut [T]) {
		for i in (1..slice.len()).rev() {
			slice.swap(i, self.below(i + 1));
		}
	}

	pub fn pick<'a, T>(&mut self, slice: &'a [T]) -> &'a T {
		&slice[self.below(slice.len())]
	}
}
