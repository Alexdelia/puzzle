use rand::RngExt;
use rand_xoshiro::Xoshiro256PlusPlus;

pub type Temperature = f32;

#[derive(Clone, Copy)]
pub struct ChainMeta {
	pub temperature: Temperature,
	pub stagnant: u32,
}

pub struct Plan {
	pub move_size: usize,
	pub force_accept: bool,
}

#[derive(Clone, Copy)]
pub enum Strategy {
	Anneal {
		start_temperature: Temperature,
		cooling: f32,
		floor_temperature: Temperature,
	},
	IteratedLocalSearch {
		kick_threshold: u32,
		kick_min: usize,
		kick_max: usize,
	},
}

impl Strategy {
	pub fn name(&self) -> &'static str {
		match self {
			Strategy::Anneal { .. } => "anneal",
			Strategy::IteratedLocalSearch { .. } => "ils",
		}
	}

	pub fn init_meta(&self) -> ChainMeta {
		let temperature = match self {
			Strategy::Anneal {
				start_temperature, ..
			} => *start_temperature,
			Strategy::IteratedLocalSearch { .. } => 0.0,
		};
		ChainMeta {
			temperature,
			stagnant: 0,
		}
	}

	pub fn plan(&self, meta: &mut ChainMeta, rng: &mut Xoshiro256PlusPlus) -> Plan {
		match self {
			Strategy::Anneal { .. } => Plan {
				move_size: 1,
				force_accept: false,
			},
			Strategy::IteratedLocalSearch {
				kick_threshold,
				kick_min,
				kick_max,
			} => {
				if meta.stagnant >= *kick_threshold {
					meta.stagnant = 0;
					Plan {
						move_size: rng.random_range(*kick_min..=*kick_max),
						force_accept: true,
					}
				} else {
					Plan {
						move_size: 1,
						force_accept: false,
					}
				}
			}
		}
	}

	pub fn accept(&self, delta: i32, meta: &ChainMeta, rng: &mut Xoshiro256PlusPlus) -> bool {
		if delta >= 0 {
			return true;
		}
		match self {
			Strategy::Anneal { .. } => {
				let probability = (delta as f32 / meta.temperature).exp();
				rng.random::<f32>() < probability
			}
			Strategy::IteratedLocalSearch { .. } => false,
		}
	}

	pub fn after_round(&self, meta: &mut ChainMeta, improved: bool) {
		if let Strategy::Anneal {
			start_temperature,
			cooling,
			floor_temperature,
		} = self
		{
			meta.temperature *= *cooling;
			if meta.temperature < *floor_temperature {
				meta.temperature = *start_temperature;
			}
		}
		if improved {
			meta.stagnant = 0;
		} else {
			meta.stagnant += 1;
		}
	}
}
