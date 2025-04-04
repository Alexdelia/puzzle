use crate::{Float, Layer};

// ANSWER START
pub struct Network {
	pub layers: Vec<Layer>,
}

impl Network {
	pub fn forward(&self, input: Vec<Float>) -> Vec<Float> {
		let mut output = input;

		for layer in &self.layers {
			output = layer.process(&output);
		}

		output
	}
}
// ANSWER END
