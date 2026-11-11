use std::ffi::{CStr, CString, OsStr};
use std::fmt::Display;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
use nix::dir::{Dir, Type};
use nix::errno::Errno;
use nix::fcntl::{AtFlags, OFlag};
use nix::sys::stat::{FchmodatFlags, Mode, SFlag, fchmodat, fstatat};
use nix::unistd::{UnlinkatFlags, geteuid, unlinkat};

use crate::paths::TASK_META_DIR;

pub const MAX_WORKER_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_SCAN_DEPTH: usize = 64;
pub const MAX_SCANNED_ENTRIES: usize = 1_000_000;
const MAX_REMOVE_ATTEMPTS: usize = 3;
const PLANTED_NAMES: [&str; 5] = [".git", ".envrc", ".direnv", ".claude", ".mcp.json"];
const GIT_DIR_NAME: &str = ".git";
const ENVRC_NAME: &str = ".envrc";
const SUPERVISOR_DIR: &str = "run";
const EXEMPT_TOP_LEVEL_DIRS: [&str; 2] = [".claude", ".direnv"];
const WORKER_WRITABLE_RUN_DIRS: [&str; 2] = ["tasks", "requests"];

fn open_worker_file(path: &Path, options: &mut OpenOptions) -> Result<File> {
	let file = options
		.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
		.open(path)
		.with_context(|| format!("open {path:?}"))?;
	let meta = file.metadata()?;
	ensure!(meta.is_file(), "{path:?} is not a regular file");
	Ok(file)
}

fn ensure_within_cap(path: &Path, len: u64) -> Result<()> {
	ensure!(
		len <= MAX_WORKER_FILE_BYTES,
		"{path:?} is larger than {MAX_WORKER_FILE_BYTES} bytes"
	);
	Ok(())
}

pub fn read_worker_file(path: &Path) -> Result<Vec<u8>> {
	let file = open_worker_file(path, OpenOptions::new().read(true))?;
	ensure_within_cap(path, file.metadata()?.len())?;
	let mut bytes = Vec::new();
	file.take(MAX_WORKER_FILE_BYTES + 1)
		.read_to_end(&mut bytes)?;
	ensure_within_cap(path, bytes.len() as u64)?;
	Ok(bytes)
}

pub fn append_worker_file(path: &Path, bytes: &[u8]) -> Result<()> {
	let mut file = open_worker_file(path, OpenOptions::new().create(true).append(true))?;
	file.write_all(bytes)
		.with_context(|| format!("append to {path:?}"))
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TopLevelExempt {
	pub envrc: bool,
}

impl TopLevelExempt {
	pub fn pin_at_init(contest_root: &Path) -> Result<Self> {
		let envrc = contest_root.join(ENVRC_NAME);
		match fs::symlink_metadata(&envrc) {
			Ok(meta) if meta.file_type().is_file() => Ok(Self { envrc: true }),
			Ok(_) => bail!(
				"{envrc:?} is not a regular file (a symlink or another type): the sandbox only binds a plain .envrc"
			),
			Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self { envrc: false }),
			Err(error) => Err(error).with_context(|| format!("inspect {envrc:?}")),
		}
	}
}

#[derive(Debug, Default, PartialEq)]
pub struct ScanReport {
	pub removed: Vec<PathBuf>,
	pub failures: Vec<(PathBuf, String)>,
}

impl ScanReport {
	pub fn must_fail_closed(&self) -> bool {
		!self.failures.is_empty()
	}
}

#[derive(Debug, Clone, Copy)]
enum Level {
	ContestTop(TopLevelExempt),
	TaskTop,
	Nested,
}

impl Level {
	fn is_planted(self, name: &OsStr) -> bool {
		match self {
			Self::ContestTop(exempt) => {
				name == GIT_DIR_NAME || (!exempt.envrc && name == ENVRC_NAME)
			}
			Self::TaskTop | Self::Nested => PLANTED_NAMES.iter().any(|planted| name == *planted),
		}
	}

	fn is_skipped(self, name: &OsStr) -> bool {
		match self {
			Self::ContestTop(_) => {
				name == SUPERVISOR_DIR || EXEMPT_TOP_LEVEL_DIRS.iter().any(|dir| name == *dir)
			}
			Self::TaskTop => name == TASK_META_DIR,
			Self::Nested => false,
		}
	}
}

type Entry = (CString, Option<Type>);

struct Frame {
	dir: Dir,
	path: PathBuf,
	level: Level,
	pending: Vec<Entry>,
}

struct RemovalFrame {
	dir: Dir,
	name: CString,
	pending: Vec<Entry>,
}

enum RemovalError {
	StillWritten,
	Fatal(String),
}

impl From<Errno> for RemovalError {
	fn from(error: Errno) -> Self {
		Self::Fatal(error.to_string())
	}
}

fn directory_flags() -> OFlag {
	OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC
}

fn open_dir_no_follow(parent: &Dir, name: &CStr) -> nix::Result<Dir> {
	Dir::openat(parent, name, directory_flags(), Mode::empty())
}

fn grant_owner_access(parent: &Dir, name: &CStr) -> nix::Result<()> {
	let stat = fstatat(parent, name, AtFlags::AT_SYMLINK_NOFOLLOW)?;
	let is_dir = stat.st_mode & SFlag::S_IFMT.bits() == SFlag::S_IFDIR.bits();
	if !is_dir || stat.st_uid != geteuid().as_raw() {
		return Err(Errno::EACCES);
	}
	fchmodat(parent, name, Mode::S_IRWXU, FchmodatFlags::NoFollowSymlink)
}

fn open_child_dir(parent: &Dir, name: &CStr) -> nix::Result<Dir> {
	match open_dir_no_follow(parent, name) {
		Err(Errno::EACCES) => {
			grant_owner_access(parent, name)?;
			open_dir_no_follow(parent, name)
		}
		opened => opened,
	}
}

fn may_be_directory(kind: Option<Type>) -> bool {
	matches!(kind, None | Some(Type::Directory))
}

fn os_name(name: &CStr) -> &OsStr {
	OsStr::from_bytes(name.to_bytes())
}

struct Scanner {
	entries_left: usize,
	over_budget: bool,
	report: ScanReport,
}

impl Scanner {
	fn fail(&mut self, path: &Path, error: impl Display) {
		self.report
			.failures
			.push((path.to_path_buf(), error.to_string()));
	}

	fn list(&mut self, dir: &mut Dir) -> Result<Vec<Entry>, String> {
		let mut found = Vec::new();
		for entry in dir.iter() {
			let entry = entry.map_err(|error| error.to_string())?;
			let name = entry.file_name();
			if matches!(name.to_bytes(), b"." | b"..") {
				continue;
			}
			if self.entries_left == 0 {
				self.over_budget = true;
				return Err(format!("scan stopped after {MAX_SCANNED_ENTRIES} entries"));
			}
			self.entries_left -= 1;
			found.push((name.to_owned(), entry.file_type()));
		}
		Ok(found)
	}

	fn frame(&mut self, mut dir: Dir, path: PathBuf, level: Level) -> Option<Frame> {
		match self.list(&mut dir) {
			Ok(pending) => Some(Frame {
				dir,
				path,
				level,
				pending,
			}),
			Err(error) => {
				self.fail(&path, error);
				None
			}
		}
	}

	fn walk(&mut self, root: Dir, path: PathBuf, level: Level) {
		let mut stack: Vec<Frame> = self.frame(root, path, level).into_iter().collect();
		while !self.over_budget {
			let depth = stack.len();
			let Some(frame) = stack.last_mut() else {
				break;
			};
			let Some((name, kind)) = frame.pending.pop() else {
				stack.pop();
				continue;
			};
			let file_name = os_name(&name);
			let child = frame.path.join(file_name);
			if frame.level.is_planted(file_name) {
				self.remove(&frame.dir, &name, child);
				continue;
			}
			if frame.level.is_skipped(file_name) || !may_be_directory(kind) {
				continue;
			}
			if depth >= MAX_SCAN_DEPTH {
				self.fail(&child, format!("deeper than {MAX_SCAN_DEPTH} levels"));
				continue;
			}
			match open_child_dir(&frame.dir, &name) {
				Ok(dir) => stack.extend(self.frame(dir, child, Level::Nested)),
				Err(Errno::ENOTDIR | Errno::ELOOP | Errno::ENOENT) => {}
				Err(error) => self.fail(&child, error),
			}
		}
	}

	fn remove(&mut self, parent: &Dir, name: &CStr, path: PathBuf) {
		match self.remove_entry(parent, name) {
			Ok(()) => self.report.removed.push(path),
			Err(RemovalError::StillWritten) => self.fail(
				&path,
				format!("still being written after {MAX_REMOVE_ATTEMPTS} removal attempts"),
			),
			Err(RemovalError::Fatal(error)) => self.fail(&path, error),
		}
	}

	fn remove_entry(&mut self, parent: &Dir, name: &CStr) -> Result<(), RemovalError> {
		for _ in 0..MAX_REMOVE_ATTEMPTS {
			let dir = match open_child_dir(parent, name) {
				Ok(dir) => dir,
				Err(Errno::ENOENT) => return Ok(()),
				Err(Errno::ENOTDIR | Errno::ELOOP) => {
					return Ok(unlinkat(parent, name, UnlinkatFlags::NoRemoveDir)?);
				}
				Err(error) => return Err(error.into()),
			};
			match self.empty_tree(dir) {
				Err(RemovalError::StillWritten) => continue,
				emptied => emptied?,
			}
			match unlinkat(parent, name, UnlinkatFlags::RemoveDir) {
				Ok(()) | Err(Errno::ENOENT) => return Ok(()),
				Err(Errno::ENOTEMPTY) => continue,
				Err(error) => return Err(error.into()),
			}
		}
		Err(RemovalError::StillWritten)
	}

	fn removal_frame(&mut self, mut dir: Dir, name: CString) -> Result<RemovalFrame, RemovalError> {
		let pending = self.list(&mut dir).map_err(RemovalError::Fatal)?;
		Ok(RemovalFrame { dir, name, pending })
	}

	fn empty_tree(&mut self, root: Dir) -> Result<(), RemovalError> {
		let mut stack = vec![self.removal_frame(root, CString::default())?];
		while let Some(frame) = stack.last_mut() {
			let Some((name, kind)) = frame.pending.pop() else {
				let emptied = stack.pop().map(|frame| frame.name);
				if let (Some(parent), Some(name)) = (stack.last(), emptied) {
					match unlinkat(&parent.dir, name.as_c_str(), UnlinkatFlags::RemoveDir) {
						Ok(()) | Err(Errno::ENOENT) => {}
						Err(Errno::ENOTEMPTY) => return Err(RemovalError::StillWritten),
						Err(error) => return Err(error.into()),
					}
				}
				continue;
			};
			if may_be_directory(kind) {
				match open_child_dir(&frame.dir, &name) {
					Ok(dir) => {
						if stack.len() >= MAX_SCAN_DEPTH {
							return Err(RemovalError::Fatal(format!(
								"deeper than {MAX_SCAN_DEPTH} levels"
							)));
						}
						let child = self.removal_frame(dir, name)?;
						stack.push(child);
						continue;
					}
					Err(Errno::ENOENT) => continue,
					Err(Errno::ENOTDIR | Errno::ELOOP) => {}
					Err(error) => return Err(error.into()),
				}
			}
			match unlinkat(&frame.dir, name.as_c_str(), UnlinkatFlags::NoRemoveDir) {
				Ok(()) | Err(Errno::ENOENT) => {}
				Err(error) => return Err(error.into()),
			}
		}
		Ok(())
	}

	fn walk_worker_run_dirs(&mut self, root: &Dir, contest_root: &Path) {
		let run_path = contest_root.join(SUPERVISOR_DIR);
		let run_name = CString::new(SUPERVISOR_DIR).expect("run dir name has no NUL");
		let run = match open_dir_no_follow(root, &run_name) {
			Ok(run) => run,
			Err(Errno::ENOENT) => return,
			Err(error) => return self.fail(&run_path, error),
		};
		for (sub, level) in WORKER_WRITABLE_RUN_DIRS
			.iter()
			.zip([Level::TaskTop, Level::Nested])
		{
			let sub_path = run_path.join(sub);
			let sub_name = CString::new(*sub).expect("run dir names have no NUL");
			let mut owners = match open_dir_no_follow(&run, &sub_name) {
				Ok(owners) => owners,
				Err(Errno::ENOENT) => continue,
				Err(error) => {
					self.fail(&sub_path, error);
					continue;
				}
			};
			let entries = match self.list(&mut owners) {
				Ok(entries) => entries,
				Err(error) => {
					self.fail(&sub_path, error);
					continue;
				}
			};
			for (name, kind) in entries {
				let owner_path = sub_path.join(os_name(&name));
				if self.over_budget || !may_be_directory(kind) {
					continue;
				}
				match open_child_dir(&owners, &name) {
					Ok(dir) => self.walk(dir, owner_path, level),
					Err(Errno::ENOTDIR | Errno::ELOOP | Errno::ENOENT) => {}
					Err(error) => self.fail(&owner_path, error),
				}
			}
		}
	}
}

pub fn ensure_no_top_level_git(contest_root: &Path) -> Result<()> {
	let git = contest_root.join(GIT_DIR_NAME);
	ensure!(
		fs::symlink_metadata(&git).is_err(),
		"contest dir holds {git:?}: the taint scan removes a top-level .git, use a subdir of the repo"
	);
	Ok(())
}

pub fn remove_planted_files(contest_root: &Path, exempt: TopLevelExempt) -> Result<ScanReport> {
	let root = Dir::open(contest_root, directory_flags(), Mode::empty())
		.with_context(|| format!("open contest root {contest_root:?}"))?;
	let mut scanner = Scanner {
		entries_left: MAX_SCANNED_ENTRIES,
		over_budget: false,
		report: ScanReport::default(),
	};
	scanner.walk_worker_run_dirs(&root, contest_root);
	scanner.walk(root, contest_root.to_path_buf(), Level::ContestTop(exempt));
	Ok(scanner.report)
}
