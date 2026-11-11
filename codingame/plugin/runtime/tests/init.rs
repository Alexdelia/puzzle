use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use contest::atomic::write_atomic;
use contest::config::Config;
use contest::paths::RunDir;
use contest::run_info::{Mode, RunInfo};

fn git(dir: &Path, args: &[&str]) {
	let status = Command::new("git")
		.arg("-C")
		.arg(dir)
		.args(args)
		.status()
		.unwrap();
	assert!(status.success());
}

fn git_repo_with_commit(dir: &Path) {
	git(dir, &["init", "-q"]);
	fs::write(dir.join("f"), "x").unwrap();
	git(dir, &["add", "f"]);
	git(
		dir,
		&[
			"-c",
			"user.name=t",
			"-c",
			"user.email=t@t",
			"-c",
			"commit.gpgsign=false",
			"commit",
			"-q",
			"-m",
			"m",
		],
	);
}

fn init_command(contest_dir: &Path, plugin_dir: &Path) -> Command {
	let mut command = Command::new(env!("CARGO_BIN_EXE_contest"));
	command
		.arg("init")
		.arg(contest_dir)
		.args(["--puzzle", "seabed-security", "--mode", "contest-live"])
		.arg("--plugin-dir")
		.arg(plugin_dir);
	command
}

fn fake_pasta(dir: &Path) -> PathBuf {
	let bin = dir.join("fake-bin");
	fs::create_dir_all(&bin).unwrap();
	let pasta = bin.join("pasta");
	fs::write(&pasta, "#!/bin/sh\n").unwrap();
	fs::set_permissions(&pasta, fs::Permissions::from_mode(0o755)).unwrap();
	pasta
}

fn run_init(contest_dir: &Path, plugin_dir: &Path, extra: &[&str]) -> PathBuf {
	let outside_repo = tempfile::tempdir().unwrap();
	let pasta = fs::canonicalize(fake_pasta(outside_repo.path())).unwrap();
	let status = init_command(contest_dir, plugin_dir)
		.arg("--pasta-bin")
		.arg(&pasta)
		.args(extra)
		.status()
		.unwrap();
	assert!(status.success());
	pasta
}

#[test]
fn init_creates_layout() {
	let repo = tempfile::tempdir().unwrap();
	git_repo_with_commit(repo.path());
	let contest_dir = repo.path().join("bot-programming/seabed");
	let pasta = run_init(
		&contest_dir,
		repo.path(),
		&[
			"--active-budget-hours",
			"20",
			"--deadline",
			"2026-12-01T10:00:00Z",
		],
	);

	let run = RunDir {
		root: contest_dir.clone(),
	};
	for dir in [
		run.tasks(),
		run.jobs(),
		run.inbox(),
		run.requests(),
		run.agents(),
	] {
		assert!(dir.is_dir(), "{dir:?}");
	}
	let info: RunInfo = serde_json::from_slice(&fs::read(run.run_json()).unwrap()).unwrap();
	assert_eq!(info.phase, "start");
	assert_eq!(info.active_seconds, 0);
	assert_eq!(info.active_budget_seconds, Some(72000));
	assert_eq!(info.deadline_unix, Some(1_796_119_200));
	assert_eq!(info.mode, Mode::ContestLive);
	assert_eq!(info.puzzle, "seabed-security");
	assert_eq!(info.plugin_commit.len(), 40);

	let config = Config::load(&run.config()).unwrap();
	assert_eq!(config.repo_root, fs::canonicalize(repo.path()).unwrap());
	assert_eq!(
		config.git_common_dir,
		fs::canonicalize(repo.path().join(".git")).unwrap()
	);
	let ignore = fs::read_to_string(contest_dir.join("run/.gitignore")).unwrap();
	assert_eq!(
		ignore,
		"agents/\nrequests/\nclaude-config/\njobs/*/log\ngcroots/\ntasks/*/work/\nsupervisor.log\nsupervisor.pid\nrun.json.lock\nblocks\n"
	);
	assert_eq!(config.pasta_bin, pasta);

	let config = Config::load(&run.config()).unwrap();
	assert_eq!(config.plugin_dir, repo.path());
	assert_eq!(
		run.repo_root().unwrap(),
		fs::canonicalize(repo.path()).unwrap()
	);
	assert_eq!(run.replies(), contest_dir.join("run/replies.md"));
}

#[test]
fn init_outside_git_stores_empty_commit() {
	let plain = tempfile::tempdir().unwrap();
	let contest_dir = plain.path().join("c");
	run_init(&contest_dir, plain.path(), &[]);
	let run = RunDir { root: contest_dir };
	let info: RunInfo = serde_json::from_slice(&fs::read(run.run_json()).unwrap()).unwrap();
	assert_eq!(info.plugin_commit, "");
	let config = Config::load(&run.config()).unwrap();
	assert!(config.repo_root.as_os_str().is_empty());
	assert!(config.git_common_dir.as_os_str().is_empty());
	assert_eq!(info.active_budget_seconds, None);
	assert_eq!(info.deadline_unix, None);
}

#[test]
fn config_defaults_round_trip() {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("config.toml");
	fs::write(&path, "").unwrap();
	let config = Config::load(&path).unwrap();
	assert_eq!(config, Config::default());
	assert_eq!(config.workers, 3);
	assert_eq!(config.grace_seconds, 270);
	assert_eq!(config.five_hour_limit_percent, 97);
	assert_eq!(config.ram_budget_mb, 13000);
	assert_eq!(config.claude_bin, Path::new("claude"));
	assert!(config.sandbox);
	assert_eq!(
		config.blocked_url_substrings,
		[
			"alexdelia",
			"discord.com",
			"discord.gg",
			"discordapp.com",
			"discordapp.net",
			"discord.media"
		]
	);
	assert!(config.plugin_dir.join("runtime/Cargo.toml").is_file());

	fs::write(&path, "workers = 5\n").unwrap();
	let config = Config::load(&path).unwrap();
	assert_eq!(config.workers, 5);
	assert_eq!(config.grace_seconds, 270);

	let serialized = toml::to_string(&config).unwrap();
	assert_eq!(toml::from_str::<Config>(&serialized).unwrap(), config);
}

#[test]
fn write_atomic_never_partial() {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("state.json");
	write_atomic(&path, b"old").unwrap();

	fs::write(
		dir.path().join(".state.json.tmp.99999"),
		vec![b'z'; 500_000],
	)
	.unwrap();
	assert_eq!(fs::read(&path).unwrap(), b"old");

	let big = vec![b'n'; 1_000_000];
	write_atomic(&path, &big).unwrap();
	assert_eq!(fs::read(&path).unwrap(), big);
	assert_eq!(
		fs::read(dir.path().join(".state.json.tmp.99999"))
			.unwrap()
			.len(),
		500_000
	);
}

#[test]
fn write_atomic_concurrent_writers_leave_no_temp_files() {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("state.json");
	let payloads: Vec<Vec<u8>> = (0..8u8).map(|n| vec![b'a' + n; 200_000]).collect();

	std::thread::scope(|scope| {
		for payload in &payloads {
			scope.spawn(|| {
				for _ in 0..20 {
					write_atomic(&path, payload).unwrap();
				}
			});
		}
	});

	assert!(payloads.contains(&fs::read(&path).unwrap()));
	let names: Vec<_> = fs::read_dir(dir.path())
		.unwrap()
		.map(|entry| entry.unwrap().file_name())
		.collect();
	assert_eq!(names, vec![std::ffi::OsString::from("state.json")]);
}

fn path_with_first(dir: &Path) -> std::ffi::OsString {
	let rest = std::env::var_os("PATH").unwrap_or_default();
	std::env::join_paths(std::iter::once(dir.to_path_buf()).chain(std::env::split_paths(&rest)))
		.unwrap()
}

#[test]
fn init_pins_pasta_found_on_path() {
	let plain = tempfile::tempdir().unwrap();
	let contest_dir = plain.path().join("c");
	let pasta = fake_pasta(plain.path());
	let status = init_command(&contest_dir, plain.path())
		.env("PATH", path_with_first(pasta.parent().unwrap()))
		.status()
		.unwrap();
	assert!(status.success());
	let config = Config::load(&RunDir { root: contest_dir }.config()).unwrap();
	assert_eq!(config.pasta_bin, pasta);
}

fn path_without_pasta() -> std::ffi::OsString {
	let rest = std::env::var_os("PATH").unwrap_or_default();
	std::env::join_paths(std::env::split_paths(&rest).filter(|dir| !dir.join("pasta").exists()))
		.unwrap()
}

fn nix_available() -> bool {
	Command::new("nix")
		.arg("--version")
		.output()
		.is_ok_and(|output| output.status.success())
}

fn nix_builds_passt() -> bool {
	Command::new("nix")
		.args(["build", "nixpkgs#passt", "--no-link"])
		.output()
		.is_ok_and(|output| output.status.success())
}

#[test]
fn init_pins_pasta_from_nix_with_gc_root() {
	if !nix_available() || !nix_builds_passt() {
		eprintln!("nix missing or nix build nixpkgs#passt failed (offline?), skipped");
		return;
	}
	let plain = tempfile::tempdir().unwrap();
	let contest_dir = plain.path().join("c");
	let status = init_command(&contest_dir, plain.path())
		.env("PATH", path_without_pasta())
		.status()
		.unwrap();
	assert!(status.success());
	let run = RunDir { root: contest_dir };
	let config = Config::load(&run.config()).unwrap();
	assert!(config.pasta_bin.starts_with("/nix/store"), "{config:?}");
	assert!(config.pasta_bin.ends_with("bin/pasta"), "{config:?}");
	assert!(config.pasta_bin.is_file());
	let gc_root = fs::read_link(run.root.join("run/gcroots/passt")).unwrap();
	assert_eq!(config.pasta_bin, gc_root.join("bin/pasta"));
}

fn init_with_pasta_status(pasta: &Path) -> std::process::ExitStatus {
	let plain = tempfile::tempdir().unwrap();
	init_command(&plain.path().join("c"), plain.path())
		.arg("--pasta-bin")
		.arg(pasta)
		.status()
		.unwrap()
}

#[test]
fn init_refuses_non_executable_pasta_bin() {
	let dir = tempfile::tempdir().unwrap();
	let plain_file = dir.path().join("pasta");
	fs::write(&plain_file, "").unwrap();
	assert!(!init_with_pasta_status(&plain_file).success());
	assert!(!init_with_pasta_status(dir.path()).success());
}

#[test]
fn init_refuses_pasta_bin_inside_repo() {
	let repo = tempfile::tempdir().unwrap();
	git_repo_with_commit(repo.path());
	let pasta = fake_pasta(repo.path());
	let status = init_command(&repo.path().join("bot-programming/c"), repo.path())
		.arg("--pasta-bin")
		.arg(&pasta)
		.status()
		.unwrap();
	assert!(!status.success());
}

#[test]
fn init_skips_pasta_on_path_inside_repo() {
	let repo = tempfile::tempdir().unwrap();
	git_repo_with_commit(repo.path());
	let planted = fake_pasta(repo.path());
	let outside = tempfile::tempdir().unwrap();
	let trusted = fake_pasta(outside.path());
	let path = std::env::join_paths(
		[planted.parent().unwrap(), trusted.parent().unwrap()]
			.into_iter()
			.map(Path::to_path_buf)
			.chain(std::env::split_paths(
				&std::env::var_os("PATH").unwrap_or_default(),
			)),
	)
	.unwrap();
	let contest_dir = repo.path().join("bot-programming/c");
	let status = init_command(&contest_dir, repo.path())
		.env("PATH", path)
		.status()
		.unwrap();
	assert!(status.success());
	let config = Config::load(&RunDir { root: contest_dir }.config()).unwrap();
	assert_eq!(config.pasta_bin, trusted);
}

#[test]
fn init_refuses_contest_dir_holding_git() {
	let repo = tempfile::tempdir().unwrap();
	git_repo_with_commit(repo.path());
	let output = init_command(repo.path(), repo.path()).output().unwrap();
	assert!(!output.status.success());
	assert!(String::from_utf8_lossy(&output.stderr).contains(".git"));
	assert!(!repo.path().join("run").exists());
}

#[test]
fn init_pins_contest_envrc_presence() {
	let repo = tempfile::tempdir().unwrap();
	git_repo_with_commit(repo.path());
	let plain = repo.path().join("bot-programming/plain");
	run_init(&plain, repo.path(), &[]);
	assert!(
		!Config::load(&RunDir { root: plain }.config())
			.unwrap()
			.contest_envrc
	);
	let with_envrc = repo.path().join("bot-programming/with-envrc");
	fs::create_dir_all(&with_envrc).unwrap();
	fs::write(with_envrc.join(".envrc"), "use flake").unwrap();
	run_init(&with_envrc, repo.path(), &[]);
	let run = RunDir { root: with_envrc };
	assert!(Config::load(&run.config()).unwrap().contest_envrc);
	let linked = repo.path().join("bot-programming/linked");
	fs::create_dir_all(&linked).unwrap();
	std::os::unix::fs::symlink("/etc/hostname", linked.join(".envrc")).unwrap();
	let output = init_command(&linked, repo.path()).output().unwrap();
	assert!(!output.status.success());
	assert!(String::from_utf8_lossy(&output.stderr).contains(".envrc"));
	assert!(!linked.join("run").exists());
}

fn assert_refused(output: std::process::Output, needles: &[&str], contest_dir: &Path) {
	let stderr = String::from_utf8_lossy(&output.stderr);
	assert!(!output.status.success(), "{stderr}");
	for needle in needles {
		assert!(stderr.contains(needle), "{needle} not in {stderr}");
	}
	assert!(!contest_dir.join("run").exists());
}

#[test]
fn init_refuses_contest_dir_holding_plugin_dir() {
	let repo = tempfile::tempdir().unwrap();
	git_repo_with_commit(repo.path());
	let contest_dir = repo.path().join("codingame");
	let plugin = contest_dir.join("plugin");
	fs::create_dir_all(&plugin).unwrap();
	let output = init_command(&contest_dir, &plugin).output().unwrap();
	assert_refused(output, &["plugin_dir", "dedicated"], &contest_dir);
	let output = init_command(&plugin, &plugin).output().unwrap();
	assert_refused(output, &["plugin_dir"], &plugin);
}

fn copied_contest_binary(dir: &Path) -> PathBuf {
	let bin = dir.join("bin");
	fs::create_dir_all(&bin).unwrap();
	let copy = bin.join("contest");
	fs::copy(env!("CARGO_BIN_EXE_contest"), &copy).unwrap();
	copy
}

fn run_copied_init(binary: &Path, contest_dir: &Path, extra: &[&Path]) -> std::process::Output {
	let mut attempts = 0;
	loop {
		let output = Command::new(binary)
			.arg("init")
			.arg(contest_dir)
			.args(["--puzzle", "p", "--mode", "contest-live"])
			.args(extra)
			.output();
		match output {
			Err(error)
				if error.kind() == std::io::ErrorKind::ExecutableFileBusy && attempts < 100 =>
			{
				attempts += 1;
				std::thread::sleep(std::time::Duration::from_millis(20));
			}
			output => return output.unwrap(),
		}
	}
}

#[test]
fn init_refuses_contest_dir_holding_contest_binary() {
	let outside = tempfile::tempdir().unwrap();
	let binary = copied_contest_binary(outside.path());
	let plugin = tempfile::tempdir().unwrap();
	let output = run_copied_init(
		&binary,
		outside.path(),
		&[Path::new("--plugin-dir"), plugin.path()],
	);
	assert_refused(output, &["contest binary"], outside.path());
}

#[test]
fn init_refuses_contest_dir_holding_git_common_dir() {
	let dir = tempfile::tempdir().unwrap();
	let repo = dir.path().join("repo");
	let contest_dir = repo.join("contest");
	fs::create_dir_all(&contest_dir).unwrap();
	let git_dir = contest_dir.join("store");
	let status = Command::new("git")
		.args(["init", "-q", "--separate-git-dir"])
		.arg(&git_dir)
		.arg(&repo)
		.status()
		.unwrap();
	assert!(status.success());
	let output = init_command(&contest_dir, dir.path()).output().unwrap();
	assert_refused(output, &["git_common_dir"], &contest_dir);
}

#[test]
fn init_refuses_planted_names_below_top_level_and_lists_them() {
	let repo = tempfile::tempdir().unwrap();
	git_repo_with_commit(repo.path());
	let contest_dir = repo.path().join("bot-programming");
	let planted = [
		contest_dir.join("other-contest/.envrc"),
		contest_dir.join("other-contest/.claude/settings.json"),
		contest_dir.join("vendored/.git/config"),
	];
	for path in &planted {
		fs::create_dir_all(path.parent().unwrap()).unwrap();
		fs::write(path, "x").unwrap();
	}
	let output = init_command(&contest_dir, repo.path()).output().unwrap();
	assert_refused(
		output,
		&[
			"other-contest/.envrc",
			"other-contest/.claude",
			"vendored/.git",
			"dedicated",
		],
		&contest_dir,
	);
	for path in &planted {
		assert!(path.is_file(), "{path:?}");
	}
}

#[test]
fn init_refuses_unresolvable_plugin_dir() {
	let outside = tempfile::tempdir().unwrap();
	let binary = copied_contest_binary(outside.path());
	let contest_dir = outside.path().join("work/contest");
	let output = run_copied_init(&binary, &contest_dir, &[]);
	assert_refused(output, &["plugin_dir"], &contest_dir);
}

#[test]
fn init_refuses_existing_run() {
	let repo = tempfile::tempdir().unwrap();
	git_repo_with_commit(repo.path());
	let contest_dir = repo.path().join("contest");
	run_init(&contest_dir, repo.path(), &["--active-budget-hours", "2"]);
	let run = RunDir {
		root: fs::canonicalize(&contest_dir).unwrap(),
	};
	let before = fs::read(run.run_json()).unwrap();
	let outside_repo = tempfile::tempdir().unwrap();
	let output = init_command(&contest_dir, repo.path())
		.arg("--pasta-bin")
		.arg(fake_pasta(outside_repo.path()))
		.output()
		.unwrap();
	assert!(!output.status.success());
	assert!(
		String::from_utf8_lossy(&output.stderr).contains("contest extend"),
		"{stderr}",
		stderr = String::from_utf8_lossy(&output.stderr)
	);
	assert_eq!(fs::read(run.run_json()).unwrap(), before);
}

#[test]
fn extend_updates_limits_and_keeps_progress() {
	let repo = tempfile::tempdir().unwrap();
	git_repo_with_commit(repo.path());
	let contest_dir = repo.path().join("contest");
	run_init(&contest_dir, repo.path(), &["--active-budget-hours", "2"]);
	let run = RunDir {
		root: fs::canonicalize(&contest_dir).unwrap(),
	};
	let mut info: RunInfo = serde_json::from_slice(&fs::read(run.run_json()).unwrap()).unwrap();
	info.active_seconds = 1234;
	info.phase = "bots".into();
	write_atomic(&run.run_json(), &serde_json::to_vec(&info).unwrap()).unwrap();
	let output = Command::new(env!("CARGO_BIN_EXE_contest"))
		.arg("extend")
		.arg(&contest_dir)
		.args(["--active-budget-hours", "5"])
		.args(["--deadline", "2030-01-02T03:04:05Z"])
		.output()
		.unwrap();
	assert!(output.status.success(), "{output:?}");
	let extended: RunInfo = serde_json::from_slice(&fs::read(run.run_json()).unwrap()).unwrap();
	assert_eq!(extended.active_budget_seconds, Some(5 * 3600));
	assert_eq!(extended.deadline_unix, Some(1893553445));
	assert_eq!(extended.active_seconds, 1234);
	assert_eq!(extended.phase, "bots");
	let refused = Command::new(env!("CARGO_BIN_EXE_contest"))
		.arg("extend")
		.arg(&contest_dir)
		.output()
		.unwrap();
	assert!(!refused.status.success());
}

#[test]
fn init_refuses_relative_path_entries() {
	let repo = tempfile::tempdir().unwrap();
	git_repo_with_commit(repo.path());
	let contest_dir = repo.path().join("contest");
	let outside_repo = tempfile::tempdir().unwrap();
	let mut entries = vec![PathBuf::from("bin")];
	entries.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
	let output = init_command(&contest_dir, repo.path())
		.arg("--pasta-bin")
		.arg(fake_pasta(outside_repo.path()))
		.env("PATH", std::env::join_paths(entries).unwrap())
		.output()
		.unwrap();
	assert!(!output.status.success());
	let stderr = String::from_utf8_lossy(&output.stderr);
	assert!(stderr.contains("relative"), "{stderr}");
	assert!(!contest_dir.join("run/run.json").exists());
}

#[test]
fn extend_refuses_a_total_budget_already_used() {
	let repo = tempfile::tempdir().unwrap();
	git_repo_with_commit(repo.path());
	let contest_dir = repo.path().join("contest");
	run_init(&contest_dir, repo.path(), &["--active-budget-hours", "2"]);
	let run = RunDir {
		root: fs::canonicalize(&contest_dir).unwrap(),
	};
	let mut info: RunInfo = serde_json::from_slice(&fs::read(run.run_json()).unwrap()).unwrap();
	info.active_seconds = 3 * 3600 + 1800;
	write_atomic(&run.run_json(), &serde_json::to_vec(&info).unwrap()).unwrap();
	let before = fs::read(run.run_json()).unwrap();
	let output = Command::new(env!("CARGO_BIN_EXE_contest"))
		.arg("extend")
		.arg(&contest_dir)
		.args(["--active-budget-hours", "3"])
		.output()
		.unwrap();
	assert!(!output.status.success());
	let stderr = String::from_utf8_lossy(&output.stderr);
	assert!(stderr.contains("3.5 h"), "{stderr}");
	assert!(stderr.contains("total"), "{stderr}");
	assert_eq!(fs::read(run.run_json()).unwrap(), before);
	let help = Command::new(env!("CARGO_BIN_EXE_contest"))
		.args(["extend", "--help"])
		.output()
		.unwrap();
	assert!(String::from_utf8_lossy(&help.stdout).contains("TOTAL"));
}
