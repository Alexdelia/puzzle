use block_the_spreading_fire::{Action, Fault, Game, Map};
use std::fmt;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitCode, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};
use std::{env, fs, thread};

const FIRST_TURN_LIMIT: Duration = Duration::from_millis(5000);
const TURN_LIMIT: Duration = Duration::from_millis(100);
const DEFAULT_TEST_DIR: &str = "validator/test";

enum Stop {
	Rule(Fault),
	Timeout(Duration),
	BotExited,
}

impl fmt::Display for Stop {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Stop::Rule(fault) => write!(f, "{fault}"),
			Stop::Timeout(limit) => {
				let limit_ms = limit.as_millis();
				write!(f, "timeout over {limit_ms}ms")
			}
			Stop::BotExited => write!(f, "bot exited"),
		}
	}
}

#[derive(Default)]
struct Outcome {
	score: u32,
	turns: u32,
	first_turn: Duration,
	slowest_turn: Duration,
	stop: Option<(u32, Stop)>,
}

struct Bot {
	child: Child,
	stdin: ChildStdin,
	lines: Receiver<String>,
}

impl Bot {
	fn spawn(command: &[String]) -> Bot {
		let mut child = Command::new(&command[0])
			.args(&command[1..])
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.stderr(Stdio::inherit())
			.spawn()
			.unwrap_or_else(|e| panic!("cannot start {program:?}: {e}", program = command[0]));
		let stdin = child.stdin.take().unwrap();
		let stdout = BufReader::new(child.stdout.take().unwrap());
		let (sender, lines) = mpsc::channel();
		thread::spawn(move || {
			for line in stdout.lines().map_while(Result::ok) {
				if sender.send(line).is_err() {
					break;
				}
			}
		});
		Bot {
			child,
			stdin,
			lines,
		}
	}

	fn ask(&mut self, input: &str, limit: Duration) -> Result<(String, Duration), Stop> {
		let asked = Instant::now();
		self.stdin
			.write_all(input.as_bytes())
			.and_then(|()| self.stdin.flush())
			.map_err(|_| Stop::BotExited)?;
		match self.lines.recv_timeout(limit) {
			Ok(line) => Ok((line, asked.elapsed())),
			Err(RecvTimeoutError::Timeout) => Err(Stop::Timeout(limit)),
			Err(RecvTimeoutError::Disconnected) => Err(Stop::BotExited),
		}
	}
}

impl Drop for Bot {
	fn drop(&mut self) {
		let _ = self.child.kill();
		let _ = self.child.wait();
	}
}

fn finish(mut game: Game, mut outcome: Outcome) -> Outcome {
	outcome.turns = game.turn();
	game.burn_out();
	outcome.score = game.remaining_value();
	outcome
}

fn play(map: &Map, command: &[String]) -> Outcome {
	let mut bot = Bot::spawn(command);
	let mut game = Game::new(map);
	let mut outcome = Outcome::default();
	while !game.is_over() {
		let first_turn = game.turn() == 0;
		let (input, limit) = if first_turn {
			(map.init_input() + &game.turn_input(), FIRST_TURN_LIMIT)
		} else {
			(game.turn_input(), TURN_LIMIT)
		};
		let played = bot.ask(&input, limit).and_then(|(line, elapsed)| {
			if first_turn {
				outcome.first_turn = elapsed;
			} else {
				outcome.slowest_turn = outcome.slowest_turn.max(elapsed);
			}
			Action::parse(&line, map)
				.and_then(|action| game.play(action))
				.map_err(Stop::Rule)
		});
		if let Err(stop) = played {
			outcome.stop = Some((game.turn() + 1, stop));
			break;
		}
	}
	finish(game, outcome)
}

fn replay(map: &Map, actions: &str) -> Outcome {
	let mut game = Game::new(map);
	let mut outcome = Outcome::default();
	for line in actions.lines() {
		if game.is_over() {
			break;
		}
		if let Err(fault) = Action::parse(line, map).and_then(|action| game.play(action)) {
			outcome.stop = Some((game.turn() + 1, Stop::Rule(fault)));
			break;
		}
	}
	finish(game, outcome)
}

fn load_map(path: &Path) -> Map {
	let shown = path.display();
	let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{shown}: {e}"));
	Map::parse(&text).unwrap_or_else(|e| panic!("{shown}: {e}"))
}

fn default_tests() -> Vec<PathBuf> {
	let mut tests = fs::read_dir(DEFAULT_TEST_DIR)
		.unwrap_or_else(|e| panic!("{DEFAULT_TEST_DIR}: {e}"))
		.map(|entry| entry.unwrap().path())
		.filter(|path| path.extension().is_some_and(|ext| ext == "txt"))
		.collect::<Vec<_>>();
	tests.sort();
	tests
}

fn report(name: &str, outcome: &Outcome) {
	let stop = outcome
		.stop
		.as_ref()
		.map(|(turn, stop)| format!("  stopped turn {turn}: {stop}"))
		.unwrap_or_default();
	println!(
		"{name:<40} {score:>6}  turns {turns:>4}  first {first_ms:>7.1}ms  max {slowest_ms:>6.1}ms{stop}",
		score = outcome.score,
		turns = outcome.turns,
		first_ms = outcome.first_turn.as_secs_f64() * 1e3,
		slowest_ms = outcome.slowest_turn.as_secs_f64() * 1e3,
	);
}

fn main() -> ExitCode {
	let args = env::args().skip(1).collect::<Vec<_>>();
	match args.first().map(String::as_str) {
		Some("play") => {
			let Some(separator) = args
				.iter()
				.position(|a| a == "--")
				.filter(|&i| i + 1 < args.len())
			else {
				return ExitCode::from(2);
			};
			let tests = match &args[1..separator] {
				[] => default_tests(),
				paths => paths.iter().map(PathBuf::from).collect(),
			};
			let command = &args[separator + 1..];
			let mut total = 0;
			for test in &tests {
				let outcome = play(&load_map(test), command);
				report(&test.file_stem().unwrap().to_string_lossy(), &outcome);
				total += outcome.score;
			}
			println!("total {total}");
		}
		Some("replay") if args.len() == 3 => {
			let actions = fs::read_to_string(&args[2])
				.unwrap_or_else(|e| panic!("{path}: {e}", path = args[2]));
			report(&args[1], &replay(&load_map(Path::new(&args[1])), &actions));
		}
		_ => return ExitCode::from(2),
	}
	ExitCode::SUCCESS
}
