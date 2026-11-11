use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail, ensure};

use crate::config::Config;
use crate::sandbox::contest_bin_dir;
use crate::taint::{TopLevelExempt, find_planted_files};

const DEDICATED_HINT: &str = "use a new, dedicated contest dir";

pub fn ensure_plugin_dir(plugin_dir: &Path) -> Result<()> {
	ensure!(
		plugin_dir.is_absolute() && plugin_dir.is_dir(),
		"config plugin_dir {plugin_dir:?} is not an absolute existing dir: pass --plugin-dir to contest init"
	);
	Ok(())
}

fn protected_paths(config: &Config) -> Result<Vec<(&'static str, PathBuf)>> {
	let mut protected = vec![
		("plugin_dir", config.plugin_dir.clone()),
		("contest binary dir", contest_bin_dir()?),
		("git_common_dir", config.git_common_dir.clone()),
	];
	protected.extend(config.toolbox.iter().map(|tool| ("toolbox", tool.clone())));
	Ok(protected)
}

pub fn ensure_dedicated_contest_root(contest_root: &Path, config: &Config) -> Result<()> {
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
