use std::cmp::Ordering;

/// `java.util.PriorityQueue`
pub struct JPriorityQueue<T> {
	items: Vec<T>,
}

impl<T> Default for JPriorityQueue<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T> JPriorityQueue<T> {
	pub fn new() -> Self {
		JPriorityQueue { items: Vec::new() }
	}

	pub fn is_empty(&self) -> bool {
		self.items.is_empty()
	}

	pub fn clear(&mut self) {
		self.items.clear();
	}

	pub fn offer(&mut self, item: T, order: impl Fn(&T, &T) -> Ordering) {
		self.items.push(item);
		let mut k = self.items.len() - 1;
		while k > 0 {
			let parent = (k - 1) / 2;
			if order(&self.items[k], &self.items[parent]) != Ordering::Less {
				break;
			}
			self.items.swap(k, parent);
			k = parent;
		}
	}

	pub fn poll(&mut self, order: impl Fn(&T, &T) -> Ordering) -> Option<T> {
		let last = self.items.pop()?;
		if self.items.is_empty() {
			return Some(last);
		}
		let head = std::mem::replace(&mut self.items[0], last);

		let n = self.items.len();
		let mut k = 0;
		while k < n / 2 {
			let mut child = 2 * k + 1;
			let right = child + 1;
			if right < n && order(&self.items[child], &self.items[right]) == Ordering::Greater {
				child = right;
			}
			if order(&self.items[k], &self.items[child]) != Ordering::Greater {
				break;
			}
			self.items.swap(k, child);
			k = child;
		}
		Some(head)
	}
}
