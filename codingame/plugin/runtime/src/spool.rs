use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::task::Poll;
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail, ensure};
use nix::sys::signal::Signal;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::atomic::write_atomic;
use crate::broker::{JobClass, JobRequest};
use crate::job::{JobRecord, JobState, read_job_record, signal_if_same_process, write_job_record};
use crate::paths::{JobId, RunDir, TaskId};
use crate::taint::read_worker_file;
use crate::task::{TaskState, TaskStore};

const POLL_INTERVAL: Duration = Duration::from_millis(200);
const REQUEST_SUFFIX: &str = ".json";
const MAX_WAIT_SECONDS: u64 = 100;
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

fn owned_record(run: &RunDir, task: &TaskId, job: &JobId) -> Result<JobRecord> {
	let job = JobId::parse(&job.0).map_err(|message| anyhow!(message))?;
	let record = read_job_record(run, &job)?;
	ensure!(
		&record.owner_task == task,
		"job {job} belongs to another task"
	);
	Ok(record)
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
	ensure!(
		!run.stop_file().exists(),
		"stopping: no new jobs, update notes.md and end the turn"
	);
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
		starttime: None,
		sandboxed: false,
	};
	validate_class(&record.request.class)?;
	write_job_record(run, &record)?;
	Ok(job_status(&record))
}

pub fn cancel_job(run: &RunDir, task: &TaskId, job: &JobId) -> Result<Value> {
	let mut record = owned_record(run, task, job)?;
	if is_ended(&record.state) || record.state == JobState::Cancelling {
		return Ok(job_status(&record));
	}
	record.state = match record.state {
		JobState::Running => {
			signal_if_same_process(&record, Signal::SIGTERM)?;
			JobState::Cancelling
		}
		_ => JobState::Cancelled,
	};
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

pub struct CommitRequest<'a> {
	pub task: &'a TaskId,
	pub stem: &'a str,
	pub milestone: &'a str,
}

pub type Committer<'a> = &'a dyn Fn(&CommitRequest) -> Poll<Result<Option<String>>>;
pub type MilestoneCommitter<'a> = &'a dyn Fn(&str) -> Poll<Result<Option<String>>>;

fn is_pending(value: &Value) -> bool {
	value["pending"] == json!(true)
}

pub fn handle(
	run: &RunDir,
	task: &TaskId,
	request: Request,
	commit: MilestoneCommitter,
) -> Result<Value> {
	match request {
		Request::Commit { milestone } => match commit(&milestone) {
			Poll::Pending => Ok(json!({ "pending": true })),
			Poll::Ready(sha) => Ok(json!({ "sha": sha? })),
		},
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
		Request::JobStatus { job } => Ok(job_status(&owned_record(run, task, &job)?)),
		Request::JobWait { job, .. } => {
			let record = owned_record(run, task, &job)?;
			let mut status = job_status(&record);
			status["pending"] = json!(!is_ended(&record.state));
			Ok(status)
		}
		Request::JobCancel { job } => cancel_job(run, task, &job),
		Request::TaskFinish { outcome, question } => finish_task(run, task, outcome, question),
	}
}

fn request_stems(dir: &Path) -> Result<Vec<String>> {
	let mut stems = Vec::new();
	for entry in fs::read_dir(dir)? {
		let entry = entry?;
		let name = entry.file_name().to_string_lossy().into_owned();
		if name.starts_with('.') || name.ends_with(RESPONSE_SUFFIX) || entry.file_type()?.is_dir() {
			continue;
		}
		if let Some(stem) = name.strip_suffix(REQUEST_SUFFIX) {
			stems.push(stem.to_owned());
		}
	}
	stems.sort();
	Ok(stems)
}

fn request_path(dir: &Path, stem: &str) -> PathBuf {
	dir.join(format!("{stem}{REQUEST_SUFFIX}"))
}

fn response_path(dir: &Path, stem: &str) -> PathBuf {
	dir.join(format!("{stem}{RESPONSE_SUFFIX}"))
}

fn deadline_passed(path: &Path, timeout_seconds: u64) -> bool {
	let received = fs::symlink_metadata(path)
		.and_then(|meta| meta.modified())
		.unwrap_or_else(|_| SystemTime::now());
	SystemTime::now() >= received + Duration::from_secs(timeout_seconds)
}

fn remove_refused(path: &Path) -> io::Result<()> {
	match fs::remove_file(path) {
		Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
		_ => Ok(()),
	}
}

fn answer_one(
	run: &RunDir,
	task: &TaskId,
	dir: &Path,
	stem: &str,
	commit: Committer,
) -> Result<Option<Response>> {
	let path = &request_path(dir, stem);
	let bytes = match read_worker_file(path) {
		Ok(bytes) => bytes,
		Err(error) => {
			remove_refused(path)?;
			return Ok(Some(failure(&error)));
		}
	};
	let request: Request = match serde_json::from_slice(&bytes) {
		Ok(request) => request,
		Err(error) => return Ok(Some(failure(&error.into()))),
	};
	let waiting_timeout = match &request {
		Request::JobWait {
			timeout_seconds, ..
		} => Some((*timeout_seconds).min(MAX_WAIT_SECONDS)),
		_ => None,
	};
	let is_commit = matches!(request, Request::Commit { .. });
	let commit_this = |milestone: &str| {
		commit(&CommitRequest {
			task,
			stem,
			milestone,
		})
	};
	let mut value = match handle(run, task, request, &commit_this) {
		Ok(value) => value,
		Err(error) => return Ok(Some(failure(&error))),
	};
	if is_commit && is_pending(&value) {
		return Ok(None);
	}
	if let Some(timeout_seconds) = waiting_timeout
		&& is_pending(&value)
	{
		if !deadline_passed(path, timeout_seconds) {
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

fn running_task_dirs(run: &RunDir) -> Result<Vec<(TaskId, PathBuf)>> {
	let store = TaskStore { run };
	let mut dirs = Vec::new();
	for entry in fs::read_dir(run.requests())? {
		let entry = entry?;
		let Ok(task) = TaskId::parse(&entry.file_name().to_string_lossy()) else {
			continue;
		};
		if !entry.file_type()?.is_dir() || store.state(&task).ok() != Some(TaskState::Running) {
			continue;
		}
		dirs.push((task, entry.path()));
	}
	dirs.sort_by(|a, b| a.0.0.cmp(&b.0.0));
	Ok(dirs)
}

pub fn request_pending(run: &RunDir, task: &TaskId, stem: &str) -> bool {
	fs::symlink_metadata(request_path(&run.task_requests(task), stem)).is_ok()
}

pub fn prepare_task_requests(run: &RunDir, task: &TaskId) -> Result<()> {
	let dir = run.task_requests(task);
	match fs::remove_dir_all(&dir) {
		Err(error) if error.kind() != io::ErrorKind::NotFound => {
			return Err(error).with_context(|| format!("empty {dir:?}"));
		}
		_ => {}
	}
	fs::create_dir_all(&dir).with_context(|| format!("create {dir:?}"))?;
	Ok(())
}

fn serve_task(run: &RunDir, task: &TaskId, dir: &Path, commit: Committer) -> Result<usize> {
	let mut answered = 0;
	for stem in request_stems(dir)? {
		let target = response_path(dir, &stem);
		if target.exists() {
			continue;
		}
		if let Some(response) = answer_one(run, task, dir, &stem, commit)? {
			write_atomic(&target, &serde_json::to_vec(&response)?)?;
			answered += 1;
		}
	}
	Ok(answered)
}

pub fn serve_pending(run: &RunDir, commit: Committer) -> Result<usize> {
	let mut answered = 0;
	for (task, dir) in running_task_dirs(run)? {
		match serve_task(run, &task, &dir, commit) {
			Ok(count) => answered += count,
			Err(error) => {
				let _ = writeln!(io::stderr(), "spool: skipped task {task}: {error:#}");
			}
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
	let task = TaskId::parse(&task.0).map_err(|message| anyhow!(message))?;
	let dir = run.task_requests(&task);
	ensure!(dir.is_dir(), "no request directory for task {task}");
	let stem = client_stem();
	write_atomic(&request_path(&dir, &stem), &serde_json::to_vec(request)?)?;
	let started = Instant::now();
	loop {
		if let Ok(bytes) = fs::read(response_path(&dir, &stem)) {
			let response: Response = serde_json::from_slice(&bytes)?;
			let _ = fs::remove_file(response_path(&dir, &stem));
			let _ = fs::remove_file(request_path(&dir, &stem));
			return if response.ok {
				Ok(response.value.unwrap_or(Value::Null))
			} else {
				Err(anyhow!(response.error.unwrap_or_default()))
			};
		}
		if started.elapsed() >= timeout {
			let _ = fs::remove_file(request_path(&dir, &stem));
			bail!(
				"no supervisor answer within {seconds} s",
				seconds = timeout.as_secs()
			);
		}
		sleep(POLL_INTERVAL);
	}
}
