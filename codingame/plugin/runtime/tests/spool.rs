use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;

use contest::broker::{JobClass, JobRequest};
use contest::config::Config;
use contest::job::{JobRecord, JobState, read_job_record, write_job_record};
use contest::paths::{JobId, RunDir, TaskId};
use contest::spool::{Outcome, Request, handle, send_request, serve_pending};
use contest::task::{TaskSpec, TaskState, TaskStore};
use tempfile::TempDir;

fn run_dir() -> (TempDir, RunDir) {
	let dir = TempDir::new().unwrap();
	let run = RunDir {
		root: dir.path().to_path_buf(),
	};
	fs::create_dir_all(run.requests()).unwrap();
	fs::create_dir_all(run.tasks()).unwrap();
	(dir, run)
}

fn make_task(run: &RunDir) -> TaskId {
	let id = TaskStore { run }
		.create(TaskSpec {
			kind: "sim-bench".into(),
			title: "t".into(),
			priority: 0,
			model: None,
			budget_usd: None,
			writes: Vec::new(),
			body: String::new(),
			parent: None,
		})
		.unwrap();
	TaskStore { run }
		.transition(&id, TaskState::Running)
		.unwrap();
	id
}

fn submit_request(class: JobClass) -> Request {
	Request::JobSubmit {
		cmd: vec!["true".into()],
		cwd: "/tmp".into(),
		class,
		gpu: false,
		ram_mb: 100,
		priority: 0,
	}
}

fn queued_job(run: &RunDir, owner: &TaskId, id: &str) -> JobId {
	let job = JobId(id.into());
	write_job_record(
		run,
		&JobRecord {
			id: job.clone(),
			owner_task: owner.clone(),
			cmd: vec!["true".into()],
			cwd: "/tmp".into(),
			request: JobRequest {
				id: job.clone(),
				class: JobClass::Batch,
				gpu: false,
				ram_mb: 1,
				priority: 0,
			},
			state: JobState::Queued,
			pid: None,
		},
	)
	.unwrap();
	job
}

#[test]
fn spool_round_trip() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	let job = queued_job(&run, &task, "j0001");
	let server_run = run.clone();
	let server = thread::spawn(move || {
		let config = Config::default();
		for _ in 0..100 {
			if serve_pending(&server_run, &config).unwrap() > 0 {
				return;
			}
			thread::sleep(Duration::from_millis(20));
		}
		panic!("no request seen");
	});
	let value = send_request(
		&run,
		&task,
		&Request::JobStatus { job },
		Duration::from_secs(10),
	)
	.unwrap();
	server.join().unwrap();
	assert_eq!(value["state"], "Queued");
}

#[test]
fn client_times_out_without_supervisor() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	let job = JobId("j0001".into());
	let outcome = send_request(
		&run,
		&task,
		&Request::JobStatus { job },
		Duration::from_millis(500),
	);
	assert!(outcome.is_err());
	assert_eq!(fs::read_dir(run.requests()).unwrap().count(), 0);
}

#[test]
fn handler_error_reaches_client() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	let server_run = run.clone();
	let server = thread::spawn(move || {
		let config = Config::default();
		while serve_pending(&server_run, &config).unwrap() == 0 {
			thread::sleep(Duration::from_millis(20));
		}
	});
	let job = JobId("missing".into());
	let outcome = send_request(
		&run,
		&task,
		&Request::JobStatus { job },
		Duration::from_secs(10),
	);
	server.join().unwrap();
	assert!(outcome.unwrap_err().to_string().contains("missing"));
}

#[test]
fn job_submit_creates_queued_record() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	let class = JobClass::Timed {
		min_cores: 1,
		max_cores: 2,
	};
	let value = handle(&run, &Config::default(), &task, submit_request(class)).unwrap();
	let id = JobId(value["job"].as_str().unwrap().into());
	let record = read_job_record(&run, &id).unwrap();
	assert_eq!(record.state, JobState::Queued);
	assert_eq!(record.owner_task, task);
	assert_eq!(record.request.class, class);
	let second = handle(
		&run,
		&Config::default(),
		&task,
		submit_request(JobClass::Batch),
	)
	.unwrap();
	assert_ne!(second["job"], value["job"]);
}

#[test]
fn job_submit_rejects_bad_core_counts() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	for (min_cores, max_cores) in [(0, 1), (3, 2)] {
		let class = JobClass::Timed {
			min_cores,
			max_cores,
		};
		assert!(handle(&run, &Config::default(), &task, submit_request(class)).is_err());
	}
}

#[test]
fn job_cancel_marks_queued_job_cancelled() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	let job = queued_job(&run, &task, "j0001");
	handle(
		&run,
		&Config::default(),
		&task,
		Request::JobCancel { job: job.clone() },
	)
	.unwrap();
	assert_eq!(
		read_job_record(&run, &job).unwrap().state,
		JobState::Cancelled
	);
}

#[test]
fn task_finish_transitions() {
	let (_dir, run) = run_dir();
	let store = TaskStore { run: &run };
	let done = make_task(&run);
	let request = Request::TaskFinish {
		outcome: Outcome::Done,
		question: None,
	};
	handle(&run, &Config::default(), &done, request).unwrap();
	assert_eq!(store.state(&done).unwrap(), TaskState::Done);

	let asking = make_task(&run);
	let request = Request::TaskFinish {
		outcome: Outcome::NeedsAnswer,
		question: Some("which cap?".into()),
	};
	handle(&run, &Config::default(), &asking, request).unwrap();
	assert_eq!(store.state(&asking).unwrap(), TaskState::NeedsAnswer);
	let notes = fs::read_to_string(run.task_dir(&asking).join("notes.md")).unwrap();
	assert!(notes.contains("which cap?"));

	let missing_question = make_task(&run);
	let request = Request::TaskFinish {
		outcome: Outcome::NeedsAnswer,
		question: None,
	};
	assert!(handle(&run, &Config::default(), &missing_question, request).is_err());
}

#[test]
fn job_wait_stays_pending_until_job_ends_or_deadline() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	let job = queued_job(&run, &task, "j0001");
	let config = Config::default();
	let waiting = Request::JobWait {
		job: job.clone(),
		timeout_seconds: 1,
	};
	let body = serde_json::json!({ "task": task, "request": waiting });
	let path = run.requests().join("1-1.json");
	fs::write(&path, body.to_string()).unwrap();
	assert_eq!(serve_pending(&run, &config).unwrap(), 0);
	assert!(!run.requests().join("1-1.response.json").exists());

	let mut record = read_job_record(&run, &job).unwrap();
	record.state = JobState::Done { code: 0 };
	write_job_record(&run, &record).unwrap();
	assert_eq!(serve_pending(&run, &config).unwrap(), 1);
	assert!(run.requests().join("1-1.response.json").exists());
}

#[test]
fn job_wait_answers_after_deadline() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	let job = queued_job(&run, &task, "j0001");
	let waiting = Request::JobWait {
		job,
		timeout_seconds: 1,
	};
	let body = serde_json::json!({ "task": task, "request": waiting });
	fs::write(run.requests().join("1-1.json"), body.to_string()).unwrap();
	let config = Config::default();
	assert_eq!(serve_pending(&run, &config).unwrap(), 0);
	thread::sleep(Duration::from_millis(1200));
	assert_eq!(serve_pending(&run, &config).unwrap(), 1);
	let answer = fs::read_to_string(run.requests().join("1-1.response.json")).unwrap();
	assert!(answer.contains("Queued"));
}

#[test]
fn cli_has_no_push() {
	let output = Command::new(env!("CARGO_BIN_EXE_contest"))
		.arg("--help")
		.output()
		.unwrap();
	let help = String::from_utf8_lossy(&output.stdout);
	assert!(help.contains("commit"));
	assert!(!help.contains("push"));
}

#[test]
fn cli_job_submit_prints_job_id() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	let server_run = run.clone();
	let server = thread::spawn(move || {
		let config = Config::default();
		while serve_pending(&server_run, &config).unwrap() == 0 {
			thread::sleep(Duration::from_millis(20));
		}
	});
	let output = Command::new(env!("CARGO_BIN_EXE_contest"))
		.env("CONTEST_RUN", &run.root)
		.env("CONTEST_TASK", &task.0)
		.args([
			"job",
			"submit",
			"--class",
			"timed",
			"--min-cores",
			"2",
			"--ram-mb",
			"500",
			"--",
			"echo",
			"hi",
		])
		.output()
		.unwrap();
	server.join().unwrap();
	assert!(
		output.status.success(),
		"{}",
		String::from_utf8_lossy(&output.stderr)
	);
	let id = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	let record = read_job_record(&run, &JobId(id)).unwrap();
	assert_eq!(record.cmd, ["echo", "hi"]);
	assert_eq!(
		record.request.class,
		JobClass::Timed {
			min_cores: 2,
			max_cores: 2
		}
	);
}
