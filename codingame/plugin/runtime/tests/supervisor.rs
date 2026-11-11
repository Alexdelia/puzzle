use std::fs;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use contest::agent::{RateLimitInfo, Window};
use contest::atomic::write_atomic;
use contest::broker::parse_cpu_list;
use contest::broker::{JobClass, JobRequest, Machine};
use contest::config::Config;
use contest::job::{JobRecord, JobState, read_job_record, read_lease, write_job_record};
use contest::paths::{JobId, RunDir, TaskId};
use contest::run_info::{Mode, RunInfo};
use contest::supervisor::active_increment;
use contest::task::{TaskSpec, TaskState, TaskStore};
use contest::usage::record_usage;
use nix::fcntl::{Flock, FlockArg};
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use serde_json::{Value, json};
use tempfile::TempDir;

const WEEK_SECONDS: i64 = 604800;
const BIN: &str = env!("CARGO_BIN_EXE_contest");
const MANAGER_BRANCH: &str = "FAKE_CLAUDE_SCRIPT=$FAKE_MANAGER_STREAM";

fn fixture(name: &str) -> PathBuf {
	Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("tests/fixtures")
		.join(name)
}

fn plugin_dir() -> PathBuf {
	Path::new(env!("CARGO_MANIFEST_DIR"))
		.parent()
		.unwrap()
		.to_path_buf()
}

fn now() -> i64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs() as i64
}

fn usage_at(five_hour: f64) -> RateLimitInfo {
	let now = now();
	RateLimitInfo {
		status: "allowed".into(),
		resets_at: now + 3600,
		five_hour: Window {
			utilization: five_hour,
			resets_at: now + 3600,
		},
		seven_day: Window {
			utilization: 0.1,
			resets_at: now + WEEK_SECONDS / 2,
		},
	}
}

fn result_stream(path: &Path, result: &str) {
	let line = json!({
		"type": "result",
		"subtype": "success",
		"is_error": false,
		"result": result,
		"total_cost_usd": 0.01,
		"usage": {"input_tokens": 10, "output_tokens": 5},
	});
	fs::write(path, format!("{line}\n")).unwrap();
}

struct Contest {
	_dir: TempDir,
	run: RunDir,
	home: PathBuf,
	manager_stream: PathBuf,
}

fn contest(workers: usize) -> Contest {
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
		grace_seconds: 2,
		sandbox: false,
		claude_bin: fixture("fake-claude.sh"),
		plugin_dir: plugin_dir(),
		..Config::default()
	};
	write_atomic(&run.config(), toml::to_string(&config).unwrap().as_bytes()).unwrap();
	let info = RunInfo {
		puzzle: "seabed-security".into(),
		mode: Mode::PastPuzzle,
		deadline_unix: None,
		active_budget_seconds: None,
		active_seconds: 0,
		phase: "start".into(),
		plugin_commit: String::new(),
		started_at: now(),
	};
	write_atomic(&run.run_json(), &serde_json::to_vec(&info).unwrap()).unwrap();
	record_usage(&run, &usage_at(0.1), now()).unwrap();
	let home = dir.path().join("home");
	fs::create_dir_all(&home).unwrap();
	let manager_stream = dir.path().join("manager-stream.jsonl");
	result_stream(&manager_stream, "nothing to do\n```json\n[]\n```");
	Contest {
		_dir: dir,
		run,
		home,
		manager_stream,
	}
}

fn spec(kind: &str, priority: i32) -> TaskSpec {
	TaskSpec {
		kind: kind.into(),
		title: format!("{kind} task"),
		priority,
		model: None,
		budget_usd: None,
		writes: Vec::new(),
		body: format!("body of the {kind} task"),
		parent: None,
	}
}

fn create_task(run: &RunDir, kind: &str, priority: i32) -> TaskId {
	TaskStore { run }.create(spec(kind, priority)).unwrap()
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
	fn pid(&self) -> Pid {
		Pid::from_raw(self.0.id() as i32)
	}

	fn stop(mut self, run: &RunDir) {
		fs::write(run.stop_file(), "test").unwrap();
		let deadline = Instant::now() + Duration::from_secs(20);
		while self.0.try_wait().unwrap().is_none() {
			assert!(Instant::now() < deadline, "supervisor ignored STOP");
			sleep(Duration::from_millis(50));
		}
	}
}

fn edit_config(contest: &Contest, edit: impl FnOnce(&mut Config)) {
	let mut config = Config::load(&contest.run.config()).unwrap();
	edit(&mut config);
	write_atomic(
		&contest.run.config(),
		toml::to_string(&config).unwrap().as_bytes(),
	)
	.unwrap();
}

fn start(contest: &Contest, worker_action: &str) -> Supervisor {
	start_with_probe(contest, worker_action, ":")
}

fn start_with_probe(contest: &Contest, worker_action: &str, probe_action: &str) -> Supervisor {
	start_with_env(contest, worker_action, probe_action, &[])
}

fn start_with_env(
	contest: &Contest,
	worker_action: &str,
	probe_action: &str,
	env: &[(&str, &Path)],
) -> Supervisor {
	let exec = format!(
		"if [[ $prompt == ok ]]; then {probe_action}; elif [[ -n ${{CONTEST_TASK:-}} ]]; then {worker_action}; else {MANAGER_BRANCH}; fi"
	);
	let log = fs::File::create(contest.home.join("supervisor.log")).unwrap();
	let child = Command::new(BIN)
		.arg("start")
		.arg(&contest.run.root)
		.arg("--foreground")
		.env("HOME", &contest.home)
		.env("FAKE_CLAUDE_SCRIPT", fixture("stream-success.jsonl"))
		.env("FAKE_MANAGER_STREAM", &contest.manager_stream)
		.env("FAKE_CLAUDE_EXEC", exec)
		.env("ANTHROPIC_TEST_SECRET", "leak")
		.env("CLAUDE_TEST_SECRET", "leak")
		.envs(env.iter().copied())
		.stdin(Stdio::null())
		.stdout(log.try_clone().unwrap())
		.stderr(log)
		.spawn()
		.unwrap();
	Supervisor(child)
}

fn wait_until(seconds: u64, what: &str, mut condition: impl FnMut() -> bool) {
	let deadline = Instant::now() + Duration::from_secs(seconds);
	while !condition() {
		assert!(Instant::now() < deadline, "timed out waiting for {what}");
		sleep(Duration::from_millis(50));
	}
}

fn events(run: &RunDir) -> Vec<Value> {
	fs::read_to_string(run.events())
		.unwrap_or_default()
		.lines()
		.map(|line| serde_json::from_str(line).unwrap())
		.collect()
}

fn has_event(run: &RunDir, expected: Value) -> bool {
	events(run).iter().any(|event| {
		expected
			.as_object()
			.unwrap()
			.iter()
			.all(|(key, value)| &event[key] == value)
	})
}

fn running_count(run: &RunDir) -> usize {
	TaskStore { run }
		.list()
		.unwrap()
		.iter()
		.filter(|task| task.state == TaskState::Running)
		.count()
}

fn contest_say(run: &RunDir, text: &str) -> Output {
	Command::new(BIN)
		.arg("say")
		.arg(&run.root)
		.arg(text)
		.output()
		.unwrap()
}

#[test]
fn task_runs_to_done() {
	let contest = contest(1);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let prompt_dump = contest.home.join("worker-prompt");
	let env_dump = contest.home.join("worker-env");
	let action = format!(
		"printf %s \"$prompt\" > {prompt}; env > {env}; contest task done",
		prompt = prompt_dump.display(),
		env = env_dump.display()
	);
	let supervisor = start(&contest, &action);
	wait_until(20, "task finished event", || {
		has_event(
			run,
			json!({"event": "task_finished", "task": id.0, "state": "done"}),
		)
	});
	supervisor.stop(run);
	assert_eq!(state(run, &id), TaskState::Done);
	let ledger = fs::read_to_string(run.ledger()).unwrap();
	let row = ledger
		.lines()
		.find(|line| line.split('\t').nth(1) == Some(id.0.as_str()))
		.expect("ledger row for the task");
	assert_eq!(row.split('\t').nth(2), Some("opus"));
	let prompt = fs::read_to_string(&prompt_dump).unwrap();
	assert!(prompt.contains(&id.0));
	assert!(prompt.contains("body of the sim-bench task"));
	assert!(prompt.contains(&run.task_dir(&id).display().to_string()));
	let env = fs::read_to_string(&env_dump).unwrap();
	assert!(env.contains(&format!("CONTEST_TASK={id}")));
	assert!(env.contains(&format!("CONTEST_RUN={root}", root = run.root.display())));
	assert!(!env.contains("ANTHROPIC_TEST_SECRET"));
	assert!(!env.contains("CLAUDE_TEST_SECRET"));
	assert!(run.agents().join(&id.0).join("stream.jsonl").is_file());
}

#[test]
fn worker_exit_without_outcome_fails_task() {
	let contest = contest(1);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let supervisor = start(&contest, "true");
	wait_until(20, "task finished event", || {
		has_event(
			run,
			json!({"event": "task_finished", "task": id.0, "state": "failed"}),
		)
	});
	supervisor.stop(run);
	assert_eq!(state(run, &id), TaskState::Failed);
	let notes = fs::read_to_string(run.task_dir(&id).join("notes.md")).unwrap();
	assert!(notes.contains("exited without outcome"));
}

#[test]
fn planted_files_removed_after_task() {
	let contest = contest(1);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let planted = run.task_dir(&id).join("sub/.git");
	let action = format!(
		"mkdir -p {planted}; contest task done",
		planted = planted.display()
	);
	let supervisor = start(&contest, &action);
	wait_until(20, "task finished event", || {
		has_event(run, json!({"event": "task_finished", "task": id.0}))
	});
	supervisor.stop(run);
	assert!(!planted.exists());
	let journal = fs::read_to_string(run.journal()).unwrap();
	assert!(journal.contains(&planted.display().to_string()));
}

#[test]
fn planted_names_cannot_forge_journal_lines() {
	let contest = contest(1);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let forged = run
		.task_dir(&id)
		.join("x\n2026-01-01 00:00:00 forged entry");
	let planted = forged.join(".git");
	let action = format!(
		"mkdir -p \"{planted}\"; contest task done",
		planted = planted.display()
	);
	let supervisor = start(&contest, &action);
	wait_until(20, "task finished event", || {
		has_event(run, json!({"event": "task_finished", "task": id.0}))
	});
	supervisor.stop(run);
	assert!(!planted.exists());
	assert_eq!(journal_lines(run, "forged entry"), 1);
	assert!(
		fs::read_to_string(run.journal())
			.unwrap()
			.lines()
			.all(|line| !line.starts_with("2026-01-01 00:00:00 forged"))
	);
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
	edit_config(contest, |config| {
		config.repo_root = repo.clone();
		config.git_common_dir = repo.join(".git");
	});
	repo
}

#[test]
fn commit_scans_before_commit() {
	let contest = contest(1);
	let run = &contest.run;
	let repo = pin_repo(&contest);
	let id = create_task(run, "sim-bench", 0);
	let task_dir = run.task_dir(&id);
	let planted = task_dir.join("sub/.git");
	let scanned = contest.home.join("scanned-before-commit");
	let action = format!(
		"mkdir -p {planted}; echo hi > {out}; contest commit --milestone first; [ -e {planted} ] || touch {scanned}; contest task done",
		planted = planted.display(),
		out = task_dir.join("out.txt").display(),
		scanned = scanned.display()
	);
	let supervisor = start(&contest, &action);
	wait_until(20, "task finished event", || {
		has_event(
			run,
			json!({"event": "task_finished", "task": id.0, "state": "done"}),
		)
	});
	supervisor.stop(run);
	assert!(scanned.exists());
	let log = git(&repo, &["log", "--format=%s"]);
	assert_eq!(
		log.lines().next(),
		Some(format!("sim-bench: {id} first").as_str())
	);
	let files = git(&repo, &["show", "--name-only", "--format=", "HEAD"]);
	assert!(files.contains("out.txt"));
}

#[test]
fn manager_creates_tasks() {
	let contest = contest(0);
	let run = &contest.run;
	let actions = json!([
		{"action": "create-task", "kind": "study-rules", "title": "read the rules", "priority": 5, "body": "read"},
		{"action": "create-task", "kind": "bot-baseline", "title": "first bot", "priority": 3, "body": "write"},
	]);
	result_stream(
		&contest.manager_stream,
		&format!("plan\n```json\n{actions}\n```"),
	);
	assert!(
		contest_say(run, "please start with the rules")
			.status
			.success()
	);
	let supervisor = start(&contest, "true");
	wait_until(20, "two tasks", || {
		TaskStore { run }.list().unwrap().len() == 2
	});
	wait_until(10, "inbox acknowledged", || {
		fs::read_dir(run.inbox().join("read")).is_ok_and(|mut entries| entries.next().is_some())
	});
	supervisor.stop(run);
	let tasks = TaskStore { run }.list().unwrap();
	assert!(tasks.iter().all(|task| task.state == TaskState::Queued));
	assert!(run.manager_cursor().is_file());
	let ledger = fs::read_to_string(run.ledger()).unwrap();
	assert!(
		ledger
			.lines()
			.any(|line| line.split('\t').nth(1) == Some("manager"))
	);
}

#[test]
fn manager_prompt_carries_digest() {
	let contest = contest(0);
	let run = &contest.run;
	assert!(contest_say(run, "hello manager").status.success());
	let prompt_dump = contest.home.join("manager-prompt");
	let exec = format!(
		"printf %s \"$prompt\" > {dump}; FAKE_CLAUDE_SCRIPT=$FAKE_MANAGER_STREAM",
		dump = prompt_dump.display()
	);
	let log = fs::File::create(contest.home.join("supervisor.log")).unwrap();
	let child = Command::new(BIN)
		.arg("start")
		.arg(&run.root)
		.arg("--foreground")
		.env("HOME", &contest.home)
		.env("FAKE_MANAGER_STREAM", &contest.manager_stream)
		.env("FAKE_CLAUDE_EXEC", exec)
		.stdout(log.try_clone().unwrap())
		.stderr(log)
		.spawn()
		.unwrap();
	let supervisor = Supervisor(child);
	wait_until(20, "manager prompt", || prompt_dump.is_file());
	supervisor.stop(run);
	let prompt = fs::read_to_string(&prompt_dump).unwrap();
	let template = fs::read_to_string(plugin_dir().join("prompts/manager.md")).unwrap();
	assert!(prompt.starts_with(template.trim_end()));
	assert!(prompt.contains("## inbox"));
	assert!(prompt.contains("hello manager"));
}

#[test]
fn failed_manager_tick_keeps_inbox_and_cursor() {
	let contest = contest(0);
	let run = &contest.run;
	result_stream(&contest.manager_stream, "no json here");
	assert!(contest_say(run, "keep me").status.success());
	let supervisor = start(&contest, "true");
	wait_until(20, "manager error event", || {
		has_event(run, json!({"event": "manager_error"}))
	});
	supervisor.stop(run);
	assert!(!run.manager_cursor().exists());
	assert!(!run.inbox().join("read").exists());
	assert_eq!(fs::read_dir(run.inbox()).unwrap().count(), 1);
}

#[test]
fn huge_says_still_tick() {
	let contest = contest(0);
	let run = &contest.run;
	for marker in ['a', 'b'] {
		let text = marker.to_string().repeat(100 * 1024);
		assert!(contest_say(run, &text).status.success());
	}
	let supervisor = start(&contest, "true");
	wait_until(20, "both messages acknowledged", || {
		fs::read_dir(run.inbox().join("read")).is_ok_and(|entries| entries.count() == 2)
	});
	supervisor.stop(run);
	assert!(run.manager_cursor().is_file());
	assert!(
		events(run)
			.iter()
			.filter(|event| event["event"] == "user_message")
			.all(|event| event["text"].as_str().unwrap().chars().count() <= 200)
	);
}

fn manager_ticks(run: &RunDir) -> usize {
	fs::read_to_string(run.ledger())
		.unwrap_or_default()
		.lines()
		.filter(|line| line.split('\t').nth(1) == Some("manager"))
		.count()
}

#[test]
fn partial_event_line_does_not_retick_manager() {
	let contest = contest(0);
	let run = &contest.run;
	fs::write(run.events(), "{\"event\":\"budget_exh").unwrap();
	let supervisor = start(&contest, "true");
	wait_until(20, "first tick", || manager_ticks(run) >= 1);
	sleep(Duration::from_secs(4));
	supervisor.stop(run);
	assert_eq!(manager_ticks(run), 1);
}

#[test]
fn cursor_write_error_backs_off() {
	let contest = contest(0);
	let run = &contest.run;
	fs::create_dir_all(run.manager_cursor().join("blocker")).unwrap();
	assert!(contest_say(run, "hello").status.success());
	let supervisor = start(&contest, "true");
	wait_until(20, "manager error", || {
		event_count(run, "manager_error") >= 1
	});
	sleep(Duration::from_secs(4));
	supervisor.stop(run);
	assert_eq!(manager_ticks(run), 1);
	assert_eq!(event_count(run, "manager_error"), 1);
}

#[test]
fn say_writes_inbox_and_event() {
	let contest = contest(0);
	let run = &contest.run;
	let output = contest_say(run, "switch to beam search");
	assert!(output.status.success());
	let messages: Vec<PathBuf> = fs::read_dir(run.inbox())
		.unwrap()
		.map(|entry| entry.unwrap().path())
		.collect();
	assert_eq!(messages.len(), 1);
	assert!(messages[0].extension().is_some_and(|ext| ext == "md"));
	assert_eq!(
		fs::read_to_string(&messages[0]).unwrap(),
		"switch to beam search"
	);
	assert!(has_event(
		run,
		json!({"event": "user_message", "text": "switch to beam search"})
	));
}

#[test]
fn respects_worker_limit() {
	let contest = contest(2);
	let run = &contest.run;
	for kind in ["a", "b", "c", "d", "e"] {
		create_task(run, kind, 0);
	}
	let supervisor = start(&contest, "exec sleep 60");
	let mut most = 0;
	let deadline = Instant::now() + Duration::from_secs(5);
	while Instant::now() < deadline {
		let running = running_count(run);
		assert!(running <= 2, "{running} workers running");
		most = most.max(running);
		sleep(Duration::from_millis(50));
	}
	supervisor.stop(run);
	assert_eq!(most, 2);
}

#[test]
fn paces_on_five_hour() {
	let contest = contest(1);
	let run = &contest.run;
	record_usage(run, &usage_at(0.96), now()).unwrap();
	let id = create_task(run, "sim-bench", 0);
	let supervisor = start(&contest, "contest task done");
	wait_until(10, "status file", || run.status().is_file());
	sleep(Duration::from_secs(3));
	supervisor.stop(run);
	assert_eq!(state(run, &id), TaskState::Queued);
	assert!(!run.agents().join(&id.0).exists());
}

#[test]
fn second_supervisor_refused() {
	let contest = contest(0);
	let run = &contest.run;
	let supervisor = start(&contest, "true");
	wait_until(10, "status file", || run.status().is_file());
	let output = Command::new(BIN)
		.arg("start")
		.arg(&run.root)
		.arg("--foreground")
		.env("HOME", &contest.home)
		.output()
		.unwrap();
	assert!(!output.status.success());
	assert!(String::from_utf8_lossy(&output.stderr).contains("already running"));
	supervisor.stop(run);
}

#[test]
fn contest_dir_holding_git_refused() {
	let contest = contest(0);
	let run = &contest.run;
	fs::create_dir_all(run.root.join(".git")).unwrap();
	let output = Command::new(BIN)
		.arg("start")
		.arg(&run.root)
		.arg("--foreground")
		.env("HOME", &contest.home)
		.output()
		.unwrap();
	assert!(!output.status.success());
	assert!(String::from_utf8_lossy(&output.stderr).contains(".git"));
	assert!(run.root.join(".git").is_dir());
}

fn start_output(contest: &Contest) -> std::process::Output {
	start_output_with_path(contest, std::env::var_os("PATH").unwrap())
}

fn start_output_with_path(contest: &Contest, path: std::ffi::OsString) -> std::process::Output {
	let mut child = Command::new(BIN)
		.env("PATH", path)
		.arg("start")
		.arg(&contest.run.root)
		.arg("--foreground")
		.env("HOME", &contest.home)
		.stdout(Stdio::null())
		.stderr(Stdio::piped())
		.spawn()
		.unwrap();
	let deadline = Instant::now() + Duration::from_secs(10);
	while child.try_wait().unwrap().is_none() {
		if Instant::now() >= deadline {
			let _ = child.kill();
			let _ = child.wait();
			panic!("supervisor started instead of refusing");
		}
		sleep(Duration::from_millis(20));
	}
	child.wait_with_output().unwrap()
}

#[test]
fn start_refuses_contest_root_holding_toolbox_or_plugin() {
	let contest = contest(0);
	let run = &contest.run;
	let original = Config::load(&run.config()).unwrap();
	let inside = run.root.join("tools");
	fs::create_dir_all(&inside).unwrap();
	for field in ["toolbox", "plugin_dir", "git_common_dir"] {
		write_atomic(
			&run.config(),
			toml::to_string(&original).unwrap().as_bytes(),
		)
		.unwrap();
		edit_config(&contest, |config| match field {
			"toolbox" => config.toolbox = vec![inside.clone()],
			"plugin_dir" => config.plugin_dir = inside.clone(),
			_ => config.git_common_dir = inside.clone(),
		});
		let output = start_output(&contest);
		let stderr = String::from_utf8_lossy(&output.stderr);
		assert!(!output.status.success(), "{field}: {stderr}");
		assert!(stderr.contains(field), "{field}: {stderr}");
	}
	assert!(!run.status().exists());
}

#[test]
fn active_time_capped_after_pause() {
	assert_eq!(
		active_increment(Duration::from_secs(3600)),
		Duration::from_secs(5)
	);
	assert_eq!(
		active_increment(Duration::from_millis(1200)),
		Duration::from_millis(1200)
	);

	let contest = contest(0);
	let run = &contest.run;
	let supervisor = start(&contest, "true");
	wait_until(10, "status file", || run.status().is_file());
	let paused_at = Instant::now();
	kill(supervisor.pid(), Signal::SIGSTOP).unwrap();
	sleep(Duration::from_secs(20));
	kill(supervisor.pid(), Signal::SIGCONT).unwrap();
	sleep(Duration::from_secs(2));
	supervisor.stop(run);
	let wall = paused_at.elapsed().as_secs();
	let info: RunInfo = serde_json::from_slice(&fs::read(run.run_json()).unwrap()).unwrap();
	assert!(
		info.active_seconds + 12 <= wall,
		"active {active} of {wall} wall seconds",
		active = info.active_seconds
	);
	assert!(info.active_seconds >= 5);
}

#[test]
fn budget_reached_writes_stop() {
	let contest = contest(0);
	let run = &contest.run;
	let mut info: RunInfo = serde_json::from_slice(&fs::read(run.run_json()).unwrap()).unwrap();
	info.active_budget_seconds = Some(1);
	write_atomic(&run.run_json(), &serde_json::to_vec(&info).unwrap()).unwrap();
	let mut supervisor = start(&contest, "true");
	let deadline = Instant::now() + Duration::from_secs(20);
	while supervisor.0.try_wait().unwrap().is_none() {
		assert!(Instant::now() < deadline, "supervisor kept running");
		sleep(Duration::from_millis(50));
	}
	assert_eq!(fs::read_to_string(run.stop_file()).unwrap(), "budget");
	assert!(has_event(run, json!({"event": "budget_exhausted"})));
}

#[test]
fn cancelled_task_worker_killed() {
	let contest = contest(1);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let supervisor = start(&contest, "exec sleep 60");
	wait_until(20, "task running", || state(run, &id) == TaskState::Running);
	TaskStore { run }
		.transition(&id, TaskState::Cancelled)
		.unwrap();
	wait_until(10, "cancelled task finished", || {
		has_event(
			run,
			json!({"event": "task_finished", "task": id.0, "state": "cancelled"}),
		)
	});
	supervisor.stop(run);
}

#[test]
fn worker_job_runs_to_finish() {
	let contest = contest(1);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let supervisor = start(
		&contest,
		"contest job submit --class batch --ram-mb 10 -- sh -c 'exit 3' && contest job wait j0001; contest task done",
	);
	wait_until(20, "job finished event", || {
		has_event(
			run,
			json!({"event": "job_finished", "job": "j0001", "code": 3}),
		)
	});
	supervisor.stop(run);
	let record = read_job_record(run, &JobId("j0001".into())).unwrap();
	assert_eq!(record.owner_task, id);
	assert_eq!(record.state, JobState::Done { code: 3 });
	assert!(record.starttime.is_some());
}

#[test]
fn restart_marks_dead_jobs_interrupted() {
	let contest = contest(0);
	let run = &contest.run;
	let owner = create_task(run, "sim-bench", 0);
	let mut gone = Command::new("true").spawn().unwrap();
	let pid = gone.id();
	gone.wait().unwrap();
	let job = JobId("j0001".into());
	write_job_record(
		run,
		&JobRecord {
			id: job.clone(),
			owner_task: owner,
			cmd: vec!["true".into()],
			cwd: run.root.clone(),
			request: JobRequest {
				id: job.clone(),
				class: JobClass::Batch,
				gpu: false,
				ram_mb: 1,
				priority: 0,
			},
			state: JobState::Running,
			pid: Some(pid),
			starttime: Some(1),
			sandboxed: false,
		},
	)
	.unwrap();
	let supervisor = start(&contest, "true");
	wait_until(10, "job interrupted event", || {
		has_event(run, json!({"event": "job_interrupted", "job": "j0001"}))
	});
	supervisor.stop(run);
	assert_eq!(
		read_job_record(run, &job).unwrap().state,
		JobState::Interrupted
	);
}

#[test]
fn restart_removes_planted_files() {
	let contest = contest(0);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let planted = run.task_dir(&id).join(".envrc");
	fs::write(&planted, "echo planted").unwrap();
	let supervisor = start(&contest, "true");
	wait_until(10, "status file", || run.status().is_file());
	supervisor.stop(run);
	assert!(!planted.exists());
}

#[test]
fn restart_requeues_interrupted() {
	let contest = contest(1);
	let run = &contest.run;
	let first = create_task(run, "sim-bench", 0);
	let supervisor = start(&contest, "exec sleep 60");
	wait_until(20, "first task running", || {
		state(run, &first) == TaskState::Running
	});
	let second = create_task(run, "bot-baseline", 500);
	kill(supervisor.pid(), Signal::SIGKILL).unwrap();
	drop(supervisor);
	assert_eq!(state(run, &first), TaskState::Running);

	let supervisor = start(&contest, "exec sleep 60");
	wait_until(20, "first task relaunched", || {
		run.journal().is_file()
			&& fs::read_to_string(run.journal())
				.unwrap()
				.lines()
				.filter(|line| line.contains("launched"))
				.count() >= 2
	});
	sleep(Duration::from_secs(1));
	supervisor.stop(run);
	let store = TaskStore { run };
	assert!(store.spec(&first).unwrap().priority >= 1000);
	assert_eq!(store.state(&first).unwrap(), TaskState::Interrupted);
	assert_eq!(store.state(&second).unwrap(), TaskState::Queued);
}

#[test]
fn status_reports_running_task() {
	let contest = contest(1);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let supervisor = start(&contest, "exec sleep 60");
	wait_until(20, "task running", || state(run, &id) == TaskState::Running);
	let output = Command::new(BIN)
		.arg("status")
		.arg(&run.root)
		.output()
		.unwrap();
	supervisor.stop(run);
	assert!(output.status.success());
	let text = String::from_utf8_lossy(&output.stdout);
	assert!(text.lines().count() <= 40);
	assert!(text.contains("phase: start"));
	assert!(text.contains(&id.0));
	let written = fs::read_to_string(run.status()).unwrap();
	assert!(written.lines().count() <= 40);
}

#[test]
fn worker_prompt_carries_rules() {
	let worker = fs::read_to_string(plugin_dir().join("prompts/worker.md")).unwrap();
	for required in [
		"{task_id}",
		"{task_dir}",
		"{spec}",
		"{notes}",
		"contest commit --milestone",
		"contest job submit",
		"contest job wait",
		"CONTEST_JOB_WORK",
		"notes.md",
		"result.md",
		"contest task done",
		"contest task failed",
		"contest task ask",
		"git",
		"comment",
		"named format",
		"clap derive",
		"2 min",
	] {
		assert!(worker.contains(required), "worker.md lacks {required:?}");
	}
	let manager = fs::read_to_string(plugin_dir().join("prompts/manager.md")).unwrap();
	for action in [
		"create-task",
		"cancel",
		"reprioritize",
		"answer",
		"set-phase",
		"journal",
		"reply",
		"kb/index.md",
		"```json",
	] {
		assert!(manager.contains(action), "manager.md lacks {action:?}");
	}
}

fn journal_lines(run: &RunDir, needle: &str) -> usize {
	fs::read_to_string(run.journal())
		.unwrap_or_default()
		.lines()
		.filter(|line| line.contains(needle))
		.count()
}

fn event_count(run: &RunDir, name: &str) -> usize {
	events(run)
		.iter()
		.filter(|event| event["event"] == name)
		.count()
}

fn plugin_without_worker_prompt(contest: &Contest) -> PathBuf {
	let plugin = contest.home.join("plugin");
	fs::create_dir_all(plugin.join("prompts")).unwrap();
	fs::copy(
		plugin_dir().join("prompts/manager.md"),
		plugin.join("prompts/manager.md"),
	)
	.unwrap();
	plugin
}

fn stop_within(supervisor: Supervisor, run: &RunDir, seconds: u64) {
	let stopped_at = Instant::now();
	supervisor.stop(run);
	assert!(
		stopped_at.elapsed() < Duration::from_secs(seconds),
		"STOP took {elapsed:?}",
		elapsed = stopped_at.elapsed()
	);
}

fn contest_status(run: &RunDir) -> String {
	let output = Command::new(BIN)
		.arg("status")
		.arg(&run.root)
		.output()
		.unwrap();
	assert!(output.status.success());
	String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn launch_error_requeues_and_backs_off() {
	let contest = contest(3);
	let run = &contest.run;
	let plugin = plugin_without_worker_prompt(&contest);
	edit_config(&contest, |config| config.plugin_dir = plugin);
	let tasks: Vec<TaskId> = ["a", "b", "c"]
		.into_iter()
		.map(|kind| create_task(run, kind, 0))
		.collect();
	let supervisor = start(&contest, "contest task done");
	wait_until(10, "launch failed event", || {
		event_count(run, "launch_failed") >= 1
	});
	sleep(Duration::from_secs(3));
	supervisor.stop(run);
	assert_eq!(event_count(run, "launch_failed"), 1);
	assert_eq!(journal_lines(run, "launch of"), 1);
	for id in &tasks {
		assert_eq!(state(run, id), TaskState::Queued);
	}
}

#[test]
fn probe_does_not_block_loop() {
	let contest = contest(1);
	let run = &contest.run;
	fs::remove_file(run.usage()).unwrap();
	create_task(run, "sim-bench", 0);
	let supervisor = start_with_probe(&contest, "contest task done", "exec sleep 60");
	wait_until(10, "probe started", || {
		run.agents().join("probe/stream.jsonl").is_file()
	});
	sleep(Duration::from_secs(1));
	stop_within(supervisor, run, 4);
}

#[test]
fn probe_killed_at_timeout() {
	let contest = contest(1);
	let run = &contest.run;
	fs::remove_file(run.usage()).unwrap();
	edit_config(&contest, |config| config.probe_timeout_seconds = 2);
	let id = create_task(run, "sim-bench", 0);
	let pid_file = contest.home.join("probe-pid");
	let probe = format!("echo $$ > {pid}; exec sleep 60", pid = pid_file.display());
	let supervisor = start_with_probe(&contest, "contest task done", &probe);
	wait_until(10, "probe timeout", || {
		journal_lines(run, "usage probe timed out") >= 1
	});
	let pid: i32 = fs::read_to_string(&pid_file)
		.unwrap()
		.trim()
		.parse()
		.unwrap();
	wait_until(5, "probe killed", || {
		!Path::new(&format!("/proc/{pid}")).exists()
	});
	wait_until(5, "usage unknown event", || {
		event_count(run, "usage_unknown") >= 1
	});
	let status = contest_status(run);
	supervisor.stop(run);
	assert_eq!(event_count(run, "usage_unknown"), 1);
	assert!(status.contains("no launches"), "{status}");
	assert_eq!(state(run, &id), TaskState::Queued);
}

#[test]
fn stale_usage_triggers_probe() {
	let contest = contest(0);
	let run = &contest.run;
	record_usage(run, &usage_at(0.1), now() - 7200).unwrap();
	let supervisor = start(&contest, "true");
	wait_until(10, "fresh usage", || {
		let usage: Value = serde_json::from_slice(&fs::read(run.usage()).unwrap()).unwrap();
		usage["recorded_at"].as_i64().unwrap() > now() - 60
	});
	wait_until(10, "probe in ledger", || {
		fs::read_to_string(run.ledger())
			.unwrap_or_default()
			.lines()
			.any(|line| {
				let fields: Vec<&str> = line.split('\t').collect();
				fields[1..3] == ["probe", "haiku"] && fields[7] == "0.05"
			})
	});
	supervisor.stop(run);
}

#[test]
fn manager_runs_sandboxed_with_read_only_tools() {
	let contest = contest(0);
	let run = &contest.run;
	let argv_dump = contest.home.join("manager-argv");
	let pasta = contest.home.join("fake-pasta.sh");
	fs::write(
		&pasta,
		format!(
			"#!/usr/bin/env bash\nprintf '%s\\0' \"$@\" > {dump}\ncat \"$FAKE_MANAGER_STREAM\"\n",
			dump = argv_dump.display()
		),
	)
	.unwrap();
	fs::set_permissions(&pasta, fs::Permissions::from_mode(0o755)).unwrap();
	edit_config(&contest, |config| {
		config.sandbox = true;
		config.pasta_bin = pasta.clone();
		config.repo_root = run.root.clone();
	});
	let supervisor = start(&contest, "true");
	wait_until(20, "sandboxed manager in ledger", || {
		fs::read_to_string(run.ledger())
			.unwrap_or_default()
			.lines()
			.any(|line| line.split('\t').nth(1) == Some("manager"))
	});
	supervisor.stop(run);
	let dump = fs::read_to_string(&argv_dump).unwrap();
	let argv: Vec<&str> = dump.split('\0').collect();
	let claude = fixture("fake-claude.sh");
	let claude_at = argv
		.iter()
		.position(|arg| Path::new(arg) == claude)
		.unwrap();
	let bwrap_at = argv.iter().position(|arg| *arg == "bwrap").unwrap();
	assert!(bwrap_at < claude_at);
	let (sandbox, claude_args) = argv.split_at(claude_at);
	let tools_at = claude_args
		.iter()
		.position(|arg| *arg == "--tools")
		.unwrap();
	assert_eq!(claude_args[tools_at + 1], "Read,Glob,Grep");
	assert!(!claude_args.contains(&"--allowedTools"));
	let root = run.root.to_string_lossy();
	let bound = |flag: &str| {
		sandbox
			.windows(3)
			.any(|mount| mount[0] == flag && mount[1] == root && mount[2] == root)
	};
	assert!(bound("--ro-bind-fd"));
	assert!(!bound("--bind-fd") && !bound("--bind") && !bound("--ro-bind"));
}

#[test]
fn swapped_pinned_envrc_fails_launch_closed() {
	let contest = contest(1);
	let run = &contest.run;
	std::os::unix::fs::symlink(contest.home.join("secret"), run.root.join(".envrc")).unwrap();
	edit_config(&contest, |config| {
		config.sandbox = true;
		config.contest_envrc = true;
		config.pasta_bin = PathBuf::from("/nonexistent/pasta");
		config.repo_root = run.root.clone();
	});
	let id = create_task(run, "sim-bench", 0);
	let supervisor = start(&contest, "contest task done");
	wait_until(10, "launch failed event", || {
		event_count(run, "launch_failed") >= 1
	});
	supervisor.stop(run);
	let failure = events(run)
		.into_iter()
		.find(|event| event["event"] == "launch_failed")
		.unwrap();
	assert_eq!(failure["task"], id.0);
	assert!(
		failure["error"].as_str().unwrap().contains(".envrc"),
		"{failure}"
	);
	assert_eq!(state(run, &id), TaskState::Queued);
	assert!(
		fs::symlink_metadata(run.root.join(".envrc"))
			.unwrap()
			.is_symlink()
	);
}

#[test]
fn expiring_credentials_trigger_one_probe() {
	let contest = contest(0);
	let run = &contest.run;
	let credentials = contest.home.join(".claude/.credentials.json");
	fs::create_dir_all(credentials.parent().unwrap()).unwrap();
	let expires_at_ms = (now() + 60) * 1000;
	fs::write(
		&credentials,
		json!({"claudeAiOauth": {"expiresAt": expires_at_ms}}).to_string(),
	)
	.unwrap();
	let probes = contest.home.join("probes");
	let probe = format!("echo probe >> {probes}", probes = probes.display());
	let supervisor = start_with_probe(&contest, "true", &probe);
	wait_until(10, "first probe", || probes.is_file());
	sleep(Duration::from_secs(4));
	supervisor.stop(run);
	assert_eq!(fs::read_to_string(&probes).unwrap().lines().count(), 1);
}

#[test]
fn manager_timeout_kills_manager() {
	let contest = contest(0);
	let run = &contest.run;
	edit_config(&contest, |config| config.manager_timeout_seconds = 2);
	let pid_file = contest.home.join("manager-pid");
	let exec = format!(
		"if [[ $prompt != ok ]]; then echo $$ > {pid}; exec sleep 60; fi",
		pid = pid_file.display()
	);
	let log = fs::File::create(contest.home.join("supervisor.log")).unwrap();
	let child = Command::new(BIN)
		.arg("start")
		.arg(&run.root)
		.arg("--foreground")
		.env("HOME", &contest.home)
		.env("FAKE_CLAUDE_SCRIPT", fixture("stream-success.jsonl"))
		.env("FAKE_CLAUDE_EXEC", exec)
		.stdout(log.try_clone().unwrap())
		.stderr(log)
		.spawn()
		.unwrap();
	let supervisor = Supervisor(child);
	wait_until(10, "manager timeout error", || {
		events(run).iter().any(|event| {
			event["event"] == "manager_error"
				&& event["error"].as_str().unwrap().contains("timed out")
		})
	});
	let pid: i32 = fs::read_to_string(&pid_file)
		.unwrap()
		.trim()
		.parse()
		.unwrap();
	assert!(!Path::new(&format!("/proc/{pid}")).exists());
	stop_within(supervisor, run, 4);
}

#[test]
fn manager_partial_apply_saves_cursor() {
	let contest = contest(0);
	let run = &contest.run;
	let actions = json!([
		{"action": "create-task", "kind": "study-rules", "title": "read the rules", "priority": 5, "body": "read"},
		{"action": "cancel", "task": "t0099-missing"},
		{"action": "reprioritize", "task": "t0098-missing", "priority": 1},
	]);
	result_stream(
		&contest.manager_stream,
		&format!("plan\n```json\n{actions}\n```"),
	);
	assert!(contest_say(run, "go").status.success());
	let supervisor = start(&contest, "true");
	wait_until(10, "manager error", || {
		event_count(run, "manager_error") >= 1
	});
	sleep(Duration::from_secs(1));
	supervisor.stop(run);
	assert_eq!(TaskStore { run }.list().unwrap().len(), 1);
	assert!(run.manager_cursor().is_file());
	assert!(run.inbox().join("read").is_dir());
	let errors: Vec<Value> = events(run)
		.into_iter()
		.filter(|event| event["event"] == "manager_error")
		.collect();
	assert_eq!(errors.len(), 1);
	let error = errors[0]["error"].as_str().unwrap();
	assert!(
		error.contains("action 1") && error.contains("action 2"),
		"{error}"
	);
}

#[test]
fn requeued_task_with_live_old_process_is_skipped() {
	let contest = contest(2);
	let run = &contest.run;
	let asker = create_task(run, "asker", 10);
	let action = "case $CONTEST_TASK in *-asker) if [ -e \"$HOME/asked\" ]; then exec sleep 60; fi; touch \"$HOME/asked\"; contest task ask why; exec sleep 4;; *) exec sleep 60;; esac";
	let supervisor = start(&contest, action);
	wait_until(10, "question", || {
		state(run, &asker) == TaskState::NeedsAnswer
	});
	let other = create_task(run, "other", 0);
	TaskStore { run }
		.transition(&asker, TaskState::Queued)
		.unwrap();
	wait_until(5, "other launched", || {
		state(run, &other) == TaskState::Running
	});
	assert_eq!(state(run, &asker), TaskState::Queued);
	wait_until(15, "asker relaunched", || {
		state(run, &asker) == TaskState::Running
	});
	supervisor.stop(run);
	assert!(!has_event(
		run,
		json!({"event": "task_finished", "task": asker.0})
	));
	assert_eq!(journal_lines(run, &format!("finished {asker}")), 0);
}

#[test]
fn oversized_notes_are_flagged_in_prompt() {
	let contest = contest(1);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	fs::write(
		run.task_dir(&id).join("notes.md"),
		"x".repeat(2 * 1024 * 1024),
	)
	.unwrap();
	let prompt_dump = contest.home.join("worker-prompt");
	let action = format!(
		"printf %s \"$prompt\" > {prompt}; contest task done",
		prompt = prompt_dump.display()
	);
	let supervisor = start(&contest, &action);
	wait_until(20, "prompt dumped", || prompt_dump.is_file());
	supervisor.stop(run);
	let prompt = fs::read_to_string(&prompt_dump).unwrap();
	assert!(prompt.contains("notes.md unreadable or too large"));
}

#[test]
fn shutdown_finishes_after_persist_failure() {
	let contest = contest(0);
	let run = &contest.run;
	let mut supervisor = start(&contest, "true");
	wait_until(10, "status file", || run.status().is_file());
	fs::remove_file(run.run_json()).unwrap();
	fs::create_dir(run.run_json()).unwrap();
	fs::write(run.stop_file(), "test").unwrap();
	let deadline = Instant::now() + Duration::from_secs(10);
	let status = loop {
		if let Some(status) = supervisor.0.try_wait().unwrap() {
			break status;
		}
		assert!(Instant::now() < deadline, "supervisor ignored STOP");
		sleep(Duration::from_millis(50));
	};
	assert!(!status.success());
	assert_eq!(journal_lines(run, "stopped: test"), 1);
	assert_eq!(fs::read_to_string(run.supervisor_pid()).unwrap(), "");
}

#[test]
fn status_ignores_stale_pid_file() {
	let contest = contest(0);
	let run = &contest.run;
	fs::write(run.supervisor_pid(), std::process::id().to_string()).unwrap();
	assert!(contest_status(run).contains("supervisor: not running"));
}

#[test]
fn background_start_reports_early_exit() {
	let contest = contest(0);
	let run = &contest.run;
	fs::create_dir_all(run.root.join(".git")).unwrap();
	let output = Command::new(BIN)
		.arg("start")
		.arg(&run.root)
		.env("HOME", &contest.home)
		.output()
		.unwrap();
	assert!(!output.status.success());
	assert!(String::from_utf8_lossy(&output.stderr).contains("supervisor.log"));
}

#[test]
fn background_start_runs_supervisor() {
	let contest = contest(0);
	let run = &contest.run;
	let output = Command::new(BIN)
		.arg("start")
		.arg(&run.root)
		.env("HOME", &contest.home)
		.env("FAKE_CLAUDE_SCRIPT", &contest.manager_stream)
		.output()
		.unwrap();
	assert!(output.status.success());
	assert!(contest_status(run).contains("supervisor: running"));
	fs::write(run.stop_file(), "test").unwrap();
	wait_until(10, "supervisor gone", || {
		contest_status(run).contains("supervisor: stopped")
	});
}

#[test]
fn task_end_cancels_live_jobs() {
	let contest = contest(1);
	let run = &contest.run;
	create_task(run, "sim-bench", 0);
	let supervisor = start(
		&contest,
		"contest job submit --class batch --ram-mb 10 -- sleep 60 && sleep 2 && contest task done",
	);
	let job = JobId("j0001".into());
	wait_until(20, "job cancelled", || {
		read_job_record(run, &job).is_ok_and(|record| record.state == JobState::Cancelled)
	});
	supervisor.stop(run);
}

#[test]
fn unsandboxed_job_env_is_scrubbed() {
	let contest = contest(1);
	let run = &contest.run;
	create_task(run, "sim-bench", 0);
	let env_dump = contest.home.join("job-env");
	let action = format!(
		"contest job submit --class batch --ram-mb 10 -- sh -c 'env > {dump}' && contest job wait j0001 && contest task done",
		dump = env_dump.display()
	);
	let supervisor = start(&contest, &action);
	wait_until(20, "job finished", || {
		has_event(run, json!({"event": "job_finished", "job": "j0001"}))
	});
	supervisor.stop(run);
	let env = fs::read_to_string(&env_dump).unwrap();
	assert!(env.contains("CONTEST_JOB_WORK="));
	assert!(!env.contains("ANTHROPIC_TEST_SECRET"));
	assert!(!env.contains("CLAUDE_TEST_SECRET"));
}

#[test]
fn commit_refused_when_scan_fails() {
	let contest = contest(1);
	let run = &contest.run;
	let repo = pin_repo(&contest);
	let id = create_task(run, "sim-bench", 0);
	let task_dir = run.task_dir(&id);
	let deep = task_dir.join(vec!["d"; 70].join("/"));
	let action = format!(
		"mkdir -p {deep}; echo hi > {out}; contest commit --milestone first; exec sleep 60",
		deep = deep.display(),
		out = task_dir.join("out.txt").display()
	);
	let supervisor = start(&contest, &action);
	wait_until(20, "task failed", || state(run, &id) == TaskState::Failed);
	wait_until(10, "task finished event", || {
		has_event(
			run,
			json!({"event": "task_finished", "task": id.0, "state": "failed"}),
		)
	});
	supervisor.stop(run);
	let log = git(&repo, &["log", "--format=%s"]);
	assert_eq!(log.trim(), "init");
	let notes = fs::read_to_string(task_dir.join("notes.md")).unwrap();
	assert!(notes.contains("taint scan failed"));
}

#[test]
fn final_status_says_stopped() {
	let contest = contest(0);
	let run = &contest.run;
	let supervisor = start(&contest, "true");
	wait_until(10, "status file", || run.status().is_file());
	supervisor.stop(run);
	let status = fs::read_to_string(run.status()).unwrap();
	assert!(status.contains("supervisor: stopped"), "{status}");
	assert!(!status.contains("running, pid"), "{status}");
}

fn queued_job_record(run: &RunDir, id: &str, owner: &TaskId, cmd: &[&str]) -> JobId {
	let job = JobId(id.into());
	write_job_record(
		run,
		&JobRecord {
			id: job.clone(),
			owner_task: owner.clone(),
			cmd: cmd.iter().map(|part| part.to_string()).collect(),
			cwd: run.root.clone(),
			request: JobRequest {
				id: job.clone(),
				class: JobClass::Batch,
				gpu: false,
				ram_mb: 1,
				priority: 0,
			},
			state: JobState::Queued,
			pid: None,
			starttime: None,
			sandboxed: false,
		},
	)
	.unwrap();
	job
}

#[test]
fn failing_cancel_does_not_stall_jobs() {
	let contest = contest(0);
	let run = &contest.run;
	let ended = create_task(run, "ended", 0);
	let store = TaskStore { run };
	store.transition(&ended, TaskState::Running).unwrap();
	store.transition(&ended, TaskState::Done).unwrap();
	let waiting = create_task(run, "waiting", 0);
	let supervisor = start(&contest, "true");
	wait_until(10, "status file", || run.status().is_file());
	let staging = RunDir {
		root: contest.home.join("staging"),
	};
	let stuck = queued_job_record(&staging, "j0001", &ended, &["true"]);
	let staged = run.jobs().join("staged-j0001");
	fs::rename(staging.job_dir(&stuck), &staged).unwrap();
	fs::set_permissions(&staged, fs::Permissions::from_mode(0o555)).unwrap();
	let stuck_dir = run.job_dir(&stuck);
	fs::rename(&staged, &stuck_dir).unwrap();
	queued_job_record(run, "j0002", &waiting, &["true"]);
	wait_until(10, "second job finished", || {
		has_event(run, json!({"event": "job_finished", "job": "j0002"}))
	});
	sleep(Duration::from_secs(2));
	fs::set_permissions(&stuck_dir, fs::Permissions::from_mode(0o755)).unwrap();
	supervisor.stop(run);
	assert_eq!(journal_lines(run, "job j0001 cancel failed"), 1);
}

#[test]
fn cancelled_job_cores_go_to_next_job_after_reap() {
	let contest = contest(1);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let cores = Machine::detect(13000, 1).timed_cores();
	let first_pid = contest.home.join("first-pid");
	let overlap = contest.home.join("overlap");
	let script = contest.home.join("worker.sh");
	fs::write(
		&script,
		format!(
			"contest job submit --class timed --min-cores {cores} --max-cores {cores} --ram-mb 10 -- sleep 60\n\
			until [ -e {first_pid} ]; do sleep 0.1; done\n\
			contest job submit --class timed --min-cores {cores} --max-cores {cores} --ram-mb 10 -- sh -c \"kill -0 $(cat {first_pid}) 2>/dev/null && touch {overlap}; true\"\n\
			contest job cancel j0001\n\
			contest job wait j0002\n\
			contest task done\n",
			first_pid = first_pid.display(),
			overlap = overlap.display()
		),
	)
	.unwrap();
	let supervisor = start(
		&contest,
		&format!("exec bash {script}", script = script.display()),
	);
	let first = JobId("j0001".into());
	wait_until(20, "first job running", || {
		read_job_record(run, &first).is_ok_and(|record| record.state == JobState::Running)
	});
	let mut record = read_job_record(run, &first).unwrap();
	let pid = record.pid.unwrap();
	record.starttime = None;
	write_job_record(run, &record).unwrap();
	fs::write(&first_pid, pid.to_string()).unwrap();
	wait_until(20, "task done", || state(run, &id) == TaskState::Done);
	supervisor.stop(run);
	assert!(!overlap.exists(), "second job ran beside the cancelled one");
	assert!(!Path::new(&format!("/proc/{pid}")).exists());
	assert_eq!(
		read_job_record(run, &first).unwrap().state,
		JobState::Cancelled
	);
	assert_eq!(
		read_job_record(run, &JobId("j0002".into())).unwrap().state,
		JobState::Done { code: 0 }
	);
}

#[test]
fn background_start_waits_for_startup() {
	let contest = contest(0);
	let run = &contest.run;
	fs::remove_file(run.run_json()).unwrap();
	let made = Command::new("mkfifo").arg(run.run_json()).status().unwrap();
	assert!(made.success());
	let starter = Command::new(BIN)
		.arg("start")
		.arg(&run.root)
		.env("HOME", &contest.home)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.unwrap();
	sleep(Duration::from_millis(500));
	let deadline = Instant::now() + Duration::from_secs(10);
	let mut fifo = loop {
		match fs::OpenOptions::new()
			.write(true)
			.custom_flags(libc::O_NONBLOCK)
			.open(run.run_json())
		{
			Ok(fifo) => break fifo,
			Err(_) if Instant::now() < deadline => sleep(Duration::from_millis(50)),
			Err(error) => panic!("supervisor never read run.json: {error}"),
		}
	};
	std::io::Write::write_all(&mut fifo, b"not json").unwrap();
	drop(fifo);
	let output = starter.wait_with_output().unwrap();
	assert!(!output.status.success());
	assert!(String::from_utf8_lossy(&output.stderr).contains("supervisor.log"));
}

#[test]
fn start_tolerates_momentary_status_lock() {
	let contest = contest(0);
	let run = &contest.run;
	let pid_file = fs::OpenOptions::new()
		.create(true)
		.write(true)
		.truncate(false)
		.open(run.supervisor_pid())
		.unwrap();
	let reader = Flock::lock(pid_file, FlockArg::LockShared).unwrap();
	let mut supervisor = start(&contest, "true");
	sleep(Duration::from_millis(300));
	drop(reader);
	wait_until(10, "status file", || run.status().is_file());
	assert!(supervisor.0.try_wait().unwrap().is_none());
	supervisor.stop(run);
}

#[test]
fn post_spawn_journal_error_keeps_worker() {
	let contest = contest(1);
	let run = &contest.run;
	fs::create_dir_all(run.journal()).unwrap();
	let id = create_task(run, "sim-bench", 0);
	let supervisor = start(&contest, "exec sleep 60");
	wait_until(20, "task running", || state(run, &id) == TaskState::Running);
	sleep(Duration::from_secs(2));
	let launched = state(run, &id);
	supervisor.stop(run);
	assert_eq!(launched, TaskState::Running);
	assert_eq!(event_count(run, "launch_failed"), 0);
}

#[test]
fn usage_refresh_rearms_unknown_report() {
	let contest = contest(1);
	let run = &contest.run;
	edit_config(&contest, |config| config.probe_retry_seconds = 1);
	fs::remove_file(run.usage()).unwrap();
	let id = create_task(run, "sim-bench", 0);
	let supervisor = start_with_probe(&contest, "contest task done", "exit 0");
	wait_until(10, "first usage unknown", || {
		event_count(run, "usage_unknown") == 1
	});
	record_usage(run, &usage_at(0.1), now()).unwrap();
	wait_until(20, "task finished", || {
		has_event(run, json!({"event": "task_finished", "task": id.0}))
	});
	fs::remove_file(run.usage()).unwrap();
	create_task(run, "second", 0);
	wait_until(10, "second usage unknown", || {
		event_count(run, "usage_unknown") == 2
	});
	supervisor.stop(run);
}

fn hold_commit_lock(repo: &Path) -> Flock<fs::File> {
	let file = fs::OpenOptions::new()
		.create(true)
		.write(true)
		.truncate(false)
		.open(repo.join(".git/contest-commit.lock"))
		.unwrap();
	Flock::lock(file, FlockArg::LockExclusive).unwrap()
}

fn commits_with(repo: &Path, subject: &str) -> Vec<String> {
	git(repo, &["log", "--format=%H %s"])
		.lines()
		.filter(|line| line.ends_with(subject))
		.map(|line| line.split(' ').next().unwrap().to_owned())
		.collect()
}

fn worker_script(contest: &Contest, body: &str) -> String {
	let script = contest.home.join("worker.sh");
	fs::write(&script, body).unwrap();
	format!("exec bash {script}", script = script.display())
}

#[test]
fn commit_retried_after_timeout_commits_new_changes() {
	let contest = contest(1);
	let run = &contest.run;
	let repo = pin_repo(&contest);
	let id = create_task(run, "sim-bench", 0);
	let home = contest.home.display();
	let action = worker_script(
		&contest,
		&format!(
			"out=\"$CONTEST_RUN/run/tasks/$CONTEST_TASK/out.txt\"\n\
			echo one > \"$out\"\n\
			contest --timeout 2 commit --milestone m || touch {home}/timed-out\n\
			until [ -e {home}/go ]; do sleep 0.1; done\n\
			echo two > \"$out\"\n\
			contest commit --milestone m > {home}/second\n\
			contest task done\n"
		),
	);
	let lock = hold_commit_lock(&repo);
	let supervisor = start(&contest, &action);
	wait_until(20, "client timed out", || {
		contest.home.join("timed-out").exists()
	});
	drop(lock);
	let subject = format!("sim-bench: {id} m");
	wait_until(10, "first commit landed", || {
		commits_with(&repo, &subject).len() == 1
	});
	sleep(Duration::from_millis(1500));
	fs::write(contest.home.join("go"), "").unwrap();
	wait_until(20, "task done", || state(run, &id) == TaskState::Done);
	supervisor.stop(run);
	let shas = commits_with(&repo, &subject);
	assert_eq!(shas.len(), 2, "{shas:?}");
	let committed = git(
		&repo,
		&["show", &format!("HEAD:contest/run/tasks/{id}/out.txt")],
	);
	assert_eq!(committed.trim(), "two");
	let second = fs::read_to_string(contest.home.join("second")).unwrap();
	assert!(second.contains(&shas[0]), "{second}");
}

#[test]
fn concurrent_commits_get_their_own_sha() {
	let contest = contest(1);
	let run = &contest.run;
	let repo = pin_repo(&contest);
	let id = create_task(run, "sim-bench", 0);
	let home = contest.home.display();
	let action = worker_script(
		&contest,
		&format!(
			"dir=\"$CONTEST_RUN/run/tasks/$CONTEST_TASK\"\n\
			echo a > \"$dir/a.txt\"\n\
			contest commit --milestone a > {home}/sha-a &\n\
			until [ -e {home}/go ]; do sleep 0.1; done\n\
			echo b > \"$dir/b.txt\"\n\
			contest commit --milestone b > {home}/sha-b &\n\
			wait\n\
			contest task done\n"
		),
	);
	let lock = hold_commit_lock(&repo);
	let supervisor = start(&contest, &action);
	wait_until(20, "first request", || {
		fs::read_dir(run.task_requests(&id)).is_ok_and(|mut entries| entries.next().is_some())
	});
	sleep(Duration::from_secs(1));
	drop(lock);
	wait_until(10, "first commit landed", || {
		commits_with(&repo, &format!("sim-bench: {id} a")).len() == 1
	});
	fs::write(contest.home.join("go"), "").unwrap();
	wait_until(20, "task done", || state(run, &id) == TaskState::Done);
	supervisor.stop(run);
	for milestone in ["a", "b"] {
		let shas = commits_with(&repo, &format!("sim-bench: {id} {milestone}"));
		assert_eq!(shas.len(), 1, "{milestone}: {shas:?}");
		let answer = fs::read_to_string(contest.home.join(format!("sha-{milestone}"))).unwrap();
		assert!(answer.contains(&shas[0]), "{milestone}: {answer}");
		assert!(!answer.contains("null"), "{milestone}: {answer}");
	}
}

#[test]
fn file_planted_after_scan_is_refused_at_staging() {
	let contest = contest(1);
	let run = &contest.run;
	let repo = pin_repo(&contest);
	let id = create_task(run, "sim-bench", 0);
	let answer = contest.home.join("answer");
	let action = format!(
		"echo hi > \"$CONTEST_RUN/run/tasks/$CONTEST_TASK/out.txt\"; contest commit --milestone m > {answer} 2>&1; exec sleep 60",
		answer = answer.display()
	);
	let lock = hold_commit_lock(&repo);
	let supervisor = start(&contest, &action);
	wait_until(20, "commit requested", || {
		fs::read_dir(run.task_requests(&id)).is_ok_and(|mut entries| entries.next().is_some())
	});
	sleep(Duration::from_secs(1));
	let planted = run.task_dir(&id).join("sub/.envrc");
	fs::create_dir_all(planted.parent().unwrap()).unwrap();
	fs::write(&planted, "echo planted").unwrap();
	drop(lock);
	wait_until(10, "commit answered", || {
		fs::read_to_string(&answer).is_ok_and(|text| text.contains("planted"))
	});
	supervisor.stop(run);
	assert_eq!(git(&repo, &["log", "--format=%s"]).trim(), "init");
	assert_eq!(git(&repo, &["diff", "--cached", "--name-only"]), "");
}

fn open_fifo_writer(path: &Path) -> fs::File {
	let deadline = Instant::now() + Duration::from_secs(10);
	loop {
		match fs::OpenOptions::new()
			.write(true)
			.custom_flags(libc::O_NONBLOCK)
			.open(path)
		{
			Ok(fifo) => return fifo,
			Err(_) if Instant::now() < deadline => sleep(Duration::from_millis(50)),
			Err(error) => panic!("nobody read {path:?}: {error}"),
		}
	}
}

#[test]
fn background_start_reports_started_only_after_announce() {
	let contest = contest(0);
	let run = &contest.run;
	let info = fs::read(run.run_json()).unwrap();
	fs::remove_file(run.run_json()).unwrap();
	let made = Command::new("mkfifo").arg(run.run_json()).status().unwrap();
	assert!(made.success());
	let mut starter = Command::new(BIN)
		.arg("start")
		.arg(&run.root)
		.env("HOME", &contest.home)
		.env("FAKE_CLAUDE_SCRIPT", &contest.manager_stream)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.unwrap();
	let mut fifo = open_fifo_writer(&run.run_json());
	sleep(Duration::from_secs(1));
	assert!(
		starter.try_wait().unwrap().is_none(),
		"start returned before startup finished"
	);
	assert_eq!(fs::read_to_string(run.supervisor_pid()).unwrap(), "");
	std::io::Write::write_all(&mut fifo, &info).unwrap();
	drop(fifo);
	write_atomic(&run.run_json(), &info).unwrap();
	let output = starter.wait_with_output().unwrap();
	let stdout = String::from_utf8_lossy(&output.stdout);
	assert!(output.status.success(), "{stdout}");
	let pid = fs::read_to_string(run.supervisor_pid()).unwrap();
	assert!(!pid.is_empty());
	assert!(stdout.contains(&format!("started: pid {pid}")), "{stdout}");
	fs::write(run.stop_file(), "test").unwrap();
	wait_until(10, "supervisor stopped", || {
		contest_status(run).contains("supervisor: stopped")
	});
}

fn rate_limit_line(status: &str, five_hour: f64) -> Value {
	let reset = now() + 3600;
	json!({
		"type": "rate_limit_event",
		"rate_limit_info": {
			"status": status,
			"resetsAt": reset,
			"rateLimitType": "five_hour",
			"unifiedWindows": {
				"five_hour": {"utilization": five_hour, "resetsAt": reset},
				"seven_day": {"utilization": 0.2, "resetsAt": now() + WEEK_SECONDS / 2},
			},
		},
	})
}

fn limit_stream(path: &Path, status: &str, five_hour: f64) {
	let result = json!({
		"type": "result",
		"subtype": "success",
		"is_error": true,
		"result": "usage limit reached",
		"total_cost_usd": 0.0,
		"usage": {"input_tokens": 0, "output_tokens": 0},
	});
	let event = rate_limit_line(status, five_hour);
	fs::write(path, format!("{event}\n{result}\n")).unwrap();
}

#[test]
fn usage_limit_hit_requeues_and_blocks_launches() {
	let contest = contest(1);
	let run = &contest.run;
	let stream = contest.home.join("limit-stream.jsonl");
	limit_stream(&stream, "rejected", 1.0);
	let id = create_task(run, "sim-bench", 7);
	let action = format!("FAKE_CLAUDE_SCRIPT={stream}", stream = stream.display());
	let supervisor = start(&contest, &action);
	wait_until(20, "usage limit journaled", || {
		journal_lines(run, "hit the usage limit") >= 1
	});
	sleep(Duration::from_secs(3));
	let status = contest_status(run);
	supervisor.stop(run);
	assert_eq!(state(run, &id), TaskState::Queued);
	assert_eq!(TaskStore { run }.spec(&id).unwrap().priority, 7);
	assert_eq!(journal_lines(run, &format!("launched {id}")), 1);
	assert_eq!(event_count(run, "launch_failed"), 0);
	assert_eq!(event_count(run, "task_finished"), 0);
	assert!(event_count(run, "usage_blocked") >= 1);
	assert!(status.contains("blocked: usage"), "{status}");
}

fn hook_blocks_bash(run: &RunDir, task: &TaskId) -> bool {
	let mut child = Command::new(BIN)
		.args(["hook", "pre-tool-use"])
		.env("CONTEST_RUN", &run.root)
		.env("CONTEST_TASK", &task.0)
		.stdin(Stdio::piped())
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.spawn()
		.unwrap();
	let payload = json!({"tool_name": "Bash", "tool_input": {"command": "ls"}});
	child
		.stdin
		.take()
		.unwrap()
		.write_all(payload.to_string().as_bytes())
		.unwrap();
	child.wait().unwrap().code() == Some(2)
}

#[test]
fn utilization_at_cap_writes_stop_marker_and_hook_blocks() {
	let contest = contest(1);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let marker = run.task_meta(&id).join("STOP");
	let go = contest.home.join("go");
	let flag = contest.home.join("stopped-once");
	let action = format!(
		"if [ -e {flag} ]; then exec sleep 60; fi; echo '{event}'; while [ ! -e {go} ]; do sleep 0.1; done; touch {flag}",
		flag = flag.display(),
		event = rate_limit_line("allowed_warning", 0.98),
		go = go.display()
	);
	let supervisor = start(&contest, &action);
	wait_until(20, "stop marker", || marker.is_file());
	assert!(hook_blocks_bash(run, &id));
	fs::write(&go, "").unwrap();
	wait_until(20, "requeued near the cap", || {
		journal_lines(run, &format!("requeued {id}")) >= 1
	});
	wait_until(20, "relaunched", || flag.is_file() && !marker.exists());
	supervisor.stop(run);
	assert_eq!(event_count(run, "task_finished"), 0);
	assert_eq!(event_count(run, "launch_failed"), 0);
}

#[test]
fn fast_exit_is_a_launch_failure() {
	let contest = contest(3);
	let run = &contest.run;
	let tasks: Vec<TaskId> = ["a", "b", "c"]
		.into_iter()
		.map(|kind| create_task(run, kind, 0))
		.collect();
	let supervisor = start(
		&contest,
		"echo claude is broken >&2; FAKE_CLAUDE_SCRIPT=/dev/null",
	);
	wait_until(20, "launch failures", || {
		event_count(run, "launch_failed") >= 3
	});
	sleep(Duration::from_secs(3));
	supervisor.stop(run);
	for id in &tasks {
		assert_eq!(state(run, id), TaskState::Queued);
	}
	assert_eq!(journal_lines(run, "launched "), 3);
	assert_eq!(event_count(run, "task_finished"), 0);
	assert!(
		events(run)
			.iter()
			.filter(|event| event["event"] == "launch_failed")
			.all(|event| event["error"]
				.as_str()
				.unwrap()
				.contains("claude is broken"))
	);
}

#[test]
fn task_failing_launch_three_times_fails_once_another_task_ran() {
	let contest = contest(1);
	let run = &contest.run;
	edit_config(&contest, |config| config.retry_base_seconds = 1);
	let bad = create_task(run, "bad", 10);
	let good = create_task(run, "good", 0);
	let action = format!(
		"if [[ $CONTEST_TASK == {bad} ]]; then FAKE_CLAUDE_SCRIPT=/dev/null; else contest task done; fi"
	);
	let supervisor = start(&contest, &action);
	wait_until(30, "bad task failed", || {
		state(run, &bad) == TaskState::Failed
	});
	supervisor.stop(run);
	assert_eq!(state(run, &good), TaskState::Done);
	assert_eq!(event_count(run, "launch_failed"), 4);
	assert!(has_event(
		run,
		json!({"event": "task_finished", "task": bad.0, "state": "failed"})
	));
	let notes = fs::read_to_string(run.task_dir(&bad).join("notes.md")).unwrap();
	assert!(notes.contains("launch failed 3 times"), "{notes}");
}

#[test]
fn repeated_spawn_errors_fail_a_task_only_after_another_ran() {
	let contest = contest(1);
	let run = &contest.run;
	edit_config(&contest, |config| config.retry_base_seconds = 1);
	let bad = create_task(run, "bad", 10);
	let good = create_task(run, "good", 0);
	fs::create_dir_all(run.task_stop_marker(&bad).join("blocker")).unwrap();
	let supervisor = start(&contest, "contest task done");
	wait_until(30, "bad task failed", || {
		state(run, &bad) == TaskState::Failed
	});
	supervisor.stop(run);
	assert_eq!(state(run, &good), TaskState::Done);
	let notes = fs::read_to_string(run.task_dir(&bad).join("notes.md")).unwrap();
	assert!(notes.contains("launch failed 3 times"), "{notes}");
}

#[test]
fn systemic_spawn_errors_fail_no_task() {
	let contest = contest(1);
	let run = &contest.run;
	let plugin = plugin_without_worker_prompt(&contest);
	edit_config(&contest, |config| {
		config.plugin_dir = plugin;
		config.retry_base_seconds = 1;
	});
	let tasks: Vec<TaskId> = ["a", "b"]
		.into_iter()
		.map(|kind| create_task(run, kind, 0))
		.collect();
	let supervisor = start(&contest, "contest task done");
	wait_until(30, "repeated launch failures", || {
		event_count(run, "launch_failed") >= 4
	});
	supervisor.stop(run);
	for id in &tasks {
		assert_eq!(state(run, id), TaskState::Queued);
	}
	assert_eq!(event_count(run, "task_finished"), 0);
}

fn pid_in(path: &Path) -> i32 {
	fs::read_to_string(path).unwrap().trim().parse().unwrap()
}

#[test]
fn idle_worker_is_killed_and_requeued() {
	let contest = contest(1);
	let run = &contest.run;
	edit_config(&contest, |config| config.worker_idle_minutes = 1);
	let id = create_task(run, "sim-bench", 0);
	let stream = run.agents().join(&id.0).join("stream.jsonl");
	let flag = contest.home.join("hung-once");
	let pid_file = contest.home.join("hung-pid");
	let action = format!(
		"if [ -e {flag} ]; then contest task done; else touch {flag}; echo $$ > {pid}; touch -d '2 hours ago' {stream}; exec sleep 60; fi",
		flag = flag.display(),
		pid = pid_file.display(),
		stream = stream.display()
	);
	let supervisor = start(&contest, &action);
	wait_until(20, "idle kill", || {
		journal_lines(run, "no stream output") >= 1
	});
	wait_until(20, "done after relaunch", || {
		state(run, &id) == TaskState::Done
	});
	supervisor.stop(run);
	let pid = pid_in(&pid_file);
	assert!(!Path::new(&format!("/proc/{pid}")).exists());
	assert_eq!(journal_lines(run, &format!("requeued {id}")), 1);
}

#[test]
fn worker_alive_after_result_is_killed_and_failed() {
	let contest = contest(1);
	let run = &contest.run;
	edit_config(&contest, |config| config.result_grace_seconds = 1);
	let id = create_task(run, "sim-bench", 0);
	let pid_file = contest.home.join("lingering-pid");
	let action = format!(
		"echo $$ > {pid}; cat \"$FAKE_CLAUDE_SCRIPT\"; exec sleep 60",
		pid = pid_file.display()
	);
	let supervisor = start(&contest, &action);
	wait_until(20, "failed after the result grace kill", || {
		state(run, &id) == TaskState::Failed
	});
	supervisor.stop(run);
	let pid = pid_in(&pid_file);
	assert!(!Path::new(&format!("/proc/{pid}")).exists());
	assert!(journal_lines(run, "after its result") >= 1);
	assert_eq!(journal_lines(run, &format!("launched {id}")), 1);
	assert_eq!(journal_lines(run, &format!("requeued {id}")), 0);
	let notes = fs::read_to_string(run.task_dir(&id).join("notes.md")).unwrap();
	assert!(notes.contains("exited without outcome"), "{notes}");
}

#[test]
fn running_task_without_worker_is_requeued() {
	let contest = contest(0);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let supervisor = start(&contest, "true");
	wait_until(10, "status file", || run.status().is_file());
	TaskStore { run }
		.transition(&id, TaskState::Running)
		.unwrap();
	wait_until(10, "requeued", || state(run, &id) == TaskState::Queued);
	supervisor.stop(run);
	assert_eq!(journal_lines(run, &format!("requeued {id}")), 1);
}

#[test]
fn manager_failures_are_journaled_on_change_and_shown() {
	let contest = contest(0);
	let run = &contest.run;
	edit_config(&contest, |config| config.retry_base_seconds = 1);
	result_stream(&contest.manager_stream, "no json here");
	let supervisor = start(&contest, "true");
	wait_until(20, "manager failure journaled", || {
		journal_lines(run, "manager failed") >= 1
	});
	let status = contest_status(run);
	assert!(status.contains("blocked: manager"), "{status}");
	wait_until(20, "repeated manager errors", || {
		event_count(run, "manager_error") >= 3
	});
	assert_eq!(journal_lines(run, "manager failed"), 1);
	let actions = json!([{"action": "cancel", "task": "t0099-missing"}]);
	result_stream(
		&contest.manager_stream,
		&format!("plan\n```json\n{actions}\n```"),
	);
	wait_until(20, "changed failure journaled", || {
		journal_lines(run, "manager failed") >= 2
	});
	supervisor.stop(run);
	let journal = fs::read_to_string(run.journal()).unwrap();
	let first = journal
		.lines()
		.find(|line| line.contains("manager failed"))
		.unwrap();
	assert!(first.contains("no fenced json block"), "{first}");
	assert!(first.contains("retry in 1 s"), "{first}");
}

#[test]
fn usage_block_is_journaled_and_shown() {
	let contest = contest(1);
	let run = &contest.run;
	let until = now() + 3600;
	let blocked = RateLimitInfo {
		status: "rejected".into(),
		resets_at: until,
		..usage_at(0.5)
	};
	record_usage(run, &blocked, now()).unwrap();
	create_task(run, "sim-bench", 0);
	let supervisor = start(&contest, "contest task done");
	wait_until(20, "usage block journaled", || {
		journal_lines(run, "launches blocked") >= 1
	});
	let status = contest_status(run);
	supervisor.stop(run);
	assert!(status.contains("blocked: usage"), "{status}");
	assert!(journal_lines(run, &contest::journal::utc_time(until)) >= 1);
}

#[test]
fn taint_over_budget_pauses_without_failing() {
	let contest = contest(1);
	let run = &contest.run;
	edit_config(&contest, |config| {
		config.taint_scan_max_entries = 200;
		config.retry_base_seconds = 1;
	});
	let first = create_task(run, "first", 1);
	let second = create_task(run, "second", 0);
	let bulk = run.task_dir(&first).join("bulk");
	let action = format!(
		"if [[ $CONTEST_TASK == {first} ]]; then mkdir -p {bulk}; for i in $(seq 300); do : > {bulk}/f$i; done; fi; contest task done",
		bulk = bulk.display()
	);
	let supervisor = start(&contest, &action);
	wait_until(20, "over budget event", || {
		event_count(run, "taint_over_budget") >= 1
	});
	sleep(Duration::from_secs(2));
	let status = contest_status(run);
	assert_eq!(state(run, &second), TaskState::Queued);
	fs::remove_dir_all(&bulk).unwrap();
	wait_until(20, "second runs after cleanup", || {
		state(run, &second) == TaskState::Done
	});
	supervisor.stop(run);
	assert_eq!(state(run, &first), TaskState::Done);
	assert!(status.contains("blocked: taint"), "{status}");
	assert_eq!(journal_lines(run, "taint scan failed"), 0);
	assert!(journal_lines(run, "over its entry budget") >= 1);
}

fn wait_for_exit(supervisor: &mut Supervisor, seconds: u64) {
	let deadline = Instant::now() + Duration::from_secs(seconds);
	while supervisor.0.try_wait().unwrap().is_none() {
		assert!(Instant::now() < deadline, "supervisor kept running");
		sleep(Duration::from_millis(50));
	}
}

#[test]
fn broken_run_json_neither_hides_the_budget_nor_floods_the_log() {
	let contest = contest(0);
	let run = &contest.run;
	let mut info: RunInfo = serde_json::from_slice(&fs::read(run.run_json()).unwrap()).unwrap();
	info.active_budget_seconds = Some(34);
	write_atomic(&run.run_json(), &serde_json::to_vec(&info).unwrap()).unwrap();
	let mut supervisor = start(&contest, "true");
	wait_until(10, "status file", || run.status().is_file());
	fs::write(run.run_json(), "{ broken").unwrap();
	wait_for_exit(&mut supervisor, 60);
	assert_eq!(fs::read_to_string(run.stop_file()).unwrap(), "budget");
	assert_eq!(journal_lines(run, "run.json"), 1);
	let log = fs::read_to_string(contest.home.join("supervisor.log")).unwrap();
	assert!(log.lines().count() <= 2, "{log}");
	let repaired: RunInfo = serde_json::from_slice(&fs::read(run.run_json()).unwrap()).unwrap();
	assert!(repaired.active_seconds >= 30);
}

#[test]
fn extend_reaches_a_running_supervisor() {
	let contest = contest(0);
	let run = &contest.run;
	let mut supervisor = start(&contest, "true");
	wait_until(10, "status file", || run.status().is_file());
	let output = Command::new(BIN)
		.arg("extend")
		.arg(&run.root)
		.args(["--deadline", "2020-01-01T00:00:00Z"])
		.output()
		.unwrap();
	assert!(output.status.success(), "{output:?}");
	wait_for_exit(&mut supervisor, 20);
	assert_eq!(fs::read_to_string(run.stop_file()).unwrap(), "budget");
}

#[test]
fn restart_marks_queued_jobs_interrupted() {
	let contest = contest(0);
	let run = &contest.run;
	let owner = create_task(run, "sim-bench", 0);
	TaskStore { run }
		.transition(&owner, TaskState::Running)
		.unwrap();
	let marker = contest.home.join("orphan-ran");
	let job = queued_job_record(run, "j0001", &owner, &["touch", marker.to_str().unwrap()]);
	let supervisor = start(&contest, "exec sleep 60");
	wait_until(10, "job interrupted event", || {
		has_event(run, json!({"event": "job_interrupted", "job": "j0001"}))
	});
	sleep(Duration::from_secs(2));
	supervisor.stop(run);
	assert_eq!(
		read_job_record(run, &job).unwrap().state,
		JobState::Interrupted
	);
	assert!(!marker.exists());
}

fn path_with_first(dir: &Path) -> std::ffi::OsString {
	let mut entries = vec![dir.to_path_buf()];
	entries.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
	std::env::join_paths(entries).unwrap()
}

fn assert_start_refused(contest: &Contest, path: std::ffi::OsString, needle: &str) {
	let output = start_output_with_path(contest, path);
	let stderr = String::from_utf8_lossy(&output.stderr);
	assert!(!output.status.success(), "{needle}: {stderr}");
	assert!(stderr.contains(needle), "{needle}: {stderr}");
}

#[test]
fn start_refuses_host_lookups_through_the_contest_root() {
	let contest = contest(0);
	let run = &contest.run;
	let planted_bin = run.root.join("bin");
	fs::create_dir_all(&planted_bin).unwrap();
	let planted_git = planted_bin.join("git");
	fs::write(&planted_git, "#!/bin/sh\n").unwrap();
	fs::set_permissions(&planted_git, fs::Permissions::from_mode(0o755)).unwrap();
	assert_start_refused(&contest, path_with_first(&planted_bin), "PATH entry");
	assert_start_refused(&contest, path_with_first(Path::new("bin")), "relative");
	let outside = contest.home.join("tools");
	fs::create_dir_all(&outside).unwrap();
	std::os::unix::fs::symlink(&planted_git, outside.join("git")).unwrap();
	assert_start_refused(&contest, path_with_first(&outside), "git");
	fs::remove_file(outside.join("git")).unwrap();
	let original = Config::load(&run.config()).unwrap();
	edit_config(&contest, |config| config.claude_bin = planted_git.clone());
	assert_start_refused(&contest, std::env::var_os("PATH").unwrap(), "claude_bin");
	edit_config(&contest, |config| config.claude_bin = "bin/git".into());
	assert_start_refused(&contest, std::env::var_os("PATH").unwrap(), "claude_bin");
	write_atomic(
		&run.config(),
		toml::to_string(&original).unwrap().as_bytes(),
	)
	.unwrap();
	assert!(!run.status().exists());
}

#[test]
fn usage_sample_over_cap_soft_stops_then_kills_at_the_deadline() {
	let contest = contest(1);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let pid_file = contest.home.join("stubborn-pid");
	let action = format!("echo $$ > {pid}; exec sleep 60", pid = pid_file.display());
	let supervisor = start(&contest, &action);
	wait_until(20, "worker running", || pid_file.is_file());
	record_usage(run, &usage_at(0.98), now()).unwrap();
	wait_until(10, "stop marker", || run.task_stop_marker(&id).is_file());
	wait_until(20, "requeued at the deadline", || {
		journal_lines(run, &format!("requeued {id}")) >= 1
	});
	supervisor.stop(run);
	let pid = pid_in(&pid_file);
	assert!(!Path::new(&format!("/proc/{pid}")).exists());
	assert!(journal_lines(run, "after the soft stop") >= 1);
	assert_eq!(state(run, &id), TaskState::Queued);
	assert_eq!(event_count(run, "launch_failed"), 0);
}

#[test]
fn idle_requeues_are_capped_per_task() {
	let contest = contest(1);
	let run = &contest.run;
	edit_config(&contest, |config| config.worker_idle_minutes = 1);
	let id = create_task(run, "sim-bench", 0);
	let stream = run.agents().join(&id.0).join("stream.jsonl");
	let action = format!(
		"touch -d '2 hours ago' {stream}; exec sleep 60",
		stream = stream.display()
	);
	let supervisor = start(&contest, &action);
	wait_until(30, "failed after capped requeues", || {
		state(run, &id) == TaskState::Failed
	});
	supervisor.stop(run);
	assert_eq!(journal_lines(run, &format!("requeued {id}")), 3);
	let notes = fs::read_to_string(run.task_dir(&id).join("notes.md")).unwrap();
	assert!(notes.contains("requeued 3 times"), "{notes}");
}

#[test]
fn limit_hit_without_future_reset_backs_off() {
	let contest = contest(1);
	let run = &contest.run;
	let stream = contest.home.join("stale-limit-stream.jsonl");
	let past = now() - 60;
	let event = json!({
		"type": "rate_limit_event",
		"rate_limit_info": {
			"status": "rejected",
			"resetsAt": past,
			"unifiedWindows": {
				"five_hour": {"utilization": 1.0, "resetsAt": past},
				"seven_day": {"utilization": 0.2, "resetsAt": now() + WEEK_SECONDS / 2},
			},
		},
	});
	let result = json!({
		"type": "result",
		"subtype": "success",
		"is_error": true,
		"result": "usage limit reached",
		"usage": {"input_tokens": 0, "output_tokens": 0},
	});
	fs::write(&stream, format!("{event}\n{result}\n")).unwrap();
	let id = create_task(run, "sim-bench", 0);
	let action = format!("FAKE_CLAUDE_SCRIPT={stream}", stream = stream.display());
	let supervisor = start(&contest, &action);
	wait_until(20, "launch failure", || {
		event_count(run, "launch_failed") >= 1
	});
	sleep(Duration::from_secs(2));
	supervisor.stop(run);
	assert_eq!(event_count(run, "launch_failed"), 1);
	assert_eq!(journal_lines(run, &format!("launched {id}")), 1);
	assert!(events(run).iter().any(|event| {
		event["error"]
			.as_str()
			.is_some_and(|error| error.contains("no future reset"))
	}));
	assert_eq!(state(run, &id), TaskState::Queued);
}

#[test]
fn manager_ticks_while_workers_run_near_the_cap() {
	let contest = contest(3);
	let run = &contest.run;
	record_usage(run, &usage_at(0.88), now()).unwrap();
	for kind in ["a", "b", "c"] {
		create_task(run, kind, 0);
	}
	let supervisor = start(&contest, "exec sleep 60");
	wait_until(20, "three workers", || running_count(run) == 3);
	wait_until(20, "first tick", || manager_ticks(run) >= 1);
	assert!(contest_say(run, "status please").status.success());
	wait_until(20, "message acknowledged", || {
		fs::read_dir(run.inbox().join("read")).is_ok_and(|mut entries| entries.next().is_some())
	});
	supervisor.stop(run);
}

#[test]
fn stop_sweep_scans_once_for_all_workers() {
	let contest = contest(2);
	let run = &contest.run;
	let mut deep = run.root.join("deep");
	for _ in 0..70 {
		deep.push("d");
	}
	fs::create_dir_all(&deep).unwrap();
	for kind in ["a", "b"] {
		create_task(run, kind, 0);
	}
	let supervisor = start(&contest, "exec sleep 60");
	wait_until(20, "two workers", || running_count(run) == 2);
	assert_eq!(journal_lines(run, "deeper than"), 1);
	supervisor.stop(run);
	assert_eq!(journal_lines(run, "deeper than"), 2);
}

fn launch_order(run: &RunDir) -> Vec<String> {
	fs::read_to_string(run.journal())
		.unwrap_or_default()
		.lines()
		.filter_map(|line| {
			line.split_once("launched ")
				.map(|(_, rest)| rest.to_owned())
		})
		.filter_map(|rest| rest.split_whitespace().next().map(str::to_owned))
		.collect()
}

#[test]
fn launch_failure_records_reset_once_another_task_ran() {
	let contest = contest(1);
	let run = &contest.run;
	edit_config(&contest, |config| config.retry_base_seconds = 1);
	let blocked = create_task(run, "blocked", 0);
	let blocker = run.task_stop_marker(&blocked).join("blocker");
	fs::create_dir_all(&blocker).unwrap();
	let supervisor = start(&contest, "contest task done");
	wait_until(30, "three failed launches", || {
		event_count(run, "launch_failed") >= 3
	});
	let good = create_task(run, "good", 5);
	wait_until(30, "good task done", || {
		state(run, &good) == TaskState::Done
	});
	let failures_before_blip = event_count(run, "launch_failed");
	wait_until(30, "one more failed launch", || {
		event_count(run, "launch_failed") > failures_before_blip
	});
	fs::remove_dir_all(run.task_stop_marker(&blocked)).unwrap();
	wait_until(30, "blocked task done", || {
		state(run, &blocked) == TaskState::Done
	});
	supervisor.stop(run);
	assert!(!has_event(
		run,
		json!({"event": "task_finished", "task": blocked.0, "state": "failed"})
	));
}

#[test]
fn task_sorts_last_only_until_another_task_ran() {
	let contest = contest(1);
	let run = &contest.run;
	edit_config(&contest, |config| config.retry_base_seconds = 1);
	let failed_once = create_task(run, "failed-once", 0);
	let blocker = run.task_stop_marker(&failed_once).join("blocker");
	fs::create_dir_all(&blocker).unwrap();
	let supervisor = start(&contest, "contest task done");
	wait_until(30, "one failed launch", || {
		event_count(run, "launch_failed") >= 1
	});
	fs::remove_dir_all(run.task_stop_marker(&failed_once)).unwrap();
	let first = create_task(run, "first", 5);
	let later = create_task(run, "later", 0);
	wait_until(30, "all tasks done", || {
		[&failed_once, &first, &later]
			.iter()
			.all(|id| state(run, id) == TaskState::Done)
	});
	supervisor.stop(run);
	assert_eq!(launch_order(run), [first.0, failed_once.0, later.0]);
}

fn allowed_cpus(status: &str) -> Vec<usize> {
	status
		.lines()
		.find_map(|line| line.strip_prefix("Cpus_allowed_list:"))
		.map(parse_cpu_list)
		.unwrap_or_default()
}

fn recorded_cpus(path: &Path) -> Vec<usize> {
	parse_cpu_list(&fs::read_to_string(path).unwrap_or_default())
}

fn process_threads_cpus(pid: u32) -> Vec<Vec<usize>> {
	fs::read_dir(format!("/proc/{pid}/task"))
		.unwrap()
		.flatten()
		.filter_map(|thread| fs::read_to_string(thread.path().join("status")).ok())
		.map(|status| allowed_cpus(&status))
		.collect()
}

fn record_cpus_loop(out: &Path, until: &Path) -> String {
	format!(
		"while [ ! -e {until} ]; do grep Cpus_allowed_list /proc/self/status | cut -f2 > {out}.tmp && mv {out}.tmp {out}; sleep 0.1; done",
		until = until.display(),
		out = out.display()
	)
}

fn all_cpus() -> Vec<usize> {
	Machine::detect(13000, 1).all_cpus()
}

fn lease_cpus(run: &RunDir, job: &str) -> Vec<usize> {
	read_lease(run, &JobId(job.into()))
		.map(|lease| lease.cpus)
		.unwrap_or_default()
}

#[test]
fn agents_and_the_supervisor_never_run_on_timed_cores() {
	let contest = contest(1);
	let run = &contest.run;
	if Machine::detect(13000, 1).timed_cores() == 0 {
		eprintln!("machine has a single core, skipped");
		return;
	}
	let id = create_task(run, "sim-bench", 0);
	let worker_cpus = contest.home.join("worker-cpus");
	let worker_nice = contest.home.join("worker-nice");
	let release = contest.home.join("release");
	let finish = contest.home.join("finish");
	let action = worker_script(
		&contest,
		&format!(
			"cut -d' ' -f19 /proc/self/stat > {nice}\n\
			contest job submit --class timed --ram-mb 10 -- sleep 60\n\
			( {record} ) &\n\
			until [ -e {release} ]; do sleep 0.1; done\n\
			contest job cancel j0001\n\
			until [ -e {finish} ]; do sleep 0.1; done\n\
			wait\n\
			contest task done\n",
			nice = worker_nice.display(),
			record = record_cpus_loop(&worker_cpus, &finish),
			release = release.display(),
			finish = finish.display()
		),
	);
	let supervisor = start(&contest, &action);
	let job = JobId("j0001".into());
	wait_until(20, "timed job running", || {
		read_job_record(run, &job).is_ok_and(|record| record.state == JobState::Running)
	});
	let timed = lease_cpus(run, "j0001");
	let untimed: Vec<usize> = all_cpus()
		.into_iter()
		.filter(|cpu| !timed.contains(cpu))
		.collect();
	assert!(!timed.is_empty() && !untimed.is_empty(), "{timed:?}");
	wait_until(10, "worker pinned off the timed cores", || {
		recorded_cpus(&worker_cpus) == untimed
	});
	let supervisor_threads = process_threads_cpus(supervisor.0.id());
	assert!(
		supervisor_threads.iter().all(|cpus| *cpus == untimed),
		"{supervisor_threads:?} vs {untimed:?}"
	);
	assert_eq!(fs::read_to_string(&worker_nice).unwrap().trim(), "10");
	fs::write(&release, "").unwrap();
	wait_until(20, "worker repinned to every cpu", || {
		recorded_cpus(&worker_cpus) == all_cpus()
	});
	fs::write(&finish, "").unwrap();
	wait_until(20, "task done", || state(run, &id) == TaskState::Done);
	supervisor.stop(run);
}

#[test]
fn running_timed_job_keeps_its_cores_until_the_drain_ends() {
	let contest = contest(1);
	let run = &contest.run;
	if Machine::detect(13000, 1).timed_cores() < 2 {
		eprintln!("machine has fewer than 3 cores, skipped");
		return;
	}
	let drain_seconds = 4;
	edit_config(&contest, |config| config.drain_seconds = drain_seconds);
	create_task(run, "sim-bench", 0);
	let first_cpus = contest.home.join("first-cpus");
	let second_go = contest.home.join("second-go");
	let finish = contest.home.join("finish");
	let first_job = contest.home.join("first-job.sh");
	fs::write(&first_job, record_cpus_loop(&first_cpus, &finish)).unwrap();
	let action = worker_script(
		&contest,
		&format!(
			"contest job submit --class timed --ram-mb 10 -- bash {first_job}\n\
			until [ -e {second_go} ]; do sleep 0.1; done\n\
			contest job submit --class timed --ram-mb 10 -- sleep 60\n\
			exec sleep 120\n",
			first_job = first_job.display(),
			second_go = second_go.display()
		),
	);
	let supervisor = start(&contest, &action);
	let first = JobId("j0001".into());
	let second = JobId("j0002".into());
	wait_until(20, "first job pinned", || {
		let cpus = lease_cpus(run, "j0001");
		!cpus.is_empty() && recorded_cpus(&first_cpus) == cpus
	});
	let before = read_lease(run, &first).unwrap();
	assert!(before.pending_cpus.is_none());
	fs::write(&second_go, "").unwrap();
	wait_until(20, "drain announced", || {
		read_lease(run, &first).is_ok_and(|lease| lease.pending_cpus.is_some())
	});
	let draining = read_lease(run, &first).unwrap();
	assert!(draining.generation > before.generation);
	assert_eq!(draining.cpus, before.cpus);
	let pending = draining.pending_cpus.clone().unwrap();
	assert!(pending.len() < before.cpus.len(), "{pending:?}");
	let effective_at = draining.effective_at.unwrap();
	assert!(
		effective_at >= now() + drain_seconds as i64 - 2,
		"{effective_at}"
	);
	while now() < effective_at - 1 {
		assert_eq!(recorded_cpus(&first_cpus), before.cpus);
		assert_eq!(
			read_job_record(run, &second).unwrap().state,
			JobState::Queued
		);
		sleep(Duration::from_millis(200));
	}
	wait_until(20, "second job running", || {
		read_job_record(run, &second).is_ok_and(|record| record.state == JobState::Running)
	});
	assert!(now() >= effective_at);
	let drained = read_lease(run, &first).unwrap();
	assert!(drained.generation > draining.generation);
	assert_eq!(drained.cpus, pending);
	assert!(drained.pending_cpus.is_none());
	wait_until(10, "first job repinned", || {
		recorded_cpus(&first_cpus) == pending
	});
	let second_cpus = lease_cpus(run, "j0002");
	assert!(
		second_cpus.iter().all(|cpu| !pending.contains(cpu)),
		"{second_cpus:?} vs {pending:?}"
	);
	fs::write(&finish, "").unwrap();
	supervisor.stop(run);
}

#[test]
fn waiting_task_frees_its_slot_and_resumes_when_its_jobs_end() {
	let contest = contest(1);
	let run = &contest.run;
	let long = create_task(run, "train", 10);
	let quick = create_task(run, "quick", 0);
	let job_go = contest.home.join("job-go");
	let notes = run.task_dir(&long).join("notes.md");
	let action = format!(
		"if [[ $CONTEST_TASK == {quick} ]]; then contest task done; \
		elif grep -q 'waiting for j0001' {notes} 2>/dev/null; then contest task done; \
		else contest job submit --class batch --ram-mb 10 -- sh -c 'until [ -e {job_go} ]; do sleep 0.1; done' \
		&& echo 'waiting for j0001' >> {notes} && contest task wait-jobs; fi",
		notes = notes.display(),
		job_go = job_go.display()
	);
	let supervisor = start(&contest, &action);
	let job = JobId("j0001".into());
	wait_until(20, "long task waiting", || {
		state(run, &long) == TaskState::Waiting
	});
	wait_until(20, "quick task ran in the freed slot", || {
		state(run, &quick) == TaskState::Done
	});
	assert_eq!(state(run, &long), TaskState::Waiting);
	assert_eq!(read_job_record(run, &job).unwrap().state, JobState::Running);
	let status = contest_status(run);
	let waiting_section = status.split("## waiting tasks").nth(1).unwrap_or_default();
	assert!(waiting_section.contains(&long.0), "{status}");
	fs::write(&job_go, "").unwrap();
	wait_until(30, "long task done", || {
		state(run, &long) == TaskState::Done
	});
	supervisor.stop(run);
	assert_eq!(
		read_job_record(run, &job).unwrap().state,
		JobState::Done { code: 0 }
	);
	assert_eq!(journal_lines(run, &format!("{long} waits for its jobs")), 1);
	assert_eq!(
		journal_lines(run, &format!("resumed {long}: its jobs ended")),
		1
	);
	assert!(
		fs::read_to_string(&notes)
			.unwrap()
			.contains("waiting for j0001")
	);
	assert!(!has_event(
		run,
		json!({"event": "task_finished", "task": long.0, "state": "waiting"})
	));
	assert!(!has_event(
		run,
		json!({"event": "task_finished", "task": long.0, "state": "failed"})
	));
}

#[test]
fn waiting_task_resumes_after_restart() {
	let contest = contest(1);
	let run = &contest.run;
	let id = create_task(run, "train", 0);
	let store = TaskStore { run };
	store.transition(&id, TaskState::Running).unwrap();
	store.transition(&id, TaskState::Waiting).unwrap();
	let mut gone = Command::new("true").spawn().unwrap();
	let pid = gone.id();
	gone.wait().unwrap();
	let job = queued_job_record(run, "j0001", &id, &["true"]);
	let mut record = read_job_record(run, &job).unwrap();
	record.state = JobState::Running;
	record.pid = Some(pid);
	record.starttime = Some(1);
	write_job_record(run, &record).unwrap();
	let supervisor = start(&contest, "contest task done");
	wait_until(20, "task done", || state(run, &id) == TaskState::Done);
	supervisor.stop(run);
	assert_eq!(
		read_job_record(run, &job).unwrap().state,
		JobState::Interrupted
	);
	assert_eq!(
		journal_lines(run, &format!("requeued {id} after restart")),
		0
	);
	assert_eq!(
		journal_lines(run, &format!("resumed {id}: its jobs ended")),
		1
	);
	assert_eq!(store.spec(&id).unwrap().priority, 0);
}

#[test]
fn job_events_of_running_tasks_do_not_wake_the_manager() {
	let contest = contest(1);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let finish = contest.home.join("finish");
	let action = worker_script(
		&contest,
		&format!(
			"for job in j0001 j0002 j0003; do contest job submit --class batch --ram-mb 10 -- true; contest job wait $job; done\n\
			until [ -e {finish} ]; do sleep 0.1; done\n\
			contest task done\n",
			finish = finish.display()
		),
	);
	let supervisor = start(&contest, &action);
	wait_until(20, "first tick", || manager_ticks(run) >= 1);
	wait_until(30, "three job events", || {
		event_count(run, "job_finished") == 3
	});
	sleep(Duration::from_secs(3));
	assert_eq!(manager_ticks(run), 1);
	fs::write(&finish, "").unwrap();
	wait_until(20, "task done", || state(run, &id) == TaskState::Done);
	wait_until(20, "tick on the task end", || manager_ticks(run) >= 2);
	supervisor.stop(run);
}

#[test]
fn job_events_of_idle_tasks_wake_the_manager() {
	let contest = contest(0);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let supervisor = start(&contest, "true");
	wait_until(20, "first tick", || manager_ticks(run) >= 1);
	queued_job_record(run, "j0001", &id, &["true"]);
	wait_until(20, "job finished", || event_count(run, "job_finished") == 1);
	wait_until(20, "tick on the job event", || manager_ticks(run) >= 2);
	supervisor.stop(run);
}

const NEVER_FITTING_RAM_MB: u64 = 900_000;

fn age_job_record(run: &RunDir, job: &str, minutes: u64) {
	let file = fs::File::options()
		.write(true)
		.open(run.job_record_path(&JobId(job.into())))
		.unwrap();
	file.set_modified(SystemTime::now() - Duration::from_secs(minutes * 60))
		.unwrap();
}

fn never_admitted_job(run: &RunDir, id: &str, owner: &TaskId) -> JobId {
	let job = JobId(id.into());
	write_job_record(
		run,
		&JobRecord {
			id: job.clone(),
			owner_task: owner.clone(),
			cmd: vec!["true".into()],
			cwd: run.root.clone(),
			request: JobRequest {
				id: job.clone(),
				class: JobClass::Batch,
				gpu: false,
				ram_mb: NEVER_FITTING_RAM_MB,
				priority: 0,
			},
			state: JobState::Queued,
			pid: None,
			starttime: None,
			sandboxed: false,
		},
	)
	.unwrap();
	job
}

#[test]
fn job_queued_past_the_warning_is_journaled_and_shown() {
	let contest = contest(0);
	let run = &contest.run;
	edit_config(&contest, |config| config.ram_budget_mb = 1_000_000);
	let owner = create_task(run, "train", 0);
	let supervisor = start(&contest, "true");
	wait_until(10, "status file", || run.status().is_file());
	let job = never_admitted_job(run, "j0001", &owner);
	age_job_record(run, "j0001", 31);
	wait_until(20, "warning journaled", || {
		journal_lines(run, "job j0001 queued 31 min: ram available") == 1
	});
	wait_until(20, "block shown", || {
		contest_status(run).contains("blocked: job j0001: queued since")
	});
	sleep(Duration::from_secs(3));
	assert_eq!(journal_lines(run, "job j0001 queued"), 1);
	assert_eq!(read_job_record(run, &job).unwrap().state, JobState::Queued);
	supervisor.stop(run);
}

#[test]
fn job_never_started_is_interrupted_and_its_waiting_task_resumes() {
	let contest = contest(1);
	let run = &contest.run;
	edit_config(&contest, |config| config.ram_budget_mb = 1_000_000);
	let id = create_task(run, "train", 0);
	let notes = run.task_dir(&id).join("notes.md");
	let action = format!(
		"if grep -q 'j0001 never started' {notes} 2>/dev/null; then contest task done; \
		else contest job submit --class batch --ram-mb {NEVER_FITTING_RAM_MB} -- true \
		&& echo submitted >> {notes} && contest task wait-jobs; fi",
		notes = notes.display()
	);
	let supervisor = start(&contest, &action);
	let job = JobId("j0001".into());
	wait_until(20, "task waiting", || state(run, &id) == TaskState::Waiting);
	age_job_record(run, "j0001", 121);
	wait_until(20, "job interrupted", || {
		read_job_record(run, &job).is_ok_and(|record| record.state == JobState::Interrupted)
	});
	wait_until(30, "task done", || state(run, &id) == TaskState::Done);
	supervisor.stop(run);
	assert_eq!(
		journal_lines(
			run,
			"job j0001 interrupted: queued 121 min without starting: ram available"
		),
		1
	);
	assert_eq!(
		journal_lines(run, &format!("resumed {id}: its jobs ended")),
		1
	);
	let notes = fs::read_to_string(&notes).unwrap();
	assert!(notes.contains("j0001 never started"), "{notes}");
}

#[test]
fn pin_failure_pauses_only_timed_launches() {
	let contest = contest(1);
	let run = &contest.run;
	let id = create_task(run, "sim-bench", 0);
	let failure = contest.home.join("pin-failure");
	fs::write(&failure, "").unwrap();
	let finish = contest.home.join("finish");
	let action = worker_script(
		&contest,
		&format!(
			"contest job submit --class batch --ram-mb 10 -- true\n\
			contest job wait j0001\n\
			contest job submit --class timed --ram-mb 10 -- true\n\
			until [ -e {finish} ]; do sleep 0.1; done\n\
			contest job wait j0002\n\
			contest task done\n",
			finish = finish.display()
		),
	);
	let supervisor = start_with_env(
		&contest,
		&action,
		":",
		&[("CONTEST_TEST_PIN_FAILURE_FILE", failure.as_path())],
	);
	let batch = JobId("j0001".into());
	let timed = JobId("j0002".into());
	wait_until(20, "batch job done", || {
		read_job_record(run, &batch).is_ok_and(|record| record.state == JobState::Done { code: 0 })
	});
	wait_until(20, "timed job submitted", || {
		read_job_record(run, &timed).is_ok()
	});
	assert!(contest_status(run).contains("blocked: pin: "));
	sleep(Duration::from_secs(3));
	assert_eq!(
		read_job_record(run, &timed).unwrap().state,
		JobState::Queued
	);
	assert_eq!(journal_lines(run, "agents not pinned off timed cores"), 1);
	fs::remove_file(&failure).unwrap();
	fs::write(&finish, "").unwrap();
	wait_until(20, "task done", || state(run, &id) == TaskState::Done);
	supervisor.stop(run);
	assert_eq!(
		read_job_record(run, &timed).unwrap().state,
		JobState::Done { code: 0 }
	);
	assert_eq!(journal_lines(run, "agents pinned again"), 1);
}
