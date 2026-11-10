use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};

use crate::config::Config;
use crate::paths::{RunDir, TaskId};

const BWRAP: &str = "bwrap";
const ISOLATION_FLAGS: [&str; 4] = [
	"--die-with-parent",
	"--new-session",
	"--unshare-pid",
	"--unshare-ipc",
];
const HIDDEN_DIRS: [&str; 4] = ["/tmp", "/var/tmp", "/run/user", "/home"];
const NIX_DAEMON_SOCKET_DIR: &str = "/nix/var/nix/daemon-socket";
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

#[derive(Debug, Clone, PartialEq)]
pub struct SandboxSpec {
	pub repo_root: PathBuf,
	pub contest_root: PathBuf,
	pub toolbox: Vec<PathBuf>,
	pub plugin_ro: Vec<PathBuf>,
	pub claude_config: PathBuf,
	pub credentials: PathBuf,
	pub sandbox_home: PathBuf,
	pub home: PathBuf,
	pub own_task: Option<TaskId>,
	pub dropped_host_env: Vec<String>,
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

impl SandboxSpec {
	pub fn for_run(run: &RunDir, config: &Config, own_task: Option<TaskId>) -> Result<Self> {
		ensure!(
			config.repo_root.is_absolute(),
			"config repo_root {root:?} is not pinned",
			root = config.repo_root
		);
		let contest_root = fs::canonicalize(&run.root)
			.with_context(|| format!("resolve {root:?}", root = run.root))?;
		let run = RunDir { root: contest_root };
		for dir in HOST_EXECUTED_CONTEST_DIRS {
			create_host_executed_dir(&run.root.join(dir))?;
		}
		let home = PathBuf::from(std::env::var_os("HOME").context("HOME is not set")?);
		let home_owner = own_task
			.as_ref()
			.map_or(MANAGER_HOME_OWNER, |task| task.0.as_str());
		let sandbox_home = run.agents().join(home_owner).join("home");
		create_dir(&sandbox_home)?;
		let mut plugin_ro = vec![contest_bin_dir()?];
		plugin_ro.extend(
			PLUGIN_READ_ONLY_DIRS
				.iter()
				.map(|dir| config.plugin_dir.join(dir)),
		);
		Ok(Self {
			repo_root: config.repo_root.clone(),
			toolbox: config.toolbox.clone(),
			plugin_ro,
			claude_config: run.claude_config(),
			credentials: home.join(".claude/.credentials.json"),
			sandbox_home,
			home,
			own_task,
			dropped_host_env: dropped_host_env(),
			contest_root: run.root,
		})
	}
}

#[derive(Default)]
struct Argv(Vec<String>);

fn text(path: &Path) -> String {
	path.to_string_lossy().into_owned()
}

impl Argv {
	fn flag(&mut self, flag: &str) {
		self.0.push(flag.to_owned());
	}

	fn on(&mut self, flag: &str, path: &Path) {
		self.flag(flag);
		self.0.push(text(path));
	}

	fn mount(&mut self, flag: &str, source: &Path, dest: &Path) {
		self.on(flag, source);
		self.0.push(text(dest));
	}

	fn same(&mut self, flag: &str, path: &Path) {
		self.mount(flag, path, path);
	}
}

fn contest_mounts(argv: &mut Argv, contest_root: &Path) {
	argv.same("--bind", contest_root);
	for dir in HOST_EXECUTED_CONTEST_DIRS {
		argv.same("--ro-bind", &contest_root.join(dir));
	}
	argv.same(
		"--ro-bind-try",
		&contest_root.join(HOST_EXECUTED_CONTEST_FILE),
	);
	argv.same("--ro-bind", &contest_root.join("run"));
}

fn own_task_mounts(argv: &mut Argv, run: &RunDir, task: &TaskId) {
	argv.same("--bind", &run.task_dir(task));
	argv.same("--ro-bind", &run.task_meta(task));
	argv.same("--bind", &run.task_requests(task));
}

fn claude_config_mounts(argv: &mut Argv, spec: &SandboxSpec) {
	let config = &spec.claude_config;
	argv.on("--tmpfs", config);
	argv.same("--ro-bind", &config.join("settings.json"));
	argv.same("--ro-bind-try", &config.join("skills"));
	argv.mount(
		"--bind",
		&spec.credentials,
		&config.join(".credentials.json"),
	);
}

pub fn bwrap_args(spec: &SandboxSpec) -> Vec<String> {
	let mut argv = Argv::default();
	for flag in ISOLATION_FLAGS {
		argv.flag(flag);
	}
	argv.same("--ro-bind", Path::new("/"));
	argv.on("--dev", Path::new("/dev"));
	argv.on("--proc", Path::new("/proc"));
	for dir in HIDDEN_DIRS {
		argv.on("--tmpfs", Path::new(dir));
	}
	argv.mount("--bind", &spec.sandbox_home, &spec.home);
	argv.on("--tmpfs", &spec.repo_root);
	for tool in &spec.toolbox {
		argv.same("--ro-bind", tool);
	}
	contest_mounts(&mut argv, &spec.contest_root);
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
	let nix_daemon_socket = Path::new(NIX_DAEMON_SOCKET_DIR);
	if nix_daemon_socket.is_dir() {
		argv.on("--tmpfs", nix_daemon_socket);
	}
	let hidden = HIDDEN_ENV
		.iter()
		.copied()
		.chain(spec.dropped_host_env.iter().map(String::as_str));
	for name in hidden {
		argv.flag("--unsetenv");
		argv.flag(name);
	}
	argv.0
}

pub fn bwrap_command(spec: &SandboxSpec, argv: &[String]) -> Vec<String> {
	let mut command = vec![BWRAP.to_owned()];
	command.extend(bwrap_args(spec));
	command.push("--".to_owned());
	command.extend(argv.iter().cloned());
	command
}
