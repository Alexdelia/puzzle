use std::fs::{self, File, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};

use crate::config::Config;
use crate::paths::{RunDir, TaskId};
use crate::run_info::RunInfo;
use crate::task::TaskStore;

const MILESTONE_MAX_CHARS: usize = 72;
const LOCK_FILE: &str = "contest-commit.lock";

struct CommitLock(File);

impl CommitLock {
	fn acquire(repo: &Path) -> Result<Self> {
		let file = OpenOptions::new()
			.create(true)
			.write(true)
			.truncate(false)
			.open(repo.join(".git").join(LOCK_FILE))
			.context("open commit lock")?;
		let locked = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
		ensure!(locked == 0, "flock: {}", std::io::Error::last_os_error());
		Ok(Self(file))
	}
}

impl Drop for CommitLock {
	fn drop(&mut self) {
		unsafe {
			libc::flock(self.0.as_raw_fd(), libc::LOCK_UN);
		}
	}
}

fn git(repo: &Path, args: &[&str], paths: &[PathBuf]) -> Result<std::process::Output> {
	let mut command = Command::new("git");
	command.arg("-C").arg(repo).args(args);
	if !paths.is_empty() {
		command.arg("--").args(paths);
	}
	command
		.output()
		.with_context(|| format!("run git {args:?}"))
}

fn git_checked(repo: &Path, args: &[&str], paths: &[PathBuf]) -> Result<String> {
	let output = git(repo, args, paths)?;
	if !output.status.success() {
		bail!(
			"git {args:?} failed: {stderr}",
			stderr = String::from_utf8_lossy(&output.stderr).trim()
		);
	}
	Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn is_tracked(repo: &Path, path: &Path) -> Result<bool> {
	let listed = git_checked(repo, &["ls-files"], &[path.to_path_buf()])?;
	Ok(!listed.is_empty())
}

fn declared_write(contest_dir: &Path, relative: &Path) -> Result<PathBuf> {
	ensure!(
		!relative.is_absolute()
			&& !relative
				.components()
				.any(|part| part == Component::ParentDir),
		"write path {relative:?} must be relative without .."
	);
	Ok(contest_dir.join(relative))
}

fn milestone_line(milestone: &str) -> String {
	milestone
		.lines()
		.next()
		.unwrap_or_default()
		.chars()
		.filter(|c| !c.is_control())
		.take(MILESTONE_MAX_CHARS)
		.collect::<String>()
		.trim()
		.to_owned()
}

pub fn commit_task(
	run: &RunDir,
	config: &Config,
	task: &TaskId,
	milestone: &str,
) -> Result<Option<String>> {
	let store = TaskStore { run };
	let spec = store.spec(task)?;
	let info: RunInfo = serde_json::from_slice(&fs::read(run.run_json())?)?;
	let mut candidates = vec![run.task_dir(task)];
	for relative in &spec.writes {
		candidates.push(declared_write(&run.root, relative)?);
	}
	let repo = run.repo_root()?;
	let mut paths = Vec::new();
	for candidate in candidates {
		if candidate.exists() || is_tracked(&repo, &candidate)? {
			paths.push(candidate);
		}
	}
	let message = format!(
		"{kind}({puzzle}): {task} {milestone}\n\n{trailer}",
		kind = spec.kind,
		puzzle = info.puzzle,
		milestone = milestone_line(milestone),
		trailer = config.commit_trailer
	);

	let _lock = CommitLock::acquire(&repo)?;
	git_checked(&repo, &["add", "-A"], &paths)?;
	let unchanged = git(&repo, &["diff", "--cached", "--quiet"], &paths)?;
	match unchanged.status.code() {
		Some(0) => return Ok(None),
		Some(1) => {}
		_ => bail!("git diff --cached failed"),
	}
	git_checked(
		&repo,
		&["commit", "-q", "--no-verify", "-m", &message],
		&paths,
	)?;
	Ok(Some(git_checked(&repo, &["rev-parse", "HEAD"], &[])?))
}
