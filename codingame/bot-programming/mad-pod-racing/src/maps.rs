use super::geom::{V2, v2};

pub const CHECKPOINT_JITTER: i64 = 30;

pub const AGADE_MAPS: [&[V2]; 13] = [
	&[
		v2(12460.0, 1350.0),
		v2(10540.0, 5980.0),
		v2(3580.0, 5180.0),
		v2(13580.0, 7600.0),
	],
	&[
		v2(3600.0, 5280.0),
		v2(13840.0, 5080.0),
		v2(10680.0, 2280.0),
		v2(8700.0, 7460.0),
		v2(7200.0, 2160.0),
	],
	&[
		v2(4560.0, 2180.0),
		v2(7350.0, 4940.0),
		v2(3320.0, 7230.0),
		v2(14580.0, 7700.0),
		v2(10560.0, 5060.0),
		v2(13100.0, 2320.0),
	],
	&[v2(5010.0, 5260.0), v2(11480.0, 6080.0), v2(9100.0, 1840.0)],
	&[
		v2(14660.0, 1410.0),
		v2(3450.0, 7220.0),
		v2(9420.0, 7240.0),
		v2(5970.0, 4240.0),
	],
	&[
		v2(3640.0, 4420.0),
		v2(8000.0, 7900.0),
		v2(13300.0, 5540.0),
		v2(9560.0, 1400.0),
	],
	&[
		v2(4100.0, 7420.0),
		v2(13500.0, 2340.0),
		v2(12940.0, 7220.0),
		v2(5640.0, 2580.0),
	],
	&[
		v2(14520.0, 7780.0),
		v2(6320.0, 4290.0),
		v2(7800.0, 860.0),
		v2(7660.0, 5970.0),
		v2(3140.0, 7540.0),
		v2(9520.0, 4380.0),
	],
	&[
		v2(10040.0, 5970.0),
		v2(13920.0, 1940.0),
		v2(8020.0, 3260.0),
		v2(2670.0, 7020.0),
	],
	&[v2(7500.0, 6940.0), v2(6000.0, 5360.0), v2(11300.0, 2820.0)],
	&[
		v2(4060.0, 4660.0),
		v2(13040.0, 1900.0),
		v2(6560.0, 7840.0),
		v2(7480.0, 1360.0),
		v2(12700.0, 7100.0),
	],
	&[
		v2(3020.0, 5190.0),
		v2(6280.0, 7760.0),
		v2(14100.0, 7760.0),
		v2(13880.0, 1220.0),
		v2(10240.0, 4920.0),
		v2(6100.0, 2200.0),
	],
	&[
		v2(10323.0, 3366.0),
		v2(11203.0, 5425.0),
		v2(7259.0, 6656.0),
		v2(5425.0, 2838.0),
	],
];

#[derive(Clone)]
pub struct Rng(pub u64);

impl Rng {
	pub fn new(seed: u64) -> Self {
		Rng(seed)
	}

	pub fn next_u64(&mut self) -> u64 {
		self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
		let mut z = self.0;
		z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
		z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
		z ^ (z >> 31)
	}

	pub fn below(&mut self, n: u64) -> u64 {
		self.next_u64() % n
	}

	pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
		lo + self.below((hi - lo + 1) as u64) as i64
	}

	pub fn f01(&mut self) -> f64 {
		(self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
	}

	pub fn chance(&mut self, p: f64) -> bool {
		self.f01() < p
	}
}

pub fn agade_map(rng: &mut Rng) -> Vec<V2> {
	let base = AGADE_MAPS[rng.below(AGADE_MAPS.len() as u64) as usize];
	let mut cps: Vec<V2> = base
		.iter()
		.map(|c| {
			v2(
				c.x + rng.range(-CHECKPOINT_JITTER, CHECKPOINT_JITTER) as f64,
				c.y + rng.range(-CHECKPOINT_JITTER, CHECKPOINT_JITTER) as f64,
			)
		})
		.collect();
	for i in (1..cps.len()).rev() {
		let j = rng.below(i as u64 + 1) as usize;
		cps.swap(i, j);
	}
	cps
}

pub fn random_map(rng: &mut Rng) -> Vec<V2> {
	let n = rng.range(3, 8) as usize;
	let mut cps: Vec<V2> = Vec::with_capacity(n);
	while cps.len() < n {
		let c = v2(rng.range(1500, 14500) as f64, rng.range(1000, 8000) as f64);
		if cps.iter().all(|o| o.dist(c) > 2000.0) {
			cps.push(c);
		}
	}
	cps
}

pub fn parse_map(spec: &str) -> Option<Vec<V2>> {
	let mut cps = Vec::new();
	for part in spec.split(';') {
		let part = part.trim();
		if part.is_empty() {
			continue;
		}
		let mut nums = part
			.split(|c: char| c == ',' || c.is_whitespace())
			.filter(|s| !s.is_empty());
		let x: f64 = nums.next()?.parse().ok()?;
		let y: f64 = nums.next()?.parse().ok()?;
		cps.push(v2(x, y));
	}
	if cps.len() >= 2 { Some(cps) } else { None }
}

pub fn format_map(cps: &[V2]) -> String {
	cps.iter()
		.map(|c| format!("{} {}", c.x as i64, c.y as i64))
		.collect::<Vec<_>>()
		.join(";")
}
