use std::fs;
use std::marker::PhantomData;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use referee_core::process::{ReadError, Solution};
use referee_core::suite::{Case, Played, RunOptions, Runner, Verdict};
use referee_core::{Game, Step};

const ORPHAN_GRACE: Duration = Duration::from_millis(50);

pub struct ClosedStdin<G>(PhantomData<G>);

impl<G> Default for ClosedStdin<G> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<G: Game> Runner for ClosedStdin<G> {
	const EXTENSION: &'static str = "json";
	const INTERACTIVE: bool = true;

	fn run(&self, case: &Case, launcher: &Path, options: &RunOptions) -> Result<Played> {
		let mut turns = 0;
		let mut first_turn_cpu = None;
		let mut solution = Solution::spawn(launcher, options.show_stderr)?
			.kill_orphans_after_main_exit(ORPHAN_GRACE);
		let verdict = Self::play(
			case,
			launcher,
			&mut solution,
			options,
			&mut turns,
			&mut first_turn_cpu,
		)?;
		let usage = solution.stop();
		Ok(Played {
			first_turn_cpu,
			..Played::new(verdict, turns, usage)
		})
	}
}

fn first_turn_limit<G: Game>(launcher: &Path) -> Duration {
	match referee_core::lang::of_launcher(launcher).as_deref() {
		Some("clojure" | "groovy" | "java" | "kotlin" | "scala") => Duration::from_millis(1500),
		_ => G::FIRST_TURN,
	}
}

impl<G: Game> ClosedStdin<G> {
	fn play(
		case: &Case,
		launcher: &Path,
		solution: &mut Solution,
		options: &RunOptions,
		turn: &mut usize,
		first_turn_cpu: &mut Option<Duration>,
	) -> Result<Verdict> {
		let text = fs::read_to_string(&case.file)
			.with_context(|| format!("read {file}", file = case.file.display()))?;
		let test: G::Test = serde_json::from_str(&text)
			.with_context(|| format!("parse {file}", file = case.file.display()))?;
		let mut game = G::new(test);
		let started = Instant::now();
		let mut input = game.initial_input() + &game.turn_input();
		let mut limit = first_turn_limit::<G>(launcher);
		*turn = 1;
		loop {
			if started.elapsed() > options.hard_cap {
				return Ok(Verdict::Timeout(format!("turn {turn}: hard cap reached")));
			}
			if options.trace {
				eprint!("{input}");
			}
			solution.write(&input);
			let deadline = Instant::now() + limit;
			let mut action = Vec::with_capacity(game.action_lines());
			for _ in 0..game.action_lines() {
				match solution.read_line(deadline) {
					Ok(line) => action.push(line),
					Err(ReadError::Timeout) => {
						let limit_ms = limit.as_millis();
						return Ok(Verdict::Timeout(format!(
							"turn {turn}: no answer within {limit_ms} ms"
						)));
					}
					Err(ReadError::Closed) => {
						return Ok(Verdict::Fail(format!("turn {turn}: solution exited")));
					}
				}
			}
			if *turn == 1 {
				*first_turn_cpu = Some(solution.live_usage().cpu);
			}
			if options.trace {
				for line in &action {
					eprintln!("> {line}");
				}
			}
			match game.play(&action) {
				Step::Win => return Ok(Verdict::Pass),
				Step::Lose(reason) => return Ok(Verdict::Fail(format!("turn {turn}: {reason}"))),
				Step::Continue => {
					input = game.turn_input();
					limit = G::TURN;
					*turn += 1;
				}
			}
		}
	}
}
