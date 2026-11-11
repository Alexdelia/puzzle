use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use contest::commit::commit_task;
use contest::config::Config;
use contest::paths::{RunDir, TaskId};
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

fn config(run: &RunDir) -> Config {
	Config {
		repo_root: run.root.clone(),
		git_common_dir: run.root.join(".git"),
		..Config::default()
	}
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
	let sha = commit_task(&run, &config(&run), &id, "m").unwrap();
	assert!(sha.is_some());
	let files = git(repo.path(), &["show", "--name-only", "--format=", "HEAD"]);
	assert!(files.contains("referee/a.rs"));
	assert!(files.contains(&format!("run/tasks/{id}/meta/spec.toml")));
	assert!(!files.contains("other/b.rs"));
	let status = git(repo.path(), &["status", "--porcelain"]);
	assert!(status.contains("?? other/"));
}

#[test]
fn commit_message_template() {
	let (repo, run, id) = setup(&["referee/a.rs"]);
	write(&run, "referee/a.rs");
	commit_task(&run, &config(&run), &id, "fix sim\nrm -rf").unwrap();
	let message = git(repo.path(), &["log", "-1", "--format=%B"]);
	let subject = message.lines().next().unwrap();
	assert_eq!(subject, format!("referee-port: {id} fix sim"));
	assert_eq!(message.trim_end(), subject);
}

#[test]
fn non_empty_trailer_is_appended_after_blank_line() {
	let (repo, run, id) = setup(&["referee/a.rs"]);
	write(&run, "referee/a.rs");
	let config = Config {
		commit_trailer: "Trailer: yes".to_owned(),
		..config(&run)
	};
	commit_task(&run, &config, &id, "m").unwrap();
	let message = git(repo.path(), &["log", "-1", "--format=%B"]);
	assert_eq!(
		message.trim_end(),
		format!("referee-port: {id} m\n\nTrailer: yes")
	);
}

#[test]
fn milestone_is_sanitised_and_capped() {
	let (repo, run, id) = setup(&["referee/a.rs"]);
	write(&run, "referee/a.rs");
	let long = format!("a\u{1b}b{tail}", tail = "z".repeat(200));
	commit_task(&run, &config(&run), &id, &long).unwrap();
	let subject = git(repo.path(), &["log", "-1", "--format=%s"]);
	let milestone = subject.trim().split_once(&format!("{id} ")).unwrap().1;
	assert_eq!(milestone.chars().count(), 72);
	assert!(milestone.starts_with("ab"));
}

#[test]
fn writes_with_dotdot_rejected() {
	let (_repo, run, id) = setup(&["../escape"]);
	assert!(commit_task(&run, &config(&run), &id, "m").is_err());
}

#[test]
fn absolute_writes_rejected() {
	let (_repo, run, id) = setup(&["/etc/passwd"]);
	assert!(commit_task(&run, &config(&run), &id, "m").is_err());
}

#[test]
fn nothing_to_commit_returns_none() {
	let (repo, run, id) = setup(&["referee/a.rs"]);
	assert!(
		commit_task(&run, &config(&run), &id, "one")
			.unwrap()
			.is_some()
	);
	let before = git(repo.path(), &["rev-parse", "HEAD"]);
	assert_eq!(commit_task(&run, &config(&run), &id, "two").unwrap(), None);
	assert_eq!(git(repo.path(), &["rev-parse", "HEAD"]), before);
}

fn committed_files(repo: &TempDir) -> String {
	git(repo.path(), &["show", "--name-only", "--format=", "HEAD"])
}

#[test]
fn unsafe_writes_are_rejected() {
	for bad in [
		"",
		".",
		"./a",
		"x/../y",
		"x/.git/config",
		".git",
		"run/state.json",
		"run/tasks/t0002-x",
		"run",
		"run/tasks",
	] {
		let (_repo, run, id) = setup(&[bad]);
		assert!(
			commit_task(&run, &config(&run), &id, "m").is_err(),
			"{bad:?}"
		);
	}
}

#[test]
fn glob_write_is_literal() {
	let (repo, run, id) = setup(&["*"]);
	write(&run, "other/b.rs");
	commit_task(&run, &config(&run), &id, "m").unwrap();
	let files = committed_files(&repo);
	assert!(!files.contains("other/b.rs"));
	assert!(!files.contains("seed"));
}

#[test]
fn own_task_dir_write_is_allowed() {
	let (repo, run, id) = setup(&["run/tasks/t0001-referee-port/result.md"]);
	assert_eq!(id.0, "t0001-referee-port");
	write(&run, "run/tasks/t0001-referee-port/result.md");
	commit_task(&run, &config(&run), &id, "m").unwrap();
	assert!(committed_files(&repo).contains("result.md"));
}

#[test]
fn planted_git_dir_in_contest_dir_is_ignored() {
	let (repo, run, id) = setup(&["referee/a.rs"]);
	write(&run, "referee/a.rs");
	let nested = TempDir::new().unwrap();
	let mut pinned = config(&run);
	pinned.repo_root = repo.path().to_path_buf();
	pinned.git_common_dir = repo.path().join(".git");
	git(nested.path(), &["init", "-q"]);
	commit_task(&run, &pinned, &id, "m").unwrap();
	assert!(committed_files(&repo).contains("referee/a.rs"));
}

#[test]
fn unpinned_repo_is_an_error() {
	let (_repo, run, id) = setup(&["referee/a.rs"]);
	assert!(commit_task(&run, &Config::default(), &id, "m").is_err());
}

struct PlantedRepo {
	marker: PathBuf,
	_hooks: TempDir,
}

fn plant_repo(dir: &Path) -> PlantedRepo {
	fs::create_dir_all(dir).unwrap();
	git(dir, &["init", "-q"]);
	git(dir, &["config", "user.name", "t"]);
	git(dir, &["config", "user.email", "t@t"]);
	git(dir, &["config", "commit.gpgsign", "false"]);
	fs::write(dir.join("inner.rs"), "inner").unwrap();
	git(dir, &["add", "inner.rs"]);
	git(dir, &["commit", "-q", "--no-verify", "-m", "inner"]);
	let hooks = TempDir::new().unwrap();
	let marker = hooks.path().join("fsmonitor-ran");
	let hook = hooks.path().join("fsmonitor.sh");
	fs::write(
		&hook,
		format!("#!/bin/sh\ntouch '{marker}'\n", marker = marker.display()),
	)
	.unwrap();
	fs::set_permissions(&hook, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
	let config = dir.join(".git/config");
	let mut text = fs::read_to_string(&config).unwrap();
	text.push_str(&format!(
		"[core]\n\tfsmonitor = {hook}\n",
		hook = hook.display()
	));
	fs::write(&config, text).unwrap();
	PlantedRepo {
		marker,
		_hooks: hooks,
	}
}

fn staged(repo: &TempDir) -> String {
	git(repo.path(), &["diff", "--cached", "--name-only"])
}

#[test]
fn embedded_repo_in_declared_write_aborts() {
	let (repo, run, id) = setup(&["bot"]);
	write(&run, "bot/main.rs");
	let planted = plant_repo(&run.root.join("bot/sub"));
	let before = git(repo.path(), &["rev-parse", "HEAD"]);
	let error = commit_task(&run, &config(&run), &id, "m").unwrap_err();
	assert!(format!("{error:#}").contains("embedded"), "{error:#}");
	assert_eq!(git(repo.path(), &["rev-parse", "HEAD"]), before);
	assert_eq!(staged(&repo), "");
	assert!(!planted.marker.exists());
}

#[test]
fn embedded_repo_in_own_task_dir_aborts() {
	let (repo, run, id) = setup(&[]);
	let planted = plant_repo(&run.task_dir(&id).join("sub"));
	let before = git(repo.path(), &["rev-parse", "HEAD"]);
	let error = commit_task(&run, &config(&run), &id, "m").unwrap_err();
	assert!(format!("{error:#}").contains("embedded"), "{error:#}");
	assert_eq!(git(repo.path(), &["rev-parse", "HEAD"]), before);
	assert_eq!(staged(&repo), "");
	assert!(!planted.marker.exists());
}
