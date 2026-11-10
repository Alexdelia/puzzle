use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};

use crate::config::Config;
use crate::paths::{RunDir, TaskId};

const BWRAP: &str = "bwrap";
const ISOLATION_FLAGS: [&str; 4] = [
	"--die-with-parent",
	"--new-session",
	"--unshare-pid",
	"--unshare-ipc",
];
const HIDDEN_ENV: [&str; 9] = [
	"DBUS_SESSION_BUS_ADDRESS",
	"SSH_AUTH_SOCK",
	"GNOME_KEYRING_CONTROL",
	"GPG_AGENT_INFO",
	"WAYLAND_DISPLAY",
	"DISPLAY",
	"XAUTHORITY",
	"SWAYSOCK",
	"I3SOCK",
];
const HOST_SESSION_ENV_PREFIX: &str = "CLAUDE";
const KEPT_CLAUDE_ENV: &str = "CLAUDE_CONFIG_DIR";
const SUPERVISOR_OWNED_TASK_FILES: [&str; 3] = ["spec.toml", "spec.md", "state"];
const PLUGIN_READ_ONLY_DIRS: [&str; 3] = ["prompts", "skills", "kb"];

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
	pub host_session_env: Vec<String>,
}

fn host_session_env() -> Vec<String> {
	let mut names: Vec<String> = std::env::vars_os()
		.filter_map(|(name, _)| name.into_string().ok())
		.filter(|name| name.starts_with(HOST_SESSION_ENV_PREFIX) && name != KEPT_CLAUDE_ENV)
		.collect();
	names.sort();
	names
}

pub fn contest_bin_dir() -> Result<PathBuf> {
	let exe = std::env::current_exe().context("locate contest binary")?;
	exe.parent()
		.map(Path::to_path_buf)
		.context("contest binary has no parent dir")
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
		let home = PathBuf::from(std::env::var_os("HOME").context("HOME is not set")?);
		let sandbox_home = run.agents().join("home");
		fs::create_dir_all(&sandbox_home).with_context(|| format!("create {sandbox_home:?}"))?;
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
			host_session_env: host_session_env(),
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

fn own_task_mounts(argv: &mut Argv, run: &RunDir, task: &TaskId) {
	let task_dir = run.task_dir(task);
	argv.same("--bind", &task_dir);
	for file in SUPERVISOR_OWNED_TASK_FILES {
		argv.same("--ro-bind", &task_dir.join(file));
	}
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
	argv.on("--tmpfs", Path::new("/tmp"));
	argv.on("--tmpfs", Path::new("/home"));
	argv.mount("--bind", &spec.sandbox_home, &spec.home);
	argv.on("--tmpfs", &spec.repo_root);
	for tool in &spec.toolbox {
		argv.same("--ro-bind", tool);
	}
	let run = RunDir {
		root: spec.contest_root.clone(),
	};
	argv.same("--bind", &spec.contest_root);
	argv.same("--ro-bind", &spec.contest_root.join("run"));
	if let Some(task) = &spec.own_task {
		own_task_mounts(&mut argv, &run, task);
	}
	for path in &spec.plugin_ro {
		argv.same("--ro-bind-try", path);
	}
	claude_config_mounts(&mut argv, spec);
	let hidden = HIDDEN_ENV
		.iter()
		.copied()
		.chain(spec.host_session_env.iter().map(String::as_str));
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
