use btk::jhash::{JHashSet, JavaHash};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Coord {
	x: i32,
	y: i32,
}

impl JavaHash for Coord {
	fn java_hash(&self) -> i32 {
		31i32
			.wrapping_mul(31i32.wrapping_add(self.x))
			.wrapping_add(self.y)
	}
}

fn row_major(w: i32, h: i32) -> Vec<Coord> {
	(0..h)
		.flat_map(|y| (0..w).map(move |x| Coord { x, y }))
		.collect()
}

fn neighbourhood(w: i32, h: i32) -> Vec<Coord> {
	(0..9)
		.flat_map(|i| {
			(0..5).map(move |j| Coord {
				x: (i * 7 + j) % w,
				y: (i * 3 + j * 5) % h,
			})
		})
		.collect()
}

fn diagonal(w: i32, h: i32) -> Vec<Coord> {
	(0..w + h)
		.flat_map(|d| {
			(0..w).filter_map(move |x| {
				let y = d - x;
				(y >= 0 && y < h).then_some(Coord { x, y })
			})
		})
		.collect()
}

fn line(name: &str, inserted: &[Coord]) -> String {
	let mut set = JHashSet::new();
	for &coord in inserted {
		set.add(coord);
	}
	let mut out = format!("{name} {}", set.len());
	for coord in set.into_list() {
		out.push_str(&format!(" {},{}", coord.x, coord.y));
	}
	out
}

#[test]
fn matches_java_hash_set_iteration_order() {
	let mut reversed = row_major(24, 16);
	reversed.reverse();
	let ours = [
		line("rowmajor 30x20", &row_major(30, 20)),
		line("rowmajor 21x14", &row_major(21, 14)),
		line("neighbourhood", &neighbourhood(30, 20)),
		line("diagonal", &diagonal(30, 20)),
		line("reverse 24x16", &reversed),
	];
	let golden = include_str!("golden/jhash.txt");
	for (want, got) in golden.lines().zip(&ours) {
		assert_eq!(want, got);
	}
	assert_eq!(golden.lines().count(), ours.len());
}
