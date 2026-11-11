use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use contest::agent::StreamOutcome;
use contest::agent::{RateLimitInfo, Window};
use contest::config::Config;
use contest::paths::RunDir;
use contest::usage::{
	five_hour_full, five_hour_used_percent, limit_hit, load_usage, over_weekly_pace, record_usage,
	rejected_until, spawn_probe,
};

const NOW: i64 = 1_000_000;
const WEEK: i64 = 604800;

fn info(five_used: f64, seven_used: f64) -> RateLimitInfo {
	RateLimitInfo {
		status: "allowed".into(),
		resets_at: NOW + 1000,
		five_hour: Window {
			utilization: five_used,
			resets_at: NOW + 1000,
		},
		seven_day: Window {
			utilization: seven_used,
			resets_at: NOW + WEEK / 2,
		},
	}
}

#[test]
fn five_hour_full_at_reserve_edge() {
	let config = Config::default();
	let sample = info(0.88, 0.1);
	assert!(!five_hour_full(&sample, NOW, 2, &config));
	assert!(five_hour_full(&sample, NOW, 3, &config));
}

#[test]
fn expired_window_counts_zero() {
	let mut sample = info(0.9, 0.1);
	assert_eq!(five_hour_used_percent(&sample, NOW), 90);
	sample.five_hour.resets_at = NOW - 1;
	assert_eq!(five_hour_used_percent(&sample, NOW), 0);
}

#[test]
fn weekly_pace() {
	let config = Config::default();
	assert!(over_weekly_pace(&info(0.1, 0.6), NOW, &config));
	assert!(!over_weekly_pace(&info(0.1, 0.4), NOW, &config));
}

#[test]
fn rejected_until_future_only() {
	let mut sample = info(0.1, 0.1);
	assert_eq!(rejected_until(&sample, NOW), None);
	sample.status = "rejected".into();
	assert_eq!(rejected_until(&sample, NOW), Some(NOW + 1000));
	assert_eq!(rejected_until(&sample, NOW + 1000), None);
}

#[test]
fn record_and_load_round_trip() {
	let dir = tempfile::tempdir().unwrap();
	fs::create_dir_all(dir.path().join("run")).unwrap();
	let run = RunDir {
		root: dir.path().to_path_buf(),
	};
	assert!(load_usage(&run).is_none());
	let sample = info(0.5, 0.25);
	record_usage(&run, &sample, NOW).unwrap();
	assert_eq!(load_usage(&run), Some((sample, NOW)));
}

#[test]
fn probe_reads_rate_limit_from_fake_claude() {
	let dir = tempfile::tempdir().unwrap();
	let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/stream-success.jsonl");
	let script = dir.path().join("claude");
	fs::write(&script, format!("#!/usr/bin/env bash\ncat {fixture:?}\n")).unwrap();
	fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
	let config = Config {
		claude_bin: script,
		..Config::default()
	};
	let mut probe = spawn_probe(&config, dir.path().join("probe/stream.jsonl")).unwrap();
	probe.child.wait().unwrap();
	let end = probe.finish();
	assert!(!dir.path().join("probe/stream.jsonl").exists());
	assert!(end.cleanup_failures.is_empty());
	assert_eq!(end.outcome.cost_usd, 0.05);
	assert_eq!(end.outcome.rate_limit.unwrap().five_hour.utilization, 0.13);
}

#[test]
fn probe_finish_reports_cleanup_failures() {
	if nix::unistd::geteuid().is_root() {
		return;
	}
	let dir = tempfile::tempdir().unwrap();
	let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/stream-success.jsonl");
	let script = dir.path().join("claude");
	fs::write(&script, format!("#!/usr/bin/env bash\ncat {fixture:?}\n")).unwrap();
	fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
	let config = Config {
		claude_bin: script,
		..Config::default()
	};
	let probe_dir = dir.path().join("probe");
	let mut probe = spawn_probe(&config, probe_dir.join("stream.jsonl")).unwrap();
	probe.child.wait().unwrap();
	fs::set_permissions(&probe_dir, fs::Permissions::from_mode(0o500)).unwrap();
	let end = probe.finish();
	fs::set_permissions(&probe_dir, fs::Permissions::from_mode(0o755)).unwrap();
	assert_eq!(
		end.cleanup_failures.len(),
		2,
		"{failures:?}",
		failures = end.cleanup_failures
	);
	assert!(end.cleanup_failures[0].contains("stream.jsonl"));
	assert!(end.outcome.rate_limit.is_some());
}

#[test]
fn probe_runs_without_tools() {
	let dir = tempfile::tempdir().unwrap();
	let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/stream-success.jsonl");
	let argv_dump = dir.path().join("argv");
	let script = dir.path().join("claude");
	fs::write(
		&script,
		format!("#!/usr/bin/env bash\nprintf '%s\\0' \"$@\" > {argv_dump:?}\ncat {fixture:?}\n"),
	)
	.unwrap();
	fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
	let config = Config {
		claude_bin: script,
		..Config::default()
	};
	let mut probe = spawn_probe(&config, dir.path().join("probe/stream.jsonl")).unwrap();
	probe.child.wait().unwrap();
	let dump = fs::read_to_string(&argv_dump).unwrap();
	let argv: Vec<&str> = dump.split('\0').collect();
	let tools_at = argv
		.iter()
		.position(|arg| *arg == "--tools")
		.unwrap_or_else(|| panic!("no --tools in {argv:?}"));
	assert_eq!(argv[tools_at + 1], "");
	assert_eq!(argv.iter().filter(|arg| **arg == "--tools").count(), 1);
}

fn limited(status: &str, five_used: f64) -> RateLimitInfo {
	RateLimitInfo {
		status: status.into(),
		..info(five_used, 0.1)
	}
}

fn outcome(rate_limit: Option<RateLimitInfo>, is_error: bool) -> StreamOutcome {
	StreamOutcome {
		is_error,
		rate_limit,
		..StreamOutcome::default()
	}
}

#[test]
fn limit_hit_on_limited_status_or_exhausted_window_with_error() {
	for status in ["rejected", "blocked", "limited"] {
		let stream = outcome(Some(limited(status, 0.5)), false);
		assert_eq!(limit_hit(&stream, None, NOW), Some(NOW + 1000), "{status}");
	}
	for status in ["allowed", "allowed_warning"] {
		let stream = outcome(Some(limited(status, 0.5)), true);
		assert_eq!(limit_hit(&stream, None, NOW), None, "{status}");
	}
	let exhausted = limited("allowed_warning", 1.0);
	assert_eq!(
		limit_hit(&outcome(Some(exhausted.clone()), true), None, NOW),
		Some(NOW + 1000)
	);
	assert_eq!(
		limit_hit(&outcome(Some(exhausted.clone()), false), None, NOW),
		None
	);
	assert_eq!(
		limit_hit(&outcome(None, true), Some(&exhausted), NOW),
		Some(NOW + 1000)
	);
	assert_eq!(limit_hit(&outcome(None, true), None, NOW), None);
	let mut past = limited("rejected", 0.5);
	past.resets_at = NOW - 10;
	past.five_hour.resets_at = NOW - 10;
	assert_eq!(limit_hit(&outcome(Some(past), false), None, NOW), Some(NOW));
}

#[test]
fn rejected_until_matches_any_limited_status() {
	assert_eq!(
		rejected_until(&limited("blocked", 0.5), NOW),
		Some(NOW + 1000)
	);
	assert_eq!(rejected_until(&limited("allowed_warning", 0.5), NOW), None);
}
