use std::fs;
use std::path::PathBuf;
use std::process::Child;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::agent::{AgentRun, RateLimitInfo, Window, parse_stream, spawn_agent};
use crate::atomic::write_atomic;
use crate::config::Config;
use crate::paths::RunDir;

const WEEK_SECONDS: i64 = 604800;
const PROBE_MODEL: &str = "haiku";
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

impl Probe {
	pub fn finish(self) -> Option<RateLimitInfo> {
		let info = parse_stream(&self.stream_log).rate_limit;
		let _ = fs::remove_file(&self.stream_log);
		let _ = fs::remove_file(self.stream_log.with_extension("stderr.log"));
		info
	}
}

pub fn spawn_probe(config: &Config, stream_log: PathBuf) -> anyhow::Result<Probe> {
	let run = AgentRun {
		prompt: "ok".into(),
		model: PROBE_MODEL.into(),
		budget_usd: PROBE_BUDGET_USD,
		cwd: std::env::temp_dir(),
		allowed_tools: None,
		stream_log,
		env: Vec::new(),
	};
	let child = spawn_agent(config, &run, None)?;
	Ok(Probe {
		child,
		stream_log: run.stream_log,
	})
}
