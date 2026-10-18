use std::fmt::Write;

pub const DEPOT: usize = 0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Site {
	pub x: i64,
	pub y: i64,
	pub demand: u32,
}

#[derive(Clone, Debug)]
pub struct Instance {
	pub capacity: u32,
	pub sites: Vec<Site>,
}

impl Instance {
	pub fn parse(text: &str) -> Result<Instance, String> {
		let mut lines = text.lines();
		let mut numbers = |what: &str, count: usize| -> Result<Vec<i64>, String> {
			let line = lines.next().ok_or(format!("missing {what}"))?;
			let parsed = line
				.split_whitespace()
				.map(|token| token.parse::<i64>())
				.collect::<Result<Vec<_>, _>>()
				.map_err(|e| format!("{what}: {e}"))?;
			if parsed.len() != count {
				return Err(format!("{what}: expected {count} numbers, got {line:?}"));
			}
			Ok(parsed)
		};
		let site_count = numbers("site count", 1)?[0] as usize;
		if site_count == 0 {
			return Err("no depot".to_string());
		}
		let capacity = numbers("capacity", 1)?[0] as u32;
		let mut sites = Vec::with_capacity(site_count);
		for expected_index in 0..site_count {
			let what = format!("site {expected_index}");
			let [index, x, y, demand] = numbers(&what, 4)?[..] else {
				unreachable!()
			};
			if index as usize != expected_index {
				return Err(format!("{what}: listed as index {index}"));
			}
			sites.push(Site {
				x,
				y,
				demand: demand as u32,
			});
		}
		if sites[DEPOT].demand != 0 {
			return Err("depot has a demand".to_string());
		}
		Ok(Instance { capacity, sites })
	}

	pub fn site_count(&self) -> usize {
		self.sites.len()
	}

	pub fn distance(&self, a: usize, b: usize) -> u32 {
		let (a, b) = (self.sites[a], self.sites[b]);
		let (dx, dy) = (a.x - b.x, a.y - b.y);
		((dx * dx + dy * dy) as f64).sqrt().round() as u32
	}

	pub fn input(&self) -> String {
		let mut input = format!(
			"{site_count}\n{capacity}\n",
			site_count = self.site_count(),
			capacity = self.capacity,
		);
		for (index, site) in self.sites.iter().enumerate() {
			let Site { x, y, demand } = site;
			writeln!(input, "{index} {x} {y} {demand}").unwrap();
		}
		input
	}
}
