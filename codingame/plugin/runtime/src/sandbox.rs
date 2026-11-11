use std::convert::Infallible;
use std::ffi::OsString;
use std::fs;
use std::os::fd::IntoRawFd;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};
use nix::fcntl::{AT_FDCWD, OFlag, OpenHow, ResolveFlag, openat2};
use nix::sys::stat::{SFlag, fstat};
use nix::unistd::{getgid, getuid};

use crate::config::Config;
use crate::paths::{RunDir, TaskId};

const BWRAP: &str = "bwrap";
const PASTA_FLAGS: [&str; 11] = [
	"--foreground",
	"--config-net",
	"--tcp-ports",
	"none",
	"--udp-ports",
	"none",
	"--tcp-ns",
	"none",
	"--udp-ns",
	"none",
	"--no-map-gw",
];
const ISOLATION_FLAGS: [&str; 4] = [
	"--die-with-parent",
	"--new-session",
	"--unshare-pid",
	"--unshare-ipc",
];
const HIDDEN_DIRS: [&str; 4] = ["/tmp", "/var/tmp", "/run/user", "/home"];
const NIX_DAEMON_SOCKET_DIR: &str = "/nix/var/nix/daemon-socket";
const SYSTEM_BUS_DIR: &str = "/run/dbus";
const HIDDEN_ENV: [&str; 10] = [
	"DBUS_SESSION_BUS_ADDRESS",
	"SSH_AUTH_SOCK",
	"GNOME_KEYRING_CONTROL",
	"GPG_AGENT_INFO",
	"WAYLAND_DISPLAY",
	"DISPLAY",
	"XAUTHORITY",
	"SWAYSOCK",
	"I3SOCK",
	"XDG_RUNTIME_DIR",
];
const DROPPED_ENV_PREFIXES: [&str; 2] = ["CLAUDE", "ANTHROPIC"];
const DROPPED_ENV_NAMES: [&str; 2] = ["GH_TOKEN", "GITHUB_TOKEN"];
const KEPT_CLAUDE_ENV: &str = "CLAUDE_CONFIG_DIR";
const HOST_EXECUTED_CONTEST_DIRS: [&str; 2] = [".claude", ".direnv"];
const HOST_EXECUTED_CONTEST_FILE: &str = ".envrc";
const PLUGIN_READ_ONLY_DIRS: [&str; 3] = ["prompts", "skills", "kb"];
const MANAGER_HOME_OWNER: &str = "manager";
const CONTEST_BIN: &str = "contest";
const LAUNCHER_VERB: &str = "sandbox-exec";
pub const AGENT_HOME: &str = "/home/contest-agent";
const BIND_FD: &str = "--bind-fd";
const RO_BIND_FD: &str = "--ro-bind-fd";
const END_OF_OPTIONS: &str = "--";

#[derive(Debug, Clone, PartialEq)]
pub struct SandboxSpec {
	pub repo_root: PathBuf,
	pub contest_root: PathBuf,
	pub contest_envrc: bool,
	pub toolbox: Vec<PathBuf>,
	pub plugin_ro: Vec<PathBuf>,
	pub claude_config: PathBuf,
	pub credentials: PathBuf,
	pub sandbox_home: PathBuf,
	pub own_task: Option<TaskId>,
	pub dropped_host_env: Vec<String>,
	pub pasta_bin: PathBuf,
	pub launcher: PathBuf,
}

pub fn dropped_env_names(names: impl IntoIterator<Item = String>) -> Vec<String> {
	let mut dropped: Vec<String> = names
		.into_iter()
		.filter(|name| {
			name != KEPT_CLAUDE_ENV
				&& (DROPPED_ENV_PREFIXES
					.iter()
					.any(|prefix| name.starts_with(prefix))
					|| DROPPED_ENV_NAMES.contains(&name.as_str()))
		})
		.collect();
	dropped.sort();
	dropped
}

pub fn dropped_host_env() -> Vec<String> {
	dropped_env_names(std::env::vars_os().filter_map(|(name, _)| name.into_string().ok()))
}

pub fn contest_bin_dir() -> Result<PathBuf> {
	let exe = std::env::current_exe().context("locate contest binary")?;
	exe.parent()
		.map(Path::to_path_buf)
		.context("contest binary has no parent dir")
}

fn create_dir(dir: &Path) -> Result<()> {
	fs::create_dir_all(dir).with_context(|| format!("create {dir:?}"))
}

fn create_host_executed_dir(dir: &Path) -> Result<()> {
	if fs::symlink_metadata(dir).is_ok_and(|meta| meta.is_symlink()) {
		bail!("{dir:?} is a symlink, refusing to follow it");
	}
	create_dir(dir)
}

fn ensure_pinned_envrc(envrc: &Path) -> Result<()> {
	let meta = fs::symlink_metadata(envrc)
		.with_context(|| format!("pinned {envrc:?} is gone, refusing to launch"))?;
	ensure!(
		meta.file_type().is_file(),
		"pinned {envrc:?} is no longer a regular file (a symlink or another type), refusing to launch"
	);
	Ok(())
}

impl SandboxSpec {
	pub fn for_run(run: &RunDir, config: &Config, own_task: Option<TaskId>) -> Result<Self> {
		ensure!(
			config.repo_root.is_absolute(),
			"config repo_root {root:?} is not pinned",
			root = config.repo_root
		);
		ensure!(
			config.pasta_bin.is_absolute(),
			"config pasta_bin {pasta:?} is not pinned",
			pasta = config.pasta_bin
		);
		ensure!(
			!config.pasta_bin.starts_with(&config.repo_root),
			"config pasta_bin {pasta:?} lies under repo_root {root:?}",
			pasta = config.pasta_bin,
			root = config.repo_root
		);
		let contest_root = fs::canonicalize(&run.root)
			.with_context(|| format!("resolve {root:?}", root = run.root))?;
		let run = RunDir { root: contest_root };
		for dir in HOST_EXECUTED_CONTEST_DIRS {
			create_host_executed_dir(&run.root.join(dir))?;
		}
		if config.contest_envrc {
			ensure_pinned_envrc(&run.root.join(HOST_EXECUTED_CONTEST_FILE))?;
		}
		let home = PathBuf::from(std::env::var_os("HOME").context("HOME is not set")?);
		let home_owner = own_task
			.as_ref()
			.map_or(MANAGER_HOME_OWNER, |task| task.0.as_str());
		let sandbox_home = run.agents().join(home_owner).join("home");
		create_dir(&sandbox_home)?;
		let bin_dir = contest_bin_dir()?;
		let mut plugin_ro = vec![bin_dir.clone()];
		plugin_ro.extend(
			PLUGIN_READ_ONLY_DIRS
				.iter()
				.map(|dir| config.plugin_dir.join(dir)),
		);
		Ok(Self {
			repo_root: config.repo_root.clone(),
			contest_envrc: config.contest_envrc,
			toolbox: config.toolbox.clone(),
			plugin_ro,
			claude_config: run.claude_config(),
			credentials: home.join(".claude/.credentials.json"),
			sandbox_home,
			own_task,
			dropped_host_env: dropped_host_env(),
			pasta_bin: config.pasta_bin.clone(),
			launcher: bin_dir.join(CONTEST_BIN),
			contest_root: run.root,
		})
	}
}

#[derive(Default)]
struct Argv {
	args: Vec<String>,
	destinations: Vec<PathBuf>,
}

fn text(path: &Path) -> String {
	path.to_string_lossy().into_owned()
}

impl Argv {
	fn flag(&mut self, flag: &str) {
		self.args.push(flag.to_owned());
	}

	fn on(&mut self, flag: &str, dest: &Path) {
		self.flag(flag);
		self.args.push(text(dest));
		self.destinations.push(dest.to_path_buf());
	}

	fn mount(&mut self, flag: &str, source: &Path, dest: &Path) {
		self.flag(flag);
		self.args.push(text(source));
		self.args.push(text(dest));
		self.destinations.push(dest.to_path_buf());
	}

	fn setenv(&mut self, name: &str, value: &str) {
		self.flag("--setenv");
		self.flag(name);
		self.flag(value);
	}

	fn same(&mut self, flag: &str, path: &Path) {
		self.mount(flag, path, path);
	}
}

fn contest_mounts(argv: &mut Argv, spec: &SandboxSpec, contest_bind: &str) {
	let contest_root = &spec.contest_root;
	argv.same(contest_bind, contest_root);
	for dir in HOST_EXECUTED_CONTEST_DIRS {
		argv.same(RO_BIND_FD, &contest_root.join(dir));
	}
	if spec.contest_envrc {
		argv.same(RO_BIND_FD, &contest_root.join(HOST_EXECUTED_CONTEST_FILE));
	}
	argv.same(RO_BIND_FD, &contest_root.join("run"));
}

fn own_task_mounts(argv: &mut Argv, run: &RunDir, task: &TaskId) {
	argv.same(BIND_FD, &run.task_dir(task));
	argv.same(RO_BIND_FD, &run.task_meta(task));
	argv.same(BIND_FD, &run.task_requests(task));
}

fn claude_config_mounts(argv: &mut Argv, spec: &SandboxSpec) {
	let config = &spec.claude_config;
	argv.on("--tmpfs", config);
	argv.same(RO_BIND_FD, &config.join("settings.json"));
	argv.same(RO_BIND_FD, &config.join("skills"));
	argv.mount(
		"--bind",
		&spec.credentials,
		&config.join(".credentials.json"),
	);
}

fn keep_host_identity(argv: &mut Argv) {
	argv.flag("--unshare-user");
	argv.flag("--uid");
	argv.flag(&getuid().to_string());
	argv.flag("--gid");
	argv.flag(&getgid().to_string());
}

fn ensure_nothing_mounted_under(home: &Path, destinations: &[PathBuf]) -> Result<()> {
	if let Some(dest) = destinations.iter().find(|dest| dest.starts_with(home)) {
		bail!(
			"sandbox destination {dest:?} lies under the agent home {home:?}: it would resolve through a worker-writable dir"
		);
	}
	Ok(())
}

pub fn bwrap_args(spec: &SandboxSpec) -> Result<Vec<String>> {
	let mut argv = Argv::default();
	keep_host_identity(&mut argv);
	for flag in ISOLATION_FLAGS {
		argv.flag(flag);
	}
	argv.same("--ro-bind", Path::new("/"));
	argv.on("--dev", Path::new("/dev"));
	argv.on("--proc", Path::new("/proc"));
	for dir in HIDDEN_DIRS {
		argv.on("--tmpfs", Path::new(dir));
	}
	let agent_home = Path::new(AGENT_HOME);
	argv.mount(BIND_FD, &spec.sandbox_home, agent_home);
	argv.setenv("HOME", AGENT_HOME);
	let after_home = argv.destinations.len();
	argv.on("--tmpfs", &spec.repo_root);
	for tool in &spec.toolbox {
		argv.same("--ro-bind", tool);
	}
	let contest_bind = match spec.own_task {
		Some(_) => BIND_FD,
		None => RO_BIND_FD,
	};
	contest_mounts(&mut argv, spec, contest_bind);
	if let Some(task) = &spec.own_task {
		let run = RunDir {
			root: spec.contest_root.clone(),
		};
		own_task_mounts(&mut argv, &run, task);
	}
	for path in &spec.plugin_ro {
		argv.same("--ro-bind-try", path);
	}
	claude_config_mounts(&mut argv, spec);
	for socket_dir in [SYSTEM_BUS_DIR, NIX_DAEMON_SOCKET_DIR].map(Path::new) {
		if socket_dir.is_dir() {
			argv.on("--tmpfs", socket_dir);
		}
	}
	let hidden = HIDDEN_ENV
		.iter()
		.copied()
		.chain(spec.dropped_host_env.iter().map(String::as_str));
	for name in hidden {
		argv.flag("--unsetenv");
		argv.flag(name);
	}
	ensure_nothing_mounted_under(agent_home, &argv.destinations[after_home..])?;
	Ok(argv.args)
}

pub fn sandbox_command(spec: &SandboxSpec, argv: &[String]) -> Result<Vec<String>> {
	let mut command = vec![text(&spec.pasta_bin)];
	command.extend(PASTA_FLAGS.map(String::from));
	command.push(END_OF_OPTIONS.to_owned());
	command.push(text(&spec.launcher));
	command.push(LAUNCHER_VERB.to_owned());
	command.push(BWRAP.to_owned());
	command.extend(bwrap_args(spec)?);
	command.push(END_OF_OPTIONS.to_owned());
	command.extend(argv.iter().cloned());
	Ok(command)
}

fn open_bind_source(path: &Path) -> Result<i32> {
	ensure!(path.is_absolute(), "bind source {path:?} is not absolute");
	let how = OpenHow::new()
		.flags(OFlag::O_PATH)
		.resolve(ResolveFlag::RESOLVE_NO_SYMLINKS);
	let fd = openat2(AT_FDCWD, path, how).with_context(|| {
		format!("open bind source {path:?} without following symlinks, refusing to launch")
	})?;
	let kind = SFlag::from_bits_truncate(fstat(&fd)?.st_mode) & SFlag::S_IFMT;
	ensure!(
		kind == SFlag::S_IFDIR || kind == SFlag::S_IFREG,
		"bind source {path:?} is neither a dir nor a regular file, refusing to launch"
	);
	Ok(fd.into_raw_fd())
}

pub fn pin_fd_bind_sources(command: &[OsString]) -> Result<Vec<OsString>> {
	let mut pinned = Vec::with_capacity(command.len());
	let mut parts = command.iter();
	while let Some(part) = parts.next() {
		pinned.push(part.clone());
		if part == END_OF_OPTIONS {
			pinned.extend(parts.by_ref().cloned());
			break;
		}
		if part == BIND_FD || part == RO_BIND_FD {
			let source = parts
				.next()
				.with_context(|| format!("{part:?} lacks its source"))?;
			pinned.push(open_bind_source(Path::new(source))?.to_string().into());
		}
	}
	Ok(pinned)
}

pub fn exec_with_pinned_sources(command: &[OsString]) -> Result<Infallible> {
	let pinned = pin_fd_bind_sources(command)?;
	let (program, args) = pinned.split_first().context("empty sandbox command")?;
	let error = Command::new(program).args(args).exec();
	Err(error).with_context(|| format!("exec {program:?}"))
}
