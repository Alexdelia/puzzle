pub const DIGEST: usize = 20;

fn sha1(data: &[u8]) -> [u8; DIGEST] {
	let mut h: [u32; 5] = [
		0x6745_2301,
		0xEFCD_AB89,
		0x98BA_DCFE,
		0x1032_5476,
		0xC3D2_E1F0,
	];
	let mut block = Vec::with_capacity(data.len() + 72);
	block.extend_from_slice(data);
	block.push(0x80);
	while block.len() % 64 != 56 {
		block.push(0);
	}
	block.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());

	let mut w = [0u32; 80];
	for chunk in block.chunks_exact(64) {
		for (i, word) in chunk.chunks_exact(4).enumerate() {
			w[i] = u32::from_be_bytes(word.try_into().unwrap());
		}
		for i in 16..80 {
			w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
		}
		let [mut a, mut b, mut c, mut d, mut e] = h;
		for (i, &wi) in w.iter().enumerate() {
			let (f, k) = match i {
				0..=19 => ((b & c) | (!b & d), 0x5A82_7999),
				20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
				40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
				_ => (b ^ c ^ d, 0xCA62_C1D6),
			};
			let next = a
				.rotate_left(5)
				.wrapping_add(f)
				.wrapping_add(e)
				.wrapping_add(k)
				.wrapping_add(wi);
			e = d;
			d = c;
			c = b.rotate_left(30);
			b = a;
			a = next;
		}
		h = [
			h[0].wrapping_add(a),
			h[1].wrapping_add(b),
			h[2].wrapping_add(c),
			h[3].wrapping_add(d),
			h[4].wrapping_add(e),
		];
	}

	let mut out = [0u8; DIGEST];
	for (slot, word) in out.chunks_exact_mut(4).zip(h) {
		slot.copy_from_slice(&word.to_be_bytes());
	}
	out
}

/// `SecureRandom.getInstance("SHA1PRNG")` seeded with `setSeed(long)`
/// then read through the `java.util.Random`
pub struct JavaRandom {
	state: [u8; DIGEST],
	remainder: [u8; DIGEST],
	remaining: usize,
}

impl JavaRandom {
	pub fn new(seed: i64) -> Self {
		JavaRandom {
			state: sha1(&seed.to_le_bytes()),
			remainder: [0; DIGEST],
			remaining: 0,
		}
	}

	pub fn next_bytes(&mut self, result: &mut [u8]) {
		let mut output = self.remainder;
		let mut index = 0;

		if self.remaining > 0 {
			let todo = (result.len()).min(DIGEST - self.remaining);
			for i in 0..todo {
				result[i] = output[self.remaining + i];
				output[self.remaining + i] = 0;
			}
			self.remaining += todo;
			index += todo;
		}

		while index < result.len() {
			output = sha1(&self.state);
			step_state(&mut self.state, &output);
			let todo = DIGEST.min(result.len() - index);
			for byte in &mut output[..todo] {
				result[index] = *byte;
				*byte = 0;
				index += 1;
			}
			self.remaining += todo;
		}

		self.remainder = output;
		self.remaining %= DIGEST;
	}

	fn next_bits(&mut self, bits: u32) -> i32 {
		let bytes = bits.div_ceil(8) as usize;
		let mut buffer = [0u8; 4];
		self.next_bytes(&mut buffer[..bytes]);
		let mut value = 0u32;
		for &byte in &buffer[..bytes] {
			value = (value << 8) + byte as u32;
		}
		(value >> (bytes as u32 * 8 - bits)) as i32
	}

	pub fn next_int(&mut self) -> i32 {
		self.next_bits(32)
	}

	pub fn next_int_below(&mut self, bound: i32) -> i32 {
		assert!(bound > 0, "bound must be positive");
		let mut r = self.next_bits(31);
		let m = bound - 1;
		if bound & m == 0 {
			return ((bound as i64 * r as i64) >> 31) as i32;
		}
		let mut u = r;
		loop {
			r = u % bound;
			if u.wrapping_add(m).wrapping_sub(r) >= 0 {
				return r;
			}
			u = self.next_bits(31);
		}
	}

	/// `RandomGenerator.nextInt(origin, bound)`
	pub fn next_int_range(&mut self, origin: i32, bound: i32) -> i32 {
		let mut r = self.next_int();
		if origin >= bound {
			return r;
		}
		let n = bound.wrapping_sub(origin);
		let m = n.wrapping_sub(1);
		if n & m == 0 {
			return (r & m).wrapping_add(origin);
		}
		if n > 0 {
			let mut u = ((r as u32) >> 1) as i32;
			loop {
				r = u % n;
				if u.wrapping_add(m).wrapping_sub(r) >= 0 {
					break;
				}
				u = ((self.next_int() as u32) >> 1) as i32;
			}
			return r.wrapping_add(origin);
		}
		while r < origin || r >= bound {
			r = self.next_int();
		}
		r
	}

	pub fn next_float(&mut self) -> f32 {
		self.next_bits(24) as f32 / (1 << 24) as f32
	}

	pub fn next_float_below(&mut self, bound: f32) -> f32 {
		assert!(
			bound > 0.0 && bound.is_finite(),
			"bound must be finite positive"
		);
		let r = self.next_float() * bound;
		if r >= bound { bound.next_down() } else { r }
	}

	pub fn next_double(&mut self) -> f64 {
		let high = (self.next_bits(26) as i64) << 27;
		(high + self.next_bits(27) as i64) as f64 * (1.0 / (1i64 << 53) as f64)
	}

	pub fn next_boolean(&mut self) -> bool {
		self.next_bits(1) != 0
	}

	/// `Collections.shuffle(list, random)`.
	pub fn shuffle<T>(&mut self, list: &mut [T]) {
		for i in (2..=list.len()).rev() {
			list.swap(i - 1, self.next_int_below(i as i32) as usize);
		}
	}
}

fn step_state(state: &mut [u8; DIGEST], output: &[u8; DIGEST]) {
	let mut carry = 1i32;
	let mut changed = false;
	for i in 0..DIGEST {
		let sum = state[i] as i8 as i32 + output[i] as i8 as i32 + carry;
		let byte = sum as u8;
		changed |= state[i] != byte;
		state[i] = byte;
		carry = sum >> 8;
	}
	if !changed {
		state[0] = state[0].wrapping_add(1);
	}
}
