use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use crate::compare;
use crate::process::{ReadError, Solution};
use crate::suite::{Case, RunOptions, Runner, Verdict};

pub struct IoPuzzle {
	pub limit: Duration,
}

impl Default for IoPuzzle {
	fn default() -> Self {
		Self {
			limit: Duration::from_millis(6000),
		}
	}
}

impl Runner for IoPuzzle {
	const EXTENSION: &'static str = "in";

	fn run(&self, case: &Case, launcher: &Path, options: &RunOptions) -> Result<Verdict> {
		let input = fs::read_to_string(&case.file)
			.with_context(|| format!("read {file}", file = case.file.display()))?;
		let expected_file = case.file.with_extension("out");
		let expected = fs::read_to_string(&expected_file)
			.with_context(|| format!("read {file}", file = expected_file.display()))?;
		let limit = self.limit.min(options.hard_cap);
		let mut solution = Solution::spawn(launcher, options.show_stderr)?;
		let deadline = Instant::now() + limit;
		solution.write_all_then_close(input);
		let mut actual = Vec::new();
		loop {
			match solution.read_line(deadline) {
				Ok(line) => actual.push(line),
				Err(ReadError::Closed) => break,
				Err(ReadError::Timeout) => {
					let limit_ms = limit.as_millis();
					return Ok(Verdict::Timeout(format!(
						"no end of output within {limit_ms} ms"
					)));
				}
			}
		}
		solution.kill();
		Ok(
			match compare::first_difference(&compare::lines(&expected), &actual) {
				None => Verdict::Pass,
				Some(difference) => Verdict::Fail(difference),
			},
		)
	}
}
