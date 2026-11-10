use std::io::{self, Read};
use std::process::ExitCode;

use clap::Subcommand;
use serde::Deserialize;
use serde_json::Value;

use crate::config::Config;
use crate::paths::RunDir;

const STOP_MESSAGE: &str = "stop requested: update notes.md and result.md, run contest commit --milestone stop, then end your turn";
const STOP_WRITABLE_SUFFIXES: [&str; 2] = ["/notes.md", "/result.md"];
const STOP_COMMANDS: [&str; 2] = ["contest commit", "contest task"];
const SHELL_CONTROL: [&str; 7] = [";", "&", "|", "`", "$(", "\n", "\r"];
const BLOCK_EXIT: u8 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookDecision {
	Allow,
	Block(String),
}

#[derive(Debug, Subcommand)]
pub enum HookVerb {
	PreToolUse,
}

#[derive(Deserialize)]
struct HookInput {
	tool_name: String,
	#[serde(default)]
	tool_input: Value,
}

fn strings_of(value: &Value) -> Vec<&str> {
	match value {
		Value::String(text) => vec![text],
		Value::Array(items) => items.iter().flat_map(strings_of).collect(),
		Value::Object(fields) => fields.values().flat_map(strings_of).collect(),
		_ => Vec::new(),
	}
}

fn blocked_entry<'config>(tool_input: &Value, config: &'config Config) -> Option<&'config str> {
	let haystacks: Vec<String> = strings_of(tool_input)
		.into_iter()
		.map(str::to_lowercase)
		.collect();
	config
		.blocked_url_substrings
		.iter()
		.find(|entry| {
			let needle = entry.to_lowercase();
			haystacks.iter().any(|text| text.contains(&needle))
		})
		.map(String::as_str)
}

fn is_stop_command(command: &str) -> bool {
	let command = command.trim();
	let follows_prefix = STOP_COMMANDS.iter().any(|prefix| {
		command
			.strip_prefix(prefix)
			.is_some_and(|rest| rest.is_empty() || rest.starts_with(' '))
	});
	follows_prefix
		&& !SHELL_CONTROL
			.iter()
			.any(|control| command.contains(control))
}

fn allowed_while_stopping(tool_name: &str, tool_input: &Value) -> bool {
	let field = |name: &str| tool_input[name].as_str().unwrap_or_default();
	match tool_name {
		"Read" => true,
		"Write" | "Edit" => STOP_WRITABLE_SUFFIXES
			.iter()
			.any(|suffix| field("file_path").ends_with(suffix)),
		"Bash" => is_stop_command(field("command")),
		_ => false,
	}
}

pub fn pre_tool_use(
	tool_name: &str,
	tool_input: &Value,
	stop_requested: bool,
	config: &Config,
) -> HookDecision {
	if let Some(entry) = blocked_entry(tool_input, config) {
		return HookDecision::Block(format!("blocked source: {entry}"));
	}
	if stop_requested && !allowed_while_stopping(tool_name, tool_input) {
		return HookDecision::Block(STOP_MESSAGE.to_owned());
	}
	HookDecision::Allow
}

fn decide_from_stdin() -> HookDecision {
	let mut text = String::new();
	if let Err(error) = io::stdin().read_to_string(&mut text) {
		return HookDecision::Block(format!("hook input unreadable: {error}"));
	}
	let input: HookInput = match serde_json::from_str(&text) {
		Ok(input) => input,
		Err(error) => return HookDecision::Block(format!("hook input unparseable: {error}")),
	};
	let run = std::env::var_os("CONTEST_RUN").map(|root| RunDir { root: root.into() });
	let config = run
		.as_ref()
		.and_then(|run| Config::load(&run.config()).ok())
		.unwrap_or_default();
	let stop_requested = run.is_some_and(|run| run.stop_file().exists());
	pre_tool_use(&input.tool_name, &input.tool_input, stop_requested, &config)
}

pub fn run_hook(verb: HookVerb) -> ExitCode {
	match verb {
		HookVerb::PreToolUse => match decide_from_stdin() {
			HookDecision::Allow => ExitCode::SUCCESS,
			HookDecision::Block(reason) => {
				eprintln!("{reason}");
				ExitCode::from(BLOCK_EXIT)
			}
		},
	}
}
