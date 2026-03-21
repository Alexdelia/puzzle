use rand::{Rng, RngExt};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Step {
	pub tilt: TiltChange,
	pub thrust: Thrust,
}

/// titl change in degrees (-18 to 18)
type TiltChange = i8;
const MIN_TILT_CHANGE: TiltChange = -18;
const MAX_TILT_CHANGE: TiltChange = 18;

/// thrust (0 to 200)
type Thrust = u8;
const MIN_THRUST: Thrust = 0;
const MAX_THRUST: Thrust = 200;

pub type Solution = Vec<Step>;

impl Step {
	pub fn random(rng: &mut impl Rng) -> Self {
		Step {
			tilt: Self::random_titl(rng),
			thrust: Self::random_thrust(rng),
		}
	}

	#[inline]
	pub fn random_titl(rng: &mut impl Rng) -> TiltChange {
		rng.random_range(MIN_TILT_CHANGE..=MAX_TILT_CHANGE)
	}

	#[inline]
	pub fn random_thrust(rng: &mut impl Rng) -> Thrust {
		rng.random_range(MIN_THRUST..=MAX_THRUST)
	}
}
