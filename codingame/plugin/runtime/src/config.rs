use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
	pub workers: usize,
	pub grace_seconds: u64,
	pub heartbeat_minutes: u64,
	pub five_hour_limit_percent: u8,
	pub five_hour_task_reserve_percent: u8,
	pub weekly_pace_margin_percent: u8,
	pub usage_max_age_minutes: u64,
	pub digest_max_bytes: usize,
	pub bash_timeout_ms: u64,
	pub ram_budget_mb: u64,
	pub blocked_url_substrings: Vec<String>,
	pub toolbox: Vec<PathBuf>,
	pub plugin_dir: PathBuf,
	pub manager_model: String,
	pub worker_model: String,
	pub manager_budget_usd: f64,
	pub worker_budget_usd: f64,
	pub claude_bin: PathBuf,
	pub sandbox: bool,
	pub commit_trailer: String,
	pub repo_root: PathBuf,
	pub git_common_dir: PathBuf,
	pub pasta_bin: PathBuf,
	pub probe_timeout_seconds: u64,
	pub manager_timeout_seconds: u64,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			workers: 3,
			grace_seconds: 270,
			heartbeat_minutes: 20,
			five_hour_limit_percent: 97,
			five_hour_task_reserve_percent: 3,
			weekly_pace_margin_percent: 0,
			usage_max_age_minutes: 30,
			digest_max_bytes: 32000,
			bash_timeout_ms: 120000,
			ram_budget_mb: 13000,
			blocked_url_substrings: ["alexdelia", "discord.com", "discord.gg"]
				.map(String::from)
				.to_vec(),
			toolbox: Vec::new(),
			plugin_dir: default_plugin_dir(),
			manager_model: "opus".into(),
			worker_model: "opus".into(),
			manager_budget_usd: 1.0,
			worker_budget_usd: 10.0,
			claude_bin: "claude".into(),
			sandbox: true,
			commit_trailer: String::new(),
			repo_root: PathBuf::new(),
			git_common_dir: PathBuf::new(),
			pasta_bin: PathBuf::new(),
			probe_timeout_seconds: 120,
			manager_timeout_seconds: 900,
		}
	}
}

impl Config {
	pub fn load(path: &Path) -> anyhow::Result<Self> {
		let text = fs::read_to_string(path).with_context(|| format!("read {path:?}"))?;
		toml::from_str(&text).with_context(|| format!("parse {path:?}"))
	}
}

pub fn default_plugin_dir() -> PathBuf {
	std::env::current_exe()
		.ok()
		.and_then(|exe| {
			exe.ancestors()
				.find(|dir| dir.join("runtime/Cargo.toml").is_file())
				.map(Path::to_path_buf)
		})
		.unwrap_or_default()
}
