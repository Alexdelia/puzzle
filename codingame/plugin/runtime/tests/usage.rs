use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use contest::agent::{RateLimitInfo, Window};
use contest::config::Config;
use contest::paths::RunDir;
use contest::usage::{
	five_hour_full, five_hour_used_percent, load_usage, over_weekly_pace, probe_usage,
	record_usage, rejected_until,
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
	let probed = probe_usage(&config).unwrap();
	assert_eq!(probed.five_hour.utilization, 0.13);
}
