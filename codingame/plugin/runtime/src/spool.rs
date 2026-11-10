use std::fs;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail, ensure};
use nix::sys::signal::Signal;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::atomic::write_atomic;
use crate::broker::{JobClass, JobRequest};
use crate::commit::commit_task;
use crate::config::Config;
use crate::job::{JobRecord, JobState, read_job_record, signal_job, write_job_record};
use crate::paths::{JobId, RunDir, TaskId};
use crate::task::{TaskState, TaskStore};

const POLL_INTERVAL: Duration = Duration::from_millis(200);
const REQUEST_SUFFIX: &str = ".json";
const RESPONSE_SUFFIX: &str = ".response.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
	Done,
	Failed,
	NeedsAnswer,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
	Commit {
		milestone: String,
	},
	JobSubmit {
		cmd: Vec<String>,
		cwd: PathBuf,
		class: JobClass,
		gpu: bool,
		ram_mb: u64,
		priority: i32,
	},
	JobStatus {
		job: JobId,
	},
	JobWait {
		job: JobId,
		timeout_seconds: u64,
	},
	JobCancel {
		job: JobId,
	},
	TaskFinish {
		outcome: Outcome,
		question: Option<String>,
	},
}

#[derive(Serialize, Deserialize)]
struct Envelope {
	task: TaskId,
	request: Request,
}

#[derive(Serialize, Deserialize)]
struct Response {
	ok: bool,
	value: Option<Value>,
	error: Option<String>,
}

fn is_ended(state: &JobState) -> bool {
	matches!(
		state,
		JobState::Done { .. } | JobState::Interrupted | JobState::Cancelled
	)
}

fn job_status(record: &JobRecord) -> Value {
	json!({ "job": record.id, "state": record.state, "pid": record.pid })
}

fn next_job_id(run: &RunDir) -> JobId {
	let highest = fs::read_dir(run.jobs())
		.into_iter()
		.flatten()
		.flatten()
		.filter_map(|entry| {
			entry
				.file_name()
				.to_str()?
				.strip_prefix('j')?
				.parse::<u32>()
				.ok()
		})
		.max()
		.unwrap_or(0);
	JobId(format!("j{next:04}", next = highest + 1))
}

fn validate_class(class: &JobClass) -> Result<()> {
	if let JobClass::Timed {
		min_cores,
		max_cores,
	} = class
	{
		ensure!(*min_cores >= 1, "min_cores must be at least 1");
		ensure!(
			max_cores >= min_cores,
			"max_cores must be at least min_cores"
		);
	}
	Ok(())
}

fn submit_job(
	run: &RunDir,
	task: &TaskId,
	cmd: Vec<String>,
	cwd: PathBuf,
	request: impl FnOnce(JobId) -> JobRequest,
) -> Result<Value> {
	ensure!(!cmd.is_empty(), "job command is empty");
	let id = next_job_id(run);
	let record = JobRecord {
		id: id.clone(),
		owner_task: task.clone(),
		cmd,
		cwd,
		request: request(id),
		state: JobState::Queued,
		pid: None,
	};
	validate_class(&record.request.class)?;
	write_job_record(run, &record)?;
	Ok(job_status(&record))
}

fn cancel_job(run: &RunDir, job: &JobId) -> Result<Value> {
	let mut record = read_job_record(run, job)?;
	if is_ended(&record.state) {
		return Ok(job_status(&record));
	}
	if let (JobState::Running, Some(pid)) = (&record.state, record.pid) {
		signal_job(pid, Signal::SIGTERM)?;
	}
	record.state = JobState::Cancelled;
	write_job_record(run, &record)?;
	Ok(job_status(&record))
}

fn finish_task(
	run: &RunDir,
	task: &TaskId,
	outcome: Outcome,
	question: Option<String>,
) -> Result<Value> {
	let store = TaskStore { run };
	let next = match outcome {
		Outcome::Done => TaskState::Done,
		Outcome::Failed => TaskState::Failed,
		Outcome::NeedsAnswer => TaskState::NeedsAnswer,
	};
	let question = match outcome {
		Outcome::NeedsAnswer => Some(question.context("needs-answer requires a question")?),
		_ => None,
	};
	store.transition(task, next)?;
	if let Some(question) = question {
		store.append_notes(task, &format!("\n## Question\n\n{question}\n"))?;
	}
	Ok(json!({ "task": task, "state": next }))
}

pub fn handle(run: &RunDir, config: &Config, task: &TaskId, request: Request) -> Result<Value> {
	match request {
		Request::Commit { milestone } => {
			let sha = commit_task(run, config, task, &milestone)?;
			Ok(json!({ "sha": sha }))
		}
		Request::JobSubmit {
			cmd,
			cwd,
			class,
			gpu,
			ram_mb,
			priority,
		} => submit_job(run, task, cmd, cwd, |id| JobRequest {
			id,
			class,
			gpu,
			ram_mb,
			priority,
		}),
		Request::JobStatus { job } => Ok(job_status(&read_job_record(run, &job)?)),
		Request::JobWait { job, .. } => {
			let record = read_job_record(run, &job)?;
			let mut status = job_status(&record);
			status["pending"] = json!(!is_ended(&record.state));
			Ok(status)
		}
		Request::JobCancel { job } => cancel_job(run, &job),
		Request::TaskFinish { outcome, question } => finish_task(run, task, outcome, question),
	}
}

fn request_stems(run: &RunDir) -> Result<Vec<String>> {
	let mut stems = Vec::new();
	for entry in fs::read_dir(run.requests())? {
		let name = entry?.file_name().to_string_lossy().into_owned();
		if name.starts_with('.') || name.ends_with(RESPONSE_SUFFIX) {
			continue;
		}
		if let Some(stem) = name.strip_suffix(REQUEST_SUFFIX) {
			stems.push(stem.to_owned());
		}
	}
	stems.sort();
	Ok(stems)
}

fn request_path(run: &RunDir, stem: &str) -> PathBuf {
	run.requests().join(format!("{stem}{REQUEST_SUFFIX}"))
}

fn response_path(run: &RunDir, stem: &str) -> PathBuf {
	run.requests().join(format!("{stem}{RESPONSE_SUFFIX}"))
}

fn deadline_passed(path: &Path, timeout_seconds: u64) -> bool {
	let received = fs::metadata(path)
		.and_then(|meta| meta.modified())
		.unwrap_or_else(|_| SystemTime::now());
	SystemTime::now() >= received + Duration::from_secs(timeout_seconds)
}

fn answer_one(run: &RunDir, config: &Config, stem: &str) -> Result<Option<Response>> {
	let path = request_path(run, stem);
	let envelope: Envelope = match fs::read(&path)
		.map_err(anyhow::Error::from)
		.and_then(|bytes| Ok(serde_json::from_slice(&bytes)?))
	{
		Ok(envelope) => envelope,
		Err(error) => return Ok(Some(failure(&error))),
	};
	let waiting_timeout = match &envelope.request {
		Request::JobWait {
			timeout_seconds, ..
		} => Some(*timeout_seconds),
		_ => None,
	};
	let mut value = match handle(run, config, &envelope.task, envelope.request) {
		Ok(value) => value,
		Err(error) => return Ok(Some(failure(&error))),
	};
	if let Some(timeout_seconds) = waiting_timeout
		&& value["pending"] == json!(true)
	{
		if !deadline_passed(&path, timeout_seconds) {
			return Ok(None);
		}
		value["timed_out"] = json!(true);
	}
	Ok(Some(Response {
		ok: true,
		value: Some(value),
		error: None,
	}))
}

fn failure(error: &anyhow::Error) -> Response {
	Response {
		ok: false,
		value: None,
		error: Some(format!("{error:#}")),
	}
}

pub fn serve_pending(run: &RunDir, config: &Config) -> Result<usize> {
	let mut answered = 0;
	for stem in request_stems(run)? {
		let target = response_path(run, &stem);
		if target.exists() {
			continue;
		}
		if let Some(response) = answer_one(run, config, &stem)? {
			write_atomic(&target, &serde_json::to_vec(&response)?)?;
			answered += 1;
		}
	}
	Ok(answered)
}

fn client_stem() -> String {
	let nanos = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_or(0, |elapsed| elapsed.as_nanos());
	format!("{nanos}-{pid}", pid = std::process::id())
}

pub fn send_request(
	run: &RunDir,
	task: &TaskId,
	request: &Request,
	timeout: Duration,
) -> Result<Value> {
	fs::create_dir_all(run.requests())?;
	let stem = client_stem();
	let envelope = Envelope {
		task: task.clone(),
		request: request.clone(),
	};
	write_atomic(&request_path(run, &stem), &serde_json::to_vec(&envelope)?)?;
	let started = Instant::now();
	loop {
		if let Ok(bytes) = fs::read(response_path(run, &stem)) {
			let response: Response = serde_json::from_slice(&bytes)?;
			let _ = fs::remove_file(response_path(run, &stem));
			let _ = fs::remove_file(request_path(run, &stem));
			return if response.ok {
				Ok(response.value.unwrap_or(Value::Null))
			} else {
				Err(anyhow!(response.error.unwrap_or_default()))
			};
		}
		if started.elapsed() >= timeout {
			let _ = fs::remove_file(request_path(run, &stem));
			bail!(
				"no supervisor answer within {seconds} s",
				seconds = timeout.as_secs()
			);
		}
		sleep(POLL_INTERVAL);
	}
}
