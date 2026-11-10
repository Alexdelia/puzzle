use std::fs;
use std::io::Write;
use std::process::{Command, Output, Stdio};

use contest::config::Config;
use contest::hook::{HookDecision, pre_tool_use};
use contest::paths::RunDir;
use serde_json::{Value, json};
use tempfile::TempDir;

fn decide(tool: &str, input: Value, stop: bool) -> HookDecision {
	pre_tool_use(tool, &input, stop, &Config::default())
}

fn is_blocked(decision: &HookDecision) -> bool {
	matches!(decision, HookDecision::Block(_))
}

#[test]
fn blocks_github_user_url() {
	let decision = decide(
		"WebFetch",
		json!({"url": "https://github.com/Alexdelia/puzzle", "prompt": "read"}),
		false,
	);
	assert_eq!(
		decision,
		HookDecision::Block("blocked source: alexdelia".into())
	);
}

#[test]
fn blocks_curl_to_discord() {
	let decision = decide(
		"Bash",
		json!({"command": "curl https://discord.com/api"}),
		false,
	);
	assert_eq!(
		decision,
		HookDecision::Block("blocked source: discord.com".into())
	);
}

#[test]
fn block_list_ignores_case() {
	let decision = decide(
		"Bash",
		json!({"command": "curl https://GitHub.com/alexdelia/puzzle"}),
		false,
	);
	assert!(is_blocked(&decision));
}

#[test]
fn default_block_list_matches_bare_user_needle() {
	assert_eq!(
		Config::default().blocked_url_substrings,
		["alexdelia", "discord.com", "discord.gg"]
	);
	for url in [
		"https://raw.githubusercontent.com/AlexDelia/puzzle/main/x",
		"https://gitlab.com/ALEXDELIA/x",
		"https://Discord.GG/abc",
	] {
		assert!(
			is_blocked(&decide("WebFetch", json!({"url": url}), false)),
			"{url}"
		);
	}
}

#[test]
fn block_list_checks_nested_strings() {
	let decision = decide(
		"MultiTool",
		json!({"edits": [{"new_string": "see discord.gg/abc"}]}),
		false,
	);
	assert!(is_blocked(&decision));
}

#[test]
fn allows_ordinary_calls_and_config_reads() {
	assert_eq!(
		decide(
			"WebFetch",
			json!({"url": "https://github.com/someone/bot"}),
			false
		),
		HookDecision::Allow
	);
	assert_eq!(
		decide("Read", json!({"file_path": "/c/run/config.toml"}), false),
		HookDecision::Allow
	);
	assert_eq!(
		decide("Bash", json!({"command": "cat /c/run/config.toml"}), true),
		HookDecision::Block(stop_message())
	);
	assert_eq!(
		decide("Read", json!({"file_path": "/c/run/config.toml"}), true),
		HookDecision::Allow
	);
}

fn stop_message() -> String {
	"stop requested: update notes.md and result.md, run contest commit --milestone stop, then end your turn".into()
}

#[test]
fn stop_allows_notes_write() {
	for tool in ["Write", "Edit"] {
		for file in ["notes.md", "result.md"] {
			let path = format!("/c/run/tasks/t0001-x/{file}");
			assert_eq!(
				decide(tool, json!({"file_path": path, "content": "x"}), true),
				HookDecision::Allow,
				"{tool} {file}"
			);
		}
	}
	assert_eq!(
		decide(
			"Write",
			json!({"file_path": "/c/src/main.rs", "content": "x"}),
			true
		),
		HookDecision::Block(stop_message())
	);
	assert_eq!(
		decide(
			"Write",
			json!({"file_path": "/c/xnotes.md", "content": "x"}),
			true
		),
		HookDecision::Block(stop_message())
	);
}

#[test]
fn stop_allows_contest_commit() {
	for command in [
		"contest commit --milestone stop",
		"  contest task done",
		"contest task ask 'which bot?'",
	] {
		assert_eq!(
			decide("Bash", json!({"command": command}), true),
			HookDecision::Allow,
			"{command}"
		);
	}
}

#[test]
fn stop_blocks_other_bash() {
	for command in [
		"cargo build --release",
		"contest committee",
		"contest commit --milestone stop; cargo build",
		"contest commit --milestone stop && cargo build",
		"contest task done | tee x",
		"contest commit --milestone $(cargo build)",
		"contest commit --milestone `cargo build`",
		"contest commit --milestone stop\ncargo build",
		"contest task done > /tmp/x",
		"contest task done < /etc/passwd",
		"contest commit --milestone <(cargo build)",
		"contest commit --milestone >(cat)",
		"contest job submit --class batch --ram-mb 1 -- true",
	] {
		assert_eq!(
			decide("Bash", json!({"command": command}), true),
			HookDecision::Block(stop_message()),
			"{command}"
		);
	}
}

#[test]
fn stop_still_applies_block_list() {
	let decision = decide(
		"Write",
		json!({"file_path": "/c/notes.md", "content": "https://discord.com/x"}),
		true,
	);
	assert_eq!(
		decision,
		HookDecision::Block("blocked source: discord.com".into())
	);
}

fn run_dir() -> (TempDir, RunDir) {
	let dir = TempDir::new().unwrap();
	let run = RunDir {
		root: dir.path().to_path_buf(),
	};
	fs::create_dir_all(run.tasks()).unwrap();
	fs::write(run.config(), toml::to_string(&Config::default()).unwrap()).unwrap();
	(dir, run)
}

fn run_hook(run: &RunDir, payload: &Value) -> Output {
	let mut child = Command::new(env!("CARGO_BIN_EXE_contest"))
		.args(["hook", "pre-tool-use"])
		.env("CONTEST_RUN", &run.root)
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.unwrap();
	child
		.stdin
		.take()
		.unwrap()
		.write_all(payload.to_string().as_bytes())
		.unwrap();
	child.wait_with_output().unwrap()
}

fn hook_payload(tool: &str, input: Value) -> Value {
	json!({
		"session_id": "s",
		"hook_event_name": "PreToolUse",
		"cwd": "/c",
		"tool_name": tool,
		"tool_input": input,
	})
}

#[test]
fn cli_allows_with_exit_zero() {
	let (_dir, run) = run_dir();
	let output = run_hook(&run, &hook_payload("Bash", json!({"command": "ls"})));
	assert_eq!(output.status.code(), Some(0));
}

#[test]
fn cli_blocks_with_exit_two_and_stderr_reason() {
	let (_dir, run) = run_dir();
	let output = run_hook(
		&run,
		&hook_payload("WebFetch", json!({"url": "https://discord.gg/x"})),
	);
	assert_eq!(output.status.code(), Some(2));
	assert_eq!(
		String::from_utf8_lossy(&output.stderr).trim(),
		"blocked source: discord.gg"
	);
}

#[test]
fn cli_reads_stop_file() {
	let (_dir, run) = run_dir();
	let bash = hook_payload("Bash", json!({"command": "ls"}));
	assert_eq!(run_hook(&run, &bash).status.code(), Some(0));
	fs::write(run.stop_file(), "").unwrap();
	let output = run_hook(&run, &bash);
	assert_eq!(output.status.code(), Some(2));
	assert_eq!(
		String::from_utf8_lossy(&output.stderr).trim(),
		stop_message()
	);
}

#[test]
fn cli_uses_run_block_list() {
	let (_dir, run) = run_dir();
	let config = Config {
		blocked_url_substrings: vec!["example.org/secret".into()],
		..Config::default()
	};
	fs::write(run.config(), toml::to_string(&config).unwrap()).unwrap();
	let output = run_hook(
		&run,
		&hook_payload("WebFetch", json!({"url": "https://example.org/secret/x"})),
	);
	assert_eq!(output.status.code(), Some(2));
}

#[test]
fn cli_falls_back_to_default_block_list_without_config() {
	let (_dir, run) = run_dir();
	fs::remove_file(run.config()).unwrap();
	let output = run_hook(
		&run,
		&hook_payload("WebFetch", json!({"url": "https://discord.com/x"})),
	);
	assert_eq!(output.status.code(), Some(2));
}

#[test]
fn cli_blocks_unparseable_input() {
	let (_dir, run) = run_dir();
	let mut child = Command::new(env!("CARGO_BIN_EXE_contest"))
		.args(["hook", "pre-tool-use"])
		.env("CONTEST_RUN", &run.root)
		.stdin(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.unwrap();
	child.stdin.take().unwrap().write_all(b"not json").unwrap();
	assert_eq!(child.wait_with_output().unwrap().status.code(), Some(2));
}
