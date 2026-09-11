use btk::replay::{Replay, TurnRecord};

pub const LABEL: usize = 24;
pub const COLUMN: usize = 10;

pub struct View<'a> {
	pub run: &'a Replay,
	pub me: usize,
}

impl<'a> View<'a> {
	pub fn new(run: &'a Replay, me: usize) -> Self {
		View { run, me }
	}

	pub fn foe(&self) -> usize {
		1 - self.me
	}

	pub fn sides(&self) -> [usize; 2] {
		[self.me, self.foe()]
	}

	pub fn name(&self, player: usize) -> &'static str {
		if player == self.me { "us" } else { "foe" }
	}

	pub fn mine<T: Copy>(&self, values: [T; 2]) -> T {
		values[self.me]
	}

	pub fn theirs<T: Copy>(&self, values: [T; 2]) -> T {
		values[self.foe()]
	}

	pub fn heading(&self) {
		println!("{:<LABEL$} {:>COLUMN$} {:>COLUMN$}", "", "us", "foe");
	}

	pub fn row(&self, label: &str, values: [String; 2]) {
		let [first, second] = values;
		let (us, foe) = if self.me == 0 {
			(first, second)
		} else {
			(second, first)
		};
		println!("{label:<LABEL$} {us:>COLUMN$} {foe:>COLUMN$}");
	}

	pub fn count_row(&self, label: &str, values: [i32; 2]) {
		self.row(label, values.map(|value| value.to_string()));
	}

	pub fn ratio_row(&self, label: &str, values: [f64; 2], digits: usize) {
		self.row(label, values.map(|value| format!("{value:.digits$}")));
	}

	pub fn owner_name(&self, owner: i8) -> &'static str {
		match owner {
			0 | 1 if owner as usize == self.me => "us",
			0 | 1 => "foe",
			2 => "neu",
			_ => "-",
		}
	}

	pub fn share(&self, owned: [i32; 2]) -> f64 {
		let total = (owned[0] + owned[1]).max(1) as f64;
		100.0 * owned[self.me] as f64 / total
	}

	pub fn turns(&self) -> &'a [TurnRecord] {
		&self.run.turns
	}
}

pub fn section(title: &str) {
	println!("\n== {title} ==");
}

pub fn base36(value: usize) -> char {
	b"0123456789abcdefghijklmnopqrstuvwxyz"[value % 36] as char
}

pub fn percent(part: i64, whole: i64) -> f64 {
	100.0 * part as f64 / whole.max(1) as f64
}

pub fn short_answer(answer: &str) -> String {
	answer
		.split(';')
		.map(|command| {
			let words: Vec<&str> = command.split_whitespace().collect();
			match words.as_slice() {
				["PLACE_TRACKS", x, y] => format!("({x},{y})"),
				["DISRUPT", rest @ ..] => format!("ink[{}]", rest.join(",")),
				["AUTOPLACE", a, b, c, d] => format!("auto({a},{b})->({c},{d})"),
				_ => command.trim().to_string(),
			}
		})
		.collect::<Vec<_>>()
		.join(" ")
}
