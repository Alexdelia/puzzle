use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};

use crate::config::Config;
use crate::paths::{RunDir, TaskId, is_valid_kind};
use crate::task::TaskStore;

const MILESTONE_MAX_CHARS: usize = 72;
const LOCK_FILE: &str = "contest-commit.lock";
const GIT_DIR_NAME: &str = ".git";
const GITLINK_MODE: &str = "160000";

struct CommitLock(File);

impl CommitLock {
	fn acquire(git_common_dir: &Path) -> Result<Self> {
		let file = OpenOptions::new()
			.create(true)
			.write(true)
			.truncate(false)
			.open(git_common_dir.join(LOCK_FILE))
			.context("open commit lock")?;
		let locked = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
		ensure!(
			locked == 0,
			"flock: {error}",
			error = std::io::Error::last_os_error()
		);
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
	command
		.arg("--literal-pathspecs")
		.arg("-C")
		.arg(repo)
		.args(args);
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

pub fn check_write_path(relative: &Path, own_task_dir: Option<&Path>) -> Result<()> {
	let components: Vec<Component> = relative.components().collect();
	ensure!(
		!components.is_empty()
			&& components
				.iter()
				.all(|part| matches!(part, Component::Normal(name) if *name != GIT_DIR_NAME)),
		"write path {relative:?} must be a plain relative path without . .. or .git"
	);
	let under_run = components[0] == Component::Normal("run".as_ref());
	ensure!(
		!under_run || own_task_dir.is_some_and(|own| relative.starts_with(own)),
		"write path {relative:?} is inside run/ but not in the task's own dir"
	);
	Ok(())
}

fn declared_write(run: &RunDir, task: &TaskId, relative: &Path) -> Result<PathBuf> {
	check_write_path(relative, Some(&Path::new("run/tasks").join(&task.0)))?;
	Ok(run.root.join(relative))
}

fn has_git_component(path: &Path) -> bool {
	path.components()
		.any(|part| part == Component::Normal(GIT_DIR_NAME.as_ref()))
}

fn ensure_outside_git_dirs(repo: &Path, path: &Path) -> Result<()> {
	let relative = path.strip_prefix(repo).unwrap_or(path);
	ensure!(
		!has_git_component(relative),
		"refusing to stage {path:?}: it is inside a .git directory"
	);
	Ok(())
}

fn ensure_plain_index_entries(repo: &Path, paths: &[PathBuf]) -> Result<()> {
	let listed = git_checked(repo, &["ls-files", "--stage", "-z"], paths)?;
	for entry in listed.split('\0').filter(|entry| !entry.is_empty()) {
		let (mode_and_object, path) = entry.split_once('\t').unwrap_or((entry, ""));
		ensure!(
			!mode_and_object.starts_with(GITLINK_MODE),
			"refusing to stage embedded repository {path:?}"
		);
		ensure!(
			!has_git_component(Path::new(path)),
			"refusing to stage {path:?}: it is inside a .git directory"
		);
	}
	Ok(())
}

fn stage(repo: &Path, paths: &[PathBuf]) -> Result<()> {
	let staged = git_checked(repo, &["add", "-A"], paths)
		.and_then(|_| ensure_plain_index_entries(repo, paths));
	if let Err(error) = staged {
		if let Err(unstage_error) = git_checked(repo, &["reset", "-q"], paths) {
			return Err(error.context(format!("unstage failed: {unstage_error:#}")));
		}
		return Err(error);
	}
	Ok(())
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
	ensure!(
		is_valid_kind(&spec.kind),
		"invalid task kind {kind:?}",
		kind = spec.kind
	);
	ensure!(
		!config.repo_root.as_os_str().is_empty() && !config.git_common_dir.as_os_str().is_empty(),
		"config has no pinned repo_root and git_common_dir"
	);
	let mut candidates = vec![run.task_dir(task)];
	for relative in &spec.writes {
		candidates.push(declared_write(run, task, relative)?);
	}
	let repo = config.repo_root.clone();
	let mut paths = Vec::new();
	for candidate in candidates {
		ensure_outside_git_dirs(&repo, &candidate)?;
		if candidate.exists() || is_tracked(&repo, &candidate)? {
			paths.push(candidate);
		}
	}
	let mut message = format!(
		"{kind}: {task} {milestone}",
		kind = spec.kind,
		milestone = milestone_line(milestone)
	);
	if !config.commit_trailer.is_empty() {
		message.push_str(&format!("\n\n{trailer}", trailer = config.commit_trailer));
	}

	let _lock = CommitLock::acquire(&config.git_common_dir)?;
	stage(&repo, &paths)?;
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
