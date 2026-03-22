pub type Axis = f64;

#[derive(Clone, Copy, Debug)]
pub struct Coord {
	pub x: Axis,
	pub y: Axis,
}

pub type Speed = f64;

/// angle in degrees (0 to 360)
pub type Degree = i16;

pub const FRICTION: f64 = 0.85;

#[cfg(feature = "visualize")]
pub const MAX_WIDTH: Axis = 16000.0;
#[cfg(feature = "visualize")]
pub const MAX_HEIGHT: Axis = 9000.0;

pub const CHECKPOINT_RADIUS: Axis = 600.0;

pub const MAX_STEP: usize = 600;
