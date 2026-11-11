use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, ExitStatus};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail, ensure};
use nix::errno::Errno;
use nix::fcntl::{Flock, FlockArg};
use nix::sys::signal::Signal;

use crate::action::{Action, apply_actions, parse_actions};
use crate::agent::{
	AgentRun, RateLimitInfo, StreamOutcome, parse_stream, spawn_agent, spawn_agent_scrubbed,
};
use crate::atomic::write_atomic;
use crate::broker::{Grant, JobRequest, Machine, allocate};
use crate::claude_config::{agent_env, credentials_expire_within, write_claude_settings};
use crate::config::Config;
use crate::digest::{ack_inbox, build_digest};
use crate::events::{Event, emit};
use crate::job::{
	JobRecord, JobState, VmStat, list_job_records, mark_spawned, process_starttime, read_vmstat,
	repin, sandbox_job, signal_if_same_process, spawn_job, swapping_now, try_reap,
	write_job_record, write_lease,
};
use crate::journal::journal_append;
use crate::ledger::ledger_append;
use crate::paths::{JobId, RunDir, TaskId};
use crate::run_info::RunInfo;
use crate::sandbox::{SandboxSpec, contest_bin_dir, dropped_host_env, sandbox_command};
use crate::spool::{prepare_task_requests, serve_pending_guarded};
use crate::status::write_status;
use crate::taint::{ScanReport, TopLevelExempt, read_worker_file, remove_planted_files};
use crate::task::{TaskSpec, TaskState, TaskStore};
use crate::usage::{
	five_hour_full, load_usage, over_weekly_pace, probe_usage, record_usage, rejected_until,
};

const LOOP_INTERVAL: Duration = Duration::from_secs(1);
const MAX_ACTIVE_STEP: Duration = Duration::from_secs(5);
const PERSIST_INTERVAL: Duration = Duration::from_secs(30);
const STATUS_INTERVAL: Duration = Duration::from_secs(30);
const PROBE_RETRY: Duration = Duration::from_secs(60);
const MANAGER_RETRY_BASE: Duration = Duration::from_secs(60);
const MANAGER_RETRY_MAX_DOUBLINGS: u32 = 10;
const CREDENTIALS_HORIZON_SECONDS: u64 = 15 * 60;
const SECONDS_PER_MINUTE: u64 = 60;
const NOTES_PROMPT_MAX_BYTES: usize = 16 * 1024;
const MANAGER_TOOLS: [&str; 3] = ["Read", "Glob", "Grep"];
const MANAGER_WHO: &str = "manager";
const STREAM_FILE: &str = "stream.jsonl";
const MANAGER_PROMPT: &str = "prompts/manager.md";
const WORKER_PROMPT: &str = "prompts/worker.md";
const CREDENTIALS_FILE: &str = ".claude/.credentials.json";
const EXITED_WITHOUT_OUTCOME: &str = "exited without outcome";
const BUDGET_STOP_REASON: &str = "budget";
const DEFAULT_STOP_REASON: &str = "requested";
const JOB_SPAWN_FAILED_CODE: i32 = 127;
const NONE_TEXT: &str = "(none)";
const GIT_DIR_NAME: &str = ".git";

pub fn active_increment(elapsed: Duration) -> Duration {
	elapsed.min(MAX_ACTIVE_STEP)
}

fn unix_now() -> i64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_or(0, |elapsed| elapsed.as_secs() as i64)
}

fn report(step: &str, result: Result<()>) {
	if let Err(error) = result {
		eprintln!("supervisor: {step}: {error:#}");
	}
}

fn read_run_info(run: &RunDir) -> Result<RunInfo> {
	let path = run.run_json();
	let bytes = fs::read(&path).with_context(|| format!("read {path:?}"))?;
	serde_json::from_slice(&bytes).with_context(|| format!("parse {path:?}"))
}

fn lock_pid_file(run: &RunDir) -> Result<Flock<File>> {
	let path = run.supervisor_pid();
	let file = OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.truncate(false)
		.open(&path)
		.with_context(|| format!("open {path:?}"))?;
	let lock = match Flock::lock(file, FlockArg::LockExclusiveNonblock) {
		Ok(lock) => lock,
		Err((_, Errno::EWOULDBLOCK)) => bail!(
			"supervisor already running for {root:?} (pid {pid})",
			root = run.root,
			pid = fs::read_to_string(&path).unwrap_or_default().trim()
		),
		Err((_, errno)) => return Err(errno).with_context(|| format!("lock {path:?}")),
	};
	lock.set_len(0)?;
	(&*lock).write_all(std::process::id().to_string().as_bytes())?;
	Ok(lock)
}

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
	let Ok(bytes) = read_worker_file(&run.task_dir(id).join("notes.md")) else {
		return NONE_TEXT.to_owned();
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

fn remove_planted(run: &RunDir, exempt: TopLevelExempt) -> Result<ScanReport> {
	let report = remove_planted_files(&run.root, exempt)?;
	for path in &report.removed {
		journal_append(
			run,
			&format!("taint: removed {path}", path = path.display()),
		)?;
	}
	for (path, error) in &report.failures {
		journal_append(
			run,
			&format!(
				"taint: cannot remove {path}: {error}",
				path = path.display()
			),
		)?;
	}
	Ok(report)
}

fn taint_failure(report: Result<ScanReport>) -> Option<String> {
	match report {
		Ok(report) if !report.must_fail_closed() => None,
		Ok(report) => Some(format!(
			"{count} planted paths could not be removed",
			count = report.failures.len()
		)),
		Err(error) => Some(format!("{error:#}")),
	}
}

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

fn reconcile_jobs(run: &RunDir) -> Result<()> {
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

struct WorkerProcess {
	task: TaskId,
	child: Child,
	model: String,
	started: Instant,
	stream_log: PathBuf,
}

struct ManagerProcess {
	child: Child,
	model: String,
	started: Instant,
	stream_log: PathBuf,
	cursor: u64,
	inbox: Vec<PathBuf>,
}

struct RunLimits {
	active_budget_seconds: Option<u64>,
	deadline_unix: Option<i64>,
}

struct Supervisor {
	run: RunDir,
	cfg: Config,
	bin_dir: PathBuf,
	credentials: Option<PathBuf>,
	exempt: TopLevelExempt,
	machine: Machine,
	limits: RunLimits,
	workers: Vec<WorkerProcess>,
	manager: Option<ManagerProcess>,
	grants: HashMap<JobId, Grant>,
	cancelling_since: HashMap<JobId, Instant>,
	vmstat: Option<VmStat>,
	active: Duration,
	last_iteration: Instant,
	last_persist: Instant,
	last_status: Option<Instant>,
	last_tick: Option<Instant>,
	manager_retry_at: Option<Instant>,
	manager_failures: u32,
	last_probe: Option<Instant>,
	blocked_until: Option<i64>,
	pid_lock: Flock<File>,
}

pub fn run_supervisor(run: RunDir, cfg: Config) -> Result<()> {
	let mut supervisor = Supervisor::start(run, cfg)?;
	loop {
		let began = Instant::now();
		if let Some(reason) = supervisor.stop_reason() {
			return supervisor.shut_down(&reason);
		}
		supervisor.iterate();
		sleep(LOOP_INTERVAL.saturating_sub(began.elapsed()));
	}
}

impl Supervisor {
	fn start(run: RunDir, cfg: Config) -> Result<Self> {
		let root = fs::canonicalize(&run.root)
			.with_context(|| format!("resolve {root:?}", root = run.root))?;
		let run = RunDir { root };
		let git = run.root.join(GIT_DIR_NAME);
		ensure!(
			fs::symlink_metadata(&git).is_err(),
			"contest dir holds {git:?}: the taint scan removes a top-level .git, use a subdir of the repo"
		);
		for dir in [
			run.tasks(),
			run.jobs(),
			run.inbox(),
			run.requests(),
			run.agents(),
		] {
			fs::create_dir_all(&dir).with_context(|| format!("create {dir:?}"))?;
		}
		let pid_lock = lock_pid_file(&run)?;
		let exempt = TopLevelExempt::observe(&run.root);
		for id in (TaskStore { run: &run }).reconcile_after_restart()? {
			journal_append(&run, &format!("requeued {id} after restart"))?;
		}
		reconcile_jobs(&run)?;
		remove_planted(&run, exempt)?;
		write_claude_settings(&run, &cfg)?;
		let info = read_run_info(&run)?;
		let now = Instant::now();
		Ok(Self {
			bin_dir: contest_bin_dir()?,
			credentials: std::env::var_os("HOME")
				.map(|home| PathBuf::from(home).join(CREDENTIALS_FILE)),
			exempt,
			machine: Machine::detect(cfg.ram_budget_mb),
			limits: RunLimits {
				active_budget_seconds: info.active_budget_seconds,
				deadline_unix: info.deadline_unix,
			},
			workers: Vec::new(),
			manager: None,
			grants: HashMap::new(),
			cancelling_since: HashMap::new(),
			vmstat: None,
			active: Duration::from_secs(info.active_seconds),
			last_iteration: now,
			last_persist: now,
			last_status: None,
			last_tick: None,
			manager_retry_at: None,
			manager_failures: 0,
			last_probe: None,
			blocked_until: None,
			pid_lock,
			run,
			cfg,
		})
	}

	fn stop_reason(&self) -> Option<String> {
		let reason = fs::read_to_string(self.run.stop_file()).ok()?;
		let reason = reason.trim();
		Some(if reason.is_empty() {
			DEFAULT_STOP_REASON.to_owned()
		} else {
			reason.to_owned()
		})
	}

	fn shut_down(mut self, reason: &str) -> Result<()> {
		self.persist()?;
		journal_append(&self.run, &format!("stopped: {reason}"))?;
		self.pid_lock.set_len(0)?;
		write_status(&self.run, unix_now())
	}

	fn iterate(&mut self) {
		let now = unix_now();
		self.refresh_credentials(now);
		let served = self.serve_requests();
		report("spool", served);
		let reaped = self.reap_workers(now);
		report("reap workers", reaped);
		let jobs = self.reap_jobs().and_then(|()| self.schedule_jobs());
		report("jobs", jobs);
		let manager = self.manager_step(now);
		report("manager", manager);
		let launched = self.launch_workers(now);
		report("launch", launched);
		let accounted = self.account_time(now);
		report("time", accounted);
		let status = self.refresh_status(now);
		report("status", status);
	}

	fn probe_allowed(&self) -> bool {
		self.last_probe
			.is_none_or(|probed| probed.elapsed() >= PROBE_RETRY)
	}

	fn probe(&mut self, now: i64) {
		self.last_probe = Some(Instant::now());
		let recorded = match probe_usage(&self.cfg) {
			Some(info) => record_usage(&self.run, &info, now),
			None => journal_append(&self.run, "usage probe failed"),
		};
		report("usage probe", recorded);
	}

	fn refresh_credentials(&mut self, now: i64) {
		let expiring = self.credentials.as_ref().is_some_and(|path| {
			credentials_expire_within(path, CREDENTIALS_HORIZON_SECONDS).unwrap_or(false)
		});
		if expiring && self.probe_allowed() {
			self.probe(now);
		}
	}

	fn fresh_usage(&mut self, now: i64) -> Option<RateLimitInfo> {
		let max_age = (self.cfg.usage_max_age_minutes * SECONDS_PER_MINUTE) as i64;
		let fresh = |run: &RunDir| {
			load_usage(run)
				.filter(|(_, recorded_at)| now - recorded_at <= max_age)
				.map(|(info, _)| info)
		};
		if let Some(info) = fresh(&self.run) {
			return Some(info);
		}
		if !self.probe_allowed() {
			return None;
		}
		self.probe(now);
		fresh(&self.run)
	}

	fn usage_allows(&mut self, info: &RateLimitInfo, now: i64) -> bool {
		if let Some(until) = rejected_until(info, now) {
			if self.blocked_until != Some(until) {
				self.blocked_until = Some(until);
				report("usage", emit(&self.run, Event::UsageBlocked { until }));
			}
			return false;
		}
		!five_hour_full(info, now, self.workers.len(), &self.cfg)
			&& !over_weekly_pace(info, now, &self.cfg)
	}

	fn serve_requests(&mut self) -> Result<()> {
		let tainted = RefCell::new(Vec::new());
		let run = &self.run;
		let exempt = self.exempt;
		let guard = |task: &TaskId| -> Result<()> {
			if let Some(failure) = taint_failure(remove_planted(run, exempt)) {
				tainted.borrow_mut().push((task.clone(), failure.clone()));
				bail!("commit refused, taint scan failed: {failure}");
			}
			Ok(())
		};
		let served = serve_pending_guarded(run, &self.cfg, &guard);
		for (task, failure) in tainted.into_inner() {
			self.flag_tainted(&task, TaskState::Running, &failure);
			self.kill_worker(&task);
		}
		served.map(drop)
	}

	fn flag_tainted(&self, id: &TaskId, state: TaskState, failure: &str) -> TaskState {
		let store = TaskStore { run: &self.run };
		let note = format!("taint scan failed: {failure}");
		report("taint", journal_append(&self.run, &format!("{id}: {note}")));
		if !state.can_become(TaskState::Failed) {
			return state;
		}
		let flagged = store
			.transition(id, TaskState::Failed)
			.and_then(|()| store.append_notes(id, &format!("\n{note}\n")));
		report("taint", flagged);
		store.state(id).unwrap_or(TaskState::Failed)
	}

	fn kill_worker(&mut self, id: &TaskId) {
		for worker in self.workers.iter_mut().filter(|worker| &worker.task == id) {
			report("kill worker", worker.child.kill().map_err(Into::into));
		}
	}

	fn kill_cancelled_workers(&mut self) {
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

	fn reap_workers(&mut self, now: i64) -> Result<()> {
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

	fn record_agent(
		&self,
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
			report("usage", record_usage(&self.run, info, now));
		}
	}

	fn finish_worker(&mut self, worker: WorkerProcess, now: i64) -> Result<()> {
		let store = TaskStore { run: &self.run };
		let id = &worker.task;
		let mut state = store.state(id)?;
		if state == TaskState::Running {
			store.transition(id, TaskState::Failed)?;
			report(
				"notes",
				store.append_notes(id, &format!("\n{EXITED_WITHOUT_OUTCOME}\n")),
			);
			state = TaskState::Failed;
		}
		let outcome = parse_stream(&worker.stream_log);
		self.record_agent(&id.0, &worker.model, &outcome, worker.started, now);
		if let Some(failure) = taint_failure(remove_planted(&self.run, self.exempt)) {
			state = self.flag_tainted(id, state, &failure);
		}
		emit(
			&self.run,
			Event::TaskFinished {
				task: id.clone(),
				state,
			},
		)?;
		journal_append(&self.run, &format!("finished {id} {state:?}"))
	}

	fn reap_jobs(&mut self) -> Result<()> {
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

	fn schedule_jobs(&mut self) -> Result<()> {
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
		}
		spawn_job(&self.run, record, grant)
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

	fn spawn_agent_for(&self, agent: &AgentRun, task: Option<&TaskId>) -> Result<Child> {
		if self.cfg.sandbox {
			let spec = SandboxSpec::for_run(&self.run, &self.cfg, task.cloned())?;
			spawn_agent(&self.cfg, agent, Some(sandbox_command(&spec, &[])))
		} else {
			spawn_agent_scrubbed(&self.cfg, agent, None, &dropped_host_env())
		}
	}

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

	fn manager_step(&mut self, now: i64) -> Result<()> {
		if let Some(manager) = self.manager.as_mut() {
			let Some(status) = manager.child.try_wait()? else {
				return Ok(());
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

	fn finish_manager(&mut self, manager: ManagerProcess, status: ExitStatus, now: i64) {
		let outcome = parse_stream(&manager.stream_log);
		self.record_agent(MANAGER_WHO, &manager.model, &outcome, manager.started, now);
		match manager_actions(status, &outcome) {
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
		for error in errors {
			self.manager_failed(&error);
		}
	}

	fn manager_failed(&mut self, error: &str) {
		self.manager_failures += 1;
		let doublings = (self.manager_failures - 1).min(MANAGER_RETRY_MAX_DOUBLINGS);
		let heartbeat = Duration::from_secs(self.cfg.heartbeat_minutes * SECONDS_PER_MINUTE);
		let backoff = (MANAGER_RETRY_BASE * 2u32.pow(doublings)).min(heartbeat);
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

	fn launch_workers(&mut self, now: i64) -> Result<()> {
		while self.workers.len() < self.cfg.workers {
			let Some(id) = (TaskStore { run: &self.run }).next_queued()? else {
				break;
			};
			if self.workers.iter().any(|worker| worker.task == id) {
				break;
			}
			let Some(info) = self.fresh_usage(now) else {
				break;
			};
			if !self.usage_allows(&info, now) {
				break;
			}
			TaskStore { run: &self.run }.transition(&id, TaskState::Running)?;
			if let Err(error) = self.launch_worker(&id) {
				self.fail_launch(&id, &error)?;
			}
		}
		Ok(())
	}

	fn launch_worker(&mut self, id: &TaskId) -> Result<()> {
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
		journal_append(
			&self.run,
			&format!("launched {id} on {model}", model = agent.model),
		)
	}

	fn fail_launch(&self, id: &TaskId, error: &anyhow::Error) -> Result<()> {
		let store = TaskStore { run: &self.run };
		store.transition(id, TaskState::Failed)?;
		report(
			"notes",
			store.append_notes(id, &format!("\nlaunch failed: {error:#}\n")),
		);
		emit(
			&self.run,
			Event::TaskFinished {
				task: id.clone(),
				state: TaskState::Failed,
			},
		)?;
		journal_append(&self.run, &format!("launch of {id} failed: {error:#}"))
	}

	fn persist(&mut self) -> Result<()> {
		let mut info = read_run_info(&self.run)?;
		info.active_seconds = self.active.as_secs();
		write_atomic(&self.run.run_json(), &serde_json::to_vec_pretty(&info)?)?;
		self.last_persist = Instant::now();
		Ok(())
	}

	fn limit_reached(&self, now: i64) -> bool {
		self.limits
			.active_budget_seconds
			.is_some_and(|budget| self.active.as_secs() >= budget)
			|| self
				.limits
				.deadline_unix
				.is_some_and(|deadline| now >= deadline)
	}

	fn account_time(&mut self, now: i64) -> Result<()> {
		self.active += active_increment(self.last_iteration.elapsed());
		self.last_iteration = Instant::now();
		if self.last_persist.elapsed() >= PERSIST_INTERVAL {
			self.persist()?;
		}
		if self.limit_reached(now) && !self.run.stop_file().exists() {
			write_atomic(&self.run.stop_file(), BUDGET_STOP_REASON.as_bytes())?;
			emit(&self.run, Event::BudgetExhausted)?;
			journal_append(&self.run, "budget reached, stop requested")?;
		}
		Ok(())
	}

	fn refresh_status(&mut self, now: i64) -> Result<()> {
		if self
			.last_status
			.is_some_and(|written| written.elapsed() < STATUS_INTERVAL)
		{
			return Ok(());
		}
		self.last_status = Some(Instant::now());
		write_status(&self.run, now)
	}
}
