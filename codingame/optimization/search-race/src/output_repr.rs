use rand::{Rng, RngExt};

use serde::{Deserialize, Serialize, Serializer, ser::SerializeSeq};

use crate::referee::env::MAX_STEP;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[repr(C)]
pub struct Step {
	pub tilt: TiltChange,
	pub thrust: Thrust,
}

pub type TiltChange = i8;
pub const MIN_TILT_CHANGE: TiltChange = -18;
pub const MAX_TILT_CHANGE: TiltChange = 18;

pub type Thrust = u8;
pub const MIN_THRUST: Thrust = 0;
pub const MAX_THRUST: Thrust = 200;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Solution {
	pub steps: [Step; MAX_STEP],
	pub len: u16,
}

impl Default for Solution {
	fn default() -> Self {
		Self {
			steps: [Step::default(); MAX_STEP],
			len: 0,
		}
	}
}

impl Solution {
	pub fn new() -> Self {
		Self::default()
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.len as usize
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.len == 0
	}

	#[inline]
	pub fn as_slice(&self) -> &[Step] {
		&self.steps[..self.len as usize]
	}

	#[inline]
	pub fn as_mut_slice(&mut self) -> &mut [Step] {
		&mut self.steps[..self.len as usize]
	}

	#[inline]
	pub fn iter(&self) -> std::slice::Iter<'_, Step> {
		self.as_slice().iter()
	}

	#[inline]
	pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Step> {
		self.as_mut_slice().iter_mut()
	}

	#[inline]
	pub fn get(&self, index: usize) -> Option<&Step> {
		self.as_slice().get(index)
	}

	#[inline]
	pub fn push(&mut self, step: Step) {
		let i = self.len as usize;
		assert!(i < MAX_STEP, "Solution capacity exceeded");
		self.steps[i] = step;
		self.len += 1;
	}

	#[inline]
	pub fn clear(&mut self) {
		self.len = 0;
	}

	#[inline]
	pub fn truncate(&mut self, new_len: usize) {
		let new_len = new_len.min(MAX_STEP);
		if (new_len as u16) < self.len {
			self.len = new_len as u16;
		}
	}
}

impl<'a> IntoIterator for &'a Solution {
	type Item = &'a Step;
	type IntoIter = std::slice::Iter<'a, Step>;
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl Serialize for Solution {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let slice = self.as_slice();
		let mut seq = serializer.serialize_seq(Some(slice.len()))?;
		for step in slice {
			seq.serialize_element(step)?;
		}
		seq.end()
	}
}

impl<'de> Deserialize<'de> for Solution {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let v = Vec::<Step>::deserialize(deserializer)?;
		if v.len() > MAX_STEP {
			return Err(serde::de::Error::custom(format!(
				"solution length {} exceeds MAX_STEP {}",
				v.len(),
				MAX_STEP
			)));
		}
		let mut solution = Solution::default();
		for step in v {
			solution.push(step);
		}
		Ok(solution)
	}
}

impl Step {
	pub fn random(rng: &mut impl Rng) -> Self {
		Step {
			tilt: Self::random_titl(rng),
			thrust: Self::random_thrust(rng),
		}
	}

	#[inline]
	pub fn random_titl(rng: &mut impl Rng) -> TiltChange {
		rng.random_range(MIN_TILT_CHANGE..=MAX_TILT_CHANGE)
	}

	#[inline]
	pub fn random_thrust(rng: &mut impl Rng) -> Thrust {
		rng.random_range(MIN_THRUST..=MAX_THRUST)
	}
}
