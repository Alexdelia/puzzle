use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::events::events_since;
use crate::journal::journal_tail;
use crate::paths::RunDir;
use crate::run_info::RunInfo;
use crate::taint::read_worker_file;
use crate::task::{TaskState, TaskStore, TaskSummary};
use crate::usage::{five_hour_used_percent, load_usage, seven_day_used_percent};

const QUEUED_SHOWN: usize = 20;
const RECENT_SHOWN: usize = 15;
const JOURNAL_SHOWN: usize = 20;
const TITLE_MAX_CHARS: usize = 120;
const SUMMARY_MAX_CHARS: usize = 200;
const QUESTION_MAX_CHARS: usize = 1500;
const MESSAGE_MAX_CHARS: usize = 4000;
const QUESTION_HEADER: &str = "## Question";
const INBOX_READ_DIR: &str = "read";

fn one_line(text: &str, max_chars: usize) -> String {
	text.chars()
		.map(|c| if c.is_control() { ' ' } else { c })
		.take(max_chars)
		.collect::<String>()
		.trim()
		.to_owned()
}

fn task_line(task: &TaskSummary) -> String {
	format!(
		"- {id} [{kind}] p={priority} {title}",
		id = task.id,
		kind = task.kind,
		priority = task.priority,
		title = one_line(&task.title, TITLE_MAX_CHARS)
	)
}

fn run_section(run: &RunDir, now: i64) -> Vec<String> {
	let info: Option<RunInfo> = fs::read(run.run_json())
		.ok()
		.and_then(|bytes| serde_json::from_slice(&bytes).ok());
	let Some(info) = info else {
		return vec!["run info unavailable".to_owned()];
	};
	let budget = info
		.active_budget_seconds
		.map_or("none".to_owned(), |seconds| seconds.to_string());
	let deadline = info.deadline_unix.map_or("none".to_owned(), |unix| {
		format!("{seconds}s", seconds = unix - now)
	});
	vec![
		format!("puzzle: {puzzle}", puzzle = info.puzzle),
		format!("mode: {mode:?}", mode = info.mode),
		format!(
			"phase: {phase}",
			phase = one_line(&info.phase, TITLE_MAX_CHARS)
		),
		format!("active: {used}s of {budget}s", used = info.active_seconds),
		format!("deadline left: {deadline}"),
	]
}

fn usage_section(run: &RunDir, now: i64) -> Vec<String> {
	match load_usage(run) {
		Some((info, recorded_at)) => vec![
			format!("5h: {used}%", used = five_hour_used_percent(&info, now)),
			format!("7d: {used}%", used = seven_day_used_percent(&info, now)),
			format!("sampled {age}s ago", age = now - recorded_at),
		],
		None => vec!["no usage sample".to_owned()],
	}
}

fn question_of(run: &RunDir, task: &TaskSummary) -> String {
	let notes = read_worker_file(&run.task_dir(&task.id).join("notes.md")).unwrap_or_default();
	let notes = String::from_utf8_lossy(&notes);
	let question = notes
		.rsplit_once(QUESTION_HEADER)
		.map_or("", |(_, question)| question);
	question.trim().chars().take(QUESTION_MAX_CHARS).collect()
}

fn needs_answer_section(run: &RunDir, tasks: &[TaskSummary]) -> Vec<String> {
	tasks
		.iter()
		.filter(|task| task.state == TaskState::NeedsAnswer)
		.map(|task| {
			format!(
				"{line}\n  question: {question}",
				line = task_line(task),
				question = question_of(run, task)
			)
		})
		.collect()
}

fn recent_section(store: &TaskStore, tasks: &[TaskSummary]) -> Vec<String> {
	let finished: Vec<&TaskSummary> = tasks
		.iter()
		.filter(|task| matches!(task.state, TaskState::Done | TaskState::Failed))
		.collect();
	let first = finished.len().saturating_sub(RECENT_SHOWN);
	finished[first..]
		.iter()
		.map(|task| {
			format!(
				"{line} -> {state:?}: {summary}",
				line = task_line(task),
				state = task.state,
				summary = one_line(
					&store.result_summary(&task.id).unwrap_or_default(),
					SUMMARY_MAX_CHARS
				)
			)
		})
		.collect()
}

fn queued_section(tasks: &[TaskSummary]) -> Vec<String> {
	let mut queued: Vec<&TaskSummary> = tasks
		.iter()
		.filter(|task| task.state == TaskState::Queued)
		.collect();
	queued.sort_by(|a, b| {
		b.priority
			.cmp(&a.priority)
			.then_with(|| a.id.0.cmp(&b.id.0))
	});
	queued
		.into_iter()
		.take(QUEUED_SHOWN)
		.map(task_line)
		.collect()
}

fn state_section(tasks: &[TaskSummary], states: &[TaskState]) -> Vec<String> {
	tasks
		.iter()
		.filter(|task| states.contains(&task.state))
		.map(task_line)
		.collect()
}

fn consume_inbox(run: &RunDir) -> Vec<String> {
	let Ok(entries) = fs::read_dir(run.inbox()) else {
		return Vec::new();
	};
	let mut files: Vec<PathBuf> = entries
		.filter_map(Result::ok)
		.filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
		.map(|entry| entry.path())
		.collect();
	files.sort();
	let read_dir = run.inbox().join(INBOX_READ_DIR);
	let mut messages = Vec::new();
	for path in files {
		let Some(name) = path.file_name() else {
			continue;
		};
		let Ok(bytes) = fs::read(&path) else {
			continue;
		};
		if fs::create_dir_all(&read_dir).is_err() || fs::rename(&path, read_dir.join(name)).is_err()
		{
			continue;
		}
		let text = String::from_utf8_lossy(&bytes);
		messages.push(format!(
			"- {text}",
			text = text
				.trim()
				.chars()
				.take(MESSAGE_MAX_CHARS)
				.collect::<String>()
		));
	}
	messages
}

struct Sections {
	run: Vec<String>,
	usage: Vec<String>,
	running: Vec<String>,
	needs_answer: Vec<String>,
	queued: Vec<String>,
	recent: Vec<String>,
	events: Vec<String>,
	inbox: Vec<String>,
	journal: Vec<String>,
	kb: Vec<String>,
}

impl Sections {
	fn render(&self) -> String {
		let sections: [(&str, &Vec<String>); 10] = [
			("run", &self.run),
			("usage", &self.usage),
			("running", &self.running),
			("needs-answer", &self.needs_answer),
			("queued", &self.queued),
			("recent", &self.recent),
			("events", &self.events),
			("inbox", &self.inbox),
			("journal", &self.journal),
			("kb", &self.kb),
		];
		let mut text = String::new();
		for (name, lines) in sections {
			text.push_str(&format!("## {name}\n"));
			if lines.is_empty() {
				text.push_str("(none)\n");
			}
			for line in lines {
				text.push_str(line);
				text.push('\n');
			}
			text.push('\n');
		}
		text
	}

	fn shrink_once(&mut self) -> bool {
		if !self.recent.is_empty() {
			self.recent.remove(0);
			true
		} else if self.queued.pop().is_some() {
			true
		} else if !self.journal.is_empty() {
			self.journal.remove(0);
			true
		} else {
			false
		}
	}
}

pub fn build_digest(run: &RunDir, config: &Config, now: i64) -> (String, u64) {
	let store = TaskStore { run };
	let tasks = store.list().unwrap_or_default();
	let cursor = fs::read_to_string(run.manager_cursor())
		.ok()
		.and_then(|text| text.trim().parse().ok())
		.unwrap_or(0);
	let (events, new_cursor) = events_since(run, cursor);
	let mut sections = Sections {
		run: run_section(run, now),
		usage: usage_section(run, now),
		running: state_section(&tasks, &[TaskState::Running, TaskState::Interrupted]),
		needs_answer: needs_answer_section(run, &tasks),
		queued: queued_section(&tasks),
		recent: recent_section(&store, &tasks),
		events: events
			.iter()
			.filter_map(|event| serde_json::to_string(event).ok())
			.collect(),
		inbox: consume_inbox(run),
		journal: journal_tail(run, JOURNAL_SHOWN),
		kb: vec![format!(
			"{path}",
			path = config.plugin_dir.join("kb/index.md").display()
		)],
	};
	let mut text = sections.render();
	while text.len() > config.digest_max_bytes && sections.shrink_once() {
		text = sections.render();
	}
	(text, new_cursor)
}
