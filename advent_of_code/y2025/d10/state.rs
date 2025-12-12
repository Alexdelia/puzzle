use crate::StateButton;

pub type State = u16;

#[inline]
pub fn click_button(state: State, button: StateButton) -> State {
	state ^ button
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_click_button() {
		let tests = [
			(0b0000, 0b0011, 0b0011),
			(0b0011, 0b0000, 0b0011),
			(0b0011, 0b0011, 0b0000),
			(0b0101, 0b0110, 0b0011),
			(0b1111, 0b0001, 0b1110),
		];

		for (state, button, expected) in tests {
			let result = click_button(state, button);
			assert_eq!(
				result, expected,
				"click_button({state:04b}, {button:04b}) = {result:04b}, expected {expected:04b}",
			);
		}
	}
}
