use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Child;
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail, ensure};
use nix::errno::Errno;
use nix::fcntl::{Flock, FlockArg};

mod jobs;
mod manager;
mod workers;

use jobs::reconcile_jobs;
use manager::ManagerProcess;
use workers::WorkerProcess;

use crate::agent::{AgentRun, RateLimitInfo, spawn_agent, spawn_agent_scrubbed};
use crate::atomic::write_atomic;
use crate::broker::{Grant, Machine};
use crate::claude_config::{credentials_expire_within, write_claude_settings};
use crate::config::Config;
use crate::events::{Event, emit};
use crate::job::VmStat;
use crate::journal::journal_append;
use crate::paths::{JobId, RunDir, TaskId};
use crate::run_info::RunInfo;
use crate::sandbox::{SandboxSpec, contest_bin_dir, dropped_host_env, sandbox_command};
use crate::spool::serve_pending_guarded;
use crate::status::write_status;
use crate::taint::{ScanReport, TopLevelExempt, remove_planted_files};
use crate::task::{TaskState, TaskStore};
use crate::usage::{
	five_hour_full, load_usage, over_weekly_pace, probe_usage, record_usage, rejected_until,
};

const LOOP_INTERVAL: Duration = Duration::from_secs(1);
const MAX_ACTIVE_STEP: Duration = Duration::from_secs(5);
const PERSIST_INTERVAL: Duration = Duration::from_secs(30);
const STATUS_INTERVAL: Duration = Duration::from_secs(30);
const PROBE_RETRY: Duration = Duration::from_secs(60);
const CREDENTIALS_HORIZON_SECONDS: u64 = 15 * 60;
const SECONDS_PER_MINUTE: u64 = 60;
const STREAM_FILE: &str = "stream.jsonl";
const CREDENTIALS_FILE: &str = ".claude/.credentials.json";
const BUDGET_STOP_REASON: &str = "budget";
const DEFAULT_STOP_REASON: &str = "requested";
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

	fn spawn_agent_for(&self, agent: &AgentRun, task: Option<&TaskId>) -> Result<Child> {
		if self.cfg.sandbox {
			let spec = SandboxSpec::for_run(&self.run, &self.cfg, task.cloned())?;
			spawn_agent(&self.cfg, agent, Some(sandbox_command(&spec, &[])))
		} else {
			spawn_agent_scrubbed(&self.cfg, agent, None, &dropped_host_env())
		}
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
