pub fn next_joltage_button_press_combination(current_combination: &mut [usize]) -> bool {
	let i = current_combination
		.iter()
		.position(|&x| x != 0)
		.expect("at least one non-zero");

	if i == current_combination.len() - 1 {
		return false;
	}

	let value = current_combination[i];
	current_combination[i + 1] += 1;
	current_combination[i] = 0;
	current_combination[0] = value - 1;

	true
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_next_joltage_button_press_combination() {
		let joltage_x = 4;
		let button_count = 3;
		let all_combination = &[
			&[4, 0, 0],
			&[3, 1, 0],
			&[2, 2, 0],
			&[1, 3, 0],
			&[0, 4, 0],
			&[3, 0, 1],
			&[2, 1, 1],
			&[1, 2, 1],
			&[0, 3, 1],
			&[2, 0, 2],
			&[1, 1, 2],
			&[0, 2, 2],
			&[1, 0, 3],
			&[0, 1, 3],
			&[0, 0, 4],
		];

		let mut current_combination = vec![0; button_count];
		current_combination[0] = joltage_x;
		let mut index = 0;
		loop {
			assert_eq!(
				current_combination.as_slice(),
				all_combination[index],
				"at index {index}"
			);

			let res = next_joltage_button_press_combination(&mut current_combination);
			if !res {
				assert_eq!(
					index,
					all_combination.len() - 1,
					"should be last combination"
				);
				break;
			}

			index += 1;
		}
	}
}
