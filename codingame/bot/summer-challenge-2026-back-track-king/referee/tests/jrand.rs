use btk::jrand::JavaRandom;

const SEEDS: [i64; 6] = [1, -1, 3967011272695928663, 42, 7, -8888888888888];
const BOUNDS: [i32; 8] = [1, 2, 3, 7, 16, 100, 1000, 65536];
const FLOAT_BOUNDS: [(&str, f32); 5] = [
	("1.0", 1.0),
	("0.055", 0.055),
	("12.6", 12.6),
	("24.0", 24.0),
	("3.5", 3.5),
];
const SHUFFLE_SIZES: [usize; 5] = [1, 2, 5, 17, 64];

fn replay() -> String {
	let mut out = String::new();
	for seed in SEEDS {
		let mut rng = JavaRandom::new(seed);
		out.push_str(&format!("seed {seed}\n"));
		for _ in 0..8 {
			out.push_str(&format!("nextInt {}\n", rng.next_int()));
		}
		for bound in BOUNDS {
			out.push_str(&format!("nextInt({bound}) {}\n", rng.next_int_below(bound)));
		}
		for _ in 0..4 {
			out.push_str(&format!("nextInt(14,21) {}\n", rng.next_int_range(14, 21)));
		}
		out.push_str(&format!("nextInt(2,8) {}\n", rng.next_int_range(2, 8)));
		out.push_str(&format!("nextInt(0,5) {}\n", rng.next_int_range(0, 5)));
		for _ in 0..6 {
			out.push_str(&format!(
				"nextFloat {}\n",
				rng.next_float().to_bits() as i32
			));
		}
		for (label, bound) in FLOAT_BOUNDS {
			out.push_str(&format!(
				"nextFloat({label}) {}\n",
				rng.next_float_below(bound).to_bits() as i32
			));
		}
		for _ in 0..6 {
			out.push_str(&format!("nextBoolean {}\n", u8::from(rng.next_boolean())));
		}
		out.push_str(&format!(
			"nextDouble {}\n",
			rng.next_double().to_bits() as i64
		));
		for size in SHUFFLE_SIZES {
			let mut list: Vec<usize> = (0..size).collect();
			rng.shuffle(&mut list);
			out.push_str(&format!("shuffle({size})"));
			for value in list {
				out.push_str(&format!(" {value}"));
			}
			out.push('\n');
		}
		let mut bytes = [0u8; 37];
		rng.next_bytes(&mut bytes);
		out.push_str("nextBytes");
		for byte in bytes {
			out.push_str(&format!(" {byte}"));
		}
		out.push('\n');
	}
	out
}

#[test]
fn matches_java_secure_random() {
	let golden = include_str!("golden/jrand.txt");
	let ours = replay();
	for (line, (want, got)) in golden.lines().zip(ours.lines()).enumerate() {
		assert_eq!(want, got, "line {}", line + 1);
	}
	assert_eq!(golden.lines().count(), ours.lines().count());
}
