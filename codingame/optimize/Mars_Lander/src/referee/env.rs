pub type Axis = f64;

#[derive(Clone, Copy, Debug)]
pub struct Coord {
	pub x: Axis,
	pub y: Axis,
}

pub type Speed = f64;

pub type Angle = f64;

pub const MAX_WIDTH: Axis = 7000.0;
pub const MAX_HEIGHT: Axis = 3000.0;

pub const GRAVITY: Speed = 3.711;

pub const VALID_X_SPEED_THRESHOLD: Speed = 20.0;
pub const VALID_Y_SPEED_THRESHOLD: Speed = 40.0;
