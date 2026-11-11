use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail, ensure};

use crate::config::Config;
use crate::sandbox::contest_bin_dir;
use crate::taint::{TopLevelExempt, find_planted_files};

const DEDICATED_HINT: &str = "use a new, dedicated contest dir";
const HOST_PROGRAMS: [&str; 2] = ["git", "bwrap"];
const EXECUTABLE_BITS: u32 = 0o111;

pub fn ensure_plugin_dir(plugin_dir: &Path) -> Result<()> {
	ensure!(
		plugin_dir.is_absolute() && plugin_dir.is_dir(),
		"config plugin_dir {plugin_dir:?} is not an absolute existing dir: pass --plugin-dir to contest init"
	);
	Ok(())
}

fn host_path_entries() -> Vec<PathBuf> {
	std::env::var_os("PATH")
		.map(|path| std::env::split_paths(&path).collect())
		.unwrap_or_default()
}

fn is_executable_file(path: &Path) -> bool {
	fs::metadata(path)
		.is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & EXECUTABLE_BITS != 0)
}

fn on_host_path(name: &OsString) -> Option<PathBuf> {
	host_path_entries()
		.into_iter()
		.map(|dir| dir.join(name))
		.find(|candidate| is_executable_file(candidate))
}

fn resolved_program(what: &str, program: &Path) -> Result<Option<PathBuf>> {
	if program.as_os_str().is_empty() {
		return Ok(None);
	}
	if program.components().count() == 1 && !program.is_absolute() {
		return Ok(on_host_path(&program.as_os_str().to_owned()));
	}
	ensure!(
		program.is_absolute(),
		"{what} {program:?} is a relative path: it would resolve against an agent's cwd, use a bare name or an absolute path"
	);
	Ok(Some(program.to_path_buf()))
}

pub fn ensure_host_path_outside(contest_root: &Path) -> Result<()> {
	for entry in host_path_entries() {
		ensure!(
			entry.is_absolute(),
			"PATH entry {entry:?} is relative: host lookups of git, bwrap or claude could resolve inside the contest dir"
		);
		let real = fs::canonicalize(&entry).unwrap_or_else(|_| entry.clone());
		ensure!(
			!entry.starts_with(contest_root) && !real.starts_with(contest_root),
			"PATH entry {entry:?} lies in the contest dir {contest_root:?}: workers could plant git, bwrap or claude there"
		);
	}
	Ok(())
}

fn protected_paths(config: &Config) -> Result<Vec<(&'static str, PathBuf)>> {
	let mut protected = vec![
		("plugin_dir", config.plugin_dir.clone()),
		("contest binary dir", contest_bin_dir()?),
		("git_common_dir", config.git_common_dir.clone()),
		("pasta_bin", config.pasta_bin.clone()),
	];
	protected.extend(config.toolbox.iter().map(|tool| ("toolbox", tool.clone())));
	protected.extend(
		resolved_program("claude_bin", &config.claude_bin)?.map(|path| ("claude_bin", path)),
	);
	for program in HOST_PROGRAMS {
		protected
			.extend(resolved_program(program, Path::new(program))?.map(|path| (program, path)));
	}
	Ok(protected)
}

pub fn ensure_dedicated_contest_root(contest_root: &Path, config: &Config) -> Result<()> {
	ensure_host_path_outside(contest_root)?;
	for (what, path) in protected_paths(config)? {
		if path.as_os_str().is_empty() {
			continue;
		}
		let real = fs::canonicalize(&path).unwrap_or(path);
		ensure!(
			!real.starts_with(contest_root),
			"contest dir {contest_root:?} holds the {what} {real:?}: workers could rewrite it, {DEDICATED_HINT}"
		);
	}
	Ok(())
}

pub fn refuse_planted_below_top(contest_root: &Path, exempt: TopLevelExempt) -> Result<()> {
	let planted = find_planted_files(contest_root, exempt)?;
	if planted.is_empty() {
		return Ok(());
	}
	let listed: Vec<String> = planted.iter().map(|path| format!("{path:?}")).collect();
	bail!(
		"contest dir {contest_root:?} holds files the taint scan deletes at every start: {listed}; {DEDICATED_HINT}",
		listed = listed.join(", ")
	)
}
