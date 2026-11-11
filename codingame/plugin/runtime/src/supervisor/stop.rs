use std::thread::sleep;
use std::time::{Duration, Instant};

use anyhow::Result;

use super::{Supervisor, report, unix_now};
use crate::journal::journal_append;
use crate::task::{TaskState, TaskStore};

const WRAP_UP_INTERVAL: Duration = Duration::from_millis(250);
const SETTLE_INTERVAL: Duration = Duration::from_millis(50);
pub const STOP_SETTLE_SECONDS: u64 = 10;

impl Supervisor {
	pub(super) fn stop(mut self, reason: &str) -> Result<()> {
		let began = Instant::now();
		self.begin_stop(reason);
		let grace = Duration::from_secs(self.cfg.grace_seconds);
		while self.has_live_work() && began.elapsed() < grace {
			let step = Instant::now();
			self.wrap_up_step();
			sleep(WRAP_UP_INTERVAL.saturating_sub(step.elapsed()));
		}
		self.sweep();
		self.shut_down(reason)
	}

	fn begin_stop(&mut self, reason: &str) {
		self.stopping = true;
		let now = unix_now();
		let journaled = journal_append(
			&self.run,
			&format!(
				"stopping: {reason}, workers get {grace} s",
				grace = self.cfg.grace_seconds
			),
		);
		report("stop", journaled);
		self.discard_manager(now);
		self.discard_probe();
		self.terminate_jobs();
		let status = self.write_status_now(now);
		report("status", status);
	}

	fn wrap_up_step(&mut self) {
		let now = unix_now();
		let served = self.serve_requests();
		report("spool", served);
		let reaped = self.reap_workers(now);
		report("reap workers", reaped);
		self.reap_jobs();
		let accounted = self.account_time(now);
		report("time", accounted);
		let status = self.refresh_status(now);
		report("status", status);
	}

	fn has_live_work(&self) -> bool {
		!self.workers.is_empty() || self.commits.busy() || self.has_live_jobs()
	}

	fn sweep(&mut self) {
		for worker in &mut self.workers {
			report("kill worker", worker.child.kill().map_err(Into::into));
		}
		self.kill_live_jobs();
		let settle_end = Instant::now() + Duration::from_secs(STOP_SETTLE_SECONDS);
		while self.has_live_work() && Instant::now() < settle_end {
			self.wrap_up_step();
			sleep(SETTLE_INTERVAL);
		}
		let now = unix_now();
		for worker in std::mem::take(&mut self.workers) {
			let finished = self.finish_worker(worker, now);
			report("finish worker", finished);
		}
		self.interrupt_remaining_jobs();
		for task in self.commits.unfinished() {
			let journaled =
				journal_append(&self.run, &format!("commit of {task} abandoned at stop"));
			report("stop", journaled);
		}
		self.interrupt_running_tasks();
	}

	fn interrupt_running_tasks(&self) {
		let store = TaskStore { run: &self.run };
		let tasks = match store.list() {
			Ok(tasks) => tasks,
			Err(error) => return report("stop", Err(error)),
		};
		for task in tasks.iter().filter(|task| task.state == TaskState::Running) {
			let interrupted = store
				.transition(&task.id, TaskState::Interrupted)
				.and_then(|()| {
					journal_append(
						&self.run,
						&format!("interrupted {id} at stop", id = task.id),
					)
				});
			report("stop", interrupted);
		}
	}
}
