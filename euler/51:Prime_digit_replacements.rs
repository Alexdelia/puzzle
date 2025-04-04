type Limit = u32;

struct Digit(Vec<u8>);

impl From<Limit> for Digit {
	fn from(n: Limit) -> Self {
		let mut v = Vec::new();
		let mut n = n;

		while n > 0 {
			v.push((n % 10) as u8);
			n /= 10;
		}

		v.reverse();

		Digit(v)
	}
}

impl From<Digit> for Limit {
	fn from(d: Digit) -> Self {
		let mut n = 0;

		for x in d.0 {
			n = n * 10 + x as Limit;
		}

		n
	}
}

impl From<&Digit> for Limit {
	fn from(d: &Digit) -> Self {
		let mut n = 0;

		for x in &d.0 {
			n = n * 10 + *x as Limit;
		}

		n
	}
}

/// take a number and a vector of indices to change
/// return a vector of all possible combinations
///
/// # Examples
///
/// ```
/// let n = Digit(vec![5, 6]);
/// let change = vec![0];
/// let result = combinations(n, change);
/// assert_eq!(
/// 	result,
/// 	[
/// 		Digit(vec![1, 6]),
/// 		Digit(vec![2, 6]),
/// 		Digit(vec![3, 6]),
/// 		Digit(vec![4, 6]),
/// 		Digit(vec![5, 6]),
/// 		Digit(vec![6, 6]),
/// 		Digit(vec![7, 6]),
/// 		Digit(vec![8, 6]),
/// 		Digit(vec![9, 6])
/// 	]
/// );
/// // do not create Digit(vec![0, 6])
///
/// let n = Digit(vec![5, 6]);
/// let change = vec![1];
/// let result = combinations(n, change);
/// assert_eq!(
/// 	result,
/// 	[
/// 		Digit(vec![5, 0]),
/// 		Digit(vec![5, 1]),
/// 		Digit(vec![5, 2]),
/// 		Digit(vec![5, 3]),
/// 		Digit(vec![5, 4]),
/// 		Digit(vec![5, 5]),
/// 		Digit(vec![5, 7]),
/// 		Digit(vec![5, 8]),
/// 		Digit(vec![5, 9])
/// 	]
/// );
/// // do not create Digit(vec![5, 6])
///
/// let n = Digit(vec![4, 3, 2]);
/// let change = vec![0, 2];
/// let result = combinations(n, change);
/// assert_eq!(
/// 	result,
/// 	[
/// 		Digit(vec![1, 3, 1]),
/// 		Digit(vec![2, 3, 2]),
/// 		Digit(vec![3, 3, 3]),
/// 		Digit(vec![4, 3, 4]),
/// 		Digit(vec![5, 3, 5]),
/// 		Digit(vec![6, 3, 6]),
/// 		Digit(vec![7, 3, 7]),
/// 		Digit(vec![8, 3, 8]),
/// 		Digit(vec![9, 3, 9])
/// 	]
/// );
/// // do not create Digit(vec![0, 3, 0]) nor Digit(vec![0, 3, 1]) nor Digit(vec![0, 3, 2]) ...
/// ```
fn combination(n: &Digit, change: &[usize]) -> Vec<Digit> {
	let mut res = Vec::new();

	for x in 0..10 {
		let mut d = n.0.clone();

		for i in change {
			d[*i] = x;
		}

		if d[0] != 0 {
			res.push(Digit(d));
		}
	}

	res
}

/// take a number and a size
/// return a vector of all possible combinations of indices to change
///
/// # Examples
///
/// ```
/// let n = Digit(vec![5, 6]);
/// let size = 1;
/// let result = change_comb(n, size);
/// assert_eq!(result, [[0], [1]]);
///
/// let n = Digit(vec![4, 3, 2]);
/// let size = 1;
/// let result = change_comb(n, size);
/// assert_eq!(result, [[0], [1], [2]]);
///
/// let n = Digit(vec![4, 3, 2]);
/// let size = 2;
/// let result = change_comb(n, size);
/// assert_eq!(result, [[0, 1], [0, 2], [1, 2]]);
/// /* do not create:
/// 	[
/// 		[1, 0],
/// 		[2, 0],
/// 		[2, 1]
/// 		[0, 0],
/// 		[1, 1],
/// 		[2, 2]
/// 	]
/// */
/// ```
fn change_comb(n: &Digit, size: usize) -> Vec<Vec<usize>> {
	let mut res = Vec::new();

	for i in 0..n.0.len() {
		let mut v = Vec::new();
		v.push(i);

		if size == 1 {
			res.push(v);
		} else {
			for j in i + 1..n.0.len() {
				let mut v = v.clone();
				v.push(j);
				res.push(v);
			}
		}
	}

	res
}

fn solve(size: u8) -> Limit {
	let sieve = primal::Sieve::new(Limit::MAX as usize);

	for p in sieve.primes_from(0) {
		let d = Digit::from(p as Limit);

		for i in 0..d.0.len() {
			let mut change = Vec::new();

			for j in i..d.0.len() {
				if d.0[j] == d.0[i] {
					change.push(j);
				}
			}

			if !change.is_empty()
				&& combination(&d, &change)
					.into_iter()
					.filter(|x| {
						let x: Limit = x.into();
						sieve.is_prime(x as usize)
					})
					.count() >= size as usize
			{
				return p as Limit;
			}
		}
	}

	panic!("Not found")
}

fn main() {
	println!("{}", solve(8));
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_solve_6() {
		assert_eq!(solve(6), 13);
	}

	#[test]
	fn test_solve_7() {
		assert_eq!(solve(7), 56003);
	}

	#[test]
	fn test_solve_8() {
		assert_eq!(solve(8), 121313);
	}

	#[test]
	fn test_combination() {
		let n = Digit(vec![5, 6]);
		let change = vec![1];
		let result = combination(&n, &change);
		assert_eq!(
			result,
			vec![
				Digit(vec![5, 0]),
				Digit(vec![5, 1]),
				Digit(vec![5, 2]),
				Digit(vec![5, 3]),
				Digit(vec![5, 4]),
				Digit(vec![5, 5]),
				Digit(vec![5, 7]),
				Digit(vec![5, 8]),
				Digit(vec![5, 9])
			]
		);
		// do not create Digit(vec![0, 6])

		let n = Digit(vec![5, 6]);
		let change = vec![1];
		let result = combination(n, change);
		assert_eq!(
			result,
			[
				Digit(vec![5, 0]),
				Digit(vec![5, 1]),
				Digit(vec![5, 2]),
				Digit(vec![5, 3]),
				Digit(vec![5, 4]),
				Digit(vec![5, 5]),
				Digit(vec![5, 7]),
				Digit(vec![5, 8]),
				Digit(vec![5, 9])
			]
		);

		let n = Digit(vec![4, 3, 2]);
		let change = vec![0, 2];
		let result = combination(&n, &change);
		assert_eq!(
			result,
			[
				Digit(vec![1, 3, 1]),
				Digit(vec![2, 3, 2]),
				Digit(vec![3, 3, 3]),
				Digit(vec![4, 3, 4]),
				Digit(vec![5, 3, 5]),
				Digit(vec![6, 3, 6]),
				Digit(vec![7, 3, 7]),
				Digit(vec![8, 3, 8]),
				Digit(vec![9, 3, 9])
			]
		);
		// do not create Digit(vec![0, 3, 0]) nor Digit(vec![0, 3, 1]) nor Digit(vec![0, 3, 2]) ...
	}
}
