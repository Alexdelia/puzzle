use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::Config;
use crate::events::{Event, events_since};
use crate::journal::journal_tail;
use crate::paths::RunDir;
use crate::run_info::RunInfo;
use crate::taint::read_worker_file;
use crate::task::{TaskState, TaskStore, TaskSummary};
use crate::usage::{
	NO_LAUNCHES_NOTE, five_hour_used_percent, is_fresh_sample, load_usage, seven_day_used_percent,
};

const QUEUED_SHOWN: usize = 20;
const RECENT_SHOWN: usize = 15;
const JOURNAL_SHOWN: usize = 20;
const TITLE_MAX_CHARS: usize = 120;
const SUMMARY_MAX_CHARS: usize = 200;
const QUESTION_MAX_CHARS: usize = 1500;
const MESSAGE_MAX_BYTES: u64 = 4000;
const QUESTION_HEADER: &str = "## Question";
const QUOTE_PREFIX: &str = "  > ";
const INBOX_READ_DIR: &str = "read";
const EVENTS_SHOWN: usize = 100;
const INBOX_SHOWN: usize = 10;
const OMITTED_EVENTS: &str = "older events omitted:";

fn one_line(text: &str, max_chars: usize) -> String {
	text.chars()
		.map(|c| if c.is_control() { ' ' } else { c })
		.take(max_chars)
		.collect::<String>()
		.trim()
		.to_owned()
}

fn quoted(text: &str, max_chars: usize) -> String {
	let flat: String = text
		.chars()
		.take(max_chars)
		.map(|c| if c != '\n' && c.is_control() { ' ' } else { c })
		.collect();
	flat.trim()
		.lines()
		.map(|line| format!("{QUOTE_PREFIX}{line}"))
		.collect::<Vec<_>>()
		.join("\n")
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
	let info = match fs::read(run.run_json())
		.context("read run.json")
		.and_then(|bytes| serde_json::from_slice::<RunInfo>(&bytes).context("parse run.json"))
	{
		Ok(info) => info,
		Err(error) => return vec![format!("run info unavailable: {error:#}")],
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

fn usage_section(run: &RunDir, config: &Config, now: i64) -> Vec<String> {
	match load_usage(run) {
		Some((info, recorded_at)) => {
			let mut lines = vec![
				format!("5h: {used}%", used = five_hour_used_percent(&info, now)),
				format!("7d: {used}%", used = seven_day_used_percent(&info, now)),
				format!("sampled {age}s ago", age = now - recorded_at),
			];
			if !is_fresh_sample(recorded_at, now, config) {
				lines.push(format!("sample stale: {NO_LAUNCHES_NOTE}"));
			}
			lines
		}
		None => vec![format!("no usage sample: {NO_LAUNCHES_NOTE}")],
	}
}

fn question_of(run: &RunDir, task: &TaskSummary) -> String {
	let notes = read_worker_file(&run.task_dir(&task.id).join("notes.md")).unwrap_or_default();
	let notes = String::from_utf8_lossy(&notes);
	let question = notes
		.rsplit_once(QUESTION_HEADER)
		.map_or("", |(_, question)| question);
	quoted(question, QUESTION_MAX_CHARS)
}

fn needs_answer_section(run: &RunDir, tasks: &[TaskSummary]) -> Vec<String> {
	tasks
		.iter()
		.filter(|task| task.state == TaskState::NeedsAnswer)
		.map(|task| {
			format!(
				"{line}\n  question:\n{question}",
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
				"{line} -> {state:?}\n{summary}",
				line = task_line(task),
				state = task.state,
				summary = quoted(
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

fn read_message(path: &Path) -> Result<(String, bool)> {
	let mut bytes = Vec::new();
	File::open(path)?
		.take(MESSAGE_MAX_BYTES + 1)
		.read_to_end(&mut bytes)?;
	let truncated = bytes.len() as u64 > MESSAGE_MAX_BYTES;
	bytes.truncate(MESSAGE_MAX_BYTES as usize);
	Ok((String::from_utf8_lossy(&bytes).into_owned(), truncated))
}

fn is_temp_name(path: &Path) -> bool {
	path.file_name()
		.is_some_and(|name| name.to_string_lossy().starts_with('.'))
}

fn pending_inbox(run: &RunDir) -> Vec<(String, PathBuf)> {
	let Ok(entries) = fs::read_dir(run.inbox()) else {
		return Vec::new();
	};
	let mut files: Vec<PathBuf> = entries
		.filter_map(Result::ok)
		.filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
		.map(|entry| entry.path())
		.filter(|path| !is_temp_name(path))
		.collect();
	files.sort();
	let mut messages = Vec::new();
	for path in files {
		if messages.len() >= INBOX_SHOWN {
			break;
		}
		let name = path
			.file_name()
			.unwrap_or_default()
			.to_string_lossy()
			.into_owned();
		let Ok((text, truncated)) = read_message(&path) else {
			continue;
		};
		let marker = if truncated {
			format!("\n{QUOTE_PREFIX}[truncated: {name}]")
		} else {
			String::new()
		};
		let message = format!("- {name}\n{body}{marker}", body = quoted(&text, usize::MAX));
		messages.push((message, path));
	}
	messages
}

pub fn ack_inbox(run: &RunDir, paths: &[PathBuf]) -> Result<()> {
	let read_dir = run.inbox().join(INBOX_READ_DIR);
	fs::create_dir_all(&read_dir).with_context(|| format!("create {read_dir:?}"))?;
	for path in paths {
		let name = path.file_name().context("inbox path has no file name")?;
		fs::rename(path, read_dir.join(name)).with_context(|| format!("ack {path:?}"))?;
	}
	Ok(())
}

pub struct Digest {
	pub text: String,
	pub cursor: u64,
	pub inbox: Vec<PathBuf>,
}

struct Sections {
	run: Vec<String>,
	usage: Vec<String>,
	running: Vec<String>,
	needs_answer: Vec<String>,
	queued: Vec<String>,
	recent: Vec<String>,
	events: Vec<(String, String)>,
	omitted_events: BTreeMap<String, usize>,
	inbox: Vec<(String, PathBuf)>,
	journal: Vec<String>,
	kb: Vec<String>,
}

fn event_lines(events: Vec<Event>) -> Vec<(String, String)> {
	events
		.iter()
		.filter_map(|event| {
			serde_json::to_string(event)
				.ok()
				.map(|line| (event.kind(), line))
		})
		.collect()
}

impl Sections {
	fn event_section(&self) -> Vec<String> {
		let mut lines = Vec::new();
		if !self.omitted_events.is_empty() {
			let counts: Vec<String> = self
				.omitted_events
				.iter()
				.map(|(kind, count)| format!("{kind} {count}"))
				.collect();
			lines.push(format!(
				"{OMITTED_EVENTS} {counts}",
				counts = counts.join(", ")
			));
		}
		lines.extend(self.events.iter().map(|(_, line)| line.clone()));
		lines
	}

	fn render(&self) -> String {
		let events = self.event_section();
		let inbox: Vec<String> = self
			.inbox
			.iter()
			.map(|(message, _)| message.clone())
			.collect();
		let sections: [(&str, &Vec<String>); 10] = [
			("run", &self.run),
			("usage", &self.usage),
			("running", &self.running),
			("needs-answer", &self.needs_answer),
			("queued", &self.queued),
			("recent", &self.recent),
			("events", &events),
			("inbox", &inbox),
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

	fn omit_oldest_event(&mut self) -> bool {
		if self.events.is_empty() {
			return false;
		}
		let (kind, _) = self.events.remove(0);
		*self.omitted_events.entry(kind).or_default() += 1;
		true
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
		} else if self.omit_oldest_event() {
			true
		} else if self.inbox.len() > 1 {
			self.inbox.pop();
			true
		} else {
			false
		}
	}
}

pub fn build_digest(run: &RunDir, config: &Config, now: i64) -> Digest {
	let store = TaskStore { run };
	let (tasks, tasks_error) = match store.list() {
		Ok(tasks) => (tasks, None),
		Err(error) => (Vec::new(), Some(format!("tasks unavailable: {error:#}"))),
	};
	let cursor = fs::read_to_string(run.manager_cursor())
		.ok()
		.and_then(|text| text.trim().parse().ok())
		.unwrap_or(0);
	let (events, new_cursor) = events_since(run, cursor);
	let mut sections = Sections {
		run: run_section(run, now)
			.into_iter()
			.chain(tasks_error)
			.collect(),
		usage: usage_section(run, config, now),
		running: state_section(&tasks, &[TaskState::Running, TaskState::Interrupted]),
		needs_answer: needs_answer_section(run, &tasks),
		queued: queued_section(&tasks),
		recent: recent_section(&store, &tasks),
		events: event_lines(events),
		omitted_events: BTreeMap::new(),
		inbox: pending_inbox(run),
		journal: journal_tail(run, JOURNAL_SHOWN),
		kb: vec![config.plugin_dir.join("kb/index.md").display().to_string()],
	};
	while sections.events.len() > EVENTS_SHOWN {
		sections.omit_oldest_event();
	}
	let mut text = sections.render();
	while text.len() > config.digest_max_bytes && sections.shrink_once() {
		text = sections.render();
	}
	Digest {
		text,
		cursor: new_cursor,
		inbox: sections.inbox.into_iter().map(|(_, path)| path).collect(),
	}
}
