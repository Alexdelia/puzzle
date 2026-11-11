use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::FileExt;
use std::path::PathBuf;
use std::process::Child;
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use nix::errno::Errno;
use nix::fcntl::{Flock, FlockArg};

mod commits;
mod jobs;
mod manager;
mod probe;
mod stop;
mod workers;

pub use stop::STOP_SETTLE_SECONDS;

use commits::{CommitThread, Commits, spawn_commit};
use jobs::reconcile_jobs;
use manager::ManagerProcess;
use probe::RunningProbe;
use workers::WorkerProcess;

use crate::agent::{AgentRun, kill_agent_group, spawn_agent, spawn_agent_scrubbed};
use crate::atomic::write_atomic;
use crate::broker::{Grant, Machine};
use crate::claude_config::write_claude_settings;
use crate::commit::unstage_task;
use crate::config::Config;
use crate::events::{Event, emit};
use crate::job::VmStat;
use crate::journal::{journal_append, journal_tail};
use crate::paths::{JobId, RunDir, TaskId};
use crate::run_info::RunInfo;
use crate::sandbox::{SandboxSpec, contest_bin_dir, dropped_host_env, sandbox_command};
use crate::spool::{CommitRequest, request_pending, serve_pending};
use crate::status::write_status;
use crate::taint::{ScanReport, TopLevelExempt, ensure_no_top_level_git, remove_planted_files};
use crate::task::{TaskState, TaskStore};

const LOOP_INTERVAL: Duration = Duration::from_secs(1);
const MAX_ACTIVE_STEP: Duration = Duration::from_secs(5);
const PERSIST_INTERVAL: Duration = Duration::from_secs(30);
const STATUS_INTERVAL: Duration = Duration::from_secs(30);
const RETRY_BASE: Duration = Duration::from_secs(60);
const RETRY_MAX_DOUBLINGS: u32 = 10;
const SECONDS_PER_MINUTE: u64 = 60;
const STREAM_FILE: &str = "stream.jsonl";
const CREDENTIALS_FILE: &str = ".claude/.credentials.json";
const BUDGET_STOP_REASON: &str = "budget";
const KILL_REAP_TIMEOUT: Duration = Duration::from_secs(3);
const KILL_REAP_POLL: Duration = Duration::from_millis(20);
const PID_LOCK_RETRY: Duration = Duration::from_secs(1);
const PID_LOCK_POLL: Duration = Duration::from_millis(20);
const STOPPED: &str = "stopped: ";
pub const DEFAULT_STOP_REASON: &str = "requested";

pub fn active_increment(elapsed: Duration) -> Duration {
	elapsed.min(MAX_ACTIVE_STEP)
}

fn unix_now() -> i64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_or(0, |elapsed| elapsed.as_secs() as i64)
}

fn retry_backoff(failures: u32, heartbeat_minutes: u64) -> Duration {
	let doublings = failures.saturating_sub(1).min(RETRY_MAX_DOUBLINGS);
	let heartbeat = Duration::from_secs(heartbeat_minutes * SECONDS_PER_MINUTE);
	(RETRY_BASE * 2u32.pow(doublings)).min(heartbeat)
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
	let mut file = OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.truncate(false)
		.open(&path)
		.with_context(|| format!("open {path:?}"))?;
	let deadline = Instant::now() + PID_LOCK_RETRY;
	loop {
		match Flock::lock(file, FlockArg::LockExclusiveNonblock) {
			Ok(lock) => {
				lock.set_len(0)?;
				return Ok(lock);
			}
			Err((returned, Errno::EWOULDBLOCK)) if Instant::now() < deadline => {
				file = returned;
				sleep(PID_LOCK_POLL);
			}
			Err((_, Errno::EWOULDBLOCK)) => bail!(
				"supervisor already running for {root:?} (pid {pid})",
				root = run.root,
				pid = fs::read_to_string(&path).unwrap_or_default().trim()
			),
			Err((_, errno)) => return Err(errno).with_context(|| format!("lock {path:?}")),
		}
	}
}

fn read_stop_reason(run: &RunDir) -> Option<String> {
	let reason = fs::read_to_string(run.stop_file()).ok()?;
	let reason = reason.trim();
	Some(if reason.is_empty() {
		DEFAULT_STOP_REASON.to_owned()
	} else {
		reason.to_owned()
	})
}

fn journal_resumed_after_stop(run: &RunDir) -> Result<()> {
	let last = journal_tail(run, 1);
	let Some((_, reason)) = last.first().and_then(|line| line.split_once(STOPPED)) else {
		return Ok(());
	};
	journal_append(run, &format!("resumed after stop: {reason}"))
}

fn abandoned_commits(run: &RunDir) -> Vec<TaskId> {
	fs::read_to_string(run.abandoned_commits())
		.unwrap_or_default()
		.lines()
		.filter_map(|line| TaskId::parse(line.trim()).ok())
		.collect()
}

fn unstage_abandoned_commits(run: &RunDir, cfg: &Config) -> Result<()> {
	for task in abandoned_commits(run) {
		unstage_task(run, cfg, &task).with_context(|| format!("unstage {task}"))?;
		journal_append(run, &format!("unstaged {task} after abandoned commit"))?;
	}
	let path = run.abandoned_commits();
	match fs::remove_file(&path) {
		Err(error) if error.kind() != io::ErrorKind::NotFound => {
			Err(error).with_context(|| format!("remove {path:?}"))
		}
		_ => Ok(()),
	}
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
	stopping: bool,
	workers: Vec<WorkerProcess>,
	manager: Option<ManagerProcess>,
	commits: Commits,
	grants: HashMap<JobId, Grant>,
	spawned_jobs: HashMap<JobId, u32>,
	cancelling_since: HashMap<JobId, Instant>,
	cancel_failures: HashSet<JobId>,
	vmstat: Option<VmStat>,
	active: Duration,
	last_iteration: Instant,
	last_persist: Instant,
	last_status: Option<Instant>,
	last_tick: Option<Instant>,
	manager_retry_at: Option<Instant>,
	manager_failures: u32,
	last_probe: Option<Instant>,
	probe: Option<RunningProbe>,
	usage_unknown_reported: bool,
	launch_retry_at: Option<Instant>,
	launch_failures: u32,
	blocked_until: Option<i64>,
	pid_lock: Flock<File>,
}

pub fn run_supervisor(run: RunDir, cfg: Config) -> Result<()> {
	let mut supervisor = Supervisor::start(run, cfg)?;
	loop {
		let began = Instant::now();
		let accounted = supervisor.account_time(unix_now());
		report("time", accounted);
		if let Some(reason) = read_stop_reason(&supervisor.run) {
			return supervisor.stop(&reason);
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
		ensure_no_top_level_git(&run.root)?;
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
		unstage_abandoned_commits(&run, &cfg)?;
		journal_resumed_after_stop(&run)?;
		let exempt = TopLevelExempt::observe(&run.root);
		for id in (TaskStore { run: &run }).reconcile_after_restart()? {
			journal_append(&run, &format!("requeued {id} after restart"))?;
		}
		reconcile_jobs(&run)?;
		remove_planted(&run, exempt)?;
		write_claude_settings(&run, &cfg)?;
		let info = read_run_info(&run)?;
		let now = Instant::now();
		let supervisor = Self {
			bin_dir: contest_bin_dir()?,
			credentials: std::env::var_os("HOME")
				.map(|home| PathBuf::from(home).join(CREDENTIALS_FILE)),
			exempt,
			machine: Machine::detect(cfg.ram_budget_mb),
			limits: RunLimits {
				active_budget_seconds: info.active_budget_seconds,
				deadline_unix: info.deadline_unix,
			},
			stopping: false,
			workers: Vec::new(),
			manager: None,
			commits: Commits::default(),
			grants: HashMap::new(),
			spawned_jobs: HashMap::new(),
			cancelling_since: HashMap::new(),
			cancel_failures: HashSet::new(),
			vmstat: None,
			active: Duration::from_secs(info.active_seconds),
			last_iteration: now,
			last_persist: now,
			last_status: None,
			last_tick: None,
			manager_retry_at: None,
			manager_failures: 0,
			last_probe: None,
			probe: None,
			usage_unknown_reported: false,
			launch_retry_at: None,
			launch_failures: 0,
			blocked_until: None,
			pid_lock,
			run,
			cfg,
		};
		supervisor
			.pid_lock
			.write_all_at(std::process::id().to_string().as_bytes(), 0)
			.context("announce supervisor pid")?;
		Ok(supervisor)
	}

	fn shut_down(mut self, reason: &str) -> Result<()> {
		let persisted = self.persist();
		let journaled = journal_append(&self.run, &format!("{STOPPED}{reason}"));
		let cleared = self.pid_lock.set_len(0).map_err(anyhow::Error::from);
		let status = write_status(&self.run, unix_now(), None);
		let errors: Vec<String> = [persisted, journaled, cleared, status]
			.into_iter()
			.filter_map(Result::err)
			.map(|error| format!("{error:#}"))
			.collect();
		if !errors.is_empty() {
			bail!("shutdown incomplete: {errors}", errors = errors.join("; "));
		}
		Ok(())
	}

	fn iterate(&mut self) {
		let now = unix_now();
		let probed = self.poll_probe(now);
		report("usage probe", probed);
		self.refresh_credentials();
		let served = self.serve_requests();
		report("spool", served);
		let reaped = self.reap_workers(now);
		report("reap workers", reaped);
		self.cancel_jobs_of_ended_tasks();
		self.reap_jobs();
		let scheduled = self.schedule_jobs();
		report("schedule jobs", scheduled);
		let manager = self.manager_step(now);
		report("manager", manager);
		let launched = self.launch_workers(now);
		report("launch", launched);
		let status = self.refresh_status(now);
		report("status", status);
	}

	fn start_commit(
		&self,
		request: &CommitRequest,
		tainted: &RefCell<Vec<(TaskId, String)>>,
	) -> Result<CommitThread> {
		if let Some(failure) = taint_failure(remove_planted(&self.run, self.exempt)) {
			tainted
				.borrow_mut()
				.push((request.task.clone(), failure.clone()));
			bail!("commit refused, taint scan failed: {failure}");
		}
		spawn_commit(
			self.run.clone(),
			self.cfg.clone(),
			request.task.clone(),
			request.milestone.to_owned(),
		)
	}

	fn serve_requests(&mut self) -> Result<()> {
		let tainted = RefCell::new(Vec::new());
		let commits = RefCell::new(std::mem::take(&mut self.commits));
		let committer = |request: &CommitRequest| {
			commits
				.borrow_mut()
				.poll(request, || self.start_commit(request, &tainted))
		};
		let served = serve_pending(&self.run, &committer);
		self.commits = commits.into_inner();
		let store = TaskStore { run: &self.run };
		self.commits.forget_finished_unless(|key| {
			store.state(&key.task).ok() == Some(TaskState::Running)
				&& request_pending(&self.run, &key.task, &key.stem)
		});
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

	fn kill_and_reap(&self, child: &mut Child, who: &str) {
		report(who, kill_agent_group(child));
		let deadline = Instant::now() + KILL_REAP_TIMEOUT;
		loop {
			match child.try_wait() {
				Ok(Some(_)) => return,
				Ok(None) if Instant::now() < deadline => sleep(KILL_REAP_POLL),
				Ok(None) => {
					let journaled = journal_append(
						&self.run,
						&format!(
							"{who} pid {pid} still alive {seconds} s after SIGKILL, left behind",
							pid = child.id(),
							seconds = KILL_REAP_TIMEOUT.as_secs()
						),
					);
					return report(who, journaled);
				}
				Err(error) => return report(who, Err(error.into())),
			}
		}
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
		self.write_status_now(now)
	}

	fn write_status_now(&mut self, now: i64) -> Result<()> {
		self.last_status = Some(Instant::now());
		write_status(&self.run, now, Some(std::process::id()))
	}
}
