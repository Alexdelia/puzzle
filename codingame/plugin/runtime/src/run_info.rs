use std::fs::{self, OpenOptions};

use anyhow::{Context, Result};
use clap::ValueEnum;
use nix::fcntl::{Flock, FlockArg};
use serde::{Deserialize, Serialize};

use crate::atomic::write_atomic;
use crate::paths::RunDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
	ContestLive,
	PastPuzzle,
	Optimization,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunInfo {
	pub puzzle: String,
	pub mode: Mode,
	pub deadline_unix: Option<i64>,
	pub active_budget_seconds: Option<u64>,
	pub active_seconds: u64,
	pub phase: String,
	pub plugin_commit: String,
	pub started_at: i64,
}

pub fn read_run_info(run: &RunDir) -> Result<RunInfo> {
	let path = run.run_json();
	let bytes = fs::read(&path).with_context(|| format!("read {path:?}"))?;
	serde_json::from_slice(&bytes).with_context(|| format!("parse {path:?}"))
}

pub fn update_run_info(
	run: &RunDir,
	edit: impl FnOnce(Result<RunInfo>) -> Result<RunInfo>,
) -> Result<RunInfo> {
	let lock_path = run.run_json_lock();
	let lock_file = OpenOptions::new()
		.write(true)
		.create(true)
		.truncate(false)
		.open(&lock_path)
		.with_context(|| format!("open {lock_path:?}"))?;
	let _lock = Flock::lock(lock_file, FlockArg::LockExclusive)
		.map_err(|(_, errno)| errno)
		.with_context(|| format!("lock {lock_path:?}"))?;
	let info = edit(read_run_info(run))?;
	write_atomic(&run.run_json(), &serde_json::to_vec_pretty(&info)?)?;
	Ok(info)
}
