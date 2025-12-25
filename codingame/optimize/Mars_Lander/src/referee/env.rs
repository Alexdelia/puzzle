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
