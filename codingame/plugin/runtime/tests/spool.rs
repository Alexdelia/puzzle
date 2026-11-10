use std::fs;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::Command;
use std::thread;
use std::time::Duration;

use contest::broker::{JobClass, JobRequest};
use contest::config::Config;
use contest::job::{
	JobRecord, JobState, mark_spawned, process_starttime, read_job_record, write_job_record,
};
use contest::paths::{JobId, RunDir, TaskId};
use contest::spool::{
	Outcome, Request, handle, prepare_task_requests, send_request, serve_pending,
};
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
	prepare_task_requests(run, &id).unwrap();
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
			starttime: None,
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
	assert_eq!(fs::read_dir(run.task_requests(&task)).unwrap().count(), 0);
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
	let dir = run.task_requests(&task);
	fs::write(
		dir.join("1-1.json"),
		serde_json::to_string(&waiting).unwrap(),
	)
	.unwrap();
	assert_eq!(serve_pending(&run, &config).unwrap(), 0);
	assert!(!run.task_requests(&task).join("1-1.response.json").exists());

	let mut record = read_job_record(&run, &job).unwrap();
	record.state = JobState::Done { code: 0 };
	write_job_record(&run, &record).unwrap();
	assert_eq!(serve_pending(&run, &config).unwrap(), 1);
	assert!(run.task_requests(&task).join("1-1.response.json").exists());
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
	let dir = run.task_requests(&task);
	fs::write(
		dir.join("1-1.json"),
		serde_json::to_string(&waiting).unwrap(),
	)
	.unwrap();
	let config = Config::default();
	assert_eq!(serve_pending(&run, &config).unwrap(), 0);
	thread::sleep(Duration::from_millis(1200));
	assert_eq!(serve_pending(&run, &config).unwrap(), 1);
	let answer = fs::read_to_string(run.task_requests(&task).join("1-1.response.json")).unwrap();
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
		"{stderr}",
		stderr = String::from_utf8_lossy(&output.stderr)
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

fn write_request(dir: &std::path::Path, name: &str, body: &str) {
	fs::write(dir.join(name), body).unwrap();
}

fn read_response(dir: &std::path::Path, name: &str) -> serde_json::Value {
	serde_json::from_slice(&fs::read(dir.join(name)).unwrap()).unwrap()
}

#[test]
fn envelope_naming_another_task_is_ignored() {
	let (_dir, run) = run_dir();
	let store = TaskStore { run: &run };
	let mine = make_task(&run);
	let victim = make_task(&run);
	let body = format!(
		r#"{{"task":"{victim}","request":{{"TaskFinish":{{"outcome":"Done","question":null}}}}}}"#
	);
	write_request(&run.task_requests(&mine), "1-1.json", &body);
	serve_pending(&run, &Config::default()).unwrap();
	assert_eq!(store.state(&victim).unwrap(), TaskState::Running);
	assert_eq!(store.state(&mine).unwrap(), TaskState::Running);

	let finish = r#"{"TaskFinish":{"outcome":"Done","question":null}}"#;
	write_request(&run.task_requests(&mine), "2-1.json", finish);
	serve_pending(&run, &Config::default()).unwrap();
	assert_eq!(store.state(&mine).unwrap(), TaskState::Done);
	assert_eq!(store.state(&victim).unwrap(), TaskState::Running);
}

#[test]
fn symlinked_or_unknown_request_dirs_are_not_served() {
	let (dir, run) = run_dir();
	let store = TaskStore { run: &run };
	let target = make_task(&run);
	let outside = dir.path().join("outside");
	fs::create_dir_all(&outside).unwrap();
	let finish = r#"{"TaskFinish":{"outcome":"Done","question":null}}"#;
	write_request(&outside, "1-1.json", finish);
	std::os::unix::fs::symlink(&outside, run.requests().join("t0099-ghost")).unwrap();
	fs::create_dir_all(run.requests().join("t0098-ghost")).unwrap();
	write_request(&run.requests().join("t0098-ghost"), "1-1.json", finish);
	fs::create_dir_all(run.requests().join("not-a-task")).unwrap();
	write_request(&run.requests().join("not-a-task"), "1-1.json", finish);
	assert_eq!(serve_pending(&run, &Config::default()).unwrap(), 0);
	assert_eq!(store.state(&target).unwrap(), TaskState::Running);
	assert!(!outside.join("1-1.response.json").exists());
}

#[test]
fn symlinked_task_dir_pointing_at_other_task_is_not_served() {
	let (_dir, run) = run_dir();
	let store = TaskStore { run: &run };
	let victim = make_task(&run);
	let attacker = make_task(&run);
	fs::remove_dir_all(run.task_requests(&attacker)).unwrap();
	std::os::unix::fs::symlink(run.task_requests(&victim), run.task_requests(&attacker)).unwrap();
	let finish = r#"{"TaskFinish":{"outcome":"Done","question":null}}"#;
	write_request(&run.task_requests(&victim), "1-1.json", finish);
	let dir = run.task_requests(&victim);
	serve_pending(&run, &Config::default()).unwrap();
	assert_eq!(store.state(&victim).unwrap(), TaskState::Done);
	assert_eq!(store.state(&attacker).unwrap(), TaskState::Running);
	assert_eq!(read_response(&dir, "1-1.response.json")["ok"], true);
}

#[test]
fn only_running_tasks_are_served() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	TaskStore { run: &run }
		.transition(&task, TaskState::Done)
		.unwrap();
	let status = r#"{"JobStatus":{"job":"j0001"}}"#;
	write_request(&run.task_requests(&task), "1-1.json", status);
	assert_eq!(serve_pending(&run, &Config::default()).unwrap(), 0);
}

#[test]
fn job_verbs_refuse_another_tasks_job() {
	let (_dir, run) = run_dir();
	let owner = make_task(&run);
	let intruder = make_task(&run);
	let job = queued_job(&run, &owner, "j0001");
	let config = Config::default();
	for request in [
		Request::JobStatus { job: job.clone() },
		Request::JobWait {
			job: job.clone(),
			timeout_seconds: 1,
		},
		Request::JobCancel { job: job.clone() },
	] {
		assert!(handle(&run, &config, &intruder, request).is_err());
	}
	assert_eq!(read_job_record(&run, &job).unwrap().state, JobState::Queued);
}

#[test]
fn malformed_job_ids_are_rejected() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	for id in ["../x", "j1", "j123", "J0001", ""] {
		let request = Request::JobCancel {
			job: JobId(id.into()),
		};
		assert!(
			handle(&run, &Config::default(), &task, request).is_err(),
			"{id}"
		);
	}
}

fn spawn_sleeper() -> std::process::Child {
	Command::new("sleep")
		.arg("30")
		.process_group(0)
		.spawn()
		.unwrap()
}

fn alive(pid: u32) -> bool {
	std::fs::read_to_string(format!("/proc/{pid}/stat"))
		.map(|stat| !stat.contains(") Z "))
		.unwrap_or(false)
}

fn running_job_of(run: &RunDir, owner: &TaskId, pid: u32, starttime: Option<u64>) -> JobId {
	let job = queued_job(run, owner, "j0001");
	let mut record = read_job_record(run, &job).unwrap();
	record.state = JobState::Running;
	mark_spawned(&mut record, pid);
	record.starttime = starttime;
	write_job_record(run, &record).unwrap();
	job
}

#[test]
fn forged_starttime_is_never_signalled() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	let mut child = spawn_sleeper();
	let real = process_starttime(child.id()).unwrap();
	for forged in [Some(real + 1), None] {
		let job = running_job_of(&run, &task, child.id(), forged);
		handle(&run, &Config::default(), &task, Request::JobCancel { job }).unwrap();
		thread::sleep(Duration::from_millis(200));
		assert!(alive(child.id()));
	}
	child.kill().unwrap();
	child.wait().unwrap();
}

#[test]
fn matching_starttime_is_signalled_and_job_is_cancelling() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	let mut child = spawn_sleeper();
	let real = process_starttime(child.id());
	let job = running_job_of(&run, &task, child.id(), real);
	let value = handle(
		&run,
		&Config::default(),
		&task,
		Request::JobCancel { job: job.clone() },
	)
	.unwrap();
	assert_eq!(value["state"], "Cancelling");
	assert_eq!(child.wait().unwrap().signal(), Some(15));
	assert_eq!(
		read_job_record(&run, &job).unwrap().state,
		JobState::Cancelling
	);
}

#[test]
fn job_wait_does_not_return_ended_while_cancelling() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	let job = queued_job(&run, &task, "j0001");
	let mut record = read_job_record(&run, &job).unwrap();
	record.state = JobState::Cancelling;
	write_job_record(&run, &record).unwrap();
	let value = handle(
		&run,
		&Config::default(),
		&task,
		Request::JobWait {
			job,
			timeout_seconds: 5,
		},
	)
	.unwrap();
	assert_eq!(value["pending"], true);
}

#[test]
fn job_wait_timeout_is_clamped_server_side() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	let job = queued_job(&run, &task, "j0001");
	let waiting = Request::JobWait {
		job,
		timeout_seconds: u64::MAX,
	};
	let dir = run.task_requests(&task);
	write_request(&dir, "1-1.json", &serde_json::to_string(&waiting).unwrap());
	assert_eq!(serve_pending(&run, &Config::default()).unwrap(), 0);
}

fn make_fifo(path: &std::path::Path) {
	let name = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).unwrap();
	assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
}

fn assert_rejected_and_removed(dir: &std::path::Path, name: &str) {
	let response = read_response(dir, &name.replace(".json", ".response.json"));
	assert_eq!(response["ok"], false, "{response}");
	assert!(fs::symlink_metadata(dir.join(name)).is_err());
}

#[test]
fn fifo_request_is_refused_and_removed() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	let dir = run.task_requests(&task);
	make_fifo(&dir.join("1-1.json"));
	assert_eq!(serve_pending(&run, &Config::default()).unwrap(), 1);
	assert_rejected_and_removed(&dir, "1-1.json");
}

#[test]
fn symlinked_request_is_refused_and_removed() {
	let (dir, run) = run_dir();
	let task = make_task(&run);
	let outside = dir.path().join("outside.json");
	fs::write(
		&outside,
		r#"{"TaskFinish":{"outcome":"Done","question":null}}"#,
	)
	.unwrap();
	let requests = run.task_requests(&task);
	std::os::unix::fs::symlink(&outside, requests.join("1-1.json")).unwrap();
	assert_eq!(serve_pending(&run, &Config::default()).unwrap(), 1);
	assert_rejected_and_removed(&requests, "1-1.json");
	assert!(outside.exists());
	assert_eq!(
		TaskStore { run: &run }.state(&task).unwrap(),
		TaskState::Running
	);
}

#[test]
fn oversized_request_is_refused_and_removed() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	let requests = run.task_requests(&task);
	let padding = " ".repeat(1024 * 1024);
	let body = format!(r#"{{"JobStatus":{{"job":"j0001"}}}}{padding}"#);
	write_request(&requests, "1-1.json", &body);
	assert_eq!(serve_pending(&run, &Config::default()).unwrap(), 1);
	assert_rejected_and_removed(&requests, "1-1.json");
}

#[test]
fn prepare_task_requests_empties_the_dir() {
	let (_dir, run) = run_dir();
	let task = make_task(&run);
	let requests = run.task_requests(&task);
	write_request(&requests, "1-1.json", "{}");
	write_request(&requests, "1-1.response.json", "{}");
	fs::create_dir(requests.join("nested")).unwrap();
	std::os::unix::fs::symlink("/", requests.join("root")).unwrap();
	prepare_task_requests(&run, &task).unwrap();
	assert!(requests.is_dir());
	assert_eq!(fs::read_dir(&requests).unwrap().count(), 0);
}

#[test]
fn job_ids_grow_past_four_digits() {
	assert!(JobId::parse("j10000").is_ok());
	assert!(JobId::parse("j0001").is_ok());
	for id in ["j123", "j", "j1000a", "k10000", "j-1000"] {
		assert!(JobId::parse(id).is_err(), "{id}");
	}
}

#[test]
fn one_broken_task_dir_does_not_stop_serving_others() {
	use std::os::unix::fs::PermissionsExt;
	let (_dir, run) = run_dir();
	let store = TaskStore { run: &run };
	let broken = make_task(&run);
	let healthy = make_task(&run);
	let finish = r#"{"TaskFinish":{"outcome":"Done","question":null}}"#;
	write_request(&run.task_requests(&broken), "1-1.json", finish);
	write_request(&run.task_requests(&healthy), "1-1.json", finish);
	let broken_dir = run.task_requests(&broken);
	fs::set_permissions(&broken_dir, fs::Permissions::from_mode(0o000)).unwrap();
	let served = serve_pending(&run, &Config::default());
	fs::set_permissions(&broken_dir, fs::Permissions::from_mode(0o755)).unwrap();
	assert_eq!(served.unwrap(), 1);
	assert_eq!(store.state(&healthy).unwrap(), TaskState::Done);
	assert_eq!(store.state(&broken).unwrap(), TaskState::Running);
}
