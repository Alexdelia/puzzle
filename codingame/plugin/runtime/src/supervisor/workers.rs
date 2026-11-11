use std::fs;
use std::path::PathBuf;
use std::process::Child;
use std::time::Instant;

use anyhow::{Context, Result};

use super::{STREAM_FILE, Supervisor, remove_planted, report, retry_backoff, taint_failure};
use crate::agent::{AgentRun, StreamOutcome, parse_stream};
use crate::claude_config::agent_env;
use crate::events::{Event, emit};
use crate::journal::journal_append;
use crate::ledger::ledger_append;
use crate::paths::{RunDir, TaskId};
use crate::spool::prepare_task_requests;
use crate::taint::read_worker_file;
use crate::task::{TaskSpec, TaskState, TaskStore};
use crate::usage::record_usage;

const NOTES_PROMPT_MAX_BYTES: usize = 16 * 1024;
const WORKER_PROMPT: &str = "prompts/worker.md";
const EXITED_WITHOUT_OUTCOME: &str = "exited without outcome";
const NONE_TEXT: &str = "(none)";
const NOTES_UNREADABLE: &str = "notes.md unreadable or too large";

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

pub(super) struct WorkerProcess {
	task: TaskId,
	pub(super) child: Child,
	model: String,
	started: Instant,
	stream_log: PathBuf,
}

impl Supervisor {
	pub(super) fn kill_worker(&mut self, id: &TaskId) {
		for worker in self.workers.iter_mut().filter(|worker| &worker.task == id) {
			report("kill worker", worker.child.kill().map_err(Into::into));
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

	pub(super) fn reap_workers(&mut self, now: i64) -> Result<()> {
		self.kill_cancelled_workers();
		let mut index = 0;
		while index < self.workers.len() {
			match self.workers[index].child.try_wait() {
				Ok(Some(_)) => {
					let worker = self.workers.swap_remove(index);
					let finished = self.finish_worker(worker, now);
					report("finish worker", finished);
				}
				Ok(None) => index += 1,
				Err(error) => {
					report("wait worker", Err(error.into()));
					index += 1;
				}
			}
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

	fn settle_unfinished(&self, id: &TaskId) -> Result<TaskState> {
		let store = TaskStore { run: &self.run };
		if self.stopping {
			store.transition(id, TaskState::Interrupted)?;
			return Ok(TaskState::Interrupted);
		}
		store.transition(id, TaskState::Failed)?;
		report(
			"notes",
			store.append_notes(id, &format!("\n{EXITED_WITHOUT_OUTCOME}\n")),
		);
		Ok(TaskState::Failed)
	}

	pub(super) fn finish_worker(&mut self, worker: WorkerProcess, now: i64) -> Result<()> {
		let id = &worker.task;
		let outcome = parse_stream(&worker.stream_log);
		self.record_agent(&id.0, &worker.model, &outcome, worker.started, now);
		let mut state = TaskStore { run: &self.run }.state(id)?;
		if let Some(failure) = taint_failure(remove_planted(&self.run, self.exempt)) {
			state = self.flag_tainted(id, state, &failure);
		}
		if state == TaskState::Running {
			state = self.settle_unfinished(id)?;
		}
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

	pub(super) fn launch_workers(&mut self, now: i64) -> Result<()> {
		if self
			.launch_retry_at
			.is_some_and(|retry_at| Instant::now() < retry_at)
		{
			return Ok(());
		}
		for id in (TaskStore { run: &self.run }).queued()? {
			if self.workers.len() >= self.cfg.workers {
				break;
			}
			if self.workers.iter().any(|worker| worker.task == id) {
				continue;
			}
			let Some(info) = self.fresh_usage(now) else {
				break;
			};
			if !self.usage_allows(&info, now) {
				break;
			}
			TaskStore { run: &self.run }.transition(&id, TaskState::Running)?;
			match self.launch_worker(&id) {
				Ok(model) => {
					self.launch_failures = 0;
					self.launch_retry_at = None;
					let journaled = journal_append(&self.run, &format!("launched {id} on {model}"));
					report("journal", journaled);
				}
				Err(error) => {
					self.launch_failed(&id, &error)?;
					break;
				}
			}
		}
		Ok(())
	}

	fn launch_worker(&mut self, id: &TaskId) -> Result<String> {
		prepare_task_requests(&self.run, id)?;
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
			allowed_tools: None,
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
		});
		Ok(agent.model)
	}

	fn launch_failed(&mut self, id: &TaskId, error: &anyhow::Error) -> Result<()> {
		self.launch_failures += 1;
		let backoff = retry_backoff(self.launch_failures, self.cfg.heartbeat_minutes);
		self.launch_retry_at = Some(Instant::now() + backoff);
		let store = TaskStore { run: &self.run };
		store.transition(id, TaskState::Interrupted)?;
		store.transition(id, TaskState::Queued)?;
		emit(
			&self.run,
			Event::LaunchFailed {
				task: id.clone(),
				error: format!("{error:#}"),
			},
		)?;
		journal_append(
			&self.run,
			&format!(
				"launch of {id} failed, launches paused {seconds} s: {error:#}",
				seconds = backoff.as_secs()
			),
		)
	}
}
