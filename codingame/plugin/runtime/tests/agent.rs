use std::path::{Path, PathBuf};
use std::time::Duration;

use contest::agent::{AgentRun, RateLimitInfo, claude_args, parse_stream, spawn_agent};
use contest::config::Config;
use contest::ledger::ledger_append;
use contest::paths::RunDir;

fn fixture(name: &str) -> PathBuf {
	Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("tests/fixtures")
		.join(name)
}

fn agent_run(stream_log: PathBuf) -> AgentRun {
	AgentRun {
		prompt: "do it".into(),
		model: "opus".into(),
		budget_usd: 2.5,
		cwd: std::env::temp_dir(),
		allowed_tools: None,
		stream_log,
		env: Vec::new(),
	}
}

#[test]
fn parse_success_stream() {
	let outcome = parse_stream(&fixture("stream-success.jsonl"));
	assert_eq!(outcome.result_text.as_deref(), Some("done"));
	assert!(!outcome.is_error);
	assert_eq!(outcome.input_tokens, 1200);
	assert_eq!(outcome.output_tokens, 300);
	assert_eq!(outcome.cache_read_tokens, 4000);
	assert_eq!(outcome.cache_write_tokens, 500);
	assert_eq!(outcome.cost_usd, 0.05);
	let limit = outcome.rate_limit.unwrap();
	assert_eq!(limit.status, "allowed");
	assert_eq!(limit.resets_at, 1792972200);
	assert_eq!(limit.five_hour.utilization, 0.13);
	assert_eq!(limit.seven_day.resets_at, 1793224800);
}

#[test]
fn parse_truncated_last_line() {
	let outcome = parse_stream(&fixture("stream-truncated.jsonl"));
	assert_eq!(outcome.result_text, None);
	assert_eq!(outcome.rate_limit.unwrap().five_hour.utilization, 0.13);
}

#[test]
fn parse_missing_file_is_empty() {
	let outcome = parse_stream(Path::new("/nonexistent/stream"));
	assert_eq!(outcome.result_text, None);
	assert_eq!(outcome.rate_limit, None);
}

#[test]
fn rate_limit_info_serializes_in_stream_shape() {
	let outcome = parse_stream(&fixture("stream-success.jsonl"));
	let info = outcome.rate_limit.unwrap();
	let json = serde_json::to_value(&info).unwrap();
	assert_eq!(json["unifiedWindows"]["five_hour"]["resetsAt"], 1792972200);
	assert_eq!(json["resetsAt"], 1792972200);
	let back: RateLimitInfo = serde_json::from_value(json).unwrap();
	assert_eq!(back, info);
}

#[test]
fn args_contain_budget_and_model() {
	let mut run = agent_run(PathBuf::from("s.jsonl"));
	let args = claude_args(&run);
	assert_eq!(args[..2], ["-p", "do it"]);
	let position = |flag: &str| args.iter().position(|arg| arg == flag).unwrap();
	assert_eq!(args[position("--model") + 1], "opus");
	assert_eq!(args[position("--max-budget-usd") + 1], "2.5");
	assert_eq!(args[position("--output-format") + 1], "stream-json");
	assert_eq!(args[position("--permission-mode") + 1], "bypassPermissions");
	assert!(args.contains(&"--verbose".to_owned()));
	assert!(args.contains(&"--no-session-persistence".to_owned()));
	assert!(!args.contains(&"--allowedTools".to_owned()));

	run.allowed_tools = Some(vec!["Read".into(), "Bash".into()]);
	let args = claude_args(&run);
	let at = args.iter().position(|arg| arg == "--allowedTools").unwrap();
	assert_eq!(args[at + 1..], ["Read", "Bash"]);
}

#[test]
fn spawn_fake_writes_stream_log() {
	let dir = tempfile::tempdir().unwrap();
	let config = Config {
		claude_bin: fixture("fake-claude.sh"),
		..Config::default()
	};
	let mut run = agent_run(dir.path().join("agents/a/stream.jsonl"));
	run.env = vec![(
		"FAKE_CLAUDE_SCRIPT".into(),
		fixture("stream-success.jsonl")
			.to_string_lossy()
			.into_owned(),
	)];
	let status = spawn_agent(&config, &run, None).unwrap().wait().unwrap();
	assert!(status.success());
	let outcome = parse_stream(&run.stream_log);
	assert_eq!(outcome.result_text.as_deref(), Some("done"));
}

#[test]
fn spawn_applies_wrapper_prefix_and_exec_hook() {
	let dir = tempfile::tempdir().unwrap();
	let marker = dir.path().join("marker");
	let config = Config {
		claude_bin: fixture("fake-claude.sh"),
		..Config::default()
	};
	let mut run = agent_run(dir.path().join("stream.jsonl"));
	run.env = vec![
		(
			"FAKE_CLAUDE_SCRIPT".into(),
			fixture("stream-success.jsonl")
				.to_string_lossy()
				.into_owned(),
		),
		(
			"FAKE_CLAUDE_EXEC".into(),
			format!("touch {marker}", marker = marker.display()),
		),
		("FAKE_CLAUDE_SLEEP".into(), "0.1".into()),
	];
	let wrapper = Some(vec!["env".to_owned(), "WRAPPED=1".to_owned()]);
	let mut child = spawn_agent(&config, &run, wrapper).unwrap();
	std::thread::sleep(Duration::from_millis(10));
	assert!(child.wait().unwrap().success());
	assert!(marker.exists());
}

#[test]
fn ledger_appends_tsv_line() {
	let dir = tempfile::tempdir().unwrap();
	let run = RunDir {
		root: dir.path().to_path_buf(),
	};
	let outcome = parse_stream(&fixture("stream-success.jsonl"));
	ledger_append(&run, "task-1", &outcome, 42);
	ledger_append(&run, "manager", &outcome, 7);
	let text = std::fs::read_to_string(run.ledger()).unwrap();
	let lines: Vec<Vec<&str>> = text
		.lines()
		.map(|line| line.split('\t').collect())
		.collect();
	assert_eq!(lines.len(), 2);
	assert_eq!(lines[0].len(), 9);
	assert_eq!(lines[0][1], "task-1");
	assert_eq!(lines[0][3..], ["1200", "300", "4000", "500", "0.05", "42"]);
	assert_eq!(lines[1][1], "manager");
}
