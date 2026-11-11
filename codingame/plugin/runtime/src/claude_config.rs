use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use serde_json::json;

use crate::atomic::write_atomic;
use crate::config::Config;
use crate::paths::{RunDir, TaskId};
use crate::taint::read_worker_file;

pub const HOOK_COMMAND: &str = "contest hook pre-tool-use";
const MILLIS_PER_SECOND: i64 = 1000;
const USER_SKILLS: [&str; 1] = ["contest"];

#[derive(Deserialize)]
struct CredentialsExpiry {
	#[serde(rename = "claudeAiOauth")]
	oauth: OauthExpiry,
}

#[derive(Deserialize)]
struct OauthExpiry {
	#[serde(rename = "expiresAt")]
	expires_at_ms: Option<i64>,
}

pub fn credentials_expire_within(path: &Path, seconds: u64) -> Result<bool> {
	let bytes = read_worker_file(path)?;
	let expiry: CredentialsExpiry = serde_json::from_slice(&bytes)
		.map_err(|_| anyhow!("{path:?} has no numeric claudeAiOauth.expiresAt"))?;
	let now_ms = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_or(0, |elapsed| elapsed.as_millis() as i64);
	let horizon_ms = now_ms.saturating_add((seconds as i64).saturating_mul(MILLIS_PER_SECOND));
	Ok(expiry
		.oauth
		.expires_at_ms
		.is_some_and(|expires_at_ms| expires_at_ms <= horizon_ms))
}

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

fn clear_skills_dir(dir: &Path) -> Result<()> {
	let cleared = match fs::symlink_metadata(dir) {
		Ok(meta) if meta.is_dir() => fs::remove_dir_all(dir),
		Ok(_) => fs::remove_file(dir),
		Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
		Err(error) => Err(error),
	};
	cleared.with_context(|| format!("replace {dir:?}"))
}

fn link_skills(dir: &Path, skills: &Path) -> Result<()> {
	clear_skills_dir(dir)?;
	fs::create_dir_all(dir).with_context(|| format!("create {dir:?}"))?;
	let entries = match fs::read_dir(skills) {
		Ok(entries) => entries,
		Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
		Err(error) => return Err(error).with_context(|| format!("list {skills:?}")),
	};
	for entry in entries {
		let name = entry
			.with_context(|| format!("list {skills:?}"))?
			.file_name();
		if USER_SKILLS.iter().any(|user_skill| name == *user_skill) {
			continue;
		}
		let (target, link) = (skills.join(&name), dir.join(&name));
		symlink(&target, &link).with_context(|| format!("link {link:?} to {target:?}"))?;
	}
	Ok(())
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
