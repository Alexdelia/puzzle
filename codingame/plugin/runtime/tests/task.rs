use contest::events::{Event, emit, events_since};
use contest::journal::{journal_append, journal_tail};
use contest::paths::{RunDir, TaskId};
use contest::task::{TaskSpec, TaskState, TaskStore};

fn run_dir() -> (tempfile::TempDir, RunDir) {
	let dir = tempfile::tempdir().unwrap();
	let run = RunDir {
		root: dir.path().to_path_buf(),
	};
	(dir, run)
}

fn spec(kind: &str, priority: i32) -> TaskSpec {
	TaskSpec {
		kind: kind.to_owned(),
		title: format!("title {kind}"),
		priority,
		model: None,
		budget_usd: Some(1.5),
		writes: vec!["bots/a".into()],
		body: "do the thing\n".to_owned(),
		parent: None,
	}
}

#[test]
fn create_assigns_sequential_ids_and_roundtrips_spec() {
	let (_dir, run) = run_dir();
	let store = TaskStore { run: &run };
	let first = store.create(spec("code", 5)).unwrap();
	let second = store.create(spec("plan", 5)).unwrap();
	assert_eq!(first, TaskId("t0001-code".to_owned()));
	assert_eq!(second, TaskId("t0002-plan".to_owned()));
	assert_eq!(store.spec(&first).unwrap(), spec("code", 5));
	assert_eq!(store.state(&first).unwrap(), TaskState::Queued);
}

#[test]
fn disallowed_transition_rejected() {
	let (_dir, run) = run_dir();
	let store = TaskStore { run: &run };
	let id = store.create(spec("code", 1)).unwrap();
	store.transition(&id, TaskState::Running).unwrap();
	store.transition(&id, TaskState::Done).unwrap();
	assert!(store.transition(&id, TaskState::Running).is_err());
	assert_eq!(store.state(&id).unwrap(), TaskState::Done);
}

#[test]
fn next_queued_orders_by_priority_then_id() {
	let (_dir, run) = run_dir();
	let store = TaskStore { run: &run };
	let low = store.create(spec("a", 1)).unwrap();
	let high_first = store.create(spec("b", 9)).unwrap();
	let high_second = store.create(spec("c", 9)).unwrap();
	assert_eq!(store.next_queued().unwrap(), Some(high_first.clone()));
	store.transition(&high_first, TaskState::Running).unwrap();
	assert_eq!(store.next_queued().unwrap(), Some(high_second.clone()));
	store
		.transition(&high_second, TaskState::Cancelled)
		.unwrap();
	assert_eq!(store.next_queued().unwrap(), Some(low));
}

#[test]
fn reconcile_requeues_running_first() {
	let (_dir, run) = run_dir();
	let store = TaskStore { run: &run };
	let running_a = store.create(spec("a", 10)).unwrap();
	let running_b = store.create(spec("b", 20)).unwrap();
	let queued = store.create(spec("c", 50)).unwrap();
	store.transition(&running_a, TaskState::Running).unwrap();
	store.transition(&running_b, TaskState::Running).unwrap();
	let requeued = store.reconcile_after_restart().unwrap();
	assert_eq!(requeued, vec![running_a.clone(), running_b.clone()]);
	for id in [&running_a, &running_b] {
		assert_eq!(store.state(id).unwrap(), TaskState::Queued);
		assert!(store.spec(id).unwrap().priority >= 1000);
	}
	assert_eq!(store.next_queued().unwrap(), Some(running_b));
	assert_eq!(store.spec(&queued).unwrap().priority, 50);
}

#[test]
fn reconcile_requeues_interrupted_too() {
	let (_dir, run) = run_dir();
	let store = TaskStore { run: &run };
	let id = store.create(spec("a", 3)).unwrap();
	store.transition(&id, TaskState::Running).unwrap();
	store.transition(&id, TaskState::Interrupted).unwrap();
	assert_eq!(store.reconcile_after_restart().unwrap(), vec![id.clone()]);
	assert_eq!(store.spec(&id).unwrap().priority, 1003);
	assert_eq!(store.state(&id).unwrap(), TaskState::Queued);
}

#[test]
fn notes_priority_and_result_summary() {
	let (_dir, run) = run_dir();
	let store = TaskStore { run: &run };
	let id = store.create(spec("a", 3)).unwrap();
	store.set_priority(&id, 7).unwrap();
	assert_eq!(store.spec(&id).unwrap().priority, 7);
	store.append_notes(&id, "one\n").unwrap();
	store.append_notes(&id, "two\n").unwrap();
	let notes = std::fs::read_to_string(run.task_dir(&id).join("notes.md")).unwrap();
	assert_eq!(notes, "one\ntwo\n");
	assert_eq!(store.result_summary(&id), None);
	std::fs::write(run.task_dir(&id).join("result.md"), "first\nsecond\n").unwrap();
	assert_eq!(store.result_summary(&id), Some("first".to_owned()));
	let listed = store.list().unwrap();
	assert_eq!(listed.len(), 1);
	assert_eq!(listed[0].priority, 7);
}

#[test]
fn events_since_cursor_returns_only_new() {
	let (_dir, run) = run_dir();
	let (none, cursor) = events_since(&run, 0);
	assert!(none.is_empty());
	emit(&run, Event::BudgetExhausted);
	emit(
		&run,
		Event::UserMessage {
			text: "hi".to_owned(),
		},
	);
	let (events, cursor) = events_since(&run, cursor);
	assert_eq!(
		events,
		vec![
			Event::BudgetExhausted,
			Event::UserMessage {
				text: "hi".to_owned()
			}
		]
	);
	let (again, same) = events_since(&run, cursor);
	assert!(again.is_empty());
	assert_eq!(same, cursor);
	emit(
		&run,
		Event::TaskFinished {
			task: TaskId("t0001-a".to_owned()),
			state: TaskState::NeedsAnswer,
		},
	);
	let (fresh, _) = events_since(&run, cursor);
	assert_eq!(fresh.len(), 1);
	let raw = std::fs::read_to_string(run.events()).unwrap();
	assert!(raw.contains("\"event\":\"task_finished\""));
	assert!(raw.contains("\"state\":\"needs-answer\""));
	assert!(raw.contains("\"at\":"));
}

#[test]
fn journal_tail_last_n() {
	let (_dir, run) = run_dir();
	assert!(journal_tail(&run, 3).is_empty());
	for index in 0..5 {
		journal_append(&run, &format!("line {index}"));
	}
	let tail = journal_tail(&run, 2);
	assert_eq!(tail.len(), 2);
	assert!(tail[0].ends_with(" line 3"));
	assert!(tail[1].ends_with(" line 4"));
	let stamp = &tail[1][..19];
	assert_eq!(stamp.len(), 19);
	assert_eq!(&stamp[4..5], "-");
	assert_eq!(&stamp[13..14], ":");
	assert_eq!(journal_tail(&run, 99).len(), 5);
}
