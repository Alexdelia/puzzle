use std::fs::{self, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use anyhow::{Context, bail};
use nix::errno::Errno;
use nix::sys::prctl::set_pdeathsig;
use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::Config;

const AGENT_NICE: libc::c_int = 10;

#[derive(Debug, Clone)]
pub struct AgentRun {
	pub prompt: String,
	pub model: String,
	pub budget_usd: f64,
	pub cwd: PathBuf,
	pub tools: Option<Vec<String>>,
	pub stream_log: PathBuf,
	pub env: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Window {
	pub utilization: f64,
	pub resets_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct UnifiedWindows {
	five_hour: Window,
	seven_day: Window,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RateLimitShape {
	status: String,
	resets_at: i64,
	unified_windows: UnifiedWindows,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "RateLimitShape", into = "RateLimitShape")]
pub struct RateLimitInfo {
	pub status: String,
	pub resets_at: i64,
	pub five_hour: Window,
	pub seven_day: Window,
}

impl From<RateLimitShape> for RateLimitInfo {
	fn from(shape: RateLimitShape) -> Self {
		Self {
			status: shape.status,
			resets_at: shape.resets_at,
			five_hour: shape.unified_windows.five_hour,
			seven_day: shape.unified_windows.seven_day,
		}
	}
}

impl From<RateLimitInfo> for RateLimitShape {
	fn from(info: RateLimitInfo) -> Self {
		Self {
			status: info.status,
			resets_at: info.resets_at,
			unified_windows: UnifiedWindows {
				five_hour: info.five_hour,
				seven_day: info.seven_day,
			},
		}
	}
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StreamOutcome {
	pub result_text: Option<String>,
	pub is_error: bool,
	pub input_tokens: u64,
	pub output_tokens: u64,
	pub cache_read_tokens: u64,
	pub cache_write_tokens: u64,
	pub cost_usd: f64,
	pub rate_limit: Option<RateLimitInfo>,
}

pub fn claude_args(run: &AgentRun) -> Vec<String> {
	let mut args = vec![
		"-p".to_owned(),
		"--model".to_owned(),
		run.model.clone(),
		"--output-format".to_owned(),
		"stream-json".to_owned(),
		"--verbose".to_owned(),
		"--permission-mode".to_owned(),
		"bypassPermissions".to_owned(),
		"--max-budget-usd".to_owned(),
		run.budget_usd.to_string(),
		"--no-session-persistence".to_owned(),
		"--setting-sources".to_owned(),
		"user".to_owned(),
	];
	if let Some(tools) = &run.tools {
		args.push("--tools".to_owned());
		args.push(tools.join(","));
	}
	args
}

pub fn spawn_agent(
	config: &Config,
	run: &AgentRun,
	wrapper: Option<Vec<String>>,
) -> anyhow::Result<Child> {
	spawn_agent_scrubbed(config, run, wrapper, &[])
}

pub fn spawn_agent_scrubbed(
	config: &Config,
	run: &AgentRun,
	wrapper: Option<Vec<String>>,
	removed_env: &[String],
) -> anyhow::Result<Child> {
	let mut argv = wrapper.unwrap_or_default();
	argv.push(config.claude_bin.to_string_lossy().into_owned());
	argv.extend(claude_args(run));
	let Some((program, rest)) = argv.split_first() else {
		bail!("empty agent command");
	};
	if let Some(dir) = run.stream_log.parent() {
		fs::create_dir_all(dir).with_context(|| format!("create {dir:?}"))?;
	}
	let stream = File::create(&run.stream_log)
		.with_context(|| format!("create {path:?}", path = run.stream_log))?;
	let stderr_path = run.stream_log.with_extension("stderr.log");
	let stderr = File::create(&stderr_path).with_context(|| format!("create {stderr_path:?}"))?;
	let prompt = unlinked_prompt_file(&run.stream_log, &run.prompt)?;
	let mut command = Command::new(program);
	command.process_group(0);
	die_with_spawning_thread(&mut command);
	yield_to_jobs(&mut command);
	for name in removed_env {
		command.env_remove(name);
	}
	command
		.args(rest)
		.current_dir(&run.cwd)
		.envs(run.env.iter().map(|(key, value)| (key, value)))
		.stdin(prompt)
		.stdout(stream)
		.stderr(stderr)
		.spawn()
		.with_context(|| format!("spawn {program}"))
}

fn unlinked_prompt_file(stream_log: &Path, prompt: &str) -> anyhow::Result<File> {
	let path = stream_log.with_extension("prompt");
	let mut file = OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.truncate(true)
		.open(&path)
		.with_context(|| format!("create {path:?}"))?;
	fs::remove_file(&path).with_context(|| format!("unlink {path:?}"))?;
	file.write_all(prompt.as_bytes())
		.and_then(|()| file.seek(SeekFrom::Start(0)))
		.with_context(|| format!("write prompt to {path:?}"))?;
	Ok(file)
}

pub fn kill_agent_group(child: &Child) -> anyhow::Result<()> {
	match killpg(Pid::from_raw(child.id() as i32), Signal::SIGKILL) {
		Ok(()) | Err(Errno::ESRCH) => Ok(()),
		Err(error) => Err(error).context("kill agent process group"),
	}
}

fn die_with_spawning_thread(command: &mut Command) {
	unsafe {
		command.pre_exec(|| set_pdeathsig(Signal::SIGKILL).map_err(std::io::Error::from));
	}
}

fn yield_to_jobs(command: &mut Command) {
	unsafe {
		command.pre_exec(|| {
			if libc::setpriority(libc::PRIO_PROCESS, 0, AGENT_NICE) != 0 {
				return Err(std::io::Error::last_os_error());
			}
			Ok(())
		});
	}
}

pub fn parse_stream(path: &Path) -> StreamOutcome {
	let text = fs::read(path)
		.map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
		.unwrap_or_default();
	let mut outcome = StreamOutcome::default();
	for line in text.lines() {
		let Ok(value) = serde_json::from_str::<Value>(line) else {
			continue;
		};
		match value["type"].as_str() {
			Some("result") => outcome = apply_result(outcome, &value),
			Some("rate_limit_event") => {
				if let Ok(info) = serde_json::from_value(value["rate_limit_info"].clone()) {
					outcome.rate_limit = Some(info);
				}
			}
			_ => {}
		}
	}
	outcome
}

fn apply_result(previous: StreamOutcome, value: &Value) -> StreamOutcome {
	let tokens = |key: &str| value["usage"][key].as_u64().unwrap_or(0);
	StreamOutcome {
		result_text: value["result"].as_str().map(str::to_owned),
		is_error: value["is_error"].as_bool().unwrap_or(false),
		input_tokens: tokens("input_tokens"),
		output_tokens: tokens("output_tokens"),
		cache_read_tokens: tokens("cache_read_input_tokens"),
		cache_write_tokens: tokens("cache_creation_input_tokens"),
		cost_usd: value["total_cost_usd"].as_f64().unwrap_or(0.0),
		rate_limit: previous.rate_limit,
	}
}
