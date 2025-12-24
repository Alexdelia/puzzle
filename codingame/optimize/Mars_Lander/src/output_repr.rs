pub struct Step {
	/// tilt change in degrees (-15 to 15)
	pub tilt: i8,
	pub thrust: ThrustChange,
}

pub enum ThrustChange {
	Decrease,
	Keep,
	Increase,
}

pub type Solution = Vec<Step>;
