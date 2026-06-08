pub type Axis = f64;

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct Coord {
	pub x: Axis,
	pub y: Axis,
}

pub type Speed = f64;

pub type Degree = f64;

pub const FRICTION: f64 = 0.85;

#[cfg(feature = "visualize")]
pub const MAX_WIDTH: Axis = 16000.0;
#[cfg(feature = "visualize")]
pub const MAX_HEIGHT: Axis = 9000.0;

const EPSILON: f64 = 0.00001;

#[inline]
pub fn truncate(x: f64) -> f64 {
	let rounded = x.round();
	if (rounded - x).abs() < EPSILON {
		rounded
	} else {
		x.trunc()
	}
}

pub const CHECKPOINT_RADIUS: Axis = 600.0;

pub const MAX_STEP: usize = 600;
