use std::fs;
use std::path::PathBuf;
use std::process::{Child, ExitStatus};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};

use super::{SECONDS_PER_MINUTE, STREAM_FILE, Supervisor, report, retry_backoff};
use crate::action::{Action, apply_actions, parse_actions};
use crate::agent::{AgentRun, StreamOutcome, parse_stream};
use crate::atomic::write_atomic;
use crate::claude_config::agent_env;
use crate::digest::{ack_inbox, build_digest};
use crate::events::{Event, emit};
use crate::journal::journal_append;
use crate::paths::RunDir;

const MANAGER_TOOLS: [&str; 3] = ["Read", "Glob", "Grep"];
const MANAGER_WHO: &str = "manager";
const MANAGER_PROMPT: &str = "prompts/manager.md";

fn has_inbox(run: &RunDir) -> bool {
	fs::read_dir(run.inbox()).is_ok_and(|entries| {
		entries.flatten().any(|entry| {
			!entry.file_name().to_string_lossy().starts_with('.')
				&& entry.file_type().is_ok_and(|kind| kind.is_file())
		})
	})
}

fn manager_actions(status: ExitStatus, outcome: &StreamOutcome) -> Result<Vec<Action>> {
	ensure!(status.success(), "manager exited with {status}");
	let text = outcome
		.result_text
		.as_deref()
		.context("manager stream has no result")?;
	ensure!(!outcome.is_error, "manager ended in error: {text}");
	parse_actions(text)
}

pub(super) struct ManagerProcess {
	child: Child,
	model: String,
	started: Instant,
	stream_log: PathBuf,
	cursor: u64,
	inbox: Vec<PathBuf>,
}

impl Supervisor {
	fn has_new_events(&self) -> bool {
		let cursor: u64 = fs::read_to_string(self.run.manager_cursor())
			.ok()
			.and_then(|text| text.trim().parse().ok())
			.unwrap_or(0);
		fs::metadata(self.run.events()).is_ok_and(|meta| meta.len() > cursor)
	}

	fn tick_due(&self) -> bool {
		if self
			.manager_retry_at
			.is_some_and(|retry_at| Instant::now() < retry_at)
		{
			return false;
		}
		let heartbeat = Duration::from_secs(self.cfg.heartbeat_minutes * SECONDS_PER_MINUTE);
		self.last_tick
			.is_none_or(|ticked| ticked.elapsed() >= heartbeat)
			|| self.has_new_events()
			|| has_inbox(&self.run)
	}

	pub(super) fn manager_step(&mut self, now: i64) -> Result<()> {
		if let Some(manager) = self.manager.as_mut() {
			let timeout = Duration::from_secs(self.cfg.manager_timeout_seconds);
			let status = match manager.child.try_wait()? {
				Some(status) => Ok(status),
				None if manager.started.elapsed() < timeout => return Ok(()),
				None => {
					manager.child.kill()?;
					manager.child.wait()?;
					Err(format!(
						"manager timed out after {seconds} s",
						seconds = timeout.as_secs()
					))
				}
			};
			if let Some(manager) = self.manager.take() {
				self.finish_manager(manager, status, now);
			}
			return Ok(());
		}
		if !self.tick_due() {
			return Ok(());
		}
		let Some(info) = self.fresh_usage(now) else {
			return Ok(());
		};
		if !self.usage_allows(&info, now) {
			return Ok(());
		}
		if let Err(error) = self.launch_manager(now) {
			self.manager_failed(&format!("manager launch failed: {error:#}"));
		}
		Ok(())
	}

	pub(super) fn discard_manager(&mut self, now: i64) {
		let Some(mut manager) = self.manager.take() else {
			return;
		};
		report("manager", manager.child.kill().map_err(Into::into));
		report(
			"manager",
			manager.child.wait().map(drop).map_err(Into::into),
		);
		let outcome = parse_stream(&manager.stream_log);
		self.record_agent(MANAGER_WHO, &manager.model, &outcome, manager.started, now);
		let journaled = journal_append(&self.run, "manager tick discarded at stop");
		report("manager", journaled);
	}

	fn launch_manager(&mut self, now: i64) -> Result<()> {
		self.last_tick = Some(Instant::now());
		let template_path = self.cfg.plugin_dir.join(MANAGER_PROMPT);
		let template = fs::read_to_string(&template_path)
			.with_context(|| format!("read {template_path:?}"))?;
		let digest = build_digest(&self.run, &self.cfg, now);
		let agent = AgentRun {
			prompt: format!("{template}\n{text}", text = digest.text),
			model: self.cfg.manager_model.clone(),
			budget_usd: self.cfg.manager_budget_usd,
			cwd: self.run.root.clone(),
			allowed_tools: Some(MANAGER_TOOLS.map(String::from).to_vec()),
			stream_log: self.run.agents().join(MANAGER_WHO).join(STREAM_FILE),
			env: agent_env(&self.run, None, &self.bin_dir),
		};
		let child = self.spawn_agent_for(&agent, None)?;
		self.manager = Some(ManagerProcess {
			child,
			model: agent.model,
			started: Instant::now(),
			stream_log: agent.stream_log,
			cursor: digest.cursor,
			inbox: digest.inbox,
		});
		Ok(())
	}

	fn finish_manager(
		&mut self,
		manager: ManagerProcess,
		status: Result<ExitStatus, String>,
		now: i64,
	) {
		let outcome = parse_stream(&manager.stream_log);
		self.record_agent(MANAGER_WHO, &manager.model, &outcome, manager.started, now);
		let actions = status
			.map_err(anyhow::Error::msg)
			.and_then(|status| manager_actions(status, &outcome));
		match actions {
			Ok(actions) => self.apply_manager(actions, &manager),
			Err(error) => self.manager_failed(&format!("{error:#}")),
		}
	}

	fn apply_manager(&mut self, actions: Vec<Action>, manager: &ManagerProcess) {
		let errors = apply_actions(&self.run, actions);
		let acked = write_atomic(
			&self.run.manager_cursor(),
			manager.cursor.to_string().as_bytes(),
		)
		.map_err(anyhow::Error::from)
		.and_then(|()| ack_inbox(&self.run, &manager.inbox));
		report("manager ack", acked);
		self.kill_cancelled_workers();
		if errors.is_empty() {
			self.manager_failures = 0;
			self.manager_retry_at = None;
			return;
		}
		self.manager_failed(&errors.join("; "));
	}

	fn manager_failed(&mut self, error: &str) {
		self.manager_failures += 1;
		let backoff = retry_backoff(self.manager_failures, self.cfg.heartbeat_minutes);
		self.manager_retry_at = Some(Instant::now() + backoff);
		report(
			"manager error",
			emit(
				&self.run,
				Event::ManagerError {
					error: error.to_owned(),
				},
			),
		);
	}
}
