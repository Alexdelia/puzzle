use crate::referee::env::{Angle, Axis, Speed, VALID_X_SPEED_THRESHOLD, VALID_Y_SPEED_THRESHOLD};

#[derive(Clone, Copy, Debug, Default)]
pub struct Lander {
	pub x: Axis,
	pub y: Axis,
	pub sx: Speed,
	pub sy: Speed,
	pub fuel: Fuel,
	pub rotate: Angle,
	pub power: Power,
}

impl Lander {
	pub fn valid_landing_condition(&self) -> bool {
		self.rotate == 0
			&& self.sx.abs() <= VALID_X_SPEED_THRESHOLD
			&& self.sy.abs() <= VALID_Y_SPEED_THRESHOLD
	}
}

/// from 0 to 2000
pub type Fuel = u16;

/// from 0 to 4
pub type Power = u8;
