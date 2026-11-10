use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::json;

use crate::atomic::write_atomic;
use crate::config::Config;
use crate::paths::{RunDir, TaskId};

pub const HOOK_COMMAND: &str = "contest hook pre-tool-use";

pub fn write_claude_settings(run: &RunDir, config: &Config) -> Result<()> {
	let dir = run.claude_config();
	fs::create_dir_all(&dir).with_context(|| format!("create {dir:?}"))?;
	let timeout = config.bash_timeout_ms.to_string();
	let settings = json!({
		"hooks": {
			"PreToolUse": [{
				"matcher": "*",
				"hooks": [{"type": "command", "command": HOOK_COMMAND}],
			}],
		},
		"env": {
			"BASH_DEFAULT_TIMEOUT_MS": timeout,
			"BASH_MAX_TIMEOUT_MS": timeout,
		},
		"disableClaudeAiConnectors": true,
	});
	write_atomic(
		&dir.join("settings.json"),
		&serde_json::to_vec_pretty(&settings)?,
	)?;
	link_skills(&dir.join("skills"), &config.plugin_dir.join("skills"))
}

fn link_skills(link: &Path, skills: &Path) -> Result<()> {
	match fs::remove_file(link) {
		Err(error) if error.kind() != io::ErrorKind::NotFound => {
			return Err(error).with_context(|| format!("replace {link:?}"));
		}
		_ => {}
	}
	symlink(skills, link).with_context(|| format!("link {link:?} to {skills:?}"))
}

pub fn agent_env(
	run: &RunDir,
	task: Option<&TaskId>,
	contest_bin_dir: &Path,
) -> Vec<(String, String)> {
	let text = |path: &Path| path.to_string_lossy().into_owned();
	let path = match std::env::var("PATH") {
		Ok(inherited) => format!("{bin}:{inherited}", bin = text(contest_bin_dir)),
		Err(_) => text(contest_bin_dir),
	};
	let mut env = vec![
		("CLAUDE_CONFIG_DIR".to_owned(), text(&run.claude_config())),
		("CONTEST_RUN".to_owned(), text(&run.root)),
		("PATH".to_owned(), path),
	];
	if let Some(task) = task {
		env.push(("CONTEST_TASK".to_owned(), task.0.clone()));
	}
	env
}
