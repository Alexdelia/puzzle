use crate::Joltage;

pub type JoltageButtonPressCombination = Vec<Joltage>;

pub fn first_joltage_button_press_combination(
	remaining_joltage: &(Joltage, Vec<usize>),
) -> JoltageButtonPressCombination {
	let mut combination = vec![0; remaining_joltage.1.len()];
	combination[0] = remaining_joltage.0;
	combination
}

pub fn next_joltage_button_press_combination(
	current_combination: &mut JoltageButtonPressCombination,
) -> bool {
	let i = current_combination
		.iter()
		.position(|&x| x != 0)
		.unwrap_or_else(|| panic!("current_combination={current_combination:?} has no non-zero"));

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
	fn test_first_joltage_button_press_combination() {
		let remaining_joltage = (4, vec![0, 1, 2]);
		let combination = first_joltage_button_press_combination(&remaining_joltage);
		assert_eq!(combination, vec![4, 0, 0]);
	}

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

		let mut current_combination =
			first_joltage_button_press_combination(&(joltage_x, vec![0; button_count]));
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
