fn parse_input<T: std::str::FromStr>() -> T {
	let mut input_line = String::new();
	std::io::stdin().read_line(&mut input_line).unwrap();
	input_line.trim().parse::<T>().ok().unwrap()
}

fn solve(n: u32, k: u8) -> usize {
	let mut count: usize = 0;
	let mut factor: usize = 1;
	let n = n as usize;

	while factor <= n {
		let d = ((n / factor) % 10) as u8;
		let top = n / (factor * 10);
		let bot = n % factor;

		if d < k {
			count += top * factor;
		} else if d == k {
			count += top * factor + bot + 1;
		} else {
			count += (top + 1) * factor;
		}

		factor *= 10;
	}

	count
}

fn main() {
	let n = parse_input::<u32>();
	let k = parse_input::<u8>();
	dbg!(n, k);

	println!("{}", solve(n, k));
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_example() {
		for &(n, k, expected) in &[
			(12, 2, 2),
			(0, 3, 0),
			(219, 5, 42),
			(4218, 4, 1461),
			(10000, 6, 4000),
			(248919, 7, 119682),
			(841772, 8, 458220),
			(1283048, 9, 732904),
			(824883294, 1, 767944060),
		] {
			let got = solve(n, k);
			assert_eq!(
				got, expected,
				"expected solve({n}, {k}) = {expected}, but got {got}"
			);
		}
	}
}
