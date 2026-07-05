use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V2 {
	pub x: f64,
	pub y: f64,
}

pub const fn v2(x: f64, y: f64) -> V2 {
	V2 { x, y }
}

impl V2 {
	pub fn dot(self, o: V2) -> f64 {
		self.x * o.x + self.y * o.y
	}

	pub fn norm2(self) -> f64 {
		self.x * self.x + self.y * self.y
	}

	pub fn norm(self) -> f64 {
		self.norm2().sqrt()
	}

	pub fn dist2(self, o: V2) -> f64 {
		let x = o.x - self.x;
		let y = o.y - self.y;
		x * x + y * y
	}

	pub fn dist(self, o: V2) -> f64 {
		self.dist2(o).sqrt()
	}
}

impl Add for V2 {
	type Output = V2;
	fn add(self, o: V2) -> V2 {
		v2(self.x + o.x, self.y + o.y)
	}
}

impl AddAssign for V2 {
	fn add_assign(&mut self, o: V2) {
		self.x += o.x;
		self.y += o.y;
	}
}

impl Sub for V2 {
	type Output = V2;
	fn sub(self, o: V2) -> V2 {
		v2(self.x - o.x, self.y - o.y)
	}
}

impl SubAssign for V2 {
	fn sub_assign(&mut self, o: V2) {
		self.x -= o.x;
		self.y -= o.y;
	}
}

impl Mul<f64> for V2 {
	type Output = V2;
	fn mul(self, s: f64) -> V2 {
		v2(self.x * s, self.y * s)
	}
}

impl Neg for V2 {
	type Output = V2;
	fn neg(self) -> V2 {
		v2(-self.x, -self.y)
	}
}
