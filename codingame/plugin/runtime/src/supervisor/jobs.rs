use std::time::{Duration, Instant};

use anyhow::Result;
use nix::sys::signal::Signal;

use super::{Supervisor, report};
use crate::broker::{Grant, JobRequest, Load, RunningJob, allocate, mem_available_mb};
use crate::events::{Event, emit};
use crate::job::{
	JobRecord, JobState, Lease, list_job_records, mark_spawned, pin_process_threads,
	process_starttime, read_vmstat, repin, sandbox_job, signal_if_same_process, signal_job,
	spawn_job, spawn_job_scrubbed, swapping_now, try_reap, write_job_record, write_lease,
};
use crate::journal::{journal_append, utc_time};
use crate::paths::{JobId, RunDir};
use crate::sandbox::{SandboxSpec, dropped_host_env};
use crate::spool::cancel_job;
use crate::task::{TaskState, TaskStore};

const JOB_SPAWN_FAILED_CODE: i32 = 127;
const AGENT_REPIN_INTERVAL: Duration = Duration::from_secs(10);

struct Drain {
	cpus: Vec<usize>,
	effective_at: i64,
}

pub(super) struct HeldLease {
	grant: Grant,
	generation: u64,
	drain: Option<Drain>,
}

impl HeldLease {
	fn lease(&self) -> Lease {
		Lease {
			generation: self.generation,
			pending_cpus: self.drain.as_ref().map(|drain| drain.cpus.clone()),
			effective_at: self.drain.as_ref().map(|drain| drain.effective_at),
			..Lease::first(&self.grant)
		}
	}
}

fn same_process(record: &JobRecord) -> bool {
	record
		.pid
		.is_some_and(|pid| record.starttime.is_some() && process_starttime(pid) == record.starttime)
}

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
		let orphaned: Vec<JobRecord> = self
			.jobs
			.current(&self.run)
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

	pub(super) fn has_live_jobs(&mut self) -> bool {
		self.jobs
			.current(&self.run)
			.iter()
			.any(|record| is_live_job(&record.state))
	}

	pub(super) fn reap_jobs(&mut self) {
		for record in self.jobs.current(&self.run) {
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
		self.leases.remove(job);
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
		for record in self.jobs.current(&self.run) {
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

	pub(super) fn kill_live_jobs(&mut self) {
		for record in self.jobs.current(&self.run) {
			if is_live_job(&record.state) {
				let killed = self.signal_job_of(&record, Signal::SIGKILL);
				report("kill job", killed.map(drop));
			}
		}
	}

	pub(super) fn interrupt_remaining_jobs(&mut self) {
		for record in self.jobs.current(&self.run) {
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

	pub(super) fn agents_running(&self) -> usize {
		self.workers.len() + usize::from(self.manager.is_some()) + usize::from(self.probe.is_some())
	}

	fn agent_pids(&self) -> Vec<u32> {
		let workers = self.workers.iter().map(|worker| worker.child.id());
		let manager = self.manager.iter().map(|manager| manager.child.id());
		let probe = self.probe.iter().map(|probe| probe.probe.child.id());
		workers.chain(manager).chain(probe).collect()
	}

	fn pin_agents(&mut self, cpus: &[usize]) -> Result<()> {
		let shares_with_timed = cpus != self.machine.all_cpus();
		let reinforce = shares_with_timed && self.last_agent_pin.elapsed() >= AGENT_REPIN_INTERVAL;
		if self.agent_cpus == cpus && !reinforce {
			return Ok(());
		}
		self.last_agent_pin = Instant::now();
		pin_process_threads(std::process::id(), cpus)?;
		for pid in self.agent_pids() {
			repin(pid, cpus)?;
		}
		self.agent_cpus = cpus.to_vec();
		Ok(())
	}

	fn finish_due_drains(&mut self, records: &[JobRecord], now: i64) -> Result<()> {
		for record in records {
			let Some(held) = self.leases.get_mut(&record.id) else {
				continue;
			};
			let Some(drain) = held.drain.take_if(|drain| drain.effective_at <= now) else {
				continue;
			};
			held.grant.cpus = drain.cpus;
			held.generation += 1;
			write_lease(&self.run, &record.id, &held.lease())?;
			if same_process(record) {
				repin(record.pid.unwrap_or_default(), &held.grant.cpus)?;
			}
			journal_append(
				&self.run,
				&format!(
					"job {id} drained to cpus {cpus:?}",
					id = record.id,
					cpus = held.grant.cpus
				),
			)?;
		}
		Ok(())
	}

	fn start_drain(&mut self, id: &JobId, cpus: Vec<usize>, now: i64) -> Result<()> {
		let Some(held) = self.leases.get_mut(id) else {
			return Ok(());
		};
		let effective_at = now + self.cfg.drain_seconds as i64;
		held.generation += 1;
		held.drain = Some(Drain {
			cpus: cpus.clone(),
			effective_at,
		});
		write_lease(&self.run, id, &held.lease())?;
		journal_append(
			&self.run,
			&format!(
				"job {id} keeps its cpus until {time}, then shrinks to {cpus:?}",
				time = utc_time(effective_at)
			),
		)
	}

	fn running_jobs(&self, records: &[JobRecord]) -> Vec<RunningJob> {
		records
			.iter()
			.filter(|record| is_live_job(&record.state))
			.map(|record| {
				let held = self.leases.get(&record.id);
				RunningJob {
					request: record.request.clone(),
					cpus: held.map(|held| held.grant.cpus.clone()).unwrap_or_default(),
					draining_to: held
						.and_then(|held| held.drain.as_ref())
						.map(|drain| drain.cpus.clone()),
				}
			})
			.collect()
	}

	pub(super) fn schedule_jobs(&mut self, now: i64) -> Result<()> {
		let swapping = self.swapping();
		let records = self.jobs.current(&self.run);
		self.finish_due_drains(&records, now)?;
		let running = self.running_jobs(&records);
		let queued: Vec<JobRequest> = records
			.iter()
			.filter(|record| record.state == JobState::Queued)
			.map(|record| record.request.clone())
			.collect();
		let load = Load {
			swapping,
			agent_ram_mb: self.agents_running() as u64 * self.cfg.agent_reserve_mb,
			mem_available_mb: mem_available_mb(),
		};
		let allocation = allocate(&self.machine, &running, &queued, &load);
		self.pin_agents(&allocation.agent_cpus)?;
		for (id, cpus) in allocation.shrinks {
			report("drain job", self.start_drain(&id, cpus, now));
		}
		let mut launches = Vec::new();
		for grant in allocation.grants {
			let Some(record) = records.iter().find(|record| record.id == grant.id) else {
				continue;
			};
			match record.state {
				JobState::Queued if allocation.admitted.contains(&grant.id) => {
					launches.push((record.clone(), grant));
				}
				JobState::Running | JobState::Cancelling => {
					report("schedule job", self.regrant(record, grant));
				}
				_ => {}
			}
		}
		for (record, grant) in launches {
			report("schedule job", self.launch_job(record, grant));
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
				self.leases.insert(
					record.id.clone(),
					HeldLease {
						grant,
						generation: 1,
						drain: None,
					},
				);
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
		let Some(held) = self.leases.get_mut(&record.id) else {
			self.leases.insert(
				record.id.clone(),
				HeldLease {
					grant: grant.clone(),
					generation: 1,
					drain: None,
				},
			);
			write_lease(&self.run, &record.id, &Lease::first(&grant))?;
			if same_process(record) {
				repin(record.pid.unwrap_or_default(), &grant.cpus)?;
			}
			return Ok(());
		};
		if held.grant == grant {
			return Ok(());
		}
		let moved = held.grant.cpus != grant.cpus;
		held.grant = grant;
		held.generation += 1;
		write_lease(&self.run, &record.id, &held.lease())?;
		if moved && same_process(record) {
			repin(record.pid.unwrap_or_default(), &held.grant.cpus)?;
		}
		Ok(())
	}
}
