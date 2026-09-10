use btk::arena::run_match;
use btk::driver::Driver;
use btk::game::Game;
use btk::gridmaker;
use btk::trace::{Detail, Trace};

struct Scripted {
	answers: std::vec::IntoIter<String>,
}

impl Driver for Scripted {
	fn send(&mut self, _text: &str) -> Result<(), String> {
		Ok(())
	}

	fn answer(&mut self) -> Result<String, String> {
		self.answers
			.next()
			.ok_or_else(|| "the recorded game has no more answers".into())
	}
}

fn field<'a>(golden: &'a str, key: &str) -> &'a str {
	golden
		.lines()
		.find_map(|line| line.strip_prefix(key))
		.unwrap_or_else(|| panic!("golden trace has no `{key}` line"))
}

fn answers(golden: &str, player: usize) -> Vec<String> {
	let key = format!("out{player} ");
	golden
		.lines()
		.filter_map(|line| line.strip_prefix(&key))
		.map(str::to_string)
		.collect()
}

fn replay(golden: &str) -> String {
	let seed: i64 = field(golden, "seed ").parse().unwrap();
	let league: i32 = field(golden, "league ").parse().unwrap();
	let mut game = Game::new(gridmaker::make(seed), league);
	let mut sides = [0, 1].map(|player| Scripted {
		answers: answers(golden, player).into_iter(),
	});
	let [side0, side1] = sides.each_mut();

	let mut trace = Trace::new(Detail::Digest);
	trace.header(seed, &game);
	let result = {
		let mut record =
			|game: &Game, report: &_, answers: &_, turn| trace.turn(game, report, answers, turn);
		run_match(&mut game, [side0, side1], None, Some(&mut record))
	};
	trace.footer(&game, &result);
	trace.text
}

#[test]
fn matches_the_official_referee() {
	let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/games");
	let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
		.expect("golden game directory")
		.map(|entry| entry.unwrap().path())
		.collect();
	files.sort();
	assert!(!files.is_empty(), "no golden games to replay");

	for path in files {
		let golden = std::fs::read_to_string(&path).unwrap();
		let ours = replay(&golden);
		let name = path.file_name().unwrap().to_string_lossy();
		for (line, (want, got)) in golden.lines().zip(ours.lines()).enumerate() {
			assert_eq!(want, got, "{name} line {}", line + 1);
		}
		assert_eq!(
			golden.lines().count(),
			ours.lines().count(),
			"{name} length"
		);
	}
}
