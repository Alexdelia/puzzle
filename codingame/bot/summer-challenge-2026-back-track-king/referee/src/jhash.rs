/// Java's `Object.hashCode`
pub trait JavaHash {
	fn java_hash(&self) -> i32;
}

const DEFAULT_CAPACITY: usize = 16;
const LOAD_PERCENT: usize = 75;
const TREEIFY_LENGTH: usize = 9;
const MIN_TREEIFY_CAPACITY: usize = 64;

/// `java.util.HashSet`
pub struct JHashSet<K> {
	buckets: Vec<Vec<K>>,
	len: usize,
	threshold: usize,
}

impl<K: Copy + Eq + JavaHash> Default for JHashSet<K> {
	fn default() -> Self {
		Self::new()
	}
}

impl<K: Copy + Eq + JavaHash> JHashSet<K> {
	pub fn new() -> Self {
		JHashSet {
			buckets: Vec::new(),
			len: 0,
			threshold: 0,
		}
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn is_empty(&self) -> bool {
		self.len == 0
	}

	pub fn contains(&self, key: &K) -> bool {
		if self.buckets.is_empty() {
			return false;
		}
		self.buckets[self.slot(key)].contains(key)
	}

	pub fn add(&mut self, key: K) -> bool {
		if self.buckets.is_empty() {
			self.buckets = vec![Vec::new(); DEFAULT_CAPACITY];
			self.threshold = DEFAULT_CAPACITY * LOAD_PERCENT / 100;
		}
		let slot = self.slot(&key);
		let chain = &mut self.buckets[slot];
		if chain.contains(&key) {
			return false;
		}
		chain.push(key);
		if chain.len() >= TREEIFY_LENGTH {
			assert!(
				self.buckets.len() < MIN_TREEIFY_CAPACITY,
				"a bucket of {TREEIFY_LENGTH} keys in a table of {} would turn into a red-black \
				 tree, which this port does not model",
				self.buckets.len()
			);
			self.grow();
		}
		self.len += 1;
		if self.len > self.threshold {
			self.grow();
		}
		true
	}

	pub fn into_list(self) -> Vec<K> {
		self.buckets.into_iter().flatten().collect()
	}

	fn slot(&self, key: &K) -> usize {
		spread(key.java_hash()) & (self.buckets.len() - 1)
	}

	fn grow(&mut self) {
		let was = self.buckets.len();
		let mut buckets = vec![Vec::new(); was * 2];
		for (slot, chain) in self.buckets.drain(..).enumerate() {
			for key in chain {
				let stays = spread(key.java_hash()) & was == 0;
				buckets[if stays { slot } else { slot + was }].push(key);
			}
		}
		self.buckets = buckets;
		self.threshold *= 2;
	}
}

fn spread(hash: i32) -> usize {
	(hash ^ ((hash as u32) >> 16) as i32) as u32 as usize
}
