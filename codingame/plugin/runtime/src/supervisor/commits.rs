use std::collections::HashMap;
use std::task::Poll;
use std::thread::{self, JoinHandle};

use anyhow::{Context, Result, anyhow};

use crate::commit::commit_task;
use crate::config::Config;
use crate::paths::{RunDir, TaskId};

pub(super) type CommitThread = JoinHandle<Result<Option<String>>>;

struct InFlight {
	milestone: String,
	thread: CommitThread,
}

#[derive(Default)]
pub(super) struct Commits(HashMap<TaskId, InFlight>);

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
		task: &TaskId,
		milestone: &str,
		start: impl FnOnce() -> Result<CommitThread>,
	) -> Poll<Result<Option<String>>> {
		if self
			.0
			.get(task)
			.is_some_and(|in_flight| !in_flight.thread.is_finished())
		{
			return Poll::Pending;
		}
		if let Some(finished) = self.0.remove(task)
			&& finished.milestone == milestone
		{
			return Poll::Ready(join(finished.thread));
		}
		match start() {
			Ok(thread) => {
				self.0.insert(
					task.clone(),
					InFlight {
						milestone: milestone.to_owned(),
						thread,
					},
				);
				Poll::Pending
			}
			Err(error) => Poll::Ready(Err(error)),
		}
	}

	pub(super) fn busy(&self) -> bool {
		self.0
			.values()
			.any(|in_flight| !in_flight.thread.is_finished())
	}

	pub(super) fn unfinished(&self) -> Vec<TaskId> {
		self.0
			.iter()
			.filter(|(_, in_flight)| !in_flight.thread.is_finished())
			.map(|(task, _)| task.clone())
			.collect()
	}

	pub(super) fn forget_finished_unless(&mut self, awaited: impl Fn(&TaskId) -> bool) {
		self.0
			.retain(|task, in_flight| !in_flight.thread.is_finished() || awaited(task));
	}
}
