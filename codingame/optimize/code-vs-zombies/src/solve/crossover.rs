use rand::Rng;

use crate::Coord;

pub fn crossover(a: &[Coord], b: &[Coord], rng: &mut impl Rng) -> Vec<Coord> {
	let len = a.len().min(b.len());
	let style = rng.random_range(0..3);
	let mut c = Vec::with_capacity(len);
	match style {
		0 => {
			let cut = rng.random_range(0..len);
			c.extend_from_slice(&a[..cut]);
			c.extend_from_slice(&b[cut..len]);
		}
		1 => {
			for i in 0..len {
				c.push(if rng.random::<bool>() { a[i] } else { b[i] });
			}
		}
		_ => {
			let cut1 = rng.random_range(0..len);
			let cut2 = rng.random_range(cut1..len);
			c.extend_from_slice(&a[..cut1]);
			c.extend_from_slice(&b[cut1..cut2]);
			c.extend_from_slice(&a[cut2..len]);
		}
	}
	c
}
