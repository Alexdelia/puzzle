use clap::{Parser, Subcommand};
use std::fmt;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};
use std::{fs, thread};
use vehicle_routing_problem::{Fault, Instance, score_line};

const TIME_LIMIT: Duration = Duration::from_secs(10);
const DEFAULT_TEST_DIR: &str = "validator/test";

enum Stop {
	Rule(Fault),
	Timeout,
	BotExited,
}

impl fmt::Display for Stop {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Stop::Rule(fault) => write!(f, "{fault}"),
			Stop::Timeout => write!(f, "timeout over {limit}s", limit = TIME_LIMIT.as_secs()),
			Stop::BotExited => write!(f, "bot exited"),
		}
	}
}

struct Outcome {
	score: Result<u32, Stop>,
	elapsed: Duration,
}

struct Bot(Child);

impl Drop for Bot {
	fn drop(&mut self) {
		let _ = self.0.kill();
		let _ = self.0.wait();
	}
}

fn answer(command: &[String], input: &str) -> Result<String, Stop> {
	let mut bot = Bot(Command::new(&command[0])
		.args(&command[1..])
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::inherit())
		.spawn()
		.unwrap_or_else(|e| panic!("cannot start {program:?}: {e}", program = command[0])));
	let mut stdin = bot.0.stdin.take().unwrap();
	let stdout = BufReader::new(bot.0.stdout.take().unwrap());
	let (sender, lines) = mpsc::channel();
	thread::spawn(move || {
		if let Some(line) = stdout.lines().map_while(Result::ok).next() {
			let _ = sender.send(line);
		}
	});
	stdin
		.write_all(input.as_bytes())
		.and_then(|()| stdin.flush())
		.map_err(|_| Stop::BotExited)?;
	match lines.recv_timeout(TIME_LIMIT) {
		Ok(line) => Ok(line),
		Err(RecvTimeoutError::Timeout) => Err(Stop::Timeout),
		Err(RecvTimeoutError::Disconnected) => Err(Stop::BotExited),
	}
}

fn play(instance: &Instance, command: &[String]) -> Outcome {
	let started = Instant::now();
	let line = answer(command, &instance.input());
	let elapsed = started.elapsed();
	let score = line.and_then(|line| score_line(instance, &line).map_err(Stop::Rule));
	Outcome { score, elapsed }
}

fn load_instance(path: &Path) -> Instance {
	let shown = path.display();
	let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{shown}: {e}"));
	Instance::parse(&text).unwrap_or_else(|e| panic!("{shown}: {e}"))
}

fn txt_files(dir: &Path) -> Vec<PathBuf> {
	let mut files = fs::read_dir(dir)
		.unwrap_or_else(|e| panic!("{dir}: {e}", dir = dir.display()))
		.map(|entry| entry.unwrap().path())
		.filter(|path| path.extension().is_some_and(|ext| ext == "txt"))
		.collect::<Vec<_>>();
	files.sort();
	files
}

fn test_paths(paths: &[PathBuf]) -> Vec<PathBuf> {
	paths
		.iter()
		.flat_map(|path| {
			if path.is_dir() {
				txt_files(path)
			} else {
				vec![path.clone()]
			}
		})
		.collect()
}

fn report(name: &str, outcome: &Outcome) {
	let result = match &outcome.score {
		Ok(score) => format!("{score:>7}"),
		Err(stop) => format!("   FAIL  {stop}"),
	};
	println!(
		"{name:<40} {elapsed_ms:>8.1}ms {result}",
		elapsed_ms = outcome.elapsed.as_secs_f64() * 1e3,
	);
}

fn play_all(tests: &[PathBuf], command: &[String]) -> bool {
	let mut total = 0;
	let mut failed = 0;
	for test in tests {
		let outcome = play(&load_instance(test), command);
		report(&test.file_stem().unwrap().to_string_lossy(), &outcome);
		match outcome.score {
			Ok(score) => total += score,
			Err(_) => failed += 1,
		}
	}
	println!(
		"total {total}  failed {failed}/{count}",
		count = tests.len()
	);
	failed == 0
}

fn score_file(test: &Path, answer: &Path) -> bool {
	let instance = load_instance(test);
	let shown = answer.display();
	let line = fs::read_to_string(answer).unwrap_or_else(|e| panic!("{shown}: {e}"));
	match score_line(&instance, line.lines().next().unwrap_or_default()) {
		Ok(score) => {
			println!("{shown} {score}");
			true
		}
		Err(fault) => {
			println!("{shown} FAIL {fault}");
			false
		}
	}
}

#[derive(Parser)]
#[command(about = "Vehicle Routing Problem referee")]
struct Cli {
	#[command(subcommand)]
	action: Action,
}

#[derive(Subcommand)]
enum Action {
	#[command(about = "Run a bot on each test, 10 s per test")]
	Play {
		#[arg(value_name = "TEST|DIR", default_value = DEFAULT_TEST_DIR)]
		tests: Vec<PathBuf>,
		#[arg(last = true, required = true, value_name = "COMMAND")]
		command: Vec<String>,
	},
	#[command(about = "Score the first line of an answer file")]
	Score { test: PathBuf, answer: PathBuf },
}

fn main() -> ExitCode {
	let passed = match Cli::parse().action {
		Action::Play { tests, command } => play_all(&test_paths(&tests), &command),
		Action::Score { test, answer } => score_file(&test, &answer),
	};
	if passed {
		ExitCode::SUCCESS
	} else {
		ExitCode::FAILURE
	}
}
