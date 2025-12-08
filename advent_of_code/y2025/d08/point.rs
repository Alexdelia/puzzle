#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
	pub x: PointUnit,
	pub y: PointUnit,
	pub z: PointUnit,
}

pub type PointUnit = u32;
