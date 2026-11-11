use std::collections::HashMap;
use std::task::Poll;
use std::thread::{self, JoinHandle};

use anyhow::{Context, Result, anyhow};

use crate::commit::commit_task;
use crate::config::Config;
use crate::paths::{RunDir, TaskId};
use crate::spool::CommitRequest;

pub(super) type CommitThread = JoinHandle<Result<Option<String>>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct CommitKey {
	pub(super) task: TaskId,
	pub(super) stem: String,
}

impl CommitKey {
	fn of(request: &CommitRequest) -> Self {
		Self {
			task: request.task.clone(),
			stem: request.stem.to_owned(),
		}
	}
}

#[derive(Default)]
pub(super) struct Commits(HashMap<CommitKey, CommitThread>);

fn join(thread: CommitThread) -> Result<Option<String>> {
	thread
		.join()
		.unwrap_or_else(|_| Err(anyhow!("commit thread panicked")))
}

pub(super) fn spawn_commit(
	run: RunDir,
	cfg: Config,
	task: TaskId,
	milestone: String,
) -> Result<CommitThread> {
	thread::Builder::new()
		.name(format!("commit {task}"))
		.spawn(move || commit_task(&run, &cfg, &task, &milestone))
		.context("spawn commit thread")
}

impl Commits {
	pub(super) fn poll(
		&mut self,
		request: &CommitRequest,
		start: impl FnOnce() -> Result<CommitThread>,
	) -> Poll<Result<Option<String>>> {
		let key = CommitKey::of(request);
		match self.0.get(&key) {
			Some(thread) if !thread.is_finished() => return Poll::Pending,
			Some(_) => {
				if let Some(thread) = self.0.remove(&key) {
					return Poll::Ready(join(thread));
				}
			}
			None => {}
		}
		match start() {
			Ok(thread) => {
				self.0.insert(key, thread);
				Poll::Pending
			}
			Err(error) => Poll::Ready(Err(error)),
		}
	}

	pub(super) fn busy(&self) -> bool {
		self.0.values().any(|thread| !thread.is_finished())
	}

	pub(super) fn unfinished(&self) -> Vec<TaskId> {
		let mut tasks: Vec<TaskId> = self
			.0
			.iter()
			.filter(|(_, thread)| !thread.is_finished())
			.map(|(key, _)| key.task.clone())
			.collect();
		tasks.sort_by(|a, b| a.0.cmp(&b.0));
		tasks.dedup();
		tasks
	}

	pub(super) fn forget_finished_unless(&mut self, awaited: impl Fn(&CommitKey) -> bool) {
		self.0
			.retain(|key, thread| !thread.is_finished() || awaited(key));
	}
}
