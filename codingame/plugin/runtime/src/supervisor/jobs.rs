use std::time::{Duration, Instant};

use anyhow::Result;
use nix::sys::signal::Signal;

use super::{Supervisor, report};
use crate::broker::{Grant, JobRequest, allocate};
use crate::events::{Event, emit};
use crate::job::{
	JobRecord, JobState, list_job_records, mark_spawned, process_starttime, read_vmstat, repin,
	sandbox_job, signal_if_same_process, signal_job, spawn_job, spawn_job_scrubbed, swapping_now,
	try_reap, write_job_record, write_lease,
};
use crate::journal::journal_append;
use crate::paths::{JobId, RunDir};
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
		if !is_live_job(&record.state) && record.state != JobState::Queued {
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

fn ended_task(store: &TaskStore, record: &JobRecord) -> bool {
	matches!(
		store.state(&record.owner_task),
		Ok(TaskState::Done | TaskState::Failed | TaskState::Cancelled)
	)
}

impl Supervisor {
	pub(super) fn cancel_jobs_of_ended_tasks(&mut self) {
		let store = TaskStore { run: &self.run };
		let orphaned: Vec<JobRecord> = list_job_records(&self.run)
			.into_iter()
			.filter(|record| matches!(record.state, JobState::Queued | JobState::Running))
			.filter(|record| ended_task(&store, record))
			.collect();
		for record in orphaned {
			let cancelled = cancel_job(&self.run, &record.owner_task, &record.id).and_then(|_| {
				journal_append(
					&self.run,
					&format!(
						"job {id} cancelled: {owner} ended",
						id = record.id,
						owner = record.owner_task
					),
				)
			});
			match cancelled {
				Ok(()) => {
					self.cancel_failures.remove(&record.id);
				}
				Err(error) => self.cancel_failed(&record.id, &error),
			}
		}
	}

	fn cancel_failed(&mut self, job: &JobId, error: &anyhow::Error) {
		if self.cancel_failures.insert(job.clone()) {
			let journaled =
				journal_append(&self.run, &format!("job {job} cancel failed: {error:#}"));
			report("cancel job", journaled);
		}
	}

	pub(super) fn has_live_jobs(&self) -> bool {
		list_job_records(&self.run)
			.iter()
			.any(|record| is_live_job(&record.state))
	}

	pub(super) fn reap_jobs(&mut self) {
		for record in list_job_records(&self.run) {
			if !is_live_job(&record.state) {
				continue;
			}
			let reaped = match job_end(&record) {
				Some(end) => self.finish_job(record, end),
				None if record.state == JobState::Cancelling => self.escalate_cancel(&record),
				None => Ok(()),
			};
			report("reap job", reaped);
		}
	}

	fn forget_job(&mut self, job: &JobId) {
		self.grants.remove(job);
		self.spawned_jobs.remove(job);
		self.cancelling_since.remove(job);
	}

	fn finish_job(&mut self, mut record: JobRecord, end: JobEnd) -> Result<()> {
		self.forget_job(&record.id);
		record.state = match (&record.state, &end) {
			(JobState::Cancelling, _) => JobState::Cancelled,
			_ if self.stopping => JobState::Interrupted,
			(_, JobEnd::Exited(code)) => JobState::Done { code: *code },
			(_, JobEnd::Vanished) => JobState::Interrupted,
		};
		write_job_record(&self.run, &record)?;
		let job = record.id.clone();
		emit(
			&self.run,
			match (&record.state, end) {
				(JobState::Interrupted, _) | (_, JobEnd::Vanished) => Event::JobInterrupted { job },
				(_, JobEnd::Exited(code)) => Event::JobFinished { job, code },
			},
		)?;
		journal_append(
			&self.run,
			&format!("job {id} {state:?}", id = record.id, state = record.state),
		)
	}

	fn signal_job_of(&self, record: &JobRecord, signal: Signal) -> Result<bool> {
		match self.spawned_jobs.get(&record.id) {
			Some(&pid) => signal_job(pid, record.sandboxed, signal).map(|()| true),
			None => signal_if_same_process(record, signal),
		}
	}

	fn escalate_cancel(&mut self, record: &JobRecord) -> Result<()> {
		let since = *self
			.cancelling_since
			.entry(record.id.clone())
			.or_insert_with(Instant::now);
		if since.elapsed() >= Duration::from_secs(self.cfg.grace_seconds) {
			self.signal_job_of(record, Signal::SIGKILL)?;
		}
		Ok(())
	}

	pub(super) fn terminate_jobs(&mut self) {
		for record in list_job_records(&self.run) {
			match record.state {
				JobState::Running => {
					let signalled = self.signal_job_of(&record, Signal::SIGTERM);
					report("stop job", signalled.map(drop));
				}
				JobState::Queued => self.interrupt_job(record),
				_ => {}
			}
		}
	}

	pub(super) fn kill_live_jobs(&self) {
		for record in list_job_records(&self.run) {
			if is_live_job(&record.state) {
				let killed = self.signal_job_of(&record, Signal::SIGKILL);
				report("kill job", killed.map(drop));
			}
		}
	}

	pub(super) fn interrupt_remaining_jobs(&mut self) {
		for record in list_job_records(&self.run) {
			if record.state == JobState::Queued || is_live_job(&record.state) {
				self.interrupt_job(record);
			}
		}
	}

	fn interrupt_job(&mut self, mut record: JobRecord) {
		self.forget_job(&record.id);
		record.state = match record.state {
			JobState::Cancelling => JobState::Cancelled,
			_ => JobState::Interrupted,
		};
		let job = record.id.clone();
		let interrupted = write_job_record(&self.run, &record)
			.and_then(|()| match record.state {
				JobState::Interrupted => {
					emit(&self.run, Event::JobInterrupted { job: job.clone() })
				}
				_ => Ok(()),
			})
			.and_then(|()| {
				journal_append(
					&self.run,
					&format!("job {job} {state:?} at stop", state = record.state),
				)
			});
		report("stop job", interrupted);
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
			let spec = SandboxSpec {
				gpu: grant.gpu,
				..SandboxSpec::for_run(&self.run, &self.cfg, Some(record.owner_task.clone()))?
			};
			sandbox_job(record, &spec)?;
			return spawn_job(&self.run, record, grant);
		}
		spawn_job_scrubbed(&self.run, record, grant, &dropped_host_env())
	}

	fn launch_job(&mut self, mut record: JobRecord, grant: Grant) -> Result<()> {
		match self.spawn_job_process(&mut record, &grant) {
			Ok(pid) => {
				mark_spawned(&mut record, pid);
				record.state = JobState::Running;
				self.spawned_jobs.insert(record.id.clone(), pid);
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
