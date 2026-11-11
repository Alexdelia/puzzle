use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use contest::agent::{RateLimitInfo, Window};
use contest::atomic::write_atomic;
use contest::config::Config;
use contest::job::{JobState, read_job_record};
use contest::paths::{JobId, RunDir, TaskId};
use contest::run_info::{Mode, RunInfo};
use contest::task::{TaskSpec, TaskState, TaskStore};
use contest::usage::record_usage;
use nix::fcntl::{Flock, FlockArg};
use serde_json::json;
use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_contest");
const WEEK_SECONDS: i64 = 604800;

fn fixture(name: &str) -> PathBuf {
	Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("tests/fixtures")
		.join(name)
}

fn now() -> i64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs() as i64
}

fn fresh_usage() -> RateLimitInfo {
	let now = now();
	RateLimitInfo {
		status: "allowed".into(),
		resets_at: now + 3600,
		five_hour: Window {
			utilization: 0.1,
			resets_at: now + 3600,
		},
		seven_day: Window {
			utilization: 0.1,
			resets_at: now + WEEK_SECONDS / 2,
		},
	}
}

struct Contest {
	_dir: TempDir,
	run: RunDir,
	home: PathBuf,
	manager_stream: PathBuf,
}

fn contest(workers: usize, grace_seconds: u64) -> Contest {
	let dir = TempDir::new().unwrap();
	let root = fs::canonicalize(dir.path()).unwrap().join("contest");
	let run = RunDir { root };
	for path in [
		run.tasks(),
		run.jobs(),
		run.inbox(),
		run.requests(),
		run.agents(),
	] {
		fs::create_dir_all(path).unwrap();
	}
	let config = Config {
		workers,
		grace_seconds,
		sandbox: false,
		claude_bin: fixture("fake-claude.sh"),
		plugin_dir: Path::new(env!("CARGO_MANIFEST_DIR"))
			.parent()
			.unwrap()
			.to_path_buf(),
		..Config::default()
	};
	write_atomic(&run.config(), toml::to_string(&config).unwrap().as_bytes()).unwrap();
	write_run_info(
		&run,
		&RunInfo {
			puzzle: "seabed-security".into(),
			mode: Mode::PastPuzzle,
			deadline_unix: None,
			active_budget_seconds: None,
			active_seconds: 0,
			phase: "start".into(),
			plugin_commit: String::new(),
			started_at: now(),
		},
	);
	record_usage(&run, &fresh_usage(), now()).unwrap();
	let home = dir.path().join("home");
	fs::create_dir_all(&home).unwrap();
	let manager_stream = dir.path().join("manager-stream.jsonl");
	let result = json!({
		"type": "result",
		"subtype": "success",
		"is_error": false,
		"result": "nothing to do\n```json\n[]\n```",
		"total_cost_usd": 0.01,
		"usage": {"input_tokens": 10, "output_tokens": 5},
	});
	fs::write(&manager_stream, format!("{result}\n")).unwrap();
	Contest {
		_dir: dir,
		run,
		home,
		manager_stream,
	}
}

fn write_run_info(run: &RunDir, info: &RunInfo) {
	write_atomic(&run.run_json(), &serde_json::to_vec(info).unwrap()).unwrap();
}

fn read_run_info(run: &RunDir) -> RunInfo {
	serde_json::from_slice(&fs::read(run.run_json()).unwrap()).unwrap()
}

fn create_task(run: &RunDir, kind: &str, priority: i32) -> TaskId {
	TaskStore { run }
		.create(TaskSpec {
			kind: kind.into(),
			title: format!("{kind} task"),
			priority,
			model: None,
			budget_usd: None,
			writes: Vec::new(),
			body: format!("body of the {kind} task"),
			parent: None,
		})
		.unwrap()
}

fn state(run: &RunDir, id: &TaskId) -> TaskState {
	TaskStore { run }.state(id).unwrap()
}

struct Supervisor(Child);

impl Drop for Supervisor {
	fn drop(&mut self) {
		let _ = self.0.kill();
		let _ = self.0.wait();
	}
}

impl Supervisor {
	fn exited_within(&mut self, seconds: u64) -> Duration {
		let started = Instant::now();
		while self.0.try_wait().unwrap().is_none() {
			assert!(
				started.elapsed() < Duration::from_secs(seconds),
				"supervisor still running after {seconds} s"
			);
			sleep(Duration::from_millis(50));
		}
		started.elapsed()
	}

	fn stop_within(&mut self, run: &RunDir, seconds: u64) -> Duration {
		fs::write(run.stop_file(), "test").unwrap();
		self.exited_within(seconds)
	}
}

struct StopOnDrop(RunDir);

impl Drop for StopOnDrop {
	fn drop(&mut self) {
		let _ = fs::write(self.0.stop_file(), "test cleanup");
	}
}

fn supervisor_command(contest: &Contest, exec: &str) -> Command {
	let mut command = Command::new(BIN);
	command
		.env("HOME", &contest.home)
		.env("FAKE_CLAUDE_SCRIPT", fixture("stream-success.jsonl"))
		.env("FAKE_MANAGER_STREAM", &contest.manager_stream)
		.env("FAKE_CLAUDE_EXEC", exec);
	command
}

fn agent_exec(worker: &str, probe: &str, manager: &str) -> String {
	format!(
		"if [[ $2 == ok ]]; then {probe}; elif [[ -n ${{CONTEST_TASK:-}} ]]; then {worker}; else {manager}; fi"
	)
}

fn worker_exec(worker: &str) -> String {
	agent_exec(worker, ":", "FAKE_CLAUDE_SCRIPT=$FAKE_MANAGER_STREAM")
}

fn start_exec(contest: &Contest, exec: &str) -> Supervisor {
	let log = fs::File::create(contest.home.join("supervisor.log")).unwrap();
	let child = supervisor_command(contest, exec)
		.arg("start")
		.arg(&contest.run.root)
		.arg("--foreground")
		.stdin(Stdio::null())
		.stdout(log.try_clone().unwrap())
		.stderr(log)
		.spawn()
		.unwrap();
	Supervisor(child)
}

fn start(contest: &Contest, worker: &str) -> Supervisor {
	start_exec(contest, &worker_exec(worker))
}

fn wait_until(seconds: u64, what: &str, mut condition: impl FnMut() -> bool) {
	let deadline = Instant::now() + Duration::from_secs(seconds);
	while !condition() {
		assert!(Instant::now() < deadline, "timed out waiting for {what}");
		sleep(Duration::from_millis(50));
	}
}

fn journal(run: &RunDir) -> String {
	fs::read_to_string(run.journal()).unwrap_or_default()
}

fn journal_lines(run: &RunDir, needle: &str) -> usize {
	journal(run)
		.lines()
		.filter(|line| line.contains(needle))
		.count()
}

fn alive(pid: i32) -> bool {
	fs::read_to_string(format!("/proc/{pid}/stat")).is_ok_and(|stat| !stat.contains(") Z "))
}

fn read_pid(path: &Path) -> i32 {
	fs::read_to_string(path).unwrap().trim().parse().unwrap()
}

fn contest_cli(contest: &Contest, args: &[&str]) -> Output {
	supervisor_command(contest, &worker_exec("exec sleep 60"))
		.args(args)
		.stdin(Stdio::null())
		.output()
		.unwrap()
}

fn git(dir: &Path, args: &[&str]) -> String {
	let output = Command::new("git")
		.arg("-C")
		.arg(dir)
		.args(args)
		.output()
		.unwrap();
	assert!(output.status.success(), "git {args:?} failed");
	String::from_utf8_lossy(&output.stdout).into_owned()
}

fn pin_repo(contest: &Contest) -> PathBuf {
	let repo = contest.run.root.parent().unwrap().to_path_buf();
	git(&repo, &["init", "-q"]);
	git(&repo, &["config", "user.name", "t"]);
	git(&repo, &["config", "user.email", "t@t"]);
	git(&repo, &["config", "commit.gpgsign", "false"]);
	git(&repo, &["commit", "-q", "--allow-empty", "-m", "init"]);
	let mut config = Config::load(&contest.run.config()).unwrap();
	config.repo_root = repo.clone();
	config.git_common_dir = repo.join(".git");
	write_atomic(
		&contest.run.config(),
		toml::to_string(&config).unwrap().as_bytes(),
	)
	.unwrap();
	repo
}

fn worker_script(contest: &Contest, body: &str) -> String {
	let script = contest.home.join("worker.sh");
	fs::write(&script, body).unwrap();
	fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
	format!("exec bash {script}", script = script.display())
}

#[test]
fn stop_terminates_jobs_with_checkpoint() {
	let contest = contest(1, 2);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let action = format!(
		"contest job submit --class batch --ram-mb 10 -- bash {script}; exec sleep 60",
		script = fixture("checkpoint-job.sh").display()
	);
	let mut supervisor = start(&contest, &action);
	let job = JobId("j0001".into());
	wait_until(20, "job ready", || {
		fs::read_to_string(run.job_log_path(&job)).is_ok_and(|log| log.contains("ready"))
	});
	supervisor.stop_within(run, 10);
	let checkpoint = run.job_work_dir(&id, &job).join("checkpoint");
	assert_eq!(fs::read_to_string(checkpoint).unwrap().trim(), "saved");
	assert_eq!(
		read_job_record(run, &job).unwrap().state,
		JobState::Interrupted
	);
	assert_eq!(state(run, &id), TaskState::Interrupted);
	assert_eq!(journal_lines(run, "stopped: test"), 1);
}

#[test]
fn stop_kills_stubborn_worker_after_grace() {
	let contest = contest(1, 2);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let pid_file = contest.home.join("worker-pid");
	let action = format!(
		"cat \"$FAKE_CLAUDE_SCRIPT\"; echo $$ > {pid}; exec sleep 60",
		pid = pid_file.display()
	);
	let mut supervisor = start(&contest, &action);
	wait_until(20, "worker started", || pid_file.is_file());
	let pid = read_pid(&pid_file);
	let elapsed = supervisor.stop_within(run, 5);
	assert!(
		elapsed >= Duration::from_secs(2),
		"stopped after {elapsed:?}"
	);
	assert!(!alive(pid));
	assert_eq!(state(run, &id), TaskState::Interrupted);
	let ledger = fs::read_to_string(run.ledger()).unwrap();
	let row: Vec<&str> = ledger
		.lines()
		.find(|line| line.split('\t').nth(1) == Some(id.0.as_str()))
		.expect("ledger row for the killed worker")
		.split('\t')
		.collect();
	assert_eq!(row[7], "0.05");
	assert_eq!(journal_lines(run, &format!("interrupted {id}")), 1);
}

#[test]
fn commit_during_wrap_up_answered() {
	let contest = contest(1, 20);
	let run = &contest.run;
	let repo = pin_repo(&contest);
	let id = create_task(run, "sim-bench", 0);
	let committed = contest.home.join("committed");
	let action = worker_script(
		&contest,
		&format!(
			"until [ -e \"$CONTEST_RUN/run/STOP\" ]; do sleep 0.1; done\necho hi > \"$CONTEST_RUN/run/tasks/$CONTEST_TASK/out.txt\"\ncontest commit --milestone stop && touch {committed}\ncontest task done\n",
			committed = committed.display()
		),
	);
	let mut supervisor = start(&contest, &action);
	wait_until(20, "task running", || state(run, &id) == TaskState::Running);
	sleep(Duration::from_millis(500));
	supervisor.stop_within(run, 10);
	assert!(committed.is_file());
	let log = git(&repo, &["log", "--format=%s"]);
	assert_eq!(
		log.lines().next(),
		Some(format!("sim-bench: {id} stop").as_str())
	);
	assert_eq!(state(run, &id), TaskState::Done);
}

#[test]
fn resume_after_stop_launches_interrupted_first() {
	let contest = contest(1, 2);
	let run = &contest.run;
	let first = create_task(run, "sim-bench", 0);
	let mut supervisor = start(&contest, "exec sleep 60");
	wait_until(20, "first running", || {
		state(run, &first) == TaskState::Running
	});
	supervisor.stop_within(run, 10);
	assert_eq!(state(run, &first), TaskState::Interrupted);
	let second = create_task(run, "bot-baseline", 500);

	let _cleanup = StopOnDrop(run.clone());
	let started = contest_cli(&contest, &["start", run.root.to_str().unwrap()]);
	assert!(
		started.status.success(),
		"{stderr}",
		stderr = String::from_utf8_lossy(&started.stderr)
	);
	wait_until(20, "first relaunched", || {
		journal_lines(run, &format!("launched {first}")) == 2
	});
	assert!(!run.stop_file().exists());
	assert_eq!(journal_lines(run, "resumed"), 1);
	assert_eq!(state(run, &first), TaskState::Running);
	assert_eq!(state(run, &second), TaskState::Queued);

	let stopped = contest_cli(&contest, &["stop", run.root.to_str().unwrap(), "--wait"]);
	assert!(stopped.status.success());
	assert!(
		fs::read_to_string(run.status())
			.unwrap()
			.contains("supervisor: stopped")
	);
	assert_eq!(state(run, &first), TaskState::Interrupted);
}

#[test]
fn stop_discards_manager_in_flight() {
	let contest = contest(0, 30);
	let run = &contest.run;
	let pid_file = contest.home.join("manager-pid");
	let said = contest_cli(&contest, &["say", run.root.to_str().unwrap(), "hello"]);
	assert!(said.status.success());
	let exec = agent_exec(
		"exec sleep 60",
		":",
		&format!("echo $$ > {pid}; exec sleep 60", pid = pid_file.display()),
	);
	let mut supervisor = start_exec(&contest, &exec);
	wait_until(20, "manager running", || pid_file.is_file());
	let pid = read_pid(&pid_file);
	supervisor.stop_within(run, 5);
	assert!(!alive(pid));
	assert!(!run.manager_cursor().exists());
	assert!(!run.inbox().join("read").exists());
	assert_eq!(fs::read_dir(run.inbox()).unwrap().count(), 1);
	assert_eq!(journal_lines(run, "manager tick discarded"), 1);
	assert_eq!(journal_lines(run, "stopped: test"), 1);
}

#[test]
fn stop_kills_probe_in_flight() {
	let contest = contest(1, 30);
	let run = &contest.run;
	fs::remove_file(run.usage()).unwrap();
	create_task(run, "sim-bench", 0);
	let pid_file = contest.home.join("probe-pid");
	let exec = agent_exec(
		"contest task done",
		&format!("echo $$ > {pid}; exec sleep 60", pid = pid_file.display()),
		"FAKE_CLAUDE_SCRIPT=$FAKE_MANAGER_STREAM",
	);
	let mut supervisor = start_exec(&contest, &exec);
	wait_until(20, "probe running", || pid_file.is_file());
	let pid = read_pid(&pid_file);
	supervisor.stop_within(run, 5);
	assert!(!alive(pid));
	assert_eq!(journal_lines(run, "usage probe discarded"), 1);
}

#[test]
fn stop_noticed_while_commit_in_flight() {
	let contest = contest(1, 5);
	let run = &contest.run;
	let repo = pin_repo(&contest);
	let id = create_task(run, "sim-bench", 0);
	let lock_file = fs::OpenOptions::new()
		.create(true)
		.write(true)
		.truncate(false)
		.open(repo.join(".git/contest-commit.lock"))
		.unwrap();
	let lock = Flock::lock(lock_file, FlockArg::LockExclusive).unwrap();
	let committed = contest.home.join("committed");
	let action = format!(
		"echo hi > \"$CONTEST_RUN/run/tasks/$CONTEST_TASK/out.txt\"; contest commit --milestone first && touch {committed}; exec sleep 60",
		committed = committed.display()
	);
	let mut supervisor = start(&contest, &action);
	let requests = run.task_requests(&id);
	wait_until(20, "commit requested", || {
		fs::read_dir(&requests).is_ok_and(|mut entries| entries.next().is_some())
	});
	sleep(Duration::from_secs(1));
	fs::write(run.stop_file(), "test").unwrap();
	wait_until(3, "stop noticed", || {
		journal_lines(run, "stopping: test") == 1
	});
	assert!(supervisor.0.try_wait().unwrap().is_none());
	drop(lock);
	wait_until(5, "commit answered", || committed.is_file());
	supervisor.exited_within(10);
	let log = git(&repo, &["log", "--format=%s"]);
	assert_eq!(
		log.lines().next(),
		Some(format!("sim-bench: {id} first").as_str())
	);
}

#[test]
fn resumed_budget_stop_stops_again() {
	let contest = contest(1, 2);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let mut info = read_run_info(run);
	info.active_budget_seconds = Some(10);
	info.active_seconds = 20;
	write_run_info(run, &info);
	fs::write(run.journal(), "2026-01-01 00:00:00 stopped: budget\n").unwrap();
	fs::write(run.stop_file(), "budget").unwrap();
	let mut supervisor = start(&contest, "contest task done");
	supervisor.exited_within(10);
	let text = journal(run);
	let resumed = text.find("resumed").expect("resumed journaled");
	let reached = text
		.find("budget reached")
		.expect("budget reason journaled");
	assert!(resumed < reached);
	assert_eq!(journal_lines(run, "stopped: budget"), 2);
	assert_eq!(fs::read_to_string(run.stop_file()).unwrap(), "budget");
	assert_eq!(state(run, &id), TaskState::Queued);
	assert!(!run.agents().join(&id.0).exists());
}

#[test]
fn stop_cli_returns_at_once_and_wait_blocks() {
	let contest = contest(1, 3);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let mut supervisor = start(&contest, "exec sleep 60");
	wait_until(20, "task running", || state(run, &id) == TaskState::Running);
	let asked = Instant::now();
	let stopped = contest_cli(
		&contest,
		&["stop", run.root.to_str().unwrap(), "--reason", "user break"],
	);
	assert!(stopped.status.success());
	assert!(asked.elapsed() < Duration::from_secs(1));
	assert!(supervisor.0.try_wait().unwrap().is_none());
	assert_eq!(fs::read_to_string(run.stop_file()).unwrap(), "user break");
	let waited = contest_cli(&contest, &["stop", run.root.to_str().unwrap(), "--wait"]);
	assert!(waited.status.success());
	assert!(String::from_utf8_lossy(&waited.stdout).contains("stopped"));
	supervisor.exited_within(1);
	assert_eq!(journal_lines(run, "stopped: user break"), 1);
	assert_eq!(fs::read_to_string(run.stop_file()).unwrap(), "user break");
}

#[test]
fn abandoned_commit_is_unstaged_at_next_start() {
	let contest = contest(1, 2);
	let run = &contest.run;
	let repo = pin_repo(&contest);
	let id = create_task(run, "sim-bench", 0);
	let lock_file = fs::OpenOptions::new()
		.create(true)
		.write(true)
		.truncate(false)
		.open(repo.join(".git/contest-commit.lock"))
		.unwrap();
	let lock = Flock::lock(lock_file, FlockArg::LockExclusive).unwrap();
	let out = run.task_dir(&id).join("out.txt");
	let action = format!(
		"echo hi > {out}; contest commit --milestone first; exec sleep 60",
		out = out.display()
	);
	let mut supervisor = start(&contest, &action);
	wait_until(20, "commit requested", || {
		fs::read_dir(run.task_requests(&id)).is_ok_and(|mut entries| entries.next().is_some())
	});
	sleep(Duration::from_secs(1));
	supervisor.stop_within(run, 20);
	assert_eq!(journal_lines(run, "may have staged or landed"), 1);
	let marker = run.abandoned_commits();
	assert_eq!(fs::read_to_string(&marker).unwrap().trim(), id.0);
	git(&repo, &["add", "--", out.to_str().unwrap()]);
	drop(lock);

	let mut supervisor = start(&contest, "exec sleep 60");
	wait_until(10, "unstaged", || {
		journal_lines(run, &format!("unstaged {id}")) == 1
	});
	assert_eq!(git(&repo, &["diff", "--cached", "--name-only"]), "");
	assert!(!marker.exists());
	supervisor.stop_within(run, 10);
}

fn pid_file_lock(run: &RunDir, arg: FlockArg) -> Flock<fs::File> {
	let file = fs::OpenOptions::new()
		.create(true)
		.write(true)
		.truncate(false)
		.open(run.supervisor_pid())
		.unwrap();
	Flock::lock(file, arg).unwrap()
}

#[test]
fn stop_landing_during_startup_is_honored() {
	let contest = contest(0, 2);
	let run = &contest.run;
	fs::write(run.stop_file(), "earlier").unwrap();
	let reader = pid_file_lock(run, FlockArg::LockShared);
	let mut supervisor = start(&contest, "exec sleep 60");
	wait_until(5, "STOP cleared by start", || !run.stop_file().exists());
	fs::write(run.stop_file(), "late").unwrap();
	drop(reader);
	supervisor.exited_within(10);
	assert_eq!(journal_lines(run, "stopped: late"), 1);
	assert_eq!(journal_lines(run, "resumed"), 0);
}

#[test]
fn stop_while_starting_prints_no_empty_pid() {
	let contest = contest(0, 2);
	let run = &contest.run;
	let _starting = pid_file_lock(run, FlockArg::LockExclusive);
	let stopped = contest_cli(&contest, &["stop", run.root.to_str().unwrap()]);
	assert!(stopped.status.success());
	let text = String::from_utf8_lossy(&stopped.stdout);
	assert!(text.contains("starting"), "{text}");
	assert!(!text.contains("pid  "), "{text}");
	assert!(!text.contains("pid stops"), "{text}");
}

#[test]
fn stop_kills_manager_process_group() {
	let contest = contest(0, 30);
	let run = &contest.run;
	let grandchild_file = contest.home.join("grandchild");
	let said = contest_cli(&contest, &["say", run.root.to_str().unwrap(), "hello"]);
	assert!(said.status.success());
	let exec = agent_exec(
		"exec sleep 60",
		":",
		&format!(
			"sleep 60 & echo $! > {grandchild}; exec sleep 60",
			grandchild = grandchild_file.display()
		),
	);
	let mut supervisor = start_exec(&contest, &exec);
	wait_until(20, "manager running", || {
		fs::read_to_string(&grandchild_file).is_ok_and(|text| text.ends_with('\n'))
	});
	let grandchild = read_pid(&grandchild_file);
	supervisor.stop_within(run, 5);
	let survived = alive(grandchild);
	if survived {
		let _ = nix::sys::signal::kill(
			nix::unistd::Pid::from_raw(grandchild),
			nix::sys::signal::Signal::SIGKILL,
		);
	}
	assert!(!survived, "manager grandchild survived the stop");
}
