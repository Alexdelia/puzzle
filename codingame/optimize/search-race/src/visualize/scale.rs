use crate::{
	referee::env::Axis,
	visualize::{CONVERSION_HEIGHT, CONVERSION_WIDTH},
};

#[inline]
pub fn scale_x(x: Axis) -> Axis {
	x * CONVERSION_WIDTH
}

#[inline]
pub fn scale_y(y: Axis) -> Axis {
	y * CONVERSION_HEIGHT
}
