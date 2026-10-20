pub fn lines(text: &str) -> Vec<String> {
	text.split('\n').map(str::to_owned).collect()
}

fn without_trailing_empty(lines: &[String]) -> &[String] {
	let kept = lines
		.iter()
		.rposition(|line| !line.is_empty())
		.map_or(0, |last| last + 1);
	&lines[..kept]
}

fn without_one_trailing_empty(lines: &[String]) -> &[String] {
	match lines.split_last() {
		Some((last, rest)) if last.is_empty() => rest,
		_ => lines,
	}
}

pub fn first_difference(expected: &[String], actual: &[String]) -> Option<String> {
	let expected = without_trailing_empty(expected);
	let actual = without_one_trailing_empty(actual);
	let length = expected.len().max(actual.len());
	(0..length).find_map(|index| {
		let wanted = expected.get(index);
		let found = actual.get(index);
		(wanted != found).then(|| {
			let line = index + 1;
			let wanted = wanted.map_or("<none>".to_owned(), |text| format!("{text:?}"));
			let found = found.map_or("<none>".to_owned(), |text| format!("{text:?}"));
			format!("line {line}: expected {wanted} found {found}")
		})
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	fn owned(items: &[&str]) -> Vec<String> {
		items.iter().map(|item| (*item).to_owned()).collect()
	}

	#[test]
	fn one_trailing_empty_line_is_free() {
		assert_eq!(first_difference(&owned(&["a", ""]), &owned(&["a"])), None);
		assert_eq!(
			first_difference(&owned(&["a", ""]), &owned(&["a", ""])),
			None
		);
	}

	#[test]
	fn two_trailing_empty_lines_fail() {
		assert!(first_difference(&owned(&["a", ""]), &owned(&["a", "", ""])).is_some());
	}

	#[test]
	fn trailing_space_and_carriage_return_fail() {
		assert!(first_difference(&owned(&["a"]), &owned(&["a "])).is_some());
		assert!(first_difference(&owned(&["a"]), &owned(&["a\r"])).is_some());
	}

	#[test]
	fn leading_empty_line_and_extra_line_fail() {
		assert!(first_difference(&owned(&["a"]), &owned(&["", "a"])).is_some());
		assert!(first_difference(&owned(&["a"]), &owned(&["a", "x"])).is_some());
	}
}
