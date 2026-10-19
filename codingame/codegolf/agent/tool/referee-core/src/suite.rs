use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;

use crate::process;

#[derive(Parser)]
struct Args {
	launcher: PathBuf,
	#[arg(long, default_value = ".")]
	root: PathBuf,
	#[arg(long)]
	validators: bool,
	#[arg(long)]
	bench: Option<usize>,
	#[arg(long)]
	stderr: bool,
	#[arg(long)]
	trace: bool,
	#[arg(long, default_value_t = 60_000)]
	hard_cap_ms: u64,
	names: Vec<String>,
}

pub enum Verdict {
	Pass,
	Fail(String),
	Timeout(String),
}

pub struct Case {
	pub name: String,
	pub file: PathBuf,
}

pub struct RunOptions {
	pub show_stderr: bool,
	pub trace: bool,
	pub hard_cap: Duration,
}

pub trait Runner {
	const EXTENSION: &'static str;

	fn run(&self, case: &Case, launcher: &Path, options: &RunOptions) -> Result<Verdict>;
}

fn discover(root: &Path, set: &str, extension: &str) -> Vec<Case> {
	let Ok(entries) = fs::read_dir(root.join(set)) else {
		return Vec::new();
	};
	let mut cases: Vec<Case> = entries
		.filter_map(Result::ok)
		.map(|entry| entry.path())
		.filter(|file| file.extension().is_some_and(|found| found == extension))
		.filter_map(|file| {
			let stem = file.file_stem()?.to_str()?.to_owned();
			Some(Case {
				name: format!("{set}/{stem}"),
				file,
			})
		})
		.collect();
	cases.sort_by(|left, right| left.name.cmp(&right.name));
	cases
}

fn selected(case: &Case, names: &[String]) -> bool {
	names.is_empty()
		|| names
			.iter()
			.any(|name| case.name == *name || case.name.ends_with(&format!("/{name}")))
}

fn run_confirmed<R: Runner>(
	runner: &R,
	case: &Case,
	launcher: &Path,
	options: &RunOptions,
) -> Result<Verdict> {
	match runner.run(case, launcher, options)? {
		Verdict::Timeout(_) => runner.run(case, launcher, options),
		verdict => Ok(verdict),
	}
}

fn report(name: &str, verdict: &Result<Verdict>) -> bool {
	match verdict {
		Ok(Verdict::Pass) => {
			println!("test {name} pass");
			true
		}
		Ok(Verdict::Fail(detail)) => {
			println!("test {name} fail {detail}");
			false
		}
		Ok(Verdict::Timeout(detail)) => {
			println!("test {name} timeout {detail}");
			false
		}
		Err(error) => {
			println!("test {name} error {error:#}");
			false
		}
	}
}

fn verify<R: Runner>(runner: &R, cases: &[Case], launcher: &Path, options: &RunOptions) -> bool {
	let total = cases.len();
	let passed = cases
		.iter()
		.filter(|case| report(&case.name, &run_confirmed(runner, case, launcher, options)))
		.count();
	println!("passed {passed}/{total}");
	passed == total
}

struct Sample {
	wall: Duration,
	solution_cpu: Duration,
	referee_cpu: Duration,
}

fn milliseconds(duration: Duration) -> f64 {
	duration.as_secs_f64() * 1000.0
}

fn median(mut values: Vec<Duration>) -> Duration {
	values.sort();
	values[values.len() / 2]
}

fn bench<R: Runner>(
	runner: &R,
	cases: &[Case],
	launcher: &Path,
	options: &RunOptions,
	runs: usize,
) -> bool {
	let runs = runs.max(1);
	let mut samples: Vec<Vec<Sample>> = cases.iter().map(|_| Vec::with_capacity(runs)).collect();
	let mut all_passed = true;
	for _ in 0..runs {
		for (case, case_samples) in cases.iter().zip(&mut samples) {
			let referee_before = process::referee_cpu();
			let solution_before = process::solutions_cpu();
			let started = Instant::now();
			let verdict = runner.run(case, launcher, options);
			let wall = started.elapsed();
			all_passed &= matches!(verdict, Ok(Verdict::Pass));
			case_samples.push(Sample {
				wall,
				solution_cpu: process::solutions_cpu().saturating_sub(solution_before),
				referee_cpu: process::referee_cpu().saturating_sub(referee_before),
			});
		}
	}
	println!("case wall_ms solution_cpu_ms referee_cpu_ms (median of {runs})");
	for (case, case_samples) in cases.iter().zip(&samples) {
		let name = &case.name;
		let wall = milliseconds(median(
			case_samples.iter().map(|sample| sample.wall).collect(),
		));
		let solution_cpu = milliseconds(median(
			case_samples
				.iter()
				.map(|sample| sample.solution_cpu)
				.collect(),
		));
		let referee_cpu = milliseconds(median(
			case_samples
				.iter()
				.map(|sample| sample.referee_cpu)
				.collect(),
		));
		println!("{name} {wall:.1} {solution_cpu:.1} {referee_cpu:.1}");
	}
	let per_run = |pick: fn(&Sample) -> Duration| {
		let total: Duration = samples.iter().flatten().map(pick).sum();
		milliseconds(total) / runs as f64
	};
	let wall = per_run(|sample| sample.wall);
	let solution_cpu = per_run(|sample| sample.solution_cpu);
	let referee_cpu = per_run(|sample| sample.referee_cpu);
	let verdict = if all_passed { "pass" } else { "fail" };
	println!(
		"suite wall_ms {wall:.1} solution_cpu_ms {solution_cpu:.1} referee_cpu_ms {referee_cpu:.1} {verdict}"
	);
	all_passed
}

pub fn main<R: Runner>(runner: R) -> ExitCode {
	let args = Args::parse();
	let mut sets = vec!["tests"];
	if args.validators {
		sets.push("validators");
	}
	let cases: Vec<Case> = sets
		.iter()
		.flat_map(|set| discover(&args.root, set, R::EXTENSION))
		.filter(|case| selected(case, &args.names))
		.collect();
	if cases.is_empty() {
		let root = args.root.display();
		eprintln!("no case found under {root}");
		return ExitCode::FAILURE;
	}
	let options = RunOptions {
		show_stderr: args.stderr,
		trace: args.trace,
		hard_cap: Duration::from_millis(args.hard_cap_ms),
	};
	let passed = match args.bench {
		Some(runs) => bench(&runner, &cases, &args.launcher, &options, runs),
		None => verify(&runner, &cases, &args.launcher, &options),
	};
	if passed {
		ExitCode::SUCCESS
	} else {
		ExitCode::FAILURE
	}
}
