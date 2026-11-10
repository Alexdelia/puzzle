use std::ffi::{CStr, CString, OsStr};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use nix::dir::{Dir, Type};
use nix::errno::Errno;
use nix::fcntl::OFlag;
use nix::sys::stat::Mode;
use nix::unistd::{UnlinkatFlags, unlinkat};

pub const MAX_WORKER_FILE_BYTES: u64 = 1024 * 1024;
const PLANTED_NAMES: [&str; 5] = [".git", ".envrc", ".direnv", ".claude", ".mcp.json"];
const PLANTED_TOP_LEVEL_NAMES: [&str; 2] = [".git", ".mcp.json"];
const SUPERVISOR_DIR: &str = "run";

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

fn directory_flags() -> OFlag {
	OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC
}

fn open_child_dir(parent: &Dir, name: &CStr) -> nix::Result<Dir> {
	Dir::openat(parent, name, directory_flags(), Mode::empty())
}

fn entries(dir: &mut Dir) -> Result<Vec<(CString, Option<Type>)>> {
	let mut found = Vec::new();
	for entry in dir.iter() {
		let entry = entry?;
		let name = entry.file_name();
		if !matches!(name.to_bytes(), b"." | b"..") {
			found.push((name.to_owned(), entry.file_type()));
		}
	}
	Ok(found)
}

fn is_planted(name: &OsStr, top_level: bool) -> bool {
	let planted: &[&str] = if top_level {
		&PLANTED_TOP_LEVEL_NAMES
	} else {
		&PLANTED_NAMES
	};
	planted.iter().any(|planted| name == OsStr::new(planted))
}

fn may_be_directory(kind: Option<Type>) -> bool {
	matches!(kind, None | Some(Type::Directory))
}

fn remove_entry(parent: &Dir, name: &CStr) -> Result<()> {
	match open_child_dir(parent, name) {
		Ok(mut dir) => {
			for (child, _) in entries(&mut dir)? {
				remove_entry(&dir, &child)?;
			}
			unlinkat(parent, name, UnlinkatFlags::RemoveDir)?;
		}
		Err(Errno::ENOTDIR | Errno::ELOOP) => {
			unlinkat(parent, name, UnlinkatFlags::NoRemoveDir)?;
		}
		Err(error) => return Err(error.into()),
	}
	Ok(())
}

fn scan(dir: &mut Dir, path: &Path, top_level: bool, removed: &mut Vec<PathBuf>) -> Result<()> {
	for (name, kind) in entries(dir)? {
		let file_name = OsStr::from_bytes(name.to_bytes());
		let child = path.join(file_name);
		if is_planted(file_name, top_level) {
			remove_entry(dir, &name).with_context(|| format!("remove {child:?}"))?;
			removed.push(child);
			continue;
		}
		if !may_be_directory(kind) || (top_level && file_name == SUPERVISOR_DIR) {
			continue;
		}
		match open_child_dir(dir, &name) {
			Ok(mut sub) => scan(&mut sub, &child, false, removed)?,
			Err(Errno::ENOTDIR | Errno::ELOOP | Errno::ENOENT) => {}
			Err(error) => return Err(error).with_context(|| format!("open {child:?}")),
		}
	}
	Ok(())
}

pub fn remove_planted_files(contest_root: &Path) -> Result<Vec<PathBuf>> {
	let mut root = Dir::open(contest_root, directory_flags(), Mode::empty())
		.with_context(|| format!("open contest root {contest_root:?}"))?;
	let mut removed = Vec::new();
	scan(&mut root, contest_root, true, &mut removed)?;
	Ok(removed)
}
