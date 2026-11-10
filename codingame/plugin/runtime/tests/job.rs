use std::fs;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use contest::broker::{Grant, JobClass, JobRequest};
use contest::job::{
	JobRecord, JobState, VmStat, parse_vmstat, read_job_record, read_vmstat, repin, signal_job,
	spawn_job, swap_delta_exceeds, try_reap, write_job_record,
};
use contest::paths::{JobId, RunDir, TaskId};
use nix::sys::signal::Signal;
use tempfile::TempDir;

const THREADED_PROGRAM: &str = "import subprocess, sys, threading, time\n\
subprocess.Popen(['sleep', '30'])\n\
for _ in range(3):\n    threading.Thread(target=time.sleep, args=(30,), daemon=True).start()\n\
print('ready', flush=True)\n\
time.sleep(30)\n";

fn machine_has_cpus(cpus: &[usize]) -> bool {
	cpus.iter()
		.all(|cpu| Path::new(&format!("/sys/devices/system/cpu/cpu{cpu}")).exists())
}

fn run_dir() -> (TempDir, RunDir) {
	let dir = TempDir::new().unwrap();
	let run = RunDir {
		root: dir.path().to_path_buf(),
	};
	(dir, run)
}

fn request(id: &str, class: JobClass) -> JobRequest {
	JobRequest {
		id: JobId(id.to_string()),
		class,
		gpu: false,
		ram_mb: 100,
		priority: 0,
	}
}

fn record(id: &str, class: JobClass, cmd: &[&str]) -> JobRecord {
	JobRecord {
		id: JobId(id.to_string()),
		owner_task: TaskId("task".to_string()),
		cmd: cmd.iter().map(|part| part.to_string()).collect(),
		cwd: std::env::temp_dir(),
		request: request(id, class),
		state: JobState::Running,
		pid: None,
		starttime: None,
	}
}

fn grant(id: &str, cpus: &[usize]) -> Grant {
	Grant {
		id: JobId(id.to_string()),
		cpus: cpus.to_vec(),
		exclusive: true,
		gpu: false,
		contended: false,
	}
}

fn status_field(pid: u32, field: &str) -> String {
	let text = fs::read_to_string(format!("/proc/{pid}/status")).unwrap();
	text.lines()
		.find_map(|line| line.strip_prefix(field)?.strip_prefix(':'))
		.unwrap()
		.trim()
		.to_string()
}

fn wait_until(what: &str, mut condition: impl FnMut() -> bool) {
	let deadline = Instant::now() + Duration::from_secs(10);
	while !condition() {
		assert!(Instant::now() < deadline, "timed out waiting for {what}");
		sleep(Duration::from_millis(20));
	}
}

fn session_threads(session: u32) -> Vec<u32> {
	let mut threads = Vec::new();
	for entry in fs::read_dir("/proc").unwrap().flatten() {
		let Some(pid) = entry
			.file_name()
			.to_str()
			.and_then(|n| n.parse::<u32>().ok())
		else {
			continue;
		};
		let Ok(stat) = fs::read_to_string(format!("/proc/{pid}/stat")) else {
			continue;
		};
		let after_name = &stat[stat.rfind(')').unwrap() + 1..];
		if after_name.split_whitespace().nth(3) != Some(&session.to_string()) {
			continue;
		}
		for task in fs::read_dir(format!("/proc/{pid}/task")).unwrap().flatten() {
			threads.push(task.file_name().to_str().unwrap().parse().unwrap());
		}
	}
	threads
}

fn thread_cpus(tid: u32) -> String {
	let text = fs::read_to_string(format!("/proc/{tid}/status")).unwrap();
	text.lines()
		.find_map(|line| line.strip_prefix("Cpus_allowed_list:"))
		.unwrap()
		.trim()
		.to_string()
}

#[test]
fn spawned_job_pinned() {
	if !machine_has_cpus(&[1, 5]) {
		eprintln!("skipped: machine lacks cpus 1 and 5");
		return;
	}
	let (_dir, run) = run_dir();
	let job = record(
		"pinned",
		JobClass::Timed {
			min_cores: 1,
			max_cores: 2,
		},
		&["sleep", "5"],
	);
	let pid = spawn_job(&run, &job, &grant("pinned", &[1, 5])).unwrap();
	assert_eq!(status_field(pid, "Cpus_allowed_list"), "1,5");
	signal_job(pid, Signal::SIGKILL).unwrap();
	wait_until("reap", || try_reap(pid).unwrap().is_some());
}

#[test]
fn spawn_sets_job_environment_and_lease() {
	let (_dir, run) = run_dir();
	let job = record(
		"env",
		JobClass::Batch,
		&[
			"sh",
			"-c",
			"echo \"$CONTEST_LEASE|$CONTEST_JOB_DIR|[$CUDA_VISIBLE_DEVICES]\"",
		],
	);
	let pid = spawn_job(&run, &job, &grant("env", &[0])).unwrap();
	wait_until("exit", || try_reap(pid).unwrap().is_some());
	let log = fs::read_to_string(run.job_dir(&job.id).join("log")).unwrap();
	let job_dir = run.job_dir(&job.id);
	let lease = job_dir.join("lease.json");
	assert_eq!(
		log.trim(),
		format!(
			"{lease}|{job_dir}|[]",
			lease = lease.display(),
			job_dir = job_dir.display()
		)
	);
	let lease_json: serde_json::Value =
		serde_json::from_str(&fs::read_to_string(lease).unwrap()).unwrap();
	assert_eq!(lease_json["cpus"], serde_json::json!([0]));
	assert_eq!(lease_json["exclusive"], true);
	assert_eq!(lease_json["contended"], false);
	assert!(lease_json["updated_at"].as_u64().unwrap() > 0);
}

#[test]
fn repin_moves_all_threads() {
	if !machine_has_cpus(&[1, 5, 2, 6]) {
		eprintln!("skipped: machine lacks cpus 1, 2, 5, 6");
		return;
	}
	let (_dir, run) = run_dir();
	let job = record(
		"repin",
		JobClass::Timed {
			min_cores: 1,
			max_cores: 2,
		},
		&["python3", "-c", THREADED_PROGRAM],
	);
	let pid = spawn_job(&run, &job, &grant("repin", &[1, 5])).unwrap();
	wait_until("threads started", || {
		fs::read_to_string(run.job_dir(&job.id).join("log")).is_ok_and(|log| log.contains("ready"))
	});
	let before = session_threads(pid);
	assert!(
		before.len() >= 5,
		"expected several threads, got {before:?}"
	);
	repin(pid, &[2, 6]).unwrap();
	for tid in session_threads(pid) {
		assert_eq!(thread_cpus(tid), "2,6", "thread {tid}");
	}
	signal_job(pid, Signal::SIGKILL).unwrap();
	wait_until("reap", || try_reap(pid).unwrap().is_some());
}

#[test]
fn sigterm_lets_job_checkpoint() {
	let (_dir, run) = run_dir();
	let script = concat!(
		env!("CARGO_MANIFEST_DIR"),
		"/tests/fixtures/checkpoint-job.sh"
	);
	let job = record("ckpt", JobClass::Batch, &[script]);
	let pid = spawn_job(&run, &job, &grant("ckpt", &[0])).unwrap();
	wait_until("ready", || {
		fs::read_to_string(run.job_dir(&job.id).join("log")).is_ok_and(|log| log.contains("ready"))
	});
	signal_job(pid, Signal::SIGTERM).unwrap();
	let mut state = None;
	wait_until("exit", || {
		state = try_reap(pid).unwrap();
		state.is_some()
	});
	assert_eq!(state, Some(JobState::Done { code: 0 }));
	assert!(run.job_dir(&job.id).join("checkpoint").exists());
}

#[test]
fn batch_is_niced() {
	let (_dir, run) = run_dir();
	let batch = record("batch", JobClass::Batch, &["sleep", "5"]);
	let timed = record(
		"timed",
		JobClass::Timed {
			min_cores: 1,
			max_cores: 1,
		},
		&["sleep", "5"],
	);
	let batch_pid = spawn_job(&run, &batch, &grant("batch", &[0])).unwrap();
	let timed_pid = spawn_job(&run, &timed, &grant("timed", &[0])).unwrap();
	assert_eq!(nice_of(batch_pid), 19);
	assert_eq!(nice_of(timed_pid), 0);
	for pid in [batch_pid, timed_pid] {
		signal_job(pid, Signal::SIGKILL).unwrap();
		wait_until("reap", || try_reap(pid).unwrap().is_some());
	}
}

fn nice_of(pid: u32) -> i32 {
	let stat = fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
	let after_name = &stat[stat.rfind(')').unwrap() + 1..];
	after_name
		.split_whitespace()
		.nth(16)
		.unwrap()
		.parse()
		.unwrap()
}

#[test]
fn swap_delta_counts_in_and_out_pages_above_threshold() {
	let prev = VmStat {
		pswpin: 100,
		pswpout: 100,
	};
	assert!(!swap_delta_exceeds(
		&prev,
		&VmStat {
			pswpin: 228,
			pswpout: 228
		}
	));
	assert!(swap_delta_exceeds(
		&prev,
		&VmStat {
			pswpin: 229,
			pswpout: 228
		}
	));
	assert!(swap_delta_exceeds(
		&prev,
		&VmStat {
			pswpin: 100,
			pswpout: 357
		}
	));
	assert!(!swap_delta_exceeds(&prev, &prev));
}

fn vmstat_counter(text: &str, name: &str) -> u64 {
	text.lines()
		.find_map(|line| line.strip_prefix(name)?.strip_prefix(' ')?.parse().ok())
		.unwrap()
}

#[test]
fn read_vmstat_matches_proc_counters() {
	let before = fs::read_to_string("/proc/vmstat").unwrap();
	let sample = read_vmstat().unwrap();
	let after = fs::read_to_string("/proc/vmstat").unwrap();
	assert!(sample.pswpin >= vmstat_counter(&before, "pswpin"));
	assert!(sample.pswpin <= vmstat_counter(&after, "pswpin"));
	assert!(sample.pswpout >= vmstat_counter(&before, "pswpout"));
	assert!(sample.pswpout <= vmstat_counter(&after, "pswpout"));
}

#[test]
fn parse_vmstat_reads_both_counters() {
	let parsed = parse_vmstat("nr_free_pages 5\npswpin 12\npswpout 34\n").unwrap();
	assert_eq!(
		parsed,
		VmStat {
			pswpin: 12,
			pswpout: 34
		}
	);
}

#[test]
fn parse_vmstat_missing_counter_is_error() {
	let error = parse_vmstat("pswpin 12\npgfault 9\n").unwrap_err();
	assert!(error.to_string().contains("pswpout"), "{error}");
}

#[test]
fn spawn_rejects_grant_of_other_job() {
	let (_dir, run) = run_dir();
	let job = record("a", JobClass::Batch, &["true"]);
	assert!(spawn_job(&run, &job, &grant("b", &[0])).is_err());
}

#[test]
fn repin_of_gone_leader_is_not_an_error() {
	repin(u32::MAX - 1, &[0]).unwrap();
}

#[test]
fn job_record_roundtrips() {
	let (_dir, run) = run_dir();
	let mut job = record(
		"rt",
		JobClass::Timed {
			min_cores: 1,
			max_cores: 3,
		},
		&["a", "b"],
	);
	job.state = JobState::Done { code: 3 };
	job.pid = Some(42);
	write_job_record(&run, &job).unwrap();
	assert_eq!(read_job_record(&run, &job.id).unwrap(), job);
}
