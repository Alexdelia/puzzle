use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;

use crate::lang::{self, Limits};
use crate::process::{self, Usage};

#[derive(Parser)]
struct Args {
	launcher: PathBuf,
	#[arg(long, default_value = ".")]
	root: PathBuf,
	#[arg(long)]
	validators: bool,
	#[arg(long)]
	hidden: bool,
	#[arg(long)]
	bench: Option<usize>,
	#[arg(long)]
	stderr: bool,
	#[arg(long)]
	trace: bool,
	#[arg(long)]
	usage: bool,
	#[arg(long)]
	baseline: Option<PathBuf>,
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
	pub limits: Limits,
	pub io_cpu_bar: Option<Duration>,
}

pub struct Played {
	pub verdict: Verdict,
	pub turns: usize,
	pub usage: Usage,
	pub first_turn_cpu: Option<Duration>,
}

impl Played {
	pub fn new(verdict: Verdict, turns: usize, usage: Usage) -> Self {
		Self {
			verdict,
			turns,
			usage,
			first_turn_cpu: None,
		}
	}

	fn within(self, options: &RunOptions, interactive: bool) -> Self {
		if !matches!(self.verdict, Verdict::Pass) {
			return self;
		}
		let peak_kb = self.usage.peak_rss_kb;
		let memory_kb = options.limits.memory_kb;
		let cpu_ms = self.usage.cpu.as_millis();
		let verdict = match options.io_cpu_bar {
			_ if peak_kb > memory_kb => {
				Verdict::Fail(format!("memory {peak_kb} kB over {memory_kb} kB"))
			}
			Some(bar) if !interactive && self.usage.cpu > bar => {
				let bar_ms = bar.as_millis();
				Verdict::Timeout(format!("solution cpu {cpu_ms} ms over {bar_ms} ms"))
			}
			_ => Verdict::Pass,
		};
		Self { verdict, ..self }
	}
}

pub trait Runner {
	const EXTENSION: &'static str;
	const INTERACTIVE: bool = false;

	fn run(&self, case: &Case, launcher: &Path, options: &RunOptions) -> Result<Played>;
}

const REFEREE_BASE_BUDGET: Duration = Duration::from_millis(2);
const REFEREE_TURN_BUDGET: Duration = Duration::from_micros(20);
const SUITE_WALL_BUDGET: Duration = Duration::from_secs(5);

fn referee_budget(turns: usize) -> Duration {
	REFEREE_BASE_BUDGET + REFEREE_TURN_BUDGET * u32::try_from(turns).unwrap_or(u32::MAX)
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
) -> Result<Played> {
	let played = run_within(runner, case, launcher, options)?;
	match played.verdict {
		Verdict::Timeout(_) => run_within(runner, case, launcher, options),
		_ => Ok(played),
	}
}

fn run_within<R: Runner>(
	runner: &R,
	case: &Case,
	launcher: &Path,
	options: &RunOptions,
) -> Result<Played> {
	runner
		.run(case, launcher, options)
		.map(|played| played.within(options, R::INTERACTIVE))
}

fn usage_columns(played: &Played) -> String {
	let cpu_ms = played.usage.cpu.as_millis();
	let peak_kb = played.usage.peak_rss_kb;
	let first_turn = played
		.first_turn_cpu
		.map(|first| {
			format!(
				" first_turn_cpu_ms {first_ms}",
				first_ms = first.as_millis()
			)
		})
		.unwrap_or_default();
	format!(" (cpu_ms {cpu_ms} peak_rss_kb {peak_kb}{first_turn})")
}

fn report(name: &str, played: &Result<Played>, show_usage: bool) -> bool {
	let usage = match played {
		Ok(played) if show_usage => usage_columns(played),
		_ => String::new(),
	};
	report_verdict(name, played, &usage)
}

fn report_verdict(name: &str, played: &Result<Played>, usage: &str) -> bool {
	match played.as_ref().map(|played| &played.verdict) {
		Ok(Verdict::Pass) => {
			println!("test {name} pass{usage}");
			true
		}
		Ok(Verdict::Fail(detail)) => {
			println!("test {name} fail {detail}{usage}");
			false
		}
		Ok(Verdict::Timeout(detail)) => {
			println!("test {name} timeout {detail}{usage}");
			false
		}
		Err(error) => {
			println!("test {name} error {error:#}");
			false
		}
	}
}

fn verify<R: Runner>(
	runner: &R,
	cases: &[Case],
	launcher: &Path,
	options: &RunOptions,
	show_usage: bool,
) -> bool {
	let total = cases.len();
	let passed = cases
		.iter()
		.filter(|case| {
			report(
				&case.name,
				&run_confirmed(runner, case, launcher, options),
				show_usage,
			)
		})
		.count();
	println!("passed {passed}/{total}");
	passed == total
}

struct Sample {
	turns: usize,
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
			let played = run_within(runner, case, launcher, options);
			let wall = started.elapsed();
			all_passed &= matches!(
				&played,
				Ok(Played {
					verdict: Verdict::Pass,
					..
				})
			);
			case_samples.push(Sample {
				turns: played.as_ref().map_or(0, |played| played.turns),
				wall,
				solution_cpu: process::solutions_cpu().saturating_sub(solution_before),
				referee_cpu: process::referee_cpu().saturating_sub(referee_before),
			});
		}
	}
	println!(
		"case turns wall_ms solution_cpu_ms referee_cpu_ms referee_budget_ms gate (median of {runs})"
	);
	let mut gate_passed = true;
	for (case, case_samples) in cases.iter().zip(&samples) {
		let name = &case.name;
		let turns = case_samples
			.iter()
			.map(|sample| sample.turns)
			.max()
			.unwrap_or(0);
		let referee_cpu_median = median(
			case_samples
				.iter()
				.map(|sample| sample.referee_cpu)
				.collect(),
		);
		let budget = referee_budget(turns);
		let within_budget = referee_cpu_median <= budget;
		gate_passed &= within_budget;
		let gate = if within_budget { "ok" } else { "over" };
		let budget_ms = milliseconds(budget);
		let wall = milliseconds(median(
			case_samples.iter().map(|sample| sample.wall).collect(),
		));
		let solution_cpu = milliseconds(median(
			case_samples
				.iter()
				.map(|sample| sample.solution_cpu)
				.collect(),
		));
		let referee_cpu = milliseconds(referee_cpu_median);
		println!(
			"{name} {turns} {wall:.1} {solution_cpu:.1} {referee_cpu:.1} {budget_ms:.1} {gate}"
		);
	}
	let per_run = |pick: fn(&Sample) -> Duration| {
		let total: Duration = samples.iter().flatten().map(pick).sum();
		milliseconds(total) / runs as f64
	};
	let wall = per_run(|sample| sample.wall);
	let solution_cpu = per_run(|sample| sample.solution_cpu);
	let referee_cpu = per_run(|sample| sample.referee_cpu);
	let suite_wall_budget_ms = milliseconds(SUITE_WALL_BUDGET);
	gate_passed &= wall <= suite_wall_budget_ms;
	let gate = if gate_passed { "ok" } else { "over" };
	let passed = all_passed && gate_passed;
	let verdict = if passed { "pass" } else { "fail" };
	println!(
		"suite wall_ms {wall:.1} solution_cpu_ms {solution_cpu:.1} referee_cpu_ms {referee_cpu:.1} gate {gate} {verdict}"
	);
	passed
}

const BASELINE_RUNS: usize = 3;

fn baseline_cpu(launcher: &Path) -> Option<Duration> {
	(0..BASELINE_RUNS)
		.filter_map(|_| {
			let mut solution = process::Solution::spawn(launcher, false).ok()?;
			solution.write_all_then_close(String::new());
			Some(solution.finish_within(Duration::from_secs(10)).cpu)
		})
		.min()
}

pub fn main<R: Runner>(runner: R) -> ExitCode {
	let args = Args::parse();
	let mut sets = vec!["tests"];
	if args.validators {
		sets.push("validators");
	}
	if args.hidden {
		sets.push("hidden");
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
	let limits = lang::limits(lang::of_launcher(&args.launcher).as_deref());
	let io_cpu_bar = match (limits.io_cpu_over_baseline, &args.baseline) {
		(Some(ratio), Some(baseline)) if !R::INTERACTIVE => {
			baseline_cpu(baseline).map(|cpu| cpu.mul_f64(ratio))
		}
		_ => None,
	};
	if let Some(bar) = io_cpu_bar {
		let bar_ms = bar.as_millis();
		println!("solution cpu bar {bar_ms} ms");
	}
	let options = RunOptions {
		show_stderr: args.stderr,
		trace: args.trace,
		hard_cap: Duration::from_millis(args.hard_cap_ms),
		limits,
		io_cpu_bar,
	};
	let passed = match args.bench {
		Some(runs) => bench(&runner, &cases, &args.launcher, &options, runs),
		None => verify(&runner, &cases, &args.launcher, &options, args.usage),
	};
	if passed {
		ExitCode::SUCCESS
	} else {
		ExitCode::FAILURE
	}
}
