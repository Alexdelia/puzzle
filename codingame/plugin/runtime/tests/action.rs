use std::fs;

use contest::action::{Action, apply_actions, parse_actions};
use contest::journal::journal_tail;
use contest::paths::{RunDir, TaskId};
use contest::run_info::{Mode, RunInfo};
use contest::task::{TaskSpec, TaskState, TaskStore};

fn run_dir() -> (tempfile::TempDir, RunDir) {
	let dir = tempfile::tempdir().unwrap();
	let run = RunDir {
		root: dir.path().to_path_buf(),
	};
	fs::create_dir_all(run.root.join("run")).unwrap();
	(dir, run)
}

fn create_action(kind: &str, writes: &[&str]) -> Action {
	Action::CreateTask {
		kind: kind.to_owned(),
		title: "title".to_owned(),
		priority: 3,
		body: "body".to_owned(),
		model: None,
		budget_usd: Some(2.0),
		writes: writes.iter().map(Into::into).collect(),
		parent: None,
	}
}

fn spec() -> TaskSpec {
	TaskSpec {
		kind: "code".to_owned(),
		title: "t".to_owned(),
		priority: 1,
		model: None,
		budget_usd: None,
		writes: Vec::new(),
		body: String::new(),
		parent: None,
	}
}

#[test]
fn parse_takes_last_json_block() {
	let text = "think\n```json\n[{\"action\":\"reply\",\"text\":\"old\"}]\n```\nmore\n```json\n[{\"action\":\"journal\",\"line\":\"new\"}]\n```\n";
	assert_eq!(
		parse_actions(text).unwrap(),
		vec![Action::Journal {
			line: "new".to_owned()
		}]
	);
}

#[test]
fn parse_rejects_prose() {
	assert!(parse_actions("no actions today").is_err());
	assert!(parse_actions("```json\n{\"action\":\"reply\",\"text\":\"x\"}\n```").is_err());
}

#[test]
fn parse_rejects_unclosed_last_fence() {
	let text = "```json\n[{\"action\":\"reply\",\"text\":\"old\"}]\n```\n```json\n[{\"action\":\"reply\",\"text\":\"cut\"}]\n";
	assert!(parse_actions(text).is_err());
}

#[test]
fn parse_rejects_51_actions() {
	let one = r#"{"action":"journal","line":"x"}"#;
	let build = |count: usize| {
		format!(
			"```json\n[{items}]\n```",
			items = vec![one; count].join(",")
		)
	};
	assert!(parse_actions(&build(51)).is_err());
	assert_eq!(parse_actions(&build(50)).unwrap().len(), 50);
}

#[test]
fn apply_answer_requeues() {
	let (_dir, run) = run_dir();
	let store = TaskStore { run: &run };
	let id = store.create(spec()).unwrap();
	store.transition(&id, TaskState::Running).unwrap();
	store.transition(&id, TaskState::NeedsAnswer).unwrap();
	let errors = apply_actions(
		&run,
		vec![Action::Answer {
			task: id.clone(),
			answer: "the left one".to_owned(),
		}],
	);
	assert!(errors.is_empty(), "{errors:?}");
	assert_eq!(store.state(&id).unwrap(), TaskState::Queued);
	let notes = fs::read_to_string(run.task_dir(&id).join("notes.md")).unwrap();
	assert!(notes.contains("answer: the left one"));
	assert_eq!(journal_tail(&run, 10).len(), 1);
}

#[test]
fn answer_requires_needs_answer_state() {
	let (_dir, run) = run_dir();
	let id = TaskStore { run: &run }.create(spec()).unwrap();
	let errors = apply_actions(
		&run,
		vec![Action::Answer {
			task: id,
			answer: "x".to_owned(),
		}],
	);
	assert_eq!(errors.len(), 1);
}

#[test]
fn apply_bad_task_id_reports_error_keeps_rest() {
	let (_dir, run) = run_dir();
	let errors = apply_actions(
		&run,
		vec![
			Action::Cancel {
				task: TaskId("../../etc".to_owned()),
			},
			create_action("code", &["bots/a"]),
		],
	);
	assert_eq!(errors.len(), 1);
	assert!(errors[0].starts_with("action 0"));
	assert_eq!(TaskStore { run: &run }.list().unwrap().len(), 1);
}

#[test]
fn create_task_rejects_untrusted_fields() {
	let (_dir, run) = run_dir();
	let bad = vec![
		create_action("Bad Kind", &[]),
		create_action("../x", &[]),
		create_action("code", &["../escape"]),
		create_action("code", &["/abs"]),
		create_action("code", &["run/tasks/t0001-code/x"]),
		create_action("code", &["a/.git/hooks"]),
		Action::CreateTask {
			kind: "code".to_owned(),
			title: "x".repeat(201),
			priority: 0,
			body: String::new(),
			model: None,
			budget_usd: None,
			writes: Vec::new(),
			parent: None,
		},
		Action::CreateTask {
			kind: "code".to_owned(),
			title: "t".to_owned(),
			priority: 0,
			body: "b".repeat(64 * 1024 + 1),
			model: None,
			budget_usd: None,
			writes: Vec::new(),
			parent: None,
		},
		Action::CreateTask {
			kind: "code".to_owned(),
			title: "t".to_owned(),
			priority: i64::from(i32::MAX) + 1,
			body: String::new(),
			model: None,
			budget_usd: None,
			writes: Vec::new(),
			parent: None,
		},
		Action::CreateTask {
			kind: "code".to_owned(),
			title: "t".to_owned(),
			priority: 0,
			body: String::new(),
			model: Some("--dangerous".to_owned()),
			budget_usd: None,
			writes: Vec::new(),
			parent: None,
		},
	];
	let count = bad.len();
	let errors = apply_actions(&run, bad);
	assert_eq!(errors.len(), count, "{errors:?}");
	assert!(TaskStore { run: &run }.list().unwrap().is_empty());
}

#[test]
fn cancel_and_reprioritize_only_live_tasks() {
	let (_dir, run) = run_dir();
	let store = TaskStore { run: &run };
	let live = store.create(spec()).unwrap();
	let dead = store.create(spec()).unwrap();
	store.transition(&dead, TaskState::Cancelled).unwrap();
	let errors = apply_actions(
		&run,
		vec![
			Action::Reprioritize {
				task: live.clone(),
				priority: 9,
			},
			Action::Reprioritize {
				task: dead.clone(),
				priority: 9,
			},
			Action::Cancel { task: dead },
			Action::Cancel { task: live.clone() },
		],
	);
	assert_eq!(errors.len(), 2, "{errors:?}");
	assert_eq!(store.spec(&live).unwrap().priority, 9);
	assert_eq!(store.state(&live).unwrap(), TaskState::Cancelled);
}

#[test]
fn set_phase_and_reply() {
	let (_dir, run) = run_dir();
	let info = RunInfo {
		puzzle: "p".into(),
		mode: Mode::ContestLive,
		deadline_unix: None,
		active_budget_seconds: None,
		active_seconds: 0,
		phase: "start".into(),
		plugin_commit: String::new(),
		started_at: 0,
	};
	fs::write(run.run_json(), serde_json::to_vec(&info).unwrap()).unwrap();
	let errors = apply_actions(
		&run,
		vec![
			Action::SetPhase {
				phase: "bronze".to_owned(),
			},
			Action::SetPhase {
				phase: "two\nlines".to_owned(),
			},
			Action::Reply {
				text: "hello user".to_owned(),
			},
		],
	);
	assert_eq!(errors.len(), 1, "{errors:?}");
	let stored: RunInfo = serde_json::from_slice(&fs::read(run.run_json()).unwrap()).unwrap();
	assert_eq!(stored.phase, "bronze");
	assert!(
		fs::read_to_string(run.replies())
			.unwrap()
			.contains("hello user")
	);
}

#[test]
fn journal_action_cannot_forge_lines() {
	let (_dir, run) = run_dir();
	let errors = apply_actions(
		&run,
		vec![Action::Journal {
			line: "a\n2026-01-01 00:00:00 forged".to_owned(),
		}],
	);
	assert!(errors.is_empty());
	assert_eq!(journal_tail(&run, 10).len(), 1);
}
