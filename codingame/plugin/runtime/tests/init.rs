use std::fs;
use std::path::Path;
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

fn run_init(contest_dir: &Path, plugin_dir: &Path, extra: &[&str]) {
	let status = Command::new(env!("CARGO_BIN_EXE_contest"))
		.arg("init")
		.arg(contest_dir)
		.args(["--puzzle", "seabed-security", "--mode", "contest-live"])
		.arg("--plugin-dir")
		.arg(plugin_dir)
		.args(extra)
		.status()
		.unwrap();
	assert!(status.success());
}

#[test]
fn init_creates_layout() {
	let repo = tempfile::tempdir().unwrap();
	git_repo_with_commit(repo.path());
	let contest_dir = repo.path().join("bot-programming/seabed");
	run_init(
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
	assert_eq!(ignore, "agents/\nrequests/\nclaude-config/\njobs/*/log\n");

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
	assert_eq!(config.blocked_url_substrings.len(), 4);
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
