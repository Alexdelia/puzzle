use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};
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
mod signals;
mod stop;
mod workers;

pub use stop::STOP_SETTLE_SECONDS;

use commits::{CommitThread, Commits, spawn_commit};
use jobs::{HeldLease, reconcile_jobs};
use manager::ManagerProcess;
use probe::{Gate, RunningProbe};
use signals::{install_stop_signals, stop_signals_received};
use workers::{TaskLaunchFailures, WorkerProcess};

use crate::agent::{AgentRun, kill_agent_group, spawn_agent, spawn_agent_scrubbed};
use crate::atomic::write_atomic;
use crate::broker::Machine;
use crate::claude_config::write_claude_settings;
use crate::commit::unstage_task;
use crate::config::Config;
use crate::events::{Event, emit};
use crate::job::{JobIndex, VmStat};
use crate::journal::{journal_append, journal_tail};
use crate::paths::{JobId, RunDir, TaskId};
use crate::run_info::{RunInfo, read_run_info, update_run_info};
use crate::sandbox::{SandboxSpec, contest_bin_dir, dropped_host_env, sandbox_command};
use crate::spool::{CommitRequest, request_pending, serve_pending};
use crate::status::write_status;
use crate::taint::{
	ScanReport, TopLevelExempt, ensure_no_top_level_git, remove_planted_files_within,
};
use crate::task::{TaskState, TaskStore};

const LOOP_INTERVAL: Duration = Duration::from_secs(1);
const MAX_ACTIVE_STEP: Duration = Duration::from_secs(5);
const PERSIST_INTERVAL: Duration = Duration::from_secs(30);
const STATUS_INTERVAL: Duration = Duration::from_secs(30);
const RETRY_MAX_DOUBLINGS: u32 = 10;
const SECONDS_PER_MINUTE: u64 = 60;
const STREAM_FILE: &str = "stream.jsonl";
const CREDENTIALS_FILE: &str = ".claude/.credentials.json";
const BUDGET_STOP_REASON: &str = "budget";
const KILL_REAP_TIMEOUT: Duration = Duration::from_secs(3);
const KILL_REAP_POLL: Duration = Duration::from_millis(20);
const PID_LOCK_POLL: Duration = Duration::from_millis(20);
const STOPPED: &str = "stopped: ";
const SIGNAL_STOP_REASON: &str = "signal";
pub const DEFAULT_STOP_REASON: &str = "requested";
const USAGE_BLOCK: &str = "usage";
const MANAGER_USAGE_BLOCK: &str = "manager usage";
const LAUNCH_BLOCK: &str = "launch";
const MANAGER_BLOCK: &str = "manager";
const TAINT_BLOCK: &str = "taint";
const COMMITS_PAUSED: &str =
	"commit refused: the taint scan is over its entry budget, commits are paused";

pub fn active_increment(elapsed: Duration) -> Duration {
	elapsed.min(MAX_ACTIVE_STEP)
}

fn unix_now() -> i64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_or(0, |elapsed| elapsed.as_secs() as i64)
}

fn retry_backoff(failures: u32, cfg: &Config) -> Duration {
	let doublings = failures.saturating_sub(1).min(RETRY_MAX_DOUBLINGS);
	let heartbeat = Duration::from_secs(cfg.heartbeat_minutes * SECONDS_PER_MINUTE);
	(Duration::from_secs(cfg.retry_base_seconds) * 2u32.pow(doublings)).min(heartbeat)
}

fn remove_if_exists(path: &Path) -> Result<()> {
	match fs::remove_file(path) {
		Err(error) if error.kind() != io::ErrorKind::NotFound => {
			Err(error).with_context(|| format!("remove {path:?}"))
		}
		_ => Ok(()),
	}
}

fn report(step: &str, result: Result<()>) {
	if let Err(error) = result {
		let _ = writeln!(io::stderr(), "supervisor: {step}: {error:#}");
	}
}

fn run_json_modified(run: &RunDir) -> Option<SystemTime> {
	fs::metadata(run.run_json())
		.and_then(|meta| meta.modified())
		.ok()
}

fn lock_pid_file(run: &RunDir, retry: Duration) -> Result<Flock<File>> {
	let path = run.supervisor_pid();
	let mut file = OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.truncate(false)
		.open(&path)
		.with_context(|| format!("open {path:?}"))?;
	let deadline = Instant::now() + retry;
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
	let mut still_staged = Vec::new();
	for task in abandoned_commits(run) {
		if !run.task_meta(&task).is_dir() {
			journal_append(
				run,
				&format!("dropped abandoned commit of {task}: task no longer exists"),
			)?;
			continue;
		}
		match unstage_task(run, cfg, &task) {
			Ok(()) => journal_append(run, &format!("unstaged {task} after abandoned commit"))?,
			Err(error) => {
				journal_append(run, &format!("unstage of {task} failed: {error:#}"))?;
				still_staged.push(task);
			}
		}
	}
	let path = run.abandoned_commits();
	if !still_staged.is_empty() {
		let lines: String = still_staged
			.iter()
			.map(|task| format!("{task}\n"))
			.collect();
		return Ok(write_atomic(&path, lines.as_bytes())?);
	}
	remove_if_exists(&path)
}

fn remove_planted(run: &RunDir, exempt: TopLevelExempt, max_entries: usize) -> Result<ScanReport> {
	let report = remove_planted_files_within(&run.root, exempt, max_entries)?;
	for path in &report.removed {
		journal_append(run, &format!("taint: removed {path:?}"))?;
	}
	for (path, error) in &report.failures {
		journal_append(run, &format!("taint: cannot remove {path:?}: {error}"))?;
	}
	Ok(report)
}

struct TaintScan {
	failure: Option<String>,
	over_budget: Option<bool>,
}

fn scan_taint(run: &RunDir, exempt: TopLevelExempt, max_entries: usize) -> TaintScan {
	match remove_planted(run, exempt, max_entries) {
		Ok(report) => TaintScan {
			failure: report.must_fail_closed().then(|| {
				format!(
					"{count} planted paths could not be removed",
					count = report.failures.len()
				)
			}),
			over_budget: Some(report.over_budget),
		},
		Err(error) => TaintScan {
			failure: Some(format!("{error:#}")),
			over_budget: None,
		},
	}
}

struct StopRequest {
	reason: String,
	signals_seen: u32,
}

struct Supervisor {
	run: RunDir,
	cfg: Config,
	bin_dir: PathBuf,
	credentials: Option<PathBuf>,
	exempt: TopLevelExempt,
	machine: Machine,
	info: RunInfo,
	run_json_seen: Option<SystemTime>,
	persist_error: Option<String>,
	stopping: bool,
	workers: Vec<WorkerProcess>,
	manager: Option<ManagerProcess>,
	commits: Commits,
	jobs: JobIndex,
	leases: HashMap<JobId, HeldLease>,
	agent_cpus: Vec<usize>,
	last_agent_pin: Instant,
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
	task_launch_failures: HashMap<TaskId, TaskLaunchFailures>,
	task_requeues: HashMap<TaskId, Vec<Instant>>,
	blocked_until: Option<i64>,
	limit_until: Option<i64>,
	manager_last_error: Option<String>,
	blocks: BTreeMap<&'static str, String>,
	taint_over_budget: bool,
	last_taint_scan: Instant,
	taint_rescans: u32,
	pid_lock: Flock<File>,
}

pub fn run_supervisor(run: RunDir, cfg: Config) -> Result<()> {
	install_stop_signals()?;
	let mut supervisor = Supervisor::start(run, cfg)?;
	loop {
		let began = Instant::now();
		let accounted = supervisor.account_time(unix_now());
		report("time", accounted);
		if let Some(request) = supervisor.requested_stop() {
			return supervisor.stop(&request);
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
		let pid_lock = lock_pid_file(&run, Duration::from_millis(cfg.pid_lock_retry_ms))?;
		journal_resumed_after_stop(&run)?;
		unstage_abandoned_commits(&run, &cfg)?;
		let exempt = TopLevelExempt {
			envrc: cfg.contest_envrc,
		};
		for id in (TaskStore { run: &run }).reconcile_after_restart()? {
			journal_append(&run, &format!("requeued {id} after restart"))?;
		}
		reconcile_jobs(&run)?;
		remove_if_exists(&run.blocks())?;
		let over_budget = remove_planted(&run, exempt, cfg.taint_scan_max_entries)?.over_budget;
		write_claude_settings(&run, &cfg)?;
		let info = read_run_info(&run)?;
		let now = Instant::now();
		let mut supervisor = Self {
			bin_dir: contest_bin_dir()?,
			credentials: std::env::var_os("HOME")
				.map(|home| PathBuf::from(home).join(CREDENTIALS_FILE)),
			exempt,
			machine: Machine::detect(cfg.ram_budget_mb, cfg.agent_cores),
			run_json_seen: run_json_modified(&run),
			persist_error: None,
			stopping: false,
			workers: Vec::new(),
			manager: None,
			commits: Commits::default(),
			jobs: JobIndex::default(),
			leases: HashMap::new(),
			agent_cpus: Vec::new(),
			last_agent_pin: now,
			spawned_jobs: HashMap::new(),
			cancelling_since: HashMap::new(),
			cancel_failures: HashSet::new(),
			vmstat: None,
			active: Duration::from_secs(info.active_seconds),
			info,
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
			task_launch_failures: HashMap::new(),
			task_requeues: HashMap::new(),
			blocked_until: None,
			limit_until: None,
			manager_last_error: None,
			blocks: BTreeMap::new(),
			taint_over_budget: false,
			last_taint_scan: now,
			taint_rescans: 0,
			pid_lock,
			run,
			cfg,
		};
		supervisor
			.pid_lock
			.write_all_at(std::process::id().to_string().as_bytes(), 0)
			.context("announce supervisor pid")?;
		supervisor.note_taint_budget(over_budget);
		Ok(supervisor)
	}

	fn requested_stop(&self) -> Option<StopRequest> {
		let signals_seen = stop_signals_received();
		if signals_seen == 0 {
			return read_stop_reason(&self.run).map(|reason| StopRequest {
				reason,
				signals_seen,
			});
		}
		if !self.run.stop_file().exists() {
			let written = write_atomic(&self.run.stop_file(), SIGNAL_STOP_REASON.as_bytes());
			report("stop signal", written.map_err(Into::into));
		}
		Some(StopRequest {
			reason: read_stop_reason(&self.run).unwrap_or_else(|| SIGNAL_STOP_REASON.to_owned()),
			signals_seen,
		})
	}

	fn shut_down(mut self, reason: &str) -> Result<()> {
		let persisted = self.persist();
		let journaled = journal_append(&self.run, &format!("{STOPPED}{reason}"));
		let cleared = self.pid_lock.set_len(0).map_err(anyhow::Error::from);
		let unblocked = remove_if_exists(&self.run.blocks());
		let status = write_status(&self.run, unix_now(), None);
		let errors: Vec<String> = [persisted, journaled, cleared, unblocked, status]
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
		self.rescan_taint_if_over_budget();
		let served = self.serve_requests();
		report("spool", served);
		self.watch_workers(now);
		let reaped = self.reap_workers(now);
		report("reap workers", reaped);
		let requeued = self.requeue_orphaned_running();
		report("requeue", requeued);
		self.cancel_jobs_of_ended_tasks();
		self.reap_jobs();
		let scheduled = self.schedule_jobs(now);
		report("schedule jobs", scheduled);
		let manager = self.manager_step(now);
		report("manager", manager);
		let resumed = self.resume_waiting_tasks();
		report("resume", resumed);
		let launched = self.launch_workers(now);
		report("launch", launched);
		let status = self.refresh_status(now);
		report("status", status);
	}

	fn start_commit(
		&self,
		request: &CommitRequest,
		tainted: &RefCell<Vec<(TaskId, String)>>,
		over_budget: &Cell<Option<bool>>,
	) -> Result<CommitThread> {
		if self.taint_over_budget {
			bail!("{COMMITS_PAUSED}");
		}
		let scan = scan_taint(&self.run, self.exempt, self.cfg.taint_scan_max_entries);
		if scan.over_budget.is_some() {
			over_budget.set(scan.over_budget);
		}
		if let Some(failure) = scan.failure {
			tainted
				.borrow_mut()
				.push((request.task.clone(), failure.clone()));
			bail!("commit refused, taint scan failed: {failure}");
		}
		if scan.over_budget == Some(true) {
			bail!("{COMMITS_PAUSED}");
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
		let over_budget = Cell::new(None);
		let commits = RefCell::new(std::mem::take(&mut self.commits));
		let committer = |request: &CommitRequest| {
			commits.borrow_mut().poll(request, || {
				self.start_commit(request, &tainted, &over_budget)
			})
		};
		let served = serve_pending(&self.run, &self.machine, &committer);
		self.commits = commits.into_inner();
		if let Some(over) = over_budget.get() {
			self.note_taint_budget(over);
		}
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

	fn scan_taint_now(&mut self) -> TaintScan {
		let scan = scan_taint(&self.run, self.exempt, self.cfg.taint_scan_max_entries);
		self.last_taint_scan = Instant::now();
		if let Some(over) = scan.over_budget {
			self.note_taint_budget(over);
		}
		scan
	}

	fn rescan_taint_if_over_budget(&mut self) {
		if !self.taint_over_budget
			|| self.last_taint_scan.elapsed() < retry_backoff(self.taint_rescans + 1, &self.cfg)
		{
			return;
		}
		self.taint_rescans += 1;
		self.scan_taint_now();
	}

	fn note_taint_budget(&mut self, over_budget: bool) {
		if over_budget == self.taint_over_budget {
			return;
		}
		self.taint_over_budget = over_budget;
		let max_entries = self.cfg.taint_scan_max_entries;
		if over_budget {
			report(
				"taint",
				emit(&self.run, Event::TaintOverBudget { max_entries }),
			);
			let reason = format!(
				"taint scan over its entry budget of {max_entries}: launches and commits paused until the contest dir shrinks"
			);
			self.set_block(TAINT_BLOCK, Some(reason.clone()));
			report("taint", journal_append(&self.run, &reason));
		} else {
			self.taint_rescans = 0;
			let journaled = journal_append(
				&self.run,
				"taint scan back under its entry budget: launches and commits resume",
			);
			report("taint", journaled);
			self.set_block(TAINT_BLOCK, None);
		}
	}

	fn set_block(&mut self, slot: &'static str, reason: Option<String>) -> bool {
		let changed = match &reason {
			Some(reason) => self.blocks.get(slot) != Some(reason),
			None => self.blocks.contains_key(slot),
		};
		if !changed {
			return false;
		}
		match reason {
			Some(reason) => self.blocks.insert(slot, reason),
			None => self.blocks.remove(slot),
		};
		report("blocks", self.write_blocks());
		true
	}

	fn write_blocks(&self) -> Result<()> {
		if self.blocks.is_empty() {
			return remove_if_exists(&self.run.blocks());
		}
		let lines: String = self
			.blocks
			.iter()
			.map(|(slot, reason)| format!("{slot}: {reason}\n"))
			.collect();
		Ok(write_atomic(&self.run.blocks(), lines.as_bytes())?)
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
			spawn_agent(&self.cfg, agent, Some(sandbox_command(&spec, &[])?))
		} else {
			spawn_agent_scrubbed(&self.cfg, agent, None, &dropped_host_env())
		}
	}

	fn adopt_run_info(&mut self, info: RunInfo) {
		self.info = info;
		self.run_json_seen = run_json_modified(&self.run);
	}

	fn persist(&mut self) -> Result<()> {
		self.last_persist = Instant::now();
		let active_seconds = self.active.as_secs();
		let last_good = self.info.clone();
		let mut unreadable = None;
		let info = update_run_info(&self.run, |current| {
			let mut info = current.unwrap_or_else(|error| {
				unreadable = Some(format!("{error:#}"));
				last_good
			});
			info.active_seconds = active_seconds;
			Ok(info)
		})?;
		self.adopt_run_info(info);
		if let Some(error) = unreadable {
			journal_append(
				&self.run,
				&format!("run.json unreadable, rewritten from the last good copy: {error}"),
			)?;
		}
		Ok(())
	}

	fn persist_on_schedule(&mut self) {
		if self.last_persist.elapsed() < PERSIST_INTERVAL {
			return;
		}
		match self.persist() {
			Ok(()) => self.persist_error = None,
			Err(error) => {
				let text = format!("{error:#}");
				if self.persist_error.as_deref() != Some(text.as_str()) {
					report("persist", Err(error));
					self.persist_error = Some(text);
				}
			}
		}
	}

	fn reload_run_info_on_change(&mut self) {
		let modified = run_json_modified(&self.run);
		if modified == self.run_json_seen {
			return;
		}
		self.run_json_seen = modified;
		if let Ok(info) = read_run_info(&self.run) {
			self.info = info;
		}
	}

	fn limit_reached(&self, now: i64) -> bool {
		self.info
			.active_budget_seconds
			.is_some_and(|budget| self.active.as_secs() >= budget)
			|| self
				.info
				.deadline_unix
				.is_some_and(|deadline| now >= deadline)
	}

	fn account_time(&mut self, now: i64) -> Result<()> {
		self.active += active_increment(self.last_iteration.elapsed());
		self.last_iteration = Instant::now();
		self.reload_run_info_on_change();
		if self.limit_reached(now) && !self.run.stop_file().exists() {
			write_atomic(&self.run.stop_file(), BUDGET_STOP_REASON.as_bytes())?;
			emit(&self.run, Event::BudgetExhausted)?;
			journal_append(&self.run, "budget reached, stop requested")?;
		}
		self.persist_on_schedule();
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
