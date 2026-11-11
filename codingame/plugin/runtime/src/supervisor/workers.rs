use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde_json::Value;

use super::{
	Gate, LAUNCH_BLOCK, SECONDS_PER_MINUTE, STREAM_FILE, Supervisor, remove_if_exists, report,
	retry_backoff, unix_now,
};
use crate::agent::{AgentRun, RateLimitInfo, StreamOutcome, kill_agent_group, parse_stream};
use crate::atomic::write_atomic;
use crate::claude_config::agent_env;
use crate::events::{Event, emit};
use crate::journal::{journal_append, utc_time};
use crate::ledger::ledger_append;
use crate::paths::{RunDir, TaskId};
use crate::spool::prepare_task_requests;
use crate::taint::read_worker_file;
use crate::task::{TaskSpec, TaskState, TaskStore};
use crate::usage::{five_hour_used_percent, fresh_usage, limit_hit, load_usage, record_usage};

const NOTES_PROMPT_MAX_BYTES: usize = 16 * 1024;
const WORKER_PROMPT: &str = "prompts/worker.md";
const EXITED_WITHOUT_OUTCOME: &str = "exited without outcome";
const NONE_TEXT: &str = "(none)";
const NOTES_UNREADABLE: &str = "notes.md unreadable or too large";
const MAX_TASK_LAUNCH_FAILURES: u32 = 3;
const STDERR_TAIL_BYTES: u64 = 400;
const STREAM_TAIL_CHUNK: usize = 1024 * 1024;
const RATE_LIMIT_TYPE: &str = "rate_limit_event";
const RESULT_TYPE: &str = "result";
const SOFT_STOP_REASON: &str = "stopped near the usage cap";
const MAX_TASK_REQUEUES: usize = 3;
const REQUEUE_WINDOW: Duration = Duration::from_secs(5 * 3600);

fn fill(template: &str, values: &[(&str, &str)]) -> String {
	let mut filled = String::with_capacity(template.len());
	let mut rest = template;
	while let Some(open) = rest.find('{') {
		filled.push_str(&rest[..open]);
		let tail = &rest[open + 1..];
		let placeholder = values.iter().find(|(key, _)| {
			tail.strip_prefix(key)
				.is_some_and(|after| after.starts_with('}'))
		});
		match placeholder {
			Some((key, value)) => {
				filled.push_str(value);
				rest = &tail[key.len() + 1..];
			}
			None => {
				filled.push('{');
				rest = tail;
			}
		}
	}
	filled.push_str(rest);
	filled
}

fn spec_text(spec: &TaskSpec) -> String {
	let writes = if spec.writes.is_empty() {
		"none".to_owned()
	} else {
		spec.writes
			.iter()
			.map(|path| path.display().to_string())
			.collect::<Vec<_>>()
			.join(", ")
	};
	let parent = spec
		.parent
		.as_ref()
		.map_or("none".to_owned(), |parent| parent.0.clone());
	format!(
		"title: {title}\nkind: {kind}\nwrites: {writes}\nparent: {parent}\n\n{body}",
		title = spec.title,
		kind = spec.kind,
		body = spec.body
	)
}

fn notes_text(run: &RunDir, id: &TaskId) -> String {
	let path = run.task_dir(id).join("notes.md");
	if fs::symlink_metadata(&path).is_err() {
		return NONE_TEXT.to_owned();
	}
	let Ok(bytes) = read_worker_file(&path) else {
		return NOTES_UNREADABLE.to_owned();
	};
	let notes = String::from_utf8_lossy(&bytes);
	let notes = notes.trim();
	if notes.is_empty() {
		return NONE_TEXT.to_owned();
	}
	if notes.len() <= NOTES_PROMPT_MAX_BYTES {
		return notes.to_owned();
	}
	let mut start = notes.len() - NOTES_PROMPT_MAX_BYTES;
	while !notes.is_char_boundary(start) {
		start += 1;
	}
	format!(
		"[earlier notes cut, read notes.md for all]\n{tail}",
		tail = &notes[start..]
	)
}

fn stderr_tail(stream_log: &Path) -> String {
	let path = stream_log.with_extension("stderr.log");
	let Ok(mut file) = File::open(&path) else {
		return String::new();
	};
	let length = file.metadata().map_or(0, |meta| meta.len());
	let mut bytes = Vec::new();
	let read = file
		.seek(SeekFrom::Start(length.saturating_sub(STDERR_TAIL_BYTES)))
		.and_then(|_| file.take(STDERR_TAIL_BYTES).read_to_end(&mut bytes));
	if read.is_err() {
		return String::new();
	}
	String::from_utf8_lossy(&bytes).trim().to_owned()
}

fn idle_for(path: &Path) -> Duration {
	fs::metadata(path)
		.and_then(|meta| meta.modified())
		.ok()
		.and_then(|modified| modified.elapsed().ok())
		.unwrap_or_default()
}

enum StreamLine {
	RateLimit(RateLimitInfo),
	Result,
}

fn classify_line(line: &str) -> Option<StreamLine> {
	if !line.contains(RATE_LIMIT_TYPE) && !line.contains(RESULT_TYPE) {
		return None;
	}
	let value: Value = serde_json::from_str(line).ok()?;
	match value["type"].as_str()? {
		RATE_LIMIT_TYPE => serde_json::from_value(value["rate_limit_info"].clone())
			.ok()
			.map(StreamLine::RateLimit),
		RESULT_TYPE => Some(StreamLine::Result),
		_ => None,
	}
}

fn read_new_lines(path: &Path, offset: &mut u64) -> io::Result<Vec<StreamLine>> {
	let mut file = File::open(path)?;
	file.seek(SeekFrom::Start(*offset))?;
	let mut bytes = Vec::new();
	file.take(STREAM_TAIL_CHUNK as u64)
		.read_to_end(&mut bytes)?;
	let complete = match bytes.iter().rposition(|&byte| byte == b'\n') {
		Some(last) => last + 1,
		None if bytes.len() == STREAM_TAIL_CHUNK => bytes.len(),
		None => 0,
	};
	*offset += complete as u64;
	Ok(String::from_utf8_lossy(&bytes[..complete])
		.lines()
		.filter_map(classify_line)
		.collect())
}

enum Kill {
	Idle(String),
	AfterResult(String),
	StopDeadline(String),
}

pub(super) struct WorkerProcess {
	task: TaskId,
	pub(super) child: Child,
	model: String,
	started: Instant,
	stream_log: PathBuf,
	stream_offset: u64,
	result_at: Option<Instant>,
	stop_asked_at: Option<Instant>,
	kill: Option<Kill>,
	exited: bool,
}

#[derive(Default)]
pub(super) struct TaskLaunchFailures {
	count: u32,
	another_task_ran: bool,
	ran_since_last_failure: bool,
}

impl TaskLaunchFailures {
	fn sorts_last(&self) -> bool {
		!self.ran_since_last_failure
	}
}

impl WorkerProcess {
	fn take_new_rate_limits(&mut self) -> Vec<RateLimitInfo> {
		let lines = read_new_lines(&self.stream_log, &mut self.stream_offset).unwrap_or_default();
		let mut limits = Vec::new();
		for line in lines {
			match line {
				StreamLine::RateLimit(info) => limits.push(info),
				StreamLine::Result => {
					self.result_at.get_or_insert_with(Instant::now);
				}
			}
		}
		limits
	}

	fn overdue(
		&self,
		idle: Duration,
		result_grace: Duration,
		stop_grace: Duration,
	) -> Option<Kill> {
		if self
			.stop_asked_at
			.is_some_and(|asked_at| asked_at.elapsed() >= stop_grace)
		{
			return Some(Kill::StopDeadline(format!(
				"still running {seconds} s after the soft stop",
				seconds = stop_grace.as_secs()
			)));
		}
		if let Some(result_at) = self.result_at {
			return (result_at.elapsed() >= result_grace).then(|| {
				Kill::AfterResult(format!(
					"still alive {seconds} s after its result",
					seconds = result_grace.as_secs()
				))
			});
		}
		(idle_for(&self.stream_log) >= idle).then(|| {
			Kill::Idle(format!(
				"no stream output for {minutes} min",
				minutes = idle.as_secs() / SECONDS_PER_MINUTE
			))
		})
	}
}

impl Supervisor {
	pub(super) fn kill_worker(&mut self, id: &TaskId) {
		for worker in self.workers.iter_mut().filter(|worker| &worker.task == id) {
			report("kill worker", kill_agent_group(&worker.child));
		}
	}

	pub(super) fn kill_cancelled_workers(&mut self) {
		let store = TaskStore { run: &self.run };
		let cancelled: Vec<TaskId> = self
			.workers
			.iter()
			.filter(|worker| store.state(&worker.task).ok() == Some(TaskState::Cancelled))
			.map(|worker| worker.task.clone())
			.collect();
		for id in cancelled {
			self.kill_worker(&id);
		}
	}

	pub(super) fn watch_workers(&mut self, now: i64) {
		let idle = Duration::from_secs(self.cfg.worker_idle_minutes * SECONDS_PER_MINUTE);
		let result_grace = Duration::from_secs(self.cfg.result_grace_seconds);
		let stop_grace = Duration::from_secs(self.cfg.grace_seconds);
		let mut fullest: Option<RateLimitInfo> = None;
		for worker in &mut self.workers {
			for info in worker.take_new_rate_limits() {
				let used = five_hour_used_percent(&info, now);
				if fullest
					.as_ref()
					.is_none_or(|kept| five_hour_used_percent(kept, now) <= used)
				{
					fullest = Some(info);
				}
			}
			if worker.kill.is_some() || worker.exited {
				continue;
			}
			if let Some(kill) = worker.overdue(idle, result_grace, stop_grace) {
				report("kill worker", kill_agent_group(&worker.child));
				worker.kill = Some(kill);
			}
		}
		if let Some(info) = fullest {
			let recorded = record_usage(&self.run, &info, now);
			if recorded.is_ok() {
				self.usage_unknown_reported = false;
			}
			report("usage", recorded);
		}
		if self.workers.is_empty() {
			return;
		}
		let Some(newest) = fresh_usage(&self.run, &self.cfg, now) else {
			return;
		};
		let used = five_hour_used_percent(&newest, now);
		if used >= self.cfg.five_hour_limit_percent {
			self.soft_stop_workers(used);
		}
	}

	fn soft_stop_workers(&mut self, used_percent: u8) {
		for worker in &mut self.workers {
			if worker.stop_asked_at.is_some() || worker.exited {
				continue;
			}
			worker.stop_asked_at = Some(Instant::now());
			let marker = self.run.task_stop_marker(&worker.task);
			let stopped = write_atomic(&marker, b"usage cap")
				.map_err(anyhow::Error::from)
				.and_then(|()| {
					journal_append(
						&self.run,
						&format!(
							"asked {id} to wrap up: 5h usage at {used_percent}%",
							id = worker.task
						),
					)
				});
			report("soft stop", stopped);
		}
	}

	pub(super) fn poll_worker_exits(&mut self) {
		for worker in self.workers.iter_mut().filter(|worker| !worker.exited) {
			match worker.child.try_wait() {
				Ok(Some(_)) => worker.exited = true,
				Ok(None) => {}
				Err(error) => report("wait worker", Err(error.into())),
			}
		}
	}

	pub(super) fn workers_alive(&self) -> bool {
		self.workers.iter().any(|worker| !worker.exited)
	}

	pub(super) fn reap_workers(&mut self, now: i64) -> Result<()> {
		self.kill_cancelled_workers();
		self.poll_worker_exits();
		let (exited, alive) = std::mem::take(&mut self.workers)
			.into_iter()
			.partition(|worker| worker.exited);
		self.workers = alive;
		self.finish_workers(exited, now);
		Ok(())
	}

	pub(super) fn finish_workers(&mut self, workers: Vec<WorkerProcess>, now: i64) {
		if workers.is_empty() {
			return;
		}
		let failure = self.scan_taint_now().failure;
		for worker in workers {
			let finished = self.finish_worker(worker, failure.as_deref(), now);
			report("finish worker", finished);
		}
	}

	pub(super) fn requeue_orphaned_running(&mut self) -> Result<()> {
		let orphaned: Vec<TaskId> = TaskStore { run: &self.run }
			.list()?
			.into_iter()
			.filter(|task| task.state == TaskState::Running)
			.map(|task| task.id)
			.filter(|id| !self.workers.iter().any(|worker| &worker.task == id))
			.collect();
		for id in orphaned {
			self.requeue(&id, "running without a worker process")?;
		}
		Ok(())
	}

	pub(super) fn record_agent(
		&mut self,
		who: &str,
		model: &str,
		outcome: &StreamOutcome,
		started: Instant,
		now: i64,
	) {
		let wall_seconds = started.elapsed().as_secs();
		report(
			"ledger",
			ledger_append(&self.run, who, model, outcome, wall_seconds),
		);
		if let Some(info) = &outcome.rate_limit {
			let recorded = record_usage(&self.run, info, now);
			if recorded.is_ok() {
				self.usage_unknown_reported = false;
			}
			report("usage", recorded);
		}
	}

	fn requeue(&self, id: &TaskId, reason: &str) -> Result<TaskState> {
		let store = TaskStore { run: &self.run };
		store.transition(id, TaskState::Interrupted)?;
		store.transition(id, TaskState::Queued)?;
		journal_append(&self.run, &format!("requeued {id}: {reason}"))?;
		Ok(TaskState::Queued)
	}

	fn requeue_counted(&mut self, id: &TaskId, reason: &str) -> Result<TaskState> {
		let requeues = self.task_requeues.entry(id.clone()).or_default();
		requeues.retain(|at| at.elapsed() < REQUEUE_WINDOW);
		if requeues.len() >= MAX_TASK_REQUEUES {
			self.task_requeues.remove(id);
			let store = TaskStore { run: &self.run };
			store.transition(id, TaskState::Failed)?;
			let note = format!(
				"requeued {MAX_TASK_REQUEUES} times within 5 h, failed instead of again: {reason}"
			);
			report("notes", store.append_notes(id, &format!("\n{note}\n")));
			journal_append(&self.run, &format!("{id}: {note}"))?;
			return Ok(TaskState::Failed);
		}
		requeues.push(Instant::now());
		self.requeue(id, reason)
	}

	fn usage_limit_hit(&mut self, until: i64, now: i64) {
		if until <= now {
			return;
		}
		self.limit_until = Some(self.limit_until.map_or(until, |known| known.max(until)));
		if self.blocked_until != Some(until) {
			self.blocked_until = Some(until);
			report("usage", emit(&self.run, Event::UsageBlocked { until }));
		}
	}

	fn is_fast_exit(&self, worker: &WorkerProcess, outcome: &StreamOutcome) -> bool {
		worker.started.elapsed() < Duration::from_secs(self.cfg.fast_exit_seconds)
			&& (outcome.result_text.is_none() || outcome.output_tokens == 0)
	}

	fn fast_exit_error(worker: &WorkerProcess, outcome: &StreamOutcome) -> String {
		let seconds = worker.started.elapsed().as_secs();
		let what = if outcome.result_text.is_none() {
			"without a result"
		} else {
			"with 0 output tokens"
		};
		let tail = stderr_tail(&worker.stream_log);
		if tail.is_empty() {
			format!("worker exited after {seconds} s {what}")
		} else {
			format!("worker exited after {seconds} s {what}, stderr: {tail}")
		}
	}

	fn settle_unfinished(
		&mut self,
		worker: &WorkerProcess,
		outcome: &StreamOutcome,
		last_known: Option<&RateLimitInfo>,
		now: i64,
	) -> Result<TaskState> {
		let id = &worker.task;
		if self.stopping {
			TaskStore { run: &self.run }.transition(id, TaskState::Interrupted)?;
			return Ok(TaskState::Interrupted);
		}
		if let Some(until) = limit_hit(outcome, last_known, now) {
			if until <= now {
				return self.launch_failed(id, "worker hit the usage limit with no future reset");
			}
			self.usage_limit_hit(until, now);
			let reason = format!(
				"hit the usage limit, launches blocked until {time}",
				time = utc_time(until)
			);
			return self.requeue_counted(id, &reason);
		}
		match &worker.kill {
			Some(Kill::Idle(reason) | Kill::StopDeadline(reason)) => {
				return self.requeue_counted(id, &format!("killed, {reason}"));
			}
			Some(Kill::AfterResult(reason)) => {
				journal_append(&self.run, &format!("killed {id}, {reason}"))?;
				return self.fail_without_outcome(id);
			}
			None => {}
		}
		if self.run.task_stop_marker(id).exists() {
			self.launch_succeeded(id);
			return self.requeue_counted(id, SOFT_STOP_REASON);
		}
		if self.is_fast_exit(worker, outcome) {
			return self.launch_failed(id, &Self::fast_exit_error(worker, outcome));
		}
		self.fail_without_outcome(id)
	}

	fn fail_without_outcome(&mut self, id: &TaskId) -> Result<TaskState> {
		self.launch_succeeded(id);
		let store = TaskStore { run: &self.run };
		store.transition(id, TaskState::Failed)?;
		report(
			"notes",
			store.append_notes(id, &format!("\n{EXITED_WITHOUT_OUTCOME}\n")),
		);
		Ok(TaskState::Failed)
	}

	pub(super) fn announce_end(&self, id: &TaskId, state: TaskState) -> Result<()> {
		match state {
			TaskState::Queued => Ok(()),
			TaskState::Interrupted => {
				journal_append(&self.run, &format!("interrupted {id} at stop"))
			}
			_ => {
				emit(
					&self.run,
					Event::TaskFinished {
						task: id.clone(),
						state,
					},
				)?;
				journal_append(&self.run, &format!("finished {id} {state:?}"))
			}
		}
	}

	fn finish_worker(
		&mut self,
		worker: WorkerProcess,
		taint_failure: Option<&str>,
		now: i64,
	) -> Result<()> {
		let id = worker.task.clone();
		let outcome = parse_stream(&worker.stream_log);
		let last_known = load_usage(&self.run).map(|(info, _)| info);
		self.record_agent(&id.0, &worker.model, &outcome, worker.started, now);
		let mut state = TaskStore { run: &self.run }.state(&id)?;
		if let Some(failure) = taint_failure {
			state = self.flag_tainted(&id, state, failure);
		}
		if state == TaskState::Running {
			state = self.settle_unfinished(&worker, &outcome, last_known.as_ref(), now)?;
		} else {
			self.launch_succeeded(&id);
		}
		self.announce_end(&id, state)
	}

	fn launch_succeeded(&mut self, id: &TaskId) {
		self.launch_failures = 0;
		self.task_launch_failures.remove(id);
		for failures in self.task_launch_failures.values_mut() {
			failures.count = 0;
			failures.another_task_ran = true;
			failures.ran_since_last_failure = true;
		}
	}

	pub(super) fn launch_workers(&mut self, now: i64) -> Result<()> {
		if self
			.launch_retry_at
			.is_some_and(|retry_at| Instant::now() < retry_at)
		{
			return Ok(());
		}
		self.launch_retry_at = None;
		self.set_block(LAUNCH_BLOCK, None);
		if self.taint_over_budget {
			return Ok(());
		}
		let mut queued = (TaskStore { run: &self.run }).queued()?;
		queued.sort_by_key(|id| {
			self.task_launch_failures
				.get(id)
				.is_some_and(TaskLaunchFailures::sorts_last)
		});
		for id in queued {
			if self.workers.len() >= self.cfg.workers {
				break;
			}
			if self.workers.iter().any(|worker| worker.task == id) {
				continue;
			}
			if !self.usage_gate(Gate::Launch, now) {
				break;
			}
			TaskStore { run: &self.run }.transition(&id, TaskState::Running)?;
			match self.launch_worker(&id) {
				Ok(model) => {
					let journaled = journal_append(&self.run, &format!("launched {id} on {model}"));
					report("journal", journaled);
				}
				Err(error) => {
					let state = self.launch_failed(&id, &format!("{error:#}"))?;
					self.announce_end(&id, state)?;
					break;
				}
			}
		}
		Ok(())
	}

	fn launch_worker(&mut self, id: &TaskId) -> Result<String> {
		prepare_task_requests(&self.run, id)?;
		remove_if_exists(&self.run.task_stop_marker(id))?;
		let spec = TaskStore { run: &self.run }.spec(id)?;
		let template_path = self.cfg.plugin_dir.join(WORKER_PROMPT);
		let template = fs::read_to_string(&template_path)
			.with_context(|| format!("read {template_path:?}"))?;
		let task_dir = self.run.task_dir(id).display().to_string();
		let prompt = fill(
			&template,
			&[
				("task_id", &id.0),
				("task_dir", &task_dir),
				("spec", &spec_text(&spec)),
				("notes", &notes_text(&self.run, id)),
			],
		);
		let agent = AgentRun {
			prompt,
			model: spec.model.unwrap_or_else(|| self.cfg.worker_model.clone()),
			budget_usd: spec.budget_usd.unwrap_or(self.cfg.worker_budget_usd),
			cwd: self.run.root.clone(),
			tools: None,
			stream_log: self.run.agents().join(&id.0).join(STREAM_FILE),
			env: agent_env(&self.run, Some(id), &self.bin_dir),
		};
		let child = self.spawn_agent_for(&agent, Some(id))?;
		self.workers.push(WorkerProcess {
			task: id.clone(),
			child,
			model: agent.model.clone(),
			started: Instant::now(),
			stream_log: agent.stream_log,
			stream_offset: 0,
			result_at: None,
			stop_asked_at: None,
			kill: None,
			exited: false,
		});
		Ok(agent.model)
	}

	fn launch_failed(&mut self, id: &TaskId, error: &str) -> Result<TaskState> {
		self.launch_failures += 1;
		let backoff = retry_backoff(self.launch_failures, &self.cfg);
		self.launch_retry_at = Some(Instant::now() + backoff);
		let (task_failures, another_task_ran) = {
			let failures = self.task_launch_failures.entry(id.clone()).or_default();
			failures.count += 1;
			failures.ran_since_last_failure = false;
			(failures.count, failures.another_task_ran)
		};
		emit(
			&self.run,
			Event::LaunchFailed {
				task: id.clone(),
				error: error.to_owned(),
			},
		)?;
		let seconds = backoff.as_secs();
		let block = format!(
			"launches paused until {time} after {count} failed launches",
			time = utc_time(unix_now() + seconds as i64),
			count = self.launch_failures
		);
		self.set_block(LAUNCH_BLOCK, Some(block));
		journal_append(
			&self.run,
			&format!("launch of {id} failed, launches paused {seconds} s: {error}"),
		)?;
		let store = TaskStore { run: &self.run };
		if task_failures >= MAX_TASK_LAUNCH_FAILURES && another_task_ran {
			self.task_launch_failures.remove(id);
			store.transition(id, TaskState::Failed)?;
			let note = format!("launch failed {task_failures} times, last error: {error}");
			report("notes", store.append_notes(id, &format!("\n{note}\n")));
			return Ok(TaskState::Failed);
		}
		store.transition(id, TaskState::Interrupted)?;
		store.transition(id, TaskState::Queued)?;
		Ok(TaskState::Queued)
	}
}
