const TABLE_SIZE: i32 = 16;
const RESIZE_THRESHOLD: usize = 12;

pub fn string_hash(text: &str) -> i32 {
	text.encode_utf16().fold(0i32, |hash, unit| {
		hash.wrapping_mul(31).wrapping_add(i32::from(unit))
	})
}

fn bucket(key: &str) -> i32 {
	let hash = string_hash(key);
	(hash ^ ((hash as u32) >> 16) as i32) & (TABLE_SIZE - 1)
}

#[derive(Clone, Debug, Default)]
pub struct CountMap {
	entries: Vec<(String, i32)>,
}

impl CountMap {
	pub fn add(&mut self, key: String, amount: i32) {
		match self
			.entries
			.iter_mut()
			.find(|(existing, _)| *existing == key)
		{
			Some((_, count)) => *count += amount,
			None => {
				assert!(
					self.entries.len() < RESIZE_THRESHOLD,
					"a HashMap past 12 keys resizes and changes its iteration order"
				);
				self.entries.push((key, amount));
			}
		}
	}

	pub fn in_java_order(&self) -> Vec<(&str, i32)> {
		let mut ordered: Vec<(&str, i32)> = self
			.entries
			.iter()
			.map(|(key, count)| (key.as_str(), *count))
			.collect();
		ordered.sort_by_key(|(key, _)| bucket(key));
		ordered
	}

	pub fn clear(&mut self) {
		self.entries.clear();
	}

	pub fn is_empty(&self) -> bool {
		self.entries.is_empty()
	}
}
