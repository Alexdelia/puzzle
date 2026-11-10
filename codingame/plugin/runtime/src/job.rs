use std::fs::{self, File};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use nix::errno::Errno;
use nix::sched::{CpuSet, sched_setaffinity};
use nix::sys::signal::{Signal, killpg};
use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};
use nix::unistd::{Pid, setsid};
use serde::{Deserialize, Serialize};

use crate::atomic::write_atomic;
use crate::broker::{Grant, JobClass, JobRequest};
use crate::paths::{JobId, RunDir, TaskId};

const BATCH_NICE: libc::c_int = 19;
const SWAP_PAGES_THRESHOLD: u64 = 256;
const VMSTAT_PATH: &str = "/proc/vmstat";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
	Queued,
	Running,
	Done { code: i32 },
	Interrupted,
	Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobRecord {
	pub id: JobId,
	pub owner_task: TaskId,
	pub cmd: Vec<String>,
	pub cwd: PathBuf,
	pub request: JobRequest,
	pub state: JobState,
	pub pid: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VmStat {
	pub pswpin: u64,
	pub pswpout: u64,
}

#[derive(Serialize)]
struct Lease<'a> {
	cpus: &'a [usize],
	exclusive: bool,
	contended: bool,
	updated_at: u64,
}

impl RunDir {
	pub fn job_record_path(&self, id: &JobId) -> PathBuf {
		self.job_dir(id).join("job.toml")
	}

	pub fn job_lease_path(&self, id: &JobId) -> PathBuf {
		self.job_dir(id).join("lease.json")
	}

	pub fn job_log_path(&self, id: &JobId) -> PathBuf {
		self.job_dir(id).join("log")
	}
}

pub fn write_job_record(run: &RunDir, record: &JobRecord) -> anyhow::Result<()> {
	fs::create_dir_all(run.job_dir(&record.id))?;
	write_atomic(
		&run.job_record_path(&record.id),
		toml::to_string(record)?.as_bytes(),
	)?;
	Ok(())
}

pub fn read_job_record(run: &RunDir, id: &JobId) -> anyhow::Result<JobRecord> {
	let path = run.job_record_path(id);
	let text = fs::read_to_string(&path).with_context(|| format!("read {path:?}"))?;
	Ok(toml::from_str(&text)?)
}

pub fn write_lease(run: &RunDir, grant: &Grant) -> anyhow::Result<()> {
	let updated_at = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_or(0, |elapsed| elapsed.as_secs());
	let lease = Lease {
		cpus: &grant.cpus,
		exclusive: grant.exclusive,
		contended: grant.contended,
		updated_at,
	};
	fs::create_dir_all(run.job_dir(&grant.id))?;
	write_atomic(
		&run.job_lease_path(&grant.id),
		serde_json::to_string(&lease)?.as_bytes(),
	)?;
	Ok(())
}

fn cpu_set(cpus: &[usize]) -> nix::Result<CpuSet> {
	let mut set = CpuSet::new();
	for &cpu in cpus {
		set.set(cpu)?;
	}
	Ok(set)
}

pub fn spawn_job(run: &RunDir, record: &JobRecord, grant: &Grant) -> anyhow::Result<u32> {
	anyhow::ensure!(
		grant.id == record.id,
		"grant {grant_id} does not belong to job {job_id}",
		grant_id = grant.id,
		job_id = record.id
	);
	let (program, args) = record.cmd.split_first().context("job command is empty")?;
	let job_dir = run.job_dir(&record.id);
	fs::create_dir_all(&job_dir)?;
	write_lease(run, grant)?;
	let log = File::create(run.job_log_path(&record.id))?;
	let set = cpu_set(&grant.cpus)?;
	let batch = matches!(record.request.class, JobClass::Batch);
	let mut command = Command::new(program);
	command
		.args(args)
		.current_dir(&record.cwd)
		.env("CONTEST_LEASE", run.job_lease_path(&record.id))
		.env("CONTEST_JOB_DIR", &job_dir)
		.env("CUDA_VISIBLE_DEVICES", if grant.gpu { "0" } else { "" })
		.stdin(Stdio::null())
		.stdout(log.try_clone()?)
		.stderr(log);
	unsafe {
		command.pre_exec(move || {
			setsid().map_err(std::io::Error::from)?;
			sched_setaffinity(Pid::from_raw(0), &set).map_err(std::io::Error::from)?;
			if batch && libc::setpriority(libc::PRIO_PROCESS, 0, BATCH_NICE) != 0 {
				return Err(std::io::Error::last_os_error());
			}
			Ok(())
		});
	}
	let child = command
		.spawn()
		.with_context(|| format!("spawn {program}"))?;
	Ok(child.id())
}

pub fn try_reap(pid: u32) -> anyhow::Result<Option<JobState>> {
	let status = waitpid(Pid::from_raw(pid as i32), Some(WaitPidFlag::WNOHANG))?;
	Ok(match status {
		WaitStatus::Exited(_, code) => Some(JobState::Done { code }),
		WaitStatus::Signaled(_, signal, _) => Some(JobState::Done {
			code: 128 + signal as i32,
		}),
		_ => None,
	})
}

pub fn signal_job(pid: u32, signal: Signal) -> anyhow::Result<()> {
	match killpg(Pid::from_raw(pid as i32), signal) {
		Ok(()) | Err(Errno::ESRCH) => Ok(()),
		Err(error) => Err(error.into()),
	}
}

fn session_of(pid: u32) -> Option<u32> {
	let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
	let after_name = &stat[stat.rfind(')')? + 1..];
	after_name.split_whitespace().nth(3)?.parse().ok()
}

fn all_pids() -> Vec<u32> {
	fs::read_dir("/proc")
		.into_iter()
		.flatten()
		.flatten()
		.filter_map(|entry| entry.file_name().to_str()?.parse().ok())
		.collect()
}

fn thread_ids(pid: u32) -> Vec<u32> {
	fs::read_dir(format!("/proc/{pid}/task"))
		.into_iter()
		.flatten()
		.flatten()
		.filter_map(|entry| entry.file_name().to_str()?.parse().ok())
		.collect()
}

pub fn repin(pid: u32, cpus: &[usize]) -> anyhow::Result<()> {
	let session = pid;
	let set = cpu_set(cpus)?;
	for process in all_pids() {
		if session_of(process) != Some(session) {
			continue;
		}
		for thread in thread_ids(process) {
			match sched_setaffinity(Pid::from_raw(thread as i32), &set) {
				Ok(()) | Err(Errno::ESRCH) => {}
				Err(error) => return Err(error).with_context(|| format!("repin thread {thread}")),
			}
		}
	}
	Ok(())
}

pub fn parse_vmstat(text: &str) -> anyhow::Result<VmStat> {
	let counter = |name: &str| -> anyhow::Result<u64> {
		text.lines()
			.find_map(|line| {
				line.strip_prefix(name)?
					.strip_prefix(' ')?
					.trim()
					.parse()
					.ok()
			})
			.with_context(|| format!("counter {name} missing or unparseable in {VMSTAT_PATH}"))
	};
	Ok(VmStat {
		pswpin: counter("pswpin")?,
		pswpout: counter("pswpout")?,
	})
}

pub fn read_vmstat() -> anyhow::Result<VmStat> {
	parse_vmstat(&fs::read_to_string(VMSTAT_PATH)?)
}

pub fn swap_delta_exceeds(prev: &VmStat, now: &VmStat) -> bool {
	let moved = now.pswpin.saturating_sub(prev.pswpin) + now.pswpout.saturating_sub(prev.pswpout);
	moved > SWAP_PAGES_THRESHOLD
}

pub fn swapping_now(prev: &VmStat) -> anyhow::Result<(bool, VmStat)> {
	let now = read_vmstat()?;
	Ok((swap_delta_exceeds(prev, &now), now))
}
