use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use contest::atomic::write_atomic;
use contest::commit::commit_task;
use contest::config::Config;
use contest::paths::{RunDir, TaskId};
use contest::run_info::{Mode, RunInfo};
use contest::task::{TaskSpec, TaskStore};
use tempfile::TempDir;

fn git(dir: &Path, args: &[&str]) -> String {
	let output = Command::new("git")
		.arg("-C")
		.arg(dir)
		.args(args)
		.output()
		.unwrap();
	assert!(output.status.success(), "git {args:?}");
	String::from_utf8_lossy(&output.stdout).into_owned()
}

fn setup(writes: &[&str]) -> (TempDir, RunDir, TaskId) {
	let repo = TempDir::new().unwrap();
	git(repo.path(), &["init", "-q"]);
	git(repo.path(), &["config", "user.name", "t"]);
	git(repo.path(), &["config", "user.email", "t@t"]);
	git(repo.path(), &["config", "commit.gpgsign", "false"]);
	fs::write(repo.path().join("seed"), "x").unwrap();
	git(repo.path(), &["add", "seed"]);
	git(repo.path(), &["commit", "-q", "--no-verify", "-m", "seed"]);
	let run = RunDir {
		root: repo.path().to_path_buf(),
	};
	fs::create_dir_all(run.tasks()).unwrap();
	let info = RunInfo {
		puzzle: "seabed-security".into(),
		mode: Mode::ContestLive,
		deadline_unix: None,
		active_budget_seconds: None,
		active_seconds: 0,
		phase: "start".into(),
		plugin_commit: String::new(),
		started_at: 0,
	};
	write_atomic(&run.run_json(), &serde_json::to_vec(&info).unwrap()).unwrap();
	let id = TaskStore { run: &run }
		.create(TaskSpec {
			kind: "referee-port".into(),
			title: "port".into(),
			priority: 0,
			model: None,
			budget_usd: None,
			writes: writes.iter().map(PathBuf::from).collect(),
			body: String::new(),
			parent: None,
		})
		.unwrap();
	(repo, run, id)
}

fn write(run: &RunDir, relative: &str) {
	let path = run.root.join(relative);
	fs::create_dir_all(path.parent().unwrap()).unwrap();
	fs::write(path, "content").unwrap();
}

#[test]
fn commit_only_task_paths() {
	let (repo, run, id) = setup(&["referee/a.rs"]);
	write(&run, "referee/a.rs");
	write(&run, "other/b.rs");
	let sha = commit_task(&run, &Config::default(), &id, "m").unwrap();
	assert!(sha.is_some());
	let files = git(repo.path(), &["show", "--name-only", "--format=", "HEAD"]);
	assert!(files.contains("referee/a.rs"));
	assert!(files.contains(&format!("run/tasks/{id}/spec.toml")));
	assert!(!files.contains("other/b.rs"));
	let status = git(repo.path(), &["status", "--porcelain"]);
	assert!(status.contains("?? other/"));
}

#[test]
fn commit_message_template() {
	let (repo, run, id) = setup(&["referee/a.rs"]);
	write(&run, "referee/a.rs");
	commit_task(&run, &Config::default(), &id, "fix sim\nrm -rf").unwrap();
	let message = git(repo.path(), &["log", "-1", "--format=%B"]);
	let subject = message.lines().next().unwrap();
	assert_eq!(
		subject,
		format!("referee-port(seabed-security): {id} fix sim")
	);
	assert!(message.contains(&Config::default().commit_trailer));
}

#[test]
fn milestone_is_sanitised_and_capped() {
	let (repo, run, id) = setup(&["referee/a.rs"]);
	write(&run, "referee/a.rs");
	let long = format!("a\u{1b}b{}", "z".repeat(200));
	commit_task(&run, &Config::default(), &id, &long).unwrap();
	let subject = git(repo.path(), &["log", "-1", "--format=%s"]);
	let milestone = subject.trim().split_once(&format!("{id} ")).unwrap().1;
	assert_eq!(milestone.chars().count(), 72);
	assert!(milestone.starts_with("ab"));
}

#[test]
fn writes_with_dotdot_rejected() {
	let (_repo, run, id) = setup(&["../escape"]);
	assert!(commit_task(&run, &Config::default(), &id, "m").is_err());
}

#[test]
fn absolute_writes_rejected() {
	let (_repo, run, id) = setup(&["/etc/passwd"]);
	assert!(commit_task(&run, &Config::default(), &id, "m").is_err());
}

#[test]
fn nothing_to_commit_returns_none() {
	let (repo, run, id) = setup(&["referee/a.rs"]);
	assert!(
		commit_task(&run, &Config::default(), &id, "one")
			.unwrap()
			.is_some()
	);
	let before = git(repo.path(), &["rev-parse", "HEAD"]);
	assert_eq!(
		commit_task(&run, &Config::default(), &id, "two").unwrap(),
		None
	);
	assert_eq!(git(repo.path(), &["rev-parse", "HEAD"]), before);
}
