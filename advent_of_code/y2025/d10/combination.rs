use std::collections::{HashMap, hash_map::Entry};

use crate::JoltageUnit;

pub type JoltageButtonPressCombination = Vec<JoltageUnit>;

pub type CombinationCache = HashMap<(JoltageUnit, usize), Vec<JoltageButtonPressCombination>>;

pub fn get_joltage_button_press_combination(
	cache: &mut CombinationCache,
	joltage_unit: JoltageUnit,
	button_count: usize,
) -> &[JoltageButtonPressCombination] {
	let entry = cache.entry((joltage_unit, button_count));
	match entry {
		Entry::Occupied(occupied) => occupied.into_mut(),
		Entry::Vacant(vacant) => {
			let mut current_combination =
				first_joltage_button_press_combination(joltage_unit, button_count);

			let mut res = Vec::new();
			res.push(current_combination.clone());

			while next_joltage_button_press_combination(&mut current_combination) {
				res.push(current_combination.clone());
			}

			vacant.insert(res)
		}
	}
}

fn first_joltage_button_press_combination(
	joltage_unit: JoltageUnit,
	button_count: usize,
) -> JoltageButtonPressCombination {
	let mut combination = vec![0; button_count];
	combination[0] = joltage_unit;
	combination
}

fn next_joltage_button_press_combination(
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
		let combination = first_joltage_button_press_combination(4, 3);
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
			first_joltage_button_press_combination(joltage_x, button_count);
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

	#[test]
	fn test_get_joltage_button_press_combination() {
		for ((joltage_unit, button_count), expected_combination) in [
			(
				(2, 2),
				vec![
					vec![2, 0], //
					vec![1, 1],
					vec![0, 2],
				],
			),
			(
				(4, 3),
				vec![
					vec![4, 0, 0],
					vec![3, 1, 0],
					vec![2, 2, 0],
					vec![1, 3, 0],
					vec![0, 4, 0],
					vec![3, 0, 1],
					vec![2, 1, 1],
					vec![1, 2, 1],
					vec![0, 3, 1],
					vec![2, 0, 2],
					vec![1, 1, 2],
					vec![0, 2, 2],
					vec![1, 0, 3],
					vec![0, 1, 3],
					vec![0, 0, 4],
				],
			),
			(
				(6, 4),
				vec![
					vec![6, 0, 0, 0],
					vec![5, 1, 0, 0],
					vec![4, 2, 0, 0],
					vec![3, 3, 0, 0],
					vec![2, 4, 0, 0],
					vec![1, 5, 0, 0],
					vec![0, 6, 0, 0],
					vec![5, 0, 1, 0],
					vec![4, 1, 1, 0],
					vec![3, 2, 1, 0],
					vec![2, 3, 1, 0],
					vec![1, 4, 1, 0],
					vec![0, 5, 1, 0],
					vec![4, 0, 2, 0],
					vec![3, 1, 2, 0],
					vec![2, 2, 2, 0],
					vec![1, 3, 2, 0],
					vec![0, 4, 2, 0],
					vec![3, 0, 3, 0],
					vec![2, 1, 3, 0],
					vec![1, 2, 3, 0],
					vec![0, 3, 3, 0],
					vec![2, 0, 4, 0],
					vec![1, 1, 4, 0],
					vec![0, 2, 4, 0],
					vec![1, 0, 5, 0],
					vec![0, 1, 5, 0],
					vec![0, 0, 6, 0],
					vec![5, 0, 0, 1],
					vec![4, 1, 0, 1],
					vec![3, 2, 0, 1],
					vec![2, 3, 0, 1],
					vec![1, 4, 0, 1],
					vec![0, 5, 0, 1],
					vec![4, 0, 1, 1],
					vec![3, 1, 1, 1],
					vec![2, 2, 1, 1],
					vec![1, 3, 1, 1],
					vec![0, 4, 1, 1],
					vec![3, 0, 2, 1],
					vec![2, 1, 2, 1],
					vec![1, 2, 2, 1],
					vec![0, 3, 2, 1],
					vec![2, 0, 3, 1],
					vec![1, 1, 3, 1],
					vec![0, 2, 3, 1],
					vec![1, 0, 4, 1],
					vec![0, 1, 4, 1],
					vec![0, 0, 5, 1],
					vec![4, 0, 0, 2],
					vec![3, 1, 0, 2],
					vec![2, 2, 0, 2],
					vec![1, 3, 0, 2],
					vec![0, 4, 0, 2],
					vec![3, 0, 1, 2],
					vec![2, 1, 1, 2],
					vec![1, 2, 1, 2],
					vec![0, 3, 1, 2],
					vec![2, 0, 2, 2],
					vec![1, 1, 2, 2],
					vec![0, 2, 2, 2],
					vec![1, 0, 3, 2],
					vec![0, 1, 3, 2],
					vec![0, 0, 4, 2],
					vec![3, 0, 0, 3],
					vec![2, 1, 0, 3],
					vec![1, 2, 0, 3],
					vec![0, 3, 0, 3],
					vec![2, 0, 1, 3],
					vec![1, 1, 1, 3],
					vec![0, 2, 1, 3],
					vec![1, 0, 2, 3],
					vec![0, 1, 2, 3],
					vec![0, 0, 3, 3],
					vec![2, 0, 0, 4],
					vec![1, 1, 0, 4],
					vec![0, 2, 0, 4],
					vec![1, 0, 1, 4],
					vec![0, 1, 1, 4],
					vec![0, 0, 2, 4],
					vec![1, 0, 0, 5],
					vec![0, 1, 0, 5],
					vec![0, 0, 1, 5],
					vec![0, 0, 0, 6],
				],
			),
		] {
			let mut cache = &mut CombinationCache::new();
			let result =
				get_joltage_button_press_combination(&mut cache, joltage_unit, button_count);
			assert_eq!(result, expected_combination);
		}
	}
}
