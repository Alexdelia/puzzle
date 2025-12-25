use crate::referee::env::Angle;

#[derive(Debug, Clone, Copy)]
pub struct Step {
	pub tilt: TiltChange,
	pub thrust: ThrustChange,
}

/// titl change in degrees (-15 to 15)
type TiltChange = Angle;

#[derive(Debug, Clone, Copy)]
pub enum ThrustChange {
	Decrease,
	Keep,
	Increase,
}

pub type Solution = Vec<Step>;

impl Step {
	pub fn random(rng: &mut impl rand::Rng) -> Self {
		Step {
			tilt: Self::random_titl(rng),
			thrust: Self::random_thrust(rng),
		}
	}

	#[inline]
	pub fn random_titl(rng: &mut impl rand::Rng) -> TiltChange {
		rng.random_range(-15..=15)
	}

	#[inline]
	pub fn random_thrust(rng: &mut impl rand::Rng) -> ThrustChange {
		match rng.random_range(0..3) {
			0 => ThrustChange::Decrease,
			1 => ThrustChange::Keep,
			_ => ThrustChange::Increase,
		}
	}
}
