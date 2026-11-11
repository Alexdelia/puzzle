use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, bail, ensure};
use clap::{Parser, Subcommand};
use nix::fcntl::{Flock, FlockArg};
use nix::unistd::setsid;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::atomic::write_atomic;
use crate::config::Config;
use crate::events::{Event, emit};
use crate::hook::{HookVerb, run_hook};
use crate::paths::RunDir;
use crate::run_info::{Mode, RunInfo};
use crate::status::{running_supervisor, status_text};
use crate::supervisor::{DEFAULT_STOP_REASON, STOP_SETTLE_SECONDS, run_supervisor};
use crate::taint::ensure_no_top_level_git;
use crate::worker::{WorkerVerb, run_worker};

const RUN_GITIGNORE: &str = "agents/\nrequests/\nclaude-config/\njobs/*/log\ngcroots/\ntasks/*/work/\nsupervisor.log\nsupervisor.pid\n";
const PASTA: &str = "pasta";
const PASST_FLAKE: &str = "nixpkgs#passt";
const SECONDS_PER_HOUR: u64 = 3600;
const START_CONFIRM_TIMEOUT: Duration = Duration::from_secs(30);
const START_POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Debug, Parser)]
#[command(name = "contest")]
pub struct Cli {
	#[arg(long, global = true, default_value_t = 60)]
	pub timeout: u64,
	#[command(subcommand)]
	pub command: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
	Init(InitArgs),
	Start(StartArgs),
	Stop(StopArgs),
	Status(StatusArgs),
	Say(SayArgs),
	#[command(subcommand)]
	Hook(HookVerb),
	#[command(flatten)]
	Worker(WorkerVerb),
}

#[derive(Debug, clap::Args)]
pub struct InitArgs {
	pub contest_dir: PathBuf,
	#[arg(long)]
	pub puzzle: String,
	#[arg(long)]
	pub mode: Mode,
	#[arg(long)]
	pub active_budget_hours: Option<u64>,
	#[arg(long)]
	pub deadline: Option<String>,
	#[arg(long)]
	pub plugin_dir: Option<PathBuf>,
	#[arg(long)]
	pub pasta_bin: Option<PathBuf>,
}

#[derive(Debug, clap::Args)]
pub struct StartArgs {
	pub contest_dir: PathBuf,
	#[arg(long)]
	pub foreground: bool,
}

#[derive(Debug, clap::Args)]
pub struct StopArgs {
	pub contest_dir: PathBuf,
	#[arg(long, default_value = DEFAULT_STOP_REASON)]
	pub reason: String,
	#[arg(long)]
	pub wait: bool,
}

#[derive(Debug, clap::Args)]
pub struct StatusArgs {
	pub contest_dir: PathBuf,
}

#[derive(Debug, clap::Args)]
pub struct SayArgs {
	pub contest_dir: PathBuf,
	pub text: String,
}

pub fn run(cli: Cli) -> anyhow::Result<ExitCode> {
	match cli.command {
		Verb::Init(args) => init(args).map(|()| ExitCode::SUCCESS),
		Verb::Start(args) => start(args).map(|()| ExitCode::SUCCESS),
		Verb::Stop(args) => stop(args).map(|()| ExitCode::SUCCESS),
		Verb::Status(args) => status(args).map(|()| ExitCode::SUCCESS),
		Verb::Say(args) => say(args).map(|()| ExitCode::SUCCESS),
		Verb::Hook(verb) => Ok(run_hook(verb)),
		Verb::Worker(verb) => run_worker(verb, cli.timeout).map(|()| ExitCode::SUCCESS),
	}
}

fn init(args: InitArgs) -> anyhow::Result<()> {
	let run = RunDir {
		root: args.contest_dir,
	};
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
	write_atomic(&run.root.join("run/.gitignore"), RUN_GITIGNORE.as_bytes())?;

	let mut config = Config::default();
	if let Some(plugin_dir) = args.plugin_dir {
		config.plugin_dir =
			fs::canonicalize(&plugin_dir).with_context(|| format!("resolve {plugin_dir:?}"))?;
	}
	let (repo_root, git_common_dir) = pinned_git_dirs(&run.root);
	config.repo_root = repo_root;
	config.git_common_dir = git_common_dir;
	config.pasta_bin = match args.pasta_bin {
		Some(pasta) => explicit_pasta_bin(&pasta, &config.repo_root)?,
		None => pinned_pasta_bin(&run, &config.repo_root)?,
	};
	write_atomic(&run.config(), toml::to_string(&config)?.as_bytes())?;

	let deadline_unix = args
		.deadline
		.map(|text| {
			OffsetDateTime::parse(&text, &Rfc3339)
				.map(OffsetDateTime::unix_timestamp)
				.with_context(|| format!("parse deadline {text:?}"))
		})
		.transpose()?;
	let info = RunInfo {
		puzzle: args.puzzle,
		mode: args.mode,
		deadline_unix,
		active_budget_seconds: args
			.active_budget_hours
			.map(|hours| hours * SECONDS_PER_HOUR),
		active_seconds: 0,
		phase: "start".into(),
		plugin_commit: plugin_commit(&config.plugin_dir),
		started_at: now_unix(),
	};
	write_atomic(&run.run_json(), &serde_json::to_vec_pretty(&info)?)?;
	Ok(())
}

fn contest_run(contest_dir: &Path) -> anyhow::Result<RunDir> {
	let root = fs::canonicalize(contest_dir).with_context(|| format!("resolve {contest_dir:?}"))?;
	Ok(RunDir { root })
}

fn start(args: StartArgs) -> anyhow::Result<()> {
	let run = contest_run(&args.contest_dir)?;
	let config = Config::load(&run.config())?;
	if args.foreground {
		return run_supervisor(run, config);
	}
	if let Some(pid) = running_supervisor(&run) {
		if run.stop_file().exists() {
			bail!(
				"supervisor pid {pid} is stopping: wait with `contest stop --wait`, then start again"
			);
		}
		bail!("supervisor already running (pid {pid})");
	}
	let log_path = run.supervisor_log();
	let log = OpenOptions::new()
		.create(true)
		.append(true)
		.open(&log_path)
		.with_context(|| format!("open {log_path:?}"))?;
	let mut command = Command::new(std::env::current_exe()?);
	command
		.arg("start")
		.arg(&run.root)
		.arg("--foreground")
		.stdin(Stdio::null())
		.stdout(log.try_clone()?)
		.stderr(log);
	unsafe {
		command.pre_exec(|| setsid().map(drop).map_err(io::Error::from));
	}
	let mut child = command.spawn().context("spawn supervisor")?;
	let pid = child.id().to_string();
	let log = log_path.display();
	let deadline = Instant::now() + START_CONFIRM_TIMEOUT;
	loop {
		if let Some(status) = child.try_wait()? {
			bail!("supervisor exited at start ({status}), see {log} and run/journal.md");
		}
		if announced_pid(&run).as_deref() == Some(pid.as_str()) {
			println!("supervisor started: pid {pid}, log {log}");
			return Ok(());
		}
		if Instant::now() >= deadline {
			println!("supervisor pid {pid} still starting, log {log}");
			return Ok(());
		}
		sleep(START_POLL_INTERVAL);
	}
}

fn announced_pid(run: &RunDir) -> Option<String> {
	fs::read_to_string(run.supervisor_pid())
		.ok()
		.map(|text| text.trim().to_owned())
}

fn stop(args: StopArgs) -> anyhow::Result<()> {
	let run = contest_run(&args.contest_dir)?;
	if !run.stop_file().exists() {
		write_atomic(&run.stop_file(), args.reason.as_bytes())?;
	}
	let Some(pid) = running_supervisor(&run) else {
		println!("no supervisor running: STOP stays until the next contest start");
		return Ok(());
	};
	println!(
		"stop requested: supervisor pid {pid} stops within {seconds} s",
		seconds = seconds_until_stopped(&run)
	);
	if args.wait {
		wait_for_supervisor_exit(&run)?;
		println!("supervisor stopped");
	}
	Ok(())
}

fn seconds_until_stopped(run: &RunDir) -> u64 {
	let grace = Config::load(&run.config())
		.unwrap_or_default()
		.grace_seconds;
	let requested_ago = fs::metadata(run.stop_file())
		.and_then(|meta| meta.modified())
		.ok()
		.and_then(|modified| modified.elapsed().ok())
		.map_or(0, |elapsed| elapsed.as_secs());
	(grace + STOP_SETTLE_SECONDS).saturating_sub(requested_ago)
}

fn wait_for_supervisor_exit(run: &RunDir) -> anyhow::Result<()> {
	let path = run.supervisor_pid();
	let file = File::open(&path).with_context(|| format!("open {path:?}"))?;
	let lock = Flock::lock(file, FlockArg::LockShared)
		.map_err(|(_, errno)| errno)
		.with_context(|| format!("wait for lock on {path:?}"))?;
	drop(lock);
	Ok(())
}

fn status(args: StatusArgs) -> anyhow::Result<()> {
	let run = contest_run(&args.contest_dir)?;
	print!("{text}", text = status_text(&run, now_unix()));
	Ok(())
}

fn say(args: SayArgs) -> anyhow::Result<()> {
	let run = contest_run(&args.contest_dir)?;
	let inbox = run.inbox();
	fs::create_dir_all(&inbox).with_context(|| format!("create {inbox:?}"))?;
	let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
	write_atomic(&inbox.join(format!("{nanos}.md")), args.text.as_bytes())?;
	emit(&run, Event::UserMessage { text: args.text })
}

fn git_path(dir: &Path, args: &[&str]) -> PathBuf {
	Command::new("git")
		.arg("-C")
		.arg(dir)
		.args(args)
		.output()
		.ok()
		.filter(|output| output.status.success())
		.map(|output| PathBuf::from(String::from_utf8_lossy(&output.stdout).trim()))
		.unwrap_or_default()
}

fn pinned_git_dirs(contest_dir: &Path) -> (PathBuf, PathBuf) {
	(
		git_path(contest_dir, &["rev-parse", "--show-toplevel"]),
		git_path(
			contest_dir,
			&["rev-parse", "--path-format=absolute", "--git-common-dir"],
		),
	)
}

fn is_executable_file(path: &Path) -> bool {
	fs::metadata(path).is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

fn inside_repo(path: &Path, repo_root: &Path) -> bool {
	!repo_root.as_os_str().is_empty()
		&& fs::canonicalize(path).map_or(true, |real| real.starts_with(repo_root))
}

fn explicit_pasta_bin(pasta: &Path, repo_root: &Path) -> anyhow::Result<PathBuf> {
	let real = fs::canonicalize(pasta).with_context(|| format!("resolve {pasta:?}"))?;
	ensure!(
		is_executable_file(&real),
		"pasta {real:?} is not an executable file"
	);
	ensure!(
		!inside_repo(&real, repo_root),
		"pasta {real:?} lies under repo_root {repo_root:?}"
	);
	Ok(real)
}

fn pasta_on_path(repo_root: &Path) -> Option<PathBuf> {
	std::env::split_paths(&std::env::var_os("PATH")?)
		.filter(|dir| dir.is_absolute())
		.map(|dir| dir.join(PASTA))
		.find(|path| is_executable_file(path) && !inside_repo(path, repo_root))
}

fn pasta_from_nix(gc_root: &Path) -> Option<PathBuf> {
	let output = Command::new("nix")
		.args(["build", PASST_FLAKE, "--print-out-paths", "--out-link"])
		.arg(gc_root)
		.output()
		.ok()
		.filter(|output| output.status.success())?;
	let store_path = String::from_utf8_lossy(&output.stdout)
		.lines()
		.next()?
		.to_owned();
	Some(PathBuf::from(store_path).join("bin").join(PASTA)).filter(|path| path.is_file())
}

fn pinned_pasta_bin(run: &RunDir, repo_root: &Path) -> anyhow::Result<PathBuf> {
	if let Some(pasta) = pasta_on_path(repo_root) {
		return Ok(pasta);
	}
	let gc_roots = run.gc_roots();
	fs::create_dir_all(&gc_roots).with_context(|| format!("create {gc_roots:?}"))?;
	Ok(pasta_from_nix(&gc_roots.join("passt")).unwrap_or_else(|| {
		eprintln!(
			"pasta not found on PATH nor via nix build {PASST_FLAKE}: sandboxed launches will fail"
		);
		PathBuf::new()
	}))
}

fn plugin_commit(plugin_dir: &Path) -> String {
	Command::new("git")
		.arg("-C")
		.arg(plugin_dir)
		.args(["rev-parse", "HEAD"])
		.output()
		.ok()
		.filter(|output| output.status.success())
		.map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
		.unwrap_or_default()
}

fn now_unix() -> i64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|elapsed| elapsed.as_secs() as i64)
		.unwrap_or_default()
}
