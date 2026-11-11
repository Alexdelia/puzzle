use std::time::{Duration, Instant};

use anyhow::Result;
use nix::sys::signal::Signal;

use super::{Supervisor, report};
use crate::broker::{Grant, JobRequest, allocate};
use crate::events::{Event, emit};
use crate::job::{
	JobRecord, JobState, list_job_records, mark_spawned, process_starttime, read_vmstat, repin,
	sandbox_job, signal_if_same_process, spawn_job, spawn_job_scrubbed, swapping_now, try_reap,
	write_job_record, write_lease,
};
use crate::journal::journal_append;
use crate::paths::RunDir;
use crate::sandbox::{SandboxSpec, dropped_host_env};
use crate::spool::cancel_job;
use crate::task::{TaskState, TaskStore};

const JOB_SPAWN_FAILED_CODE: i32 = 127;

enum JobEnd {
	Exited(i32),
	Vanished,
}

fn job_end(record: &JobRecord) -> Option<JobEnd> {
	let Some(pid) = record.pid else {
		return Some(JobEnd::Vanished);
	};
	match try_reap(pid) {
		Ok(Some(JobState::Done { code })) => Some(JobEnd::Exited(code)),
		Ok(_) => None,
		Err(_) => (record.starttime.is_none() || process_starttime(pid) != record.starttime)
			.then_some(JobEnd::Vanished),
	}
}

fn is_live_job(state: &JobState) -> bool {
	matches!(state, JobState::Running | JobState::Cancelling)
}

pub(super) fn reconcile_jobs(run: &RunDir) -> Result<()> {
	for mut record in list_job_records(run) {
		if !is_live_job(&record.state) {
			continue;
		}
		signal_if_same_process(&record, Signal::SIGKILL)?;
		record.state = match record.state {
			JobState::Cancelling => JobState::Cancelled,
			_ => JobState::Interrupted,
		};
		write_job_record(run, &record)?;
		if record.state == JobState::Interrupted {
			emit(
				run,
				Event::JobInterrupted {
					job: record.id.clone(),
				},
			)?;
		}
		journal_append(
			run,
			&format!(
				"job {id} {state:?} after restart",
				id = record.id,
				state = record.state
			),
		)?;
	}
	Ok(())
}

impl Supervisor {
	pub(super) fn cancel_jobs_of_ended_tasks(&self) -> Result<()> {
		let store = TaskStore { run: &self.run };
		for record in list_job_records(&self.run) {
			if !matches!(record.state, JobState::Queued | JobState::Running) {
				continue;
			}
			let ended = matches!(
				store.state(&record.owner_task),
				Ok(TaskState::Done | TaskState::Failed | TaskState::Cancelled)
			);
			if ended {
				cancel_job(&self.run, &record.owner_task, &record.id)?;
				journal_append(
					&self.run,
					&format!(
						"job {id} cancelled: {owner} ended",
						id = record.id,
						owner = record.owner_task
					),
				)?;
			}
		}
		Ok(())
	}

	pub(super) fn reap_jobs(&mut self) -> Result<()> {
		for record in list_job_records(&self.run) {
			if !is_live_job(&record.state) {
				continue;
			}
			match job_end(&record) {
				Some(end) => self.finish_job(record, end)?,
				None if record.state == JobState::Cancelling => self.escalate_cancel(&record)?,
				None => {}
			}
		}
		Ok(())
	}

	fn finish_job(&mut self, mut record: JobRecord, end: JobEnd) -> Result<()> {
		record.state = match (&record.state, &end) {
			(JobState::Cancelling, _) => JobState::Cancelled,
			(_, JobEnd::Exited(code)) => JobState::Done { code: *code },
			(_, JobEnd::Vanished) => JobState::Interrupted,
		};
		write_job_record(&self.run, &record)?;
		self.grants.remove(&record.id);
		self.cancelling_since.remove(&record.id);
		let job = record.id.clone();
		emit(
			&self.run,
			match end {
				JobEnd::Exited(code) => Event::JobFinished { job, code },
				JobEnd::Vanished => Event::JobInterrupted { job },
			},
		)?;
		journal_append(
			&self.run,
			&format!("job {id} {state:?}", id = record.id, state = record.state),
		)
	}

	fn escalate_cancel(&mut self, record: &JobRecord) -> Result<()> {
		let since = *self
			.cancelling_since
			.entry(record.id.clone())
			.or_insert_with(Instant::now);
		if since.elapsed() >= Duration::from_secs(self.cfg.grace_seconds) {
			signal_if_same_process(record, Signal::SIGKILL)?;
		}
		Ok(())
	}

	fn swapping(&mut self) -> bool {
		let Some(previous) = self.vmstat else {
			self.vmstat = read_vmstat().ok();
			return false;
		};
		match swapping_now(&previous) {
			Ok((swapping, current)) => {
				self.vmstat = Some(current);
				swapping
			}
			Err(error) => {
				report("vmstat", Err(error));
				false
			}
		}
	}

	pub(super) fn schedule_jobs(&mut self) -> Result<()> {
		let swapping = self.swapping();
		let records = list_job_records(&self.run);
		let requests = |live: bool| -> Vec<JobRequest> {
			records
				.iter()
				.filter(|record| {
					if live {
						is_live_job(&record.state)
					} else {
						record.state == JobState::Queued
					}
				})
				.map(|record| record.request.clone())
				.collect()
		};
		let (running, queued) = (requests(true), requests(false));
		if running.is_empty() && queued.is_empty() {
			return Ok(());
		}
		let allocation = allocate(&self.machine, &running, &queued, swapping);
		for grant in allocation.grants {
			let Some(record) = records.iter().find(|record| record.id == grant.id) else {
				continue;
			};
			let scheduled = match record.state {
				JobState::Queued if allocation.admitted.contains(&grant.id) => {
					self.launch_job(record.clone(), grant)
				}
				JobState::Running | JobState::Cancelling
					if self.grants.get(&grant.id) != Some(&grant) =>
				{
					self.regrant(record, grant)
				}
				_ => Ok(()),
			};
			report("schedule job", scheduled);
		}
		Ok(())
	}

	fn spawn_job_process(&self, record: &mut JobRecord, grant: &Grant) -> Result<u32> {
		if self.cfg.sandbox {
			let spec = SandboxSpec::for_run(&self.run, &self.cfg, Some(record.owner_task.clone()))?;
			sandbox_job(record, &spec);
			return spawn_job(&self.run, record, grant);
		}
		spawn_job_scrubbed(&self.run, record, grant, &dropped_host_env())
	}

	fn launch_job(&mut self, mut record: JobRecord, grant: Grant) -> Result<()> {
		match self.spawn_job_process(&mut record, &grant) {
			Ok(pid) => {
				mark_spawned(&mut record, pid);
				record.state = JobState::Running;
				write_job_record(&self.run, &record)?;
				journal_append(
					&self.run,
					&format!(
						"job {id} started for {owner} on cpus {cpus:?}",
						id = record.id,
						owner = record.owner_task,
						cpus = grant.cpus
					),
				)?;
				self.grants.insert(record.id.clone(), grant);
				Ok(())
			}
			Err(error) => {
				record.state = JobState::Done {
					code: JOB_SPAWN_FAILED_CODE,
				};
				write_job_record(&self.run, &record)?;
				emit(
					&self.run,
					Event::JobFinished {
						job: record.id.clone(),
						code: JOB_SPAWN_FAILED_CODE,
					},
				)?;
				journal_append(
					&self.run,
					&format!("job {id} spawn failed: {error:#}", id = record.id),
				)
			}
		}
	}

	fn regrant(&mut self, record: &JobRecord, grant: Grant) -> Result<()> {
		write_lease(&self.run, &grant)?;
		if let Some(pid) = record.pid
			&& record.starttime.is_some()
			&& process_starttime(pid) == record.starttime
		{
			repin(pid, &grant.cpus)?;
		}
		self.grants.insert(grant.id.clone(), grant);
		Ok(())
	}
}
