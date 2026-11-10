use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

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
}

fn now_unix() -> u64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_or(0, |elapsed| elapsed.as_secs())
}

pub fn emit(run: &RunDir, event: Event) {
	let Ok(Value::Object(mut fields)) = serde_json::to_value(&event) else {
		return;
	};
	fields.insert("at".to_owned(), Value::from(now_unix()));
	let path = run.events();
	if let Some(dir) = path.parent() {
		let _ = fs::create_dir_all(dir);
	}
	if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
		let _ = file.write_all(format!("{line}\n", line = Value::Object(fields)).as_bytes());
	}
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
