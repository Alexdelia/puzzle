use crate::referee::env::{Axis, Degree, Speed};

#[derive(Clone, Copy, Debug, Default)]
pub struct Car {
	pub x: Axis,
	pub y: Axis,
	pub sx: Speed,
	pub sy: Speed,
	pub angle: Degree,
}
