use std::fs;

use contest::config::Config;
use contest::digest::build_digest;
use contest::events::{Event, emit};
use contest::journal::journal_append;
use contest::paths::{JobId, RunDir};
use contest::task::{TaskSpec, TaskStore};

const NOW: i64 = 1_000_000;

fn run_dir() -> (tempfile::TempDir, RunDir) {
	let dir = tempfile::tempdir().unwrap();
	let run = RunDir {
		root: dir.path().to_path_buf(),
	};
	fs::create_dir_all(run.inbox()).unwrap();
	(dir, run)
}

fn spec(priority: i32) -> TaskSpec {
	TaskSpec {
		kind: "code".to_owned(),
		title: "a task with a reasonably long title to fill the digest".to_owned(),
		priority,
		model: None,
		budget_usd: None,
		writes: Vec::new(),
		body: String::new(),
		parent: None,
	}
}

#[test]
fn digest_bounded() {
	let (_dir, run) = run_dir();
	let store = TaskStore { run: &run };
	for index in 0..500 {
		store.create(spec(index % 7)).unwrap();
	}
	for index in 0..2000 {
		journal_append(
			&run,
			&format!("journal line {index} {pad}", pad = "x".repeat(60)),
		)
		.unwrap();
	}
	for index in 0..100 {
		emit(
			&run,
			Event::JobFinished {
				job: JobId(format!("j{index:04}")),
				code: 0,
			},
		)
		.unwrap();
	}
	let (digest, cursor) = build_digest(&run, &Config::default(), NOW);
	assert!(digest.len() <= 32000, "{len}", len = digest.len());
	for index in 0..100 {
		assert!(
			digest.contains(&format!("j{index:04}")),
			"event {index} missing"
		);
	}
	assert_eq!(cursor, fs::metadata(run.events()).unwrap().len());
}

#[test]
fn inbox_consumed_once() {
	let (_dir, run) = run_dir();
	fs::write(run.inbox().join("0001.md"), "focus on the referee\n").unwrap();
	let (first, _) = build_digest(&run, &Config::default(), NOW);
	assert!(first.contains("focus on the referee"));
	assert!(run.inbox().join("read/0001.md").is_file());
	assert!(!run.inbox().join("0001.md").exists());
	let (second, _) = build_digest(&run, &Config::default(), NOW);
	assert!(!second.contains("focus on the referee"));
}

#[test]
fn needs_answer_shows_question_and_never_dropped() {
	let (_dir, run) = run_dir();
	let store = TaskStore { run: &run };
	let id = store.create(spec(1)).unwrap();
	store
		.transition(&id, contest::task::TaskState::Running)
		.unwrap();
	store
		.transition(&id, contest::task::TaskState::NeedsAnswer)
		.unwrap();
	store
		.append_notes(&id, "\n## Question\n\nwhich bot first?\n")
		.unwrap();
	let config = Config {
		digest_max_bytes: 10,
		..Config::default()
	};
	let (digest, _) = build_digest(&run, &config, NOW);
	assert!(digest.contains("which bot first?"));
}

#[test]
fn cursor_file_limits_events() {
	let (_dir, run) = run_dir();
	emit(&run, Event::BudgetExhausted).unwrap();
	let (_, cursor) = build_digest(&run, &Config::default(), NOW);
	fs::write(run.manager_cursor(), cursor.to_string()).unwrap();
	emit(&run, Event::UsageBlocked { until: 42 }).unwrap();
	let (digest, _) = build_digest(&run, &Config::default(), NOW);
	assert!(digest.contains("usage_blocked"));
	assert!(!digest.contains("budget_exhausted"));
}
