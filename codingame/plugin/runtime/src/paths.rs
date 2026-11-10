use std::fmt;
use std::io;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaskId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JobId(pub String);

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
		self.run().join("journal.jsonl")
	}

	pub fn events(&self) -> PathBuf {
		self.run().join("events.jsonl")
	}

	pub fn manager_cursor(&self) -> PathBuf {
		self.run().join("manager-cursor")
	}

	pub fn ledger(&self) -> PathBuf {
		self.run().join("ledger.jsonl")
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
