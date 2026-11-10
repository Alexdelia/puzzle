use std::fs;
use std::path::Path;
use std::process::Command;

use contest::atomic::write_atomic;
use contest::claude_config::{agent_env, credentials_expire_within, write_claude_settings};
use contest::config::Config;
use contest::paths::{JobId, RunDir, TaskId};
use contest::sandbox::{
	SandboxSpec, bwrap_args, bwrap_command, contest_bin_dir, dropped_env_names,
};
use tempfile::TempDir;

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

const NIX_DAEMON_SOCKET_DIR: &str = "/nix/var/nix/daemon-socket";
const READY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

fn text(path: &Path) -> String {
	path.to_string_lossy().into_owned()
}

fn task() -> TaskId {
	TaskId::parse("t0001-sim-bench").unwrap()
}

fn spec_under(root: &Path, own_task: Option<TaskId>) -> SandboxSpec {
	let repo = root.join("repo");
	let contest = repo.join("bot-programming/c1");
	let home_owner = own_task
		.as_ref()
		.map_or("manager".to_owned(), |task| task.0.clone());
	SandboxSpec {
		repo_root: repo.clone(),
		contest_root: contest.clone(),
		toolbox: vec![
			repo.join("bot-programming/c0"),
			repo.join("optimization/o1"),
		],
		plugin_ro: vec![repo.join("plugin/runtime/target"), repo.join("plugin/kb")],
		claude_config: contest.join("run/claude-config"),
		credentials: root.join("host-home/.claude/.credentials.json"),
		sandbox_home: contest.join(format!("run/agents/{home_owner}/home")),
		home: root.join("host-home"),
		own_task,
		dropped_host_env: vec!["CLAUDE_CODE_MESSAGING_TOKEN".into()],
	}
}

fn spec() -> SandboxSpec {
	spec_under(Path::new("/r"), Some(task()))
}

fn mounts(args: &[String]) -> Vec<(String, String, String)> {
	let mut found = Vec::new();
	let mut index = 0;
	while index < args.len() {
		let flag = args[index].as_str();
		match flag {
			"--bind" | "--ro-bind" | "--ro-bind-try" | "--bind-try" => {
				found.push((
					flag.into(),
					args[index + 1].clone(),
					args[index + 2].clone(),
				));
				index += 3;
			}
			"--tmpfs" | "--dev" | "--proc" | "--unsetenv" => {
				found.push((flag.into(), String::new(), args[index + 1].clone()));
				index += 2;
			}
			_ => index += 1,
		}
	}
	found
}

fn position(args: &[String], flag: &str, dest: &Path) -> usize {
	let dest = text(dest);
	mounts(args)
		.iter()
		.position(|(found, _, target)| found == flag && *target == dest)
		.unwrap_or_else(|| panic!("no {flag} {dest} in {args:?}"))
}

#[test]
fn bwrap_starts_with_isolation_flags() {
	let args = bwrap_args(&spec());
	let expected: Vec<String> = [
		"--die-with-parent",
		"--new-session",
		"--unshare-pid",
		"--unshare-ipc",
		"--ro-bind",
		"/",
		"/",
		"--dev",
		"/dev",
		"--proc",
		"/proc",
		"--tmpfs",
		"/tmp",
		"--tmpfs",
		"/var/tmp",
		"--tmpfs",
		"/run/user",
		"--tmpfs",
		"/home",
		"--bind",
		"/r/repo/bot-programming/c1/run/agents/t0001-sim-bench/home",
		"/r/host-home",
		"--tmpfs",
		"/r/repo",
	]
	.map(String::from)
	.to_vec();
	assert_eq!(args[..expected.len()], expected[..]);
}

#[test]
fn bwrap_hides_nix_daemon_socket_after_every_bind() {
	let mut spec = spec();
	spec.toolbox.push("/nix".into());
	let args = bwrap_args(&spec);
	let mounts: Vec<_> = mounts(&args)
		.into_iter()
		.filter(|(flag, _, _)| flag != "--unsetenv")
		.collect();
	let hidden = mounts
		.iter()
		.position(|(flag, _, target)| flag == "--tmpfs" && target == NIX_DAEMON_SOCKET_DIR);
	if !Path::new(NIX_DAEMON_SOCKET_DIR).is_dir() {
		assert_eq!(hidden, None, "{args:?}");
		return;
	}
	assert_eq!(hidden, Some(mounts.len() - 1), "{args:?}");
}

#[test]
fn real_bwrap_hides_nix_daemon_socket_under_nix_toolbox() {
	if !bwrap_available() || !Path::new(NIX_DAEMON_SOCKET_DIR).is_dir() {
		eprintln!("bwrap or nix daemon socket not found, skipped");
		return;
	}
	let Layout { _dir, mut spec, .. } = layout();
	spec.toolbox.push("/nix/var/nix".into());
	let socket = Path::new(NIX_DAEMON_SOCKET_DIR).join("socket");
	assert_eq!(sandboxed_sh(&spec, &probe_line(&socket)).trim(), "absent");
}

#[test]
fn bwrap_hides_git() {
	let spec = spec();
	let args = bwrap_args(&spec);
	position(&args, "--tmpfs", &spec.repo_root);
	let git = text(&spec.repo_root.join(".git"));
	assert!(args.iter().all(|arg| !arg.starts_with(&git)), "{args:?}");
}

#[test]
fn toolbox_read_only() {
	let spec = spec();
	let args = bwrap_args(&spec);
	let repo_hidden = position(&args, "--tmpfs", &spec.repo_root);
	for tool in &spec.toolbox {
		let tool_text = text(tool);
		let binds: Vec<_> = mounts(&args)
			.into_iter()
			.filter(|(_, _, dest)| *dest == tool_text)
			.collect();
		assert_eq!(
			binds,
			vec![("--ro-bind".into(), tool_text.clone(), tool_text.clone())]
		);
		assert!(position(&args, "--ro-bind", tool) > repo_hidden);
	}
}

#[test]
fn run_dir_is_read_only_except_own_task_and_requests() {
	let spec = spec();
	let args = bwrap_args(&spec);
	let run = RunDir {
		root: spec.contest_root.clone(),
	};
	let task = task();
	let contest = position(&args, "--bind", &spec.contest_root);
	let run_ro = position(&args, "--ro-bind", &run.root.join("run"));
	let task_rw = position(&args, "--bind", &run.task_dir(&task));
	let meta_ro = position(&args, "--ro-bind", &run.task_meta(&task));
	let requests_rw = position(&args, "--bind", &run.task_requests(&task));
	assert!(position(&args, "--tmpfs", &spec.repo_root) < contest);
	assert!(contest < run_ro);
	assert!(run_ro < task_rw);
	assert!(task_rw < meta_ro);
	assert!(run_ro < requests_rw);
	let task_files = text(&run.task_dir(&task));
	assert!(
		mounts(&args).iter().all(
			|(_, _, dest)| !dest.starts_with(&format!("{task_files}/spec"))
				&& !dest.ends_with("/state")
		),
		"{args:?}"
	);
	let rw: Vec<String> = mounts(&args)
		.into_iter()
		.filter(|(flag, _, _)| flag == "--bind")
		.map(|(_, _, dest)| dest)
		.collect();
	assert!(!rw.contains(&text(&run.jobs())));
	assert!(!rw.contains(&text(&run.tasks())));
}

#[test]
fn host_executed_contest_files_read_only() {
	let spec = spec();
	let args = bwrap_args(&spec);
	let contest = position(&args, "--bind", &spec.contest_root);
	let claude = position(&args, "--ro-bind", &spec.contest_root.join(".claude"));
	let direnv = position(&args, "--ro-bind", &spec.contest_root.join(".direnv"));
	let envrc = position(&args, "--ro-bind-try", &spec.contest_root.join(".envrc"));
	assert!(contest < claude && contest < direnv && contest < envrc);
}

#[test]
fn manager_sandbox_has_no_task_binds() {
	let spec = spec_under(Path::new("/r"), None);
	let args = bwrap_args(&spec);
	let run = text(&spec.contest_root.join("run"));
	let rw_in_run: Vec<_> = mounts(&args)
		.into_iter()
		.filter(|(flag, _, dest)| flag == "--bind" && dest.starts_with(&run))
		.map(|(_, _, dest)| dest)
		.collect();
	assert_eq!(
		rw_in_run,
		vec![text(&spec.claude_config.join(".credentials.json"))]
	);
}

#[test]
fn plugin_paths_read_only_and_claude_config_layered() {
	let spec = spec();
	let args = bwrap_args(&spec);
	for path in &spec.plugin_ro {
		position(&args, "--ro-bind-try", path);
	}
	let config_tmpfs = position(&args, "--tmpfs", &spec.claude_config);
	let settings = position(
		&args,
		"--ro-bind",
		&spec.claude_config.join("settings.json"),
	);
	let skills = position(&args, "--ro-bind-try", &spec.claude_config.join("skills"));
	let credentials = position(
		&args,
		"--bind",
		&spec.claude_config.join(".credentials.json"),
	);
	assert!(position(&args, "--ro-bind", &spec.contest_root.join("run")) < config_tmpfs);
	assert!(config_tmpfs < settings && config_tmpfs < skills && config_tmpfs < credentials);
	let credentials_mount = &mounts(&args)[credentials];
	assert_eq!(credentials_mount.1, text(&spec.credentials));
}

#[test]
fn hides_session_environment() {
	let args = bwrap_args(&spec());
	let unset: Vec<String> = mounts(&args)
		.into_iter()
		.filter(|(flag, _, _)| flag == "--unsetenv")
		.map(|(_, _, name)| name)
		.collect();
	let mut expected = HIDDEN_ENV.map(String::from).to_vec();
	expected.push("CLAUDE_CODE_MESSAGING_TOKEN".into());
	assert_eq!(unset, expected);
}

#[test]
fn bwrap_command_wraps_argv() {
	let spec = spec();
	let argv = vec!["cargo".to_string(), "build".to_string()];
	let command = bwrap_command(&spec, &argv);
	assert_eq!(command[0], "bwrap");
	assert_eq!(command[1..command.len() - 3], bwrap_args(&spec)[..]);
	assert_eq!(command[command.len() - 3..], ["--", "cargo", "build"]);
}

struct Layout {
	_dir: TempDir,
	spec: SandboxSpec,
	run: RunDir,
	task: TaskId,
	other: TaskId,
}

fn touch(path: &Path, body: &str) {
	fs::create_dir_all(path.parent().unwrap()).unwrap();
	fs::write(path, body).unwrap();
}

fn layout() -> Layout {
	let dir = TempDir::new().unwrap();
	let spec = spec_under(dir.path(), Some(task()));
	let run = RunDir {
		root: spec.contest_root.clone(),
	};
	let task = task();
	let other = TaskId::parse("t0002-sim-bench").unwrap();
	touch(&spec.repo_root.join(".git/config"), "secret");
	touch(
		&spec.repo_root.join("bot-programming/old/bot.rs"),
		"old bot",
	);
	for tool in &spec.toolbox {
		touch(&tool.join("referee.rs"), "referee");
	}
	touch(&spec.plugin_ro[0].join("contest"), "bin");
	touch(&run.config(), "workers = 3\n");
	for id in [&task, &other] {
		for file in ["spec.toml", "spec.md", "state"] {
			touch(&run.task_meta(id).join(file), "x");
		}
		touch(&run.task_dir(id).join("notes.md"), "x");
		fs::create_dir_all(run.task_requests(id)).unwrap();
	}
	fs::create_dir_all(spec.contest_root.join(".claude")).unwrap();
	fs::create_dir_all(spec.contest_root.join(".direnv")).unwrap();
	touch(&spec.contest_root.join(".envrc"), "use flake");
	fs::create_dir_all(run.jobs().join("j0001")).unwrap();
	touch(&spec.claude_config.join("settings.json"), "{}");
	fs::create_dir_all(&spec.sandbox_home).unwrap();
	touch(&spec.credentials, "token");
	touch(&spec.contest_root.join("src/main.rs"), "fn main() {}");
	Layout {
		_dir: dir,
		spec,
		run,
		task,
		other,
	}
}

fn bwrap_available() -> bool {
	Command::new("bwrap")
		.arg("--version")
		.output()
		.is_ok_and(|output| output.status.success())
}

fn sandboxed_sh(spec: &SandboxSpec, script: &str) -> String {
	let argv = vec!["sh".to_string(), "-c".to_string(), script.to_string()];
	let command = bwrap_command(spec, &argv);
	let output = Command::new(&command[0])
		.args(&command[1..])
		.output()
		.unwrap();
	assert!(
		output.status.success(),
		"{stderr}",
		stderr = String::from_utf8_lossy(&output.stderr)
	);
	String::from_utf8_lossy(&output.stdout).into_owned()
}

fn probe_line(path: &Path) -> String {
	let path = text(path);
	format!(
		"if [ -e '{path}' ]; then if (echo w >> '{path}') 2>/dev/null; then echo rw; else echo ro; fi; else echo absent; fi"
	)
}

fn probe_empty_line(path: &Path) -> String {
	let path = text(path);
	format!("if [ -z \"$(ls -A '{path}')\" ]; then echo empty; else echo listed; fi")
}

fn probe_dir_line(path: &Path) -> String {
	let path = text(path);
	format!(
		"if [ -d '{path}' ]; then if touch '{path}/probe' 2>/dev/null; then echo rw; else echo ro; fi; else echo absent; fi"
	)
}

#[test]
fn real_bwrap_enforces_layout() {
	if !bwrap_available() {
		eprintln!("bwrap not found, skipped");
		return;
	}
	let Layout {
		_dir,
		spec,
		run,
		task,
		other,
	} = layout();
	let checks: Vec<(&str, String, &str)> = vec![
		(
			"git",
			probe_line(&spec.repo_root.join(".git/config")),
			"absent",
		),
		(
			"other repo code",
			probe_line(&spec.repo_root.join("bot-programming/old/bot.rs")),
			"absent",
		),
		(
			"toolbox",
			probe_line(&spec.toolbox[0].join("referee.rs")),
			"ro",
		),
		(
			"plugin",
			probe_line(&spec.plugin_ro[0].join("contest")),
			"ro",
		),
		(
			"contest code",
			probe_line(&spec.contest_root.join("src/main.rs")),
			"rw",
		),
		("config", probe_line(&run.config()), "ro"),
		(
			"own notes",
			probe_line(&run.task_dir(&task).join("notes.md")),
			"rw",
		),
		(
			"own spec",
			probe_line(&run.task_meta(&task).join("spec.toml")),
			"ro",
		),
		(
			"own spec body",
			probe_line(&run.task_meta(&task).join("spec.md")),
			"ro",
		),
		(
			"own state",
			probe_line(&run.task_meta(&task).join("state")),
			"ro",
		),
		("own meta dir", probe_dir_line(&run.task_meta(&task)), "ro"),
		(
			"contest .claude",
			probe_dir_line(&spec.contest_root.join(".claude")),
			"ro",
		),
		(
			"contest .direnv",
			probe_dir_line(&spec.contest_root.join(".direnv")),
			"ro",
		),
		(
			"contest .envrc",
			probe_line(&spec.contest_root.join(".envrc")),
			"ro",
		),
		(
			"session bus",
			probe_line(&Path::new("/run/user").join(uid().to_string()).join("bus")),
			"absent",
		),
		(
			"user runtime dirs",
			probe_empty_line(Path::new("/run/user")),
			"empty",
		),
		("var tmp", probe_empty_line(Path::new("/var/tmp")), "empty"),
		(
			"other notes",
			probe_line(&run.task_dir(&other).join("notes.md")),
			"ro",
		),
		(
			"own requests",
			probe_dir_line(&run.task_requests(&task)),
			"rw",
		),
		(
			"other requests",
			probe_dir_line(&run.task_requests(&other)),
			"ro",
		),
		(
			"jobs",
			probe_dir_line(&run.job_dir(&JobId::parse("j0001").unwrap())),
			"ro",
		),
		(
			"settings",
			probe_line(&spec.claude_config.join("settings.json")),
			"ro",
		),
		("config scratch", probe_dir_line(&spec.claude_config), "rw"),
		(
			"credentials",
			probe_line(&spec.claude_config.join(".credentials.json")),
			"rw",
		),
		("home", probe_dir_line(&spec.home), "rw"),
		(
			"nix daemon socket",
			probe_line(&Path::new(NIX_DAEMON_SOCKET_DIR).join("socket")),
			"absent",
		),
	];
	let script = checks
		.iter()
		.map(|(_, line, _)| line.as_str())
		.collect::<Vec<_>>()
		.join("\n");
	let lines: Vec<String> = sandboxed_sh(&spec, &script)
		.lines()
		.map(str::to_owned)
		.collect();
	assert_eq!(lines.len(), checks.len(), "{lines:?}");
	for ((name, _, expected), got) in checks.iter().zip(&lines) {
		assert_eq!(got, expected, "{name}");
	}
	assert!(
		fs::read_to_string(&spec.credentials)
			.unwrap()
			.ends_with("w\n")
	);
	assert!(!spec.claude_config.join("probe").exists());
	assert_eq!(fs::read_to_string(run.config()).unwrap(), "workers = 3\n");
}

fn uid() -> u32 {
	unsafe { libc::getuid() }
}

#[test]
fn real_bwrap_meta_stays_read_only_after_host_rewrite() {
	if !bwrap_available() {
		eprintln!("bwrap not found, skipped");
		return;
	}
	let Layout {
		_dir,
		spec,
		run,
		task,
		..
	} = layout();
	let state = run.task_meta(&task).join("state");
	let ready = run.task_dir(&task).join("ready");
	let rewritten = run.task_dir(&task).join("rewritten");
	let script = format!(
		"touch '{ready}'; while [ ! -e '{rewritten}' ]; do sleep 0.02; done; cat '{state}'; echo; {probe}",
		ready = text(&ready),
		rewritten = text(&rewritten),
		state = text(&state),
		probe = probe_line(&state),
	);
	let argv = vec!["sh".to_string(), "-c".to_string(), script];
	let command = bwrap_command(&spec, &argv);
	let mut child = Command::new(&command[0])
		.args(&command[1..])
		.stdout(std::process::Stdio::piped())
		.spawn()
		.unwrap();
	let deadline = std::time::Instant::now() + READY_TIMEOUT;
	while !ready.exists() {
		if std::time::Instant::now() >= deadline {
			let _ = child.kill();
			let _ = child.wait();
			panic!(
				"sandboxed shell never wrote {ready:?} within {seconds} s",
				seconds = READY_TIMEOUT.as_secs()
			);
		}
		std::thread::sleep(std::time::Duration::from_millis(20));
	}
	write_atomic(&state, b"running").unwrap();
	fs::write(&rewritten, "").unwrap();
	let output = child.wait_with_output().unwrap();
	assert_eq!(
		String::from_utf8_lossy(&output.stdout)
			.lines()
			.collect::<Vec<_>>(),
		["running", "ro"]
	);
	assert_eq!(fs::read_to_string(&state).unwrap(), "running");
}

#[test]
fn real_bwrap_hides_host_environment() {
	if !bwrap_available() {
		eprintln!("bwrap not found, skipped");
		return;
	}
	let Layout { _dir, spec, .. } = layout();
	let script = "echo ${SSH_AUTH_SOCK:-unset} ${XDG_RUNTIME_DIR:-unset} ${CLAUDE_CODE_MESSAGING_TOKEN:-unset} $CLAUDE_CONFIG_DIR";
	let argv = vec!["sh".to_string(), "-c".to_string(), script.to_string()];
	let command = bwrap_command(&spec, &argv);
	let output = Command::new(&command[0])
		.args(&command[1..])
		.env("SSH_AUTH_SOCK", "/run/agent.sock")
		.env("XDG_RUNTIME_DIR", "/run/user/1000")
		.env("CLAUDE_CODE_MESSAGING_TOKEN", "host-session")
		.env("CLAUDE_CONFIG_DIR", "/run-config")
		.env("HOME", &spec.home)
		.output()
		.unwrap();
	assert_eq!(
		String::from_utf8_lossy(&output.stdout).trim(),
		"unset unset unset /run-config"
	);
}

#[test]
fn claude_settings_carry_hook_timeouts_and_skills() {
	let dir = TempDir::new().unwrap();
	let run = RunDir {
		root: dir.path().join("contest"),
	};
	let config = Config {
		plugin_dir: dir.path().join("plugin"),
		bash_timeout_ms: 90000,
		..Config::default()
	};
	write_claude_settings(&run, &config).unwrap();
	write_claude_settings(&run, &config).unwrap();
	let settings: serde_json::Value =
		serde_json::from_slice(&fs::read(run.claude_config().join("settings.json")).unwrap())
			.unwrap();
	assert_eq!(
		settings["hooks"]["PreToolUse"],
		serde_json::json!([{
			"matcher": "*",
			"hooks": [{"type": "command", "command": "contest hook pre-tool-use"}],
		}])
	);
	assert_eq!(settings["env"]["BASH_DEFAULT_TIMEOUT_MS"], "90000");
	assert_eq!(settings["env"]["BASH_MAX_TIMEOUT_MS"], "90000");
	assert_eq!(settings["disableClaudeAiConnectors"], true);
	assert_eq!(
		fs::read_link(run.claude_config().join("skills")).unwrap(),
		config.plugin_dir.join("skills")
	);
}

#[test]
fn agent_env_points_at_run_config_and_contest_binary() {
	let run = RunDir { root: "/c".into() };
	let task = task();
	let env = agent_env(&run, Some(&task), Path::new("/plugin/bin"));
	let value = |key: &str| {
		env.iter()
			.find(|(name, _)| name == key)
			.map(|(_, value)| value.clone())
	};
	assert_eq!(value("CLAUDE_CONFIG_DIR").unwrap(), "/c/run/claude-config");
	assert_eq!(value("CONTEST_RUN").unwrap(), "/c");
	assert_eq!(value("CONTEST_TASK").unwrap(), "t0001-sim-bench");
	assert!(value("PATH").unwrap().starts_with("/plugin/bin:"));
	assert!(
		agent_env(&run, None, Path::new("/plugin/bin"))
			.iter()
			.all(|(name, _)| name != "CONTEST_TASK")
	);
}

#[test]
fn spec_for_run_uses_pinned_repo_and_plugin_dirs() {
	let dir = TempDir::new().unwrap();
	let run = RunDir {
		root: dir.path().join("repo/bot-programming/c1"),
	};
	fs::create_dir_all(&run.root).unwrap();
	let config = Config {
		repo_root: dir.path().join("repo"),
		plugin_dir: dir.path().join("repo/plugin"),
		toolbox: vec![dir.path().join("repo/bot-programming/c0")],
		..Config::default()
	};
	let spec = SandboxSpec::for_run(&run, &config, Some(task())).unwrap();
	let contest_root = fs::canonicalize(&run.root).unwrap();
	assert_eq!(spec.contest_root, contest_root);
	assert_eq!(spec.repo_root, config.repo_root);
	assert_eq!(spec.toolbox, config.toolbox);
	assert_eq!(spec.claude_config, contest_root.join("run/claude-config"));
	assert_eq!(
		spec.sandbox_home,
		contest_root.join("run/agents/t0001-sim-bench/home")
	);
	assert!(spec.sandbox_home.is_dir());
	assert!(contest_root.join(".claude").is_dir());
	assert!(contest_root.join(".direnv").is_dir());
	let manager = SandboxSpec::for_run(&run, &config, None).unwrap();
	assert_eq!(
		manager.sandbox_home,
		contest_root.join("run/agents/manager/home")
	);
	assert_eq!(spec.plugin_ro[0], contest_bin_dir().unwrap());
	assert_eq!(
		spec.plugin_ro[1..],
		["prompts", "skills", "kb"].map(|name| config.plugin_dir.join(name))[..]
	);
	assert!(spec.plugin_ro.iter().all(|path| !path.ends_with("corpus")));
	assert_eq!(
		spec.credentials,
		spec.home.join(".claude/.credentials.json")
	);
	assert_eq!(spec.own_task, Some(task()));
	assert_eq!(
		spec.dropped_host_env,
		dropped_env_names(std::env::vars().map(|(name, _)| name))
	);
	let unpinned = Config::default();
	assert!(SandboxSpec::for_run(&run, &unpinned, None).is_err());
}

#[test]
fn spec_for_run_refuses_symlinked_host_executed_dirs() {
	for name in [".claude", ".direnv"] {
		let dir = TempDir::new().unwrap();
		let run = RunDir {
			root: dir.path().join("repo/bot-programming/c1"),
		};
		fs::create_dir_all(&run.root).unwrap();
		let target = dir.path().join("elsewhere");
		fs::create_dir_all(&target).unwrap();
		std::os::unix::fs::symlink(&target, run.root.join(name)).unwrap();
		let config = Config {
			repo_root: dir.path().join("repo"),
			plugin_dir: dir.path().join("repo/plugin"),
			..Config::default()
		};
		let error = SandboxSpec::for_run(&run, &config, Some(task())).unwrap_err();
		assert!(
			format!("{error:#}").contains("symlink"),
			"{name}: {error:#}"
		);
	}
}

#[test]
fn dropped_env_names_cover_claude_session_and_secrets() {
	let names = [
		"CLAUDECODE",
		"CLAUDE_CODE_MESSAGING_TOKEN",
		"CLAUDE_CONFIG_DIR",
		"ANTHROPIC_API_KEY",
		"ANTHROPIC_BASE_URL",
		"GH_TOKEN",
		"GITHUB_TOKEN",
		"GITHUB_WORKSPACE",
		"HOME",
		"PATH",
	]
	.map(String::from);
	assert_eq!(
		dropped_env_names(names),
		[
			"ANTHROPIC_API_KEY",
			"ANTHROPIC_BASE_URL",
			"CLAUDECODE",
			"CLAUDE_CODE_MESSAGING_TOKEN",
			"GH_TOKEN",
			"GITHUB_TOKEN",
		]
		.map(String::from)
	);
}

fn credentials_file(dir: &TempDir, body: &str) -> std::path::PathBuf {
	let path = dir.path().join(".credentials.json");
	fs::write(&path, body).unwrap();
	path
}

fn now_ms() -> i64 {
	std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap()
		.as_millis() as i64
}

#[test]
fn credentials_expiry_reads_only_expires_at() {
	let dir = TempDir::new().unwrap();
	let soon = now_ms() + 5 * 60 * 1000;
	let later = now_ms() + 60 * 60 * 1000;
	let body = |expires: i64| {
		format!(
			r#"{{"claudeAiOauth":{{"accessToken":"a","refreshToken":"r","expiresAt":{expires},"scopes":["x"]}}}}"#
		)
	};
	let path = credentials_file(&dir, &body(soon));
	assert!(credentials_expire_within(&path, 15 * 60).unwrap());
	let path = credentials_file(&dir, &body(later));
	assert!(!credentials_expire_within(&path, 15 * 60).unwrap());
	let path = credentials_file(
		&dir,
		r#"{"claudeAiOauth":{"accessToken":"a","expiresAt":null}}"#,
	);
	assert!(!credentials_expire_within(&path, 15 * 60).unwrap());
}

#[test]
fn credentials_expiry_errors_never_echo_secrets() {
	let dir = TempDir::new().unwrap();
	for body in [
		r#"{"other":{"accessToken":"sk-secret-token"}}"#,
		r#"{"claudeAiOauth":{"accessToken":"sk-secret-token","expiresAt":"sk-secret-token"}}"#,
		"sk-secret-token not json",
	] {
		let path = credentials_file(&dir, body);
		let error = credentials_expire_within(&path, 60).unwrap_err();
		assert!(!format!("{error:#}").contains("sk-secret"), "{error:#}");
	}
	assert!(credentials_expire_within(&dir.path().join("missing"), 60).is_err());
}
