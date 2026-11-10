use std::fmt;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TaskId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JobId(pub String);

fn all_digits(text: &str) -> bool {
	text.bytes().all(|b| b.is_ascii_digit())
}

fn digits(text: &str, count: usize) -> bool {
	text.len() == count && all_digits(text)
}

const JOB_ID_MIN_DIGITS: usize = 4;

impl TaskId {
	pub fn parse(text: &str) -> Result<Self, String> {
		let valid = text.strip_prefix('t').is_some_and(|rest| {
			rest.split_once('-').is_some_and(|(number, kind)| {
				digits(number, 4)
					&& !kind.is_empty()
					&& kind
						.bytes()
						.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
			})
		});
		if valid {
			Ok(Self(text.to_owned()))
		} else {
			Err(format!("invalid task id {text:?}"))
		}
	}
}

impl JobId {
	pub fn parse(text: &str) -> Result<Self, String> {
		if text
			.strip_prefix('j')
			.is_some_and(|rest| rest.len() >= JOB_ID_MIN_DIGITS && all_digits(rest))
		{
			Ok(Self(text.to_owned()))
		} else {
			Err(format!("invalid job id {text:?}"))
		}
	}
}

pub fn is_valid_kind(kind: &str) -> bool {
	!kind.is_empty()
		&& kind
			.bytes()
			.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

impl fmt::Display for TaskId {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(&self.0)
	}
}

impl fmt::Display for JobId {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(&self.0)
	}
}

#[derive(Debug, Clone)]
pub struct RunDir {
	pub root: PathBuf,
}

impl RunDir {
	fn run(&self) -> PathBuf {
		self.root.join("run")
	}

	pub fn state(&self) -> PathBuf {
		self.run().join("state.json")
	}

	pub fn run_json(&self) -> PathBuf {
		self.run().join("run.json")
	}

	pub fn config(&self) -> PathBuf {
		self.run().join("config.toml")
	}

	pub fn journal(&self) -> PathBuf {
		self.run().join("journal.md")
	}

	pub fn events(&self) -> PathBuf {
		self.run().join("events.jsonl")
	}

	pub fn manager_cursor(&self) -> PathBuf {
		self.run().join("manager-cursor")
	}

	pub fn ledger(&self) -> PathBuf {
		self.run().join("ledger.tsv")
	}

	pub fn usage(&self) -> PathBuf {
		self.run().join("usage.json")
	}

	pub fn status(&self) -> PathBuf {
		self.run().join("status.md")
	}

	pub fn stop_file(&self) -> PathBuf {
		self.run().join("STOP")
	}

	pub fn inbox(&self) -> PathBuf {
		self.run().join("inbox")
	}

	pub fn requests(&self) -> PathBuf {
		self.run().join("requests")
	}

	pub fn tasks(&self) -> PathBuf {
		self.run().join("tasks")
	}

	pub fn task_requests(&self, id: &TaskId) -> PathBuf {
		self.requests().join(&id.0)
	}

	pub fn task_dir(&self, id: &TaskId) -> PathBuf {
		self.tasks().join(&id.0)
	}

	pub fn jobs(&self) -> PathBuf {
		self.run().join("jobs")
	}

	pub fn job_dir(&self, id: &JobId) -> PathBuf {
		self.jobs().join(&id.0)
	}

	pub fn agents(&self) -> PathBuf {
		self.run().join("agents")
	}

	pub fn claude_config(&self) -> PathBuf {
		self.run().join("claude-config")
	}

	pub fn supervisor_pid(&self) -> PathBuf {
		self.run().join("supervisor.pid")
	}

	pub fn replies(&self) -> PathBuf {
		self.run().join("replies.md")
	}

	pub fn repo_root(&self) -> io::Result<PathBuf> {
		let output = Command::new("git")
			.arg("-C")
			.arg(&self.root)
			.args(["rev-parse", "--show-toplevel"])
			.output()?;
		if !output.status.success() {
			return Err(io::Error::other("not inside a git repository"));
		}
		Ok(PathBuf::from(
			String::from_utf8_lossy(&output.stdout).trim(),
		))
	}
}
