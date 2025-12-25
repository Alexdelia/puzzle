use crate::referee::env::{Angle, Axis, Speed};

#[derive(Debug)]
pub struct Lander {
	pub x: Axis,
	pub y: Axis,
	pub sx: Speed,
	pub sy: Speed,
	pub fuel: Fuel,
	pub rotate: Angle,
	pub power: Power,
}

/// from 0 to 2000
pub type Fuel = u16;

/// from 0 to 4
pub type Power = u8;
