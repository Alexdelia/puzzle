use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::atomic::write_atomic;
use crate::paths::{RunDir, TASK_META_DIR, TaskId};
use crate::taint::{append_worker_file, read_worker_file};

const REQUEUE_PRIORITY_BONUS: i32 = 1000;
const SPEC_FILE: &str = "spec.toml";
const BODY_FILE: &str = "spec.md";
const STATE_FILE: &str = "state";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskState {
	Queued,
	Running,
	Interrupted,
	NeedsAnswer,
	Waiting,
	Done,
	Failed,
	Cancelled,
}

impl TaskState {
	fn name(self) -> &'static str {
		match self {
			Self::Queued => "queued",
			Self::Running => "running",
			Self::Interrupted => "interrupted",
			Self::NeedsAnswer => "needs-answer",
			Self::Waiting => "waiting",
			Self::Done => "done",
			Self::Failed => "failed",
			Self::Cancelled => "cancelled",
		}
	}

	fn parse(name: &str) -> Option<Self> {
		[
			Self::Queued,
			Self::Running,
			Self::Interrupted,
			Self::NeedsAnswer,
			Self::Waiting,
			Self::Done,
			Self::Failed,
			Self::Cancelled,
		]
		.into_iter()
		.find(|state| state.name() == name)
	}

	pub fn can_become(self, next: Self) -> bool {
		use TaskState::*;
		matches!(
			(self, next),
			(Queued, Running | Cancelled)
				| (
					Running,
					Done | Failed | Interrupted | NeedsAnswer | Waiting | Cancelled
				) | (Interrupted, Queued | Cancelled)
				| (NeedsAnswer | Waiting, Queued | Cancelled | Failed)
				| (Done, Failed)
				| (Failed, Queued)
		)
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskSpec {
	pub kind: String,
	pub title: String,
	pub priority: i32,
	pub model: Option<String>,
	pub budget_usd: Option<f64>,
	pub writes: Vec<PathBuf>,
	#[serde(skip)]
	pub body: String,
	pub parent: Option<TaskId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TaskSummary {
	pub id: TaskId,
	pub kind: String,
	pub title: String,
	pub priority: i32,
	pub state: TaskState,
}

pub struct TaskStore<'a> {
	pub run: &'a RunDir,
}

impl TaskStore<'_> {
	fn next_id_path(&self) -> PathBuf {
		self.run.tasks().join("next-id")
	}

	fn spec_path(&self, id: &TaskId) -> PathBuf {
		self.run.task_meta(id).join(SPEC_FILE)
	}

	fn body_path(&self, id: &TaskId) -> PathBuf {
		self.run.task_meta(id).join(BODY_FILE)
	}

	fn state_path(&self, id: &TaskId) -> PathBuf {
		self.run.task_meta(id).join(STATE_FILE)
	}

	fn take_counter(&self) -> Result<u32> {
		let path = self.next_id_path();
		let current = match fs::read_to_string(&path) {
			Ok(text) => text.trim().parse().context("invalid next-id")?,
			Err(_) => 1,
		};
		write_atomic(&path, (current + 1).to_string().as_bytes())?;
		Ok(current)
	}

	pub fn create(&self, spec: TaskSpec) -> Result<TaskId> {
		fs::create_dir_all(self.run.tasks())?;
		let counter = self.take_counter()?;
		let id = TaskId(format!("t{counter:04}-{kind}", kind = spec.kind));
		let staging = self.run.tasks().join(format!(".tmp-{id}"));
		let _ = fs::remove_dir_all(&staging);
		let staged = staging.join(TASK_META_DIR);
		fs::create_dir_all(&staged)?;
		write_atomic(&staged.join(SPEC_FILE), toml::to_string(&spec)?.as_bytes())?;
		write_atomic(&staged.join(BODY_FILE), spec.body.as_bytes())?;
		write_atomic(
			&staged.join(STATE_FILE),
			TaskState::Queued.name().as_bytes(),
		)?;
		fs::rename(&staging, self.run.task_dir(&id))?;
		Ok(id)
	}

	pub fn spec(&self, id: &TaskId) -> Result<TaskSpec> {
		let text =
			fs::read_to_string(self.spec_path(id)).with_context(|| format!("unknown task {id}"))?;
		let mut spec: TaskSpec = toml::from_str(&text)?;
		spec.body = fs::read_to_string(self.body_path(id)).unwrap_or_default();
		Ok(spec)
	}

	pub fn state(&self, id: &TaskId) -> Result<TaskState> {
		let text = fs::read_to_string(self.state_path(id))
			.with_context(|| format!("unknown task {id}"))?;
		TaskState::parse(text.trim()).with_context(|| format!("invalid state of {id}"))
	}

	pub fn transition(&self, id: &TaskId, next: TaskState) -> Result<()> {
		let current = self.state(id)?;
		if !current.can_become(next) {
			bail!(
				"task {id}: transition {from} -> {to} not allowed",
				from = current.name(),
				to = next.name()
			);
		}
		write_atomic(&self.state_path(id), next.name().as_bytes())?;
		Ok(())
	}

	pub fn set_priority(&self, id: &TaskId, priority: i32) -> Result<()> {
		let mut spec = self.spec(id)?;
		spec.priority = priority;
		write_atomic(&self.spec_path(id), toml::to_string(&spec)?.as_bytes())?;
		Ok(())
	}

	pub fn append_notes(&self, id: &TaskId, text: &str) -> Result<()> {
		append_worker_file(&self.run.task_dir(id).join("notes.md"), text.as_bytes())
	}

	pub fn result_summary(&self, id: &TaskId) -> Option<String> {
		let bytes = read_worker_file(&self.run.task_dir(id).join("result.md")).ok()?;
		String::from_utf8_lossy(&bytes)
			.lines()
			.next()
			.map(str::to_owned)
	}

	pub fn list(&self) -> Result<Vec<TaskSummary>> {
		let Ok(entries) = fs::read_dir(self.run.tasks()) else {
			return Ok(Vec::new());
		};
		let mut summaries = Vec::new();
		for entry in entries {
			let entry = entry?;
			let name = entry.file_name().to_string_lossy().into_owned();
			if name.starts_with('.') || !entry.file_type()?.is_dir() {
				continue;
			}
			let id = TaskId(name);
			let (Ok(spec), Ok(state)) = (self.spec(&id), self.state(&id)) else {
				continue;
			};
			summaries.push(TaskSummary {
				state,
				kind: spec.kind,
				title: spec.title,
				priority: spec.priority,
				id,
			});
		}
		summaries.sort_by(|a, b| a.id.0.cmp(&b.id.0));
		Ok(summaries)
	}

	pub fn queued(&self) -> Result<Vec<TaskId>> {
		let mut queued: Vec<TaskSummary> = self
			.list()?
			.into_iter()
			.filter(|task| task.state == TaskState::Queued)
			.collect();
		queued.sort_by(|a, b| {
			b.priority
				.cmp(&a.priority)
				.then_with(|| a.id.0.cmp(&b.id.0))
		});
		Ok(queued.into_iter().map(|task| task.id).collect())
	}

	pub fn next_queued(&self) -> Result<Option<TaskId>> {
		Ok(self.queued()?.into_iter().next())
	}

	pub fn reconcile_after_restart(&self) -> Result<Vec<TaskId>> {
		let mut requeued = Vec::new();
		for task in self.list()? {
			match task.state {
				TaskState::Running => self.transition(&task.id, TaskState::Interrupted)?,
				TaskState::Interrupted => {}
				_ => continue,
			}
			self.transition(&task.id, TaskState::Queued)?;
			self.set_priority(&task.id, task.priority + REQUEUE_PRIORITY_BONUS)?;
			requeued.push(task.id);
		}
		Ok(requeued)
	}
}
