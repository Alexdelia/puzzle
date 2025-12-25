mod first_generation;
pub use first_generation::init_first_generation;

use crate::referee::lander::Lander;

pub const SOLUTION_PER_GENERATION: usize = 128;

pub fn init_lander() -> [Lander; SOLUTION_PER_GENERATION] {
	let mut lander_list = Vec::with_capacity(SOLUTION_PER_GENERATION);
	for _ in 0..SOLUTION_PER_GENERATION {
		lander_list.push(Lander {
			x: 2500.0,
			y: 2500.0,
			sx: 0.0,
			sy: 0.0,
			fuel: 500,
			rotate: 0.0,
			power: 0,
		});
	}
	lander_list.try_into().expect("lander list size mismatch")
}
