use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Child;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::agent::{AgentRun, RateLimitInfo, StreamOutcome, Window, parse_stream, spawn_agent};
use crate::atomic::write_atomic;
use crate::config::Config;
use crate::paths::RunDir;

const WEEK_SECONDS: i64 = 604800;
pub const PROBE_MODEL: &str = "haiku";
const PROBE_BUDGET_USD: f64 = 0.05;

#[derive(Serialize, Deserialize)]
struct UsageRecord {
	#[serde(flatten)]
	info: RateLimitInfo,
	recorded_at: i64,
}

pub fn record_usage(run: &RunDir, info: &RateLimitInfo, now: i64) -> anyhow::Result<()> {
	let record = UsageRecord {
		info: info.clone(),
		recorded_at: now,
	};
	let path = run.usage();
	write_atomic(&path, &serde_json::to_vec(&record)?).with_context(|| format!("write {path:?}"))
}

pub fn load_usage(run: &RunDir) -> Option<(RateLimitInfo, i64)> {
	let text = fs::read_to_string(run.usage()).ok()?;
	let record: UsageRecord = serde_json::from_str(&text).ok()?;
	Some((record.info, record.recorded_at))
}

pub const NO_LAUNCHES_NOTE: &str = "no launches until a usage probe succeeds";
const SECONDS_PER_MINUTE: i64 = 60;

pub fn is_fresh_sample(recorded_at: i64, now: i64, config: &Config) -> bool {
	now - recorded_at <= config.usage_max_age_minutes as i64 * SECONDS_PER_MINUTE
}

pub fn fresh_usage(run: &RunDir, config: &Config, now: i64) -> Option<RateLimitInfo> {
	load_usage(run)
		.filter(|(_, recorded_at)| is_fresh_sample(*recorded_at, now, config))
		.map(|(info, _)| info)
}

fn percent(window: &Window) -> i64 {
	(window.utilization * 100.0 + 1e-9).floor() as i64
}

pub fn five_hour_used_percent(info: &RateLimitInfo, now: i64) -> u8 {
	if info.five_hour.resets_at > now {
		percent(&info.five_hour).clamp(0, 255) as u8
	} else {
		0
	}
}

pub fn seven_day_used_percent(info: &RateLimitInfo, now: i64) -> u8 {
	if info.seven_day.resets_at > now {
		percent(&info.seven_day).clamp(0, 255) as u8
	} else {
		0
	}
}

pub fn five_hour_full(info: &RateLimitInfo, now: i64, running: usize, config: &Config) -> bool {
	let reserved = i64::from(config.five_hour_task_reserve_percent) * (running as i64 + 1);
	i64::from(five_hour_used_percent(info, now)) + reserved
		> i64::from(config.five_hour_limit_percent)
}

pub fn over_weekly_pace(info: &RateLimitInfo, now: i64, config: &Config) -> bool {
	let elapsed_percent = 100 - (info.seven_day.resets_at - now) * 100 / WEEK_SECONDS;
	percent(&info.seven_day) + i64::from(config.weekly_pace_margin_percent) >= elapsed_percent
}

pub fn rejected_until(info: &RateLimitInfo, now: i64) -> Option<i64> {
	(info.status == "rejected" && info.resets_at > now).then_some(info.resets_at)
}

pub struct Probe {
	pub child: Child,
	stream_log: PathBuf,
}

pub struct ProbeEnd {
	pub outcome: StreamOutcome,
	pub cleanup_failures: Vec<String>,
}

fn remove_probe_file(path: &Path) -> Option<String> {
	match fs::remove_file(path) {
		Err(error) if error.kind() != io::ErrorKind::NotFound => {
			Some(format!("remove {path:?}: {error}"))
		}
		_ => None,
	}
}

impl Probe {
	pub fn finish(self) -> ProbeEnd {
		let outcome = parse_stream(&self.stream_log);
		let cleanup_failures = [
			self.stream_log.clone(),
			self.stream_log.with_extension("stderr.log"),
		]
		.iter()
		.filter_map(|path| remove_probe_file(path))
		.collect();
		ProbeEnd {
			outcome,
			cleanup_failures,
		}
	}
}

pub fn spawn_probe(config: &Config, stream_log: PathBuf) -> anyhow::Result<Probe> {
	let run = AgentRun {
		prompt: "ok".into(),
		model: PROBE_MODEL.into(),
		budget_usd: PROBE_BUDGET_USD,
		cwd: std::env::temp_dir(),
		tools: None,
		stream_log,
		env: Vec::new(),
	};
	let child = spawn_agent(config, &run, None)?;
	Ok(Probe {
		child,
		stream_log: run.stream_log,
	})
}
