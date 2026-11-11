use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::journal::clip_chars;
use crate::paths::{JobId, RunDir, TaskId};
use crate::task::TaskState;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
	TaskFinished { task: TaskId, state: TaskState },
	JobFinished { job: JobId, code: i32 },
	JobInterrupted { job: JobId },
	UserMessage { text: String },
	ManagerError { error: String },
	BudgetExhausted,
	UsageBlocked { until: i64 },
	UsageUnknown,
	LaunchFailed { task: TaskId, error: String },
	TaintOverBudget { max_entries: usize },
}

const USER_TEXT_MAX_CHARS: usize = 200;
const ERROR_MAX_CHARS: usize = 500;
const SCAN_CHUNK_BYTES: usize = 64 * 1024;

impl Event {
	fn bounded(self) -> Self {
		match self {
			Self::UserMessage { text } => Self::UserMessage {
				text: clip_chars(&text, USER_TEXT_MAX_CHARS),
			},
			Self::ManagerError { error } => Self::ManagerError {
				error: clip_chars(&error, ERROR_MAX_CHARS),
			},
			Self::LaunchFailed { task, error } => Self::LaunchFailed {
				task,
				error: clip_chars(&error, ERROR_MAX_CHARS),
			},
			other => other,
		}
	}

	pub fn kind(&self) -> String {
		serde_json::to_value(self)
			.ok()
			.and_then(|value| value["event"].as_str().map(str::to_owned))
			.unwrap_or_default()
	}
}

fn now_unix() -> u64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_or(0, |elapsed| elapsed.as_secs())
}

pub fn emit(run: &RunDir, event: Event) -> anyhow::Result<()> {
	let Value::Object(mut fields) = serde_json::to_value(event.bounded())? else {
		anyhow::bail!("event is not a json object");
	};
	fields.insert("at".to_owned(), Value::from(now_unix()));
	let path = run.events();
	if let Some(dir) = path.parent() {
		fs::create_dir_all(dir)?;
	}
	let mut file = OpenOptions::new().create(true).append(true).open(path)?;
	file.write_all(format!("{line}\n", line = Value::Object(fields)).as_bytes())?;
	Ok(())
}

pub fn events_since(run: &RunDir, cursor: u64) -> (Vec<Event>, u64) {
	let Ok(mut file) = File::open(run.events()) else {
		return (Vec::new(), cursor);
	};
	if file.seek(SeekFrom::Start(cursor)).is_err() {
		return (Vec::new(), cursor);
	}
	let mut bytes = Vec::new();
	if file.read_to_end(&mut bytes).is_err() {
		return (Vec::new(), cursor);
	}
	let complete = bytes
		.iter()
		.rposition(|&byte| byte == b'\n')
		.map_or(0, |last| last + 1);
	let events = String::from_utf8_lossy(&bytes[..complete])
		.lines()
		.filter_map(|line| serde_json::from_str(line).ok())
		.collect();
	(events, cursor + complete as u64)
}

pub fn has_complete_event_after(run: &RunDir, cursor: u64) -> bool {
	let Ok(mut file) = File::open(run.events()) else {
		return false;
	};
	if file.seek(SeekFrom::Start(cursor)).is_err() {
		return false;
	}
	let mut chunk = vec![0; SCAN_CHUNK_BYTES];
	loop {
		match file.read(&mut chunk) {
			Ok(0) | Err(_) => return false,
			Ok(read) if chunk[..read].contains(&b'\n') => return true,
			Ok(_) => {}
		}
	}
}
