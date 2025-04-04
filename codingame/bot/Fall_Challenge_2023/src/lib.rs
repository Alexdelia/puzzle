mod activation;
mod layer;
mod matrix;
mod network;

use std::ops::Range;

pub use activation::{relu, sigmoid, Activation};
pub use layer::Layer;
pub use matrix::Matrix;
pub use network::Network;

// ANSWER START

pub type Float = f32;

pub fn referencial(input: Float, in_range: &Range<Float>, out_range: &Range<Float>) -> Float {
	(input - in_range.start) / (in_range.end - in_range.start) * (out_range.end - out_range.start)
		+ out_range.start
}

pub fn referencial_bool(input: bool, out_range: &Range<Float>) -> Float {
	if input {
		out_range.end
	} else {
		out_range.start
	}
}

// ANSWER END

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_referencial() {
		assert_eq!(referencial(0.0, &(0.0..1.0), &(0.0..1.0)), 0.0);
		assert_eq!(referencial(0.5, &(0.0..1.0), &(0.0..1.0)), 0.5);
		assert_eq!(referencial(1.0, &(0.0..1.0), &(0.0..1.0)), 1.0);

		assert_eq!(referencial(0.0, &(0.0..1.0), &(0.0..2.0)), 0.0);
		assert_eq!(referencial(0.5, &(0.0..1.0), &(0.0..2.0)), 1.0);
		assert_eq!(referencial(1.0, &(0.0..1.0), &(0.0..2.0)), 2.0);

		assert_eq!(referencial(0.0, &(0.0..1.0), &(-1.0..1.0)), -1.0);
		assert_eq!(referencial(0.5, &(0.0..1.0), &(-1.0..1.0)), 0.0);
		assert_eq!(referencial(1.0, &(0.0..1.0), &(-1.0..1.0)), 1.0);

		assert_eq!(referencial(21.0, &(0.0..100.0), &(0.0..1.0)), 0.21);
		assert_eq!(referencial(21.0, &(0.0..42.0), &(0.0..1.0)), 0.5);
	}

	#[test]
	fn test_referencial_bool() {
		assert_eq!(referencial_bool(true, &(0.0..1.0)), 1.0);
		assert_eq!(referencial_bool(false, &(0.0..1.0)), 0.0);
	}
}
