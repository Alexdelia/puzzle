use clap::ValueEnum;
use serde::{Deserialize, Serialize};

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
