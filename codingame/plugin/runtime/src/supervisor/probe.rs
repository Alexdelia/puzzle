use std::time::{Duration, Instant};

use anyhow::Result;

use super::{STREAM_FILE, Supervisor, report};
use crate::agent::RateLimitInfo;
use crate::claude_config::credentials_expire_within;
use crate::events::{Event, emit};
use crate::journal::journal_append;
use crate::usage::{
	Probe, five_hour_full, fresh_usage, over_weekly_pace, record_usage, rejected_until, spawn_probe,
};

const CREDENTIALS_HORIZON_SECONDS: u64 = 15 * 60;
const PROBE_WHO: &str = "probe";

pub(super) struct RunningProbe {
	probe: Probe,
	started: Instant,
}

impl Supervisor {
	fn probe_allowed(&self) -> bool {
		let retry = Duration::from_secs(self.cfg.probe_retry_seconds);
		self.probe.is_none()
			&& self
				.last_probe
				.is_none_or(|probed| probed.elapsed() >= retry)
	}

	pub(super) fn discard_probe(&mut self) {
		let Some(mut running) = self.probe.take() else {
			return;
		};
		self.kill_and_reap(&mut running.probe.child, PROBE_WHO);
		drop(running.probe.finish());
		let journaled = journal_append(&self.run, "usage probe discarded at stop");
		report("usage probe", journaled);
	}

	fn start_probe(&mut self) {
		if !self.probe_allowed() {
			return;
		}
		self.last_probe = Some(Instant::now());
		let stream_log = self.run.agents().join(PROBE_WHO).join(STREAM_FILE);
		match spawn_probe(&self.cfg, stream_log) {
			Ok(probe) => {
				self.probe = Some(RunningProbe {
					probe,
					started: Instant::now(),
				});
			}
			Err(error) => self.probe_failed(&format!("usage probe failed to start: {error:#}")),
		}
	}

	pub(super) fn poll_probe(&mut self, now: i64) -> Result<()> {
		let Some(running) = self.probe.as_mut() else {
			return Ok(());
		};
		let timeout = Duration::from_secs(self.cfg.probe_timeout_seconds);
		if running.probe.child.try_wait()?.is_none() {
			if running.started.elapsed() < timeout {
				return Ok(());
			}
			if let Some(mut timed_out) = self.probe.take() {
				self.kill_and_reap(&mut timed_out.probe.child, PROBE_WHO);
			}
			self.probe_failed(&format!(
				"usage probe timed out after {seconds} s",
				seconds = timeout.as_secs()
			));
			return Ok(());
		}
		let Some(finished) = self.probe.take() else {
			return Ok(());
		};
		match finished.probe.finish() {
			Some(info) => {
				self.usage_unknown_reported = false;
				record_usage(&self.run, &info, now)
			}
			None => {
				self.probe_failed("usage probe failed: no rate limit in its stream");
				Ok(())
			}
		}
	}

	fn probe_failed(&mut self, failure: &str) {
		report("usage probe", journal_append(&self.run, failure));
		if !self.usage_unknown_reported {
			self.usage_unknown_reported = true;
			report("usage probe", emit(&self.run, Event::UsageUnknown));
		}
	}

	pub(super) fn refresh_credentials(&mut self) {
		let expiring = self.credentials.as_ref().is_some_and(|path| {
			credentials_expire_within(path, CREDENTIALS_HORIZON_SECONDS).unwrap_or(false)
		});
		if expiring {
			self.start_probe();
		}
	}

	pub(super) fn fresh_usage(&mut self, now: i64) -> Option<RateLimitInfo> {
		let info = fresh_usage(&self.run, &self.cfg, now);
		if info.is_none() {
			self.start_probe();
		}
		info
	}

	pub(super) fn usage_allows(&mut self, info: &RateLimitInfo, now: i64) -> bool {
		if let Some(until) = rejected_until(info, now) {
			if self.blocked_until != Some(until) {
				self.blocked_until = Some(until);
				report("usage", emit(&self.run, Event::UsageBlocked { until }));
			}
			return false;
		}
		!five_hour_full(info, now, self.workers.len(), &self.cfg)
			&& !over_weekly_pace(info, now, &self.cfg)
	}
}
