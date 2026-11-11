use std::fs::{self, File};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use nix::dir::Dir;
use nix::errno::Errno;
use nix::fcntl::OFlag;
use nix::sched::{CpuSet, sched_setaffinity};
use nix::sys::prctl::set_pdeathsig;
use nix::sys::signal::{Signal, kill, killpg};
use nix::sys::stat::{Mode, mkdirat};
use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};
use nix::unistd::{Pid, setsid};
use serde::{Deserialize, Serialize};

use crate::atomic::write_atomic;
use crate::broker::{Grant, JobClass, JobRequest};
use crate::paths::{JOB_WORK_DIR, JobId, RunDir, TaskId};
use crate::sandbox::{SandboxSpec, sandbox_command};

const BATCH_NICE: libc::c_int = 19;
const SWAP_PAGES_THRESHOLD: u64 = 256;
const VMSTAT_PATH: &str = "/proc/vmstat";
const STAT_PARENT: usize = 1;
const STAT_SESSION: usize = 3;
const STAT_STARTTIME: usize = 19;
const NAMESPACE_INIT_PID: &str = "1";
const MAX_PIN_ROUNDS: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
	Queued,
	Running,
	Done { code: i32 },
	Interrupted,
	Cancelling,
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
	#[serde(default)]
	pub starttime: Option<u64>,
	#[serde(default)]
	pub sandboxed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VmStat {
	pub pswpin: u64,
	pub pswpout: u64,
}

const FIRST_LEASE_GENERATION: u64 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lease {
	pub cpus: Vec<usize>,
	pub exclusive: bool,
	pub contended: bool,
	pub updated_at: u64,
	pub generation: u64,
	pub pending_cpus: Option<Vec<usize>>,
	pub effective_at: Option<i64>,
}

impl Lease {
	pub fn first(grant: &Grant) -> Self {
		Self {
			cpus: grant.cpus.clone(),
			exclusive: grant.exclusive,
			contended: grant.contended,
			updated_at: 0,
			generation: FIRST_LEASE_GENERATION,
			pending_cpus: None,
			effective_at: None,
		}
	}
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

pub fn list_job_records(run: &RunDir) -> Vec<JobRecord> {
	let mut records: Vec<JobRecord> = fs::read_dir(run.jobs())
		.into_iter()
		.flatten()
		.flatten()
		.filter_map(|entry| JobId::parse(entry.file_name().to_str()?).ok())
		.filter_map(|id| read_job_record(run, &id).ok())
		.collect();
	records.sort_by(|a, b| a.id.cmp(&b.id));
	records
}

pub fn write_lease(run: &RunDir, id: &JobId, lease: &Lease) -> anyhow::Result<()> {
	let lease = Lease {
		updated_at: SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.map_or(0, |elapsed| elapsed.as_secs()),
		..lease.clone()
	};
	fs::create_dir_all(run.job_dir(id))?;
	write_atomic(
		&run.job_lease_path(id),
		serde_json::to_string(&lease)?.as_bytes(),
	)?;
	Ok(())
}

pub fn read_lease(run: &RunDir, id: &JobId) -> anyhow::Result<Lease> {
	let path = run.job_lease_path(id);
	let text = fs::read_to_string(&path).with_context(|| format!("read {path:?}"))?;
	Ok(serde_json::from_str(&text)?)
}

fn cpu_set(cpus: &[usize]) -> nix::Result<CpuSet> {
	let mut set = CpuSet::new();
	for &cpu in cpus {
		set.set(cpu)?;
	}
	Ok(set)
}

fn open_dir_no_follow(parent: &Dir, name: &str) -> nix::Result<Dir> {
	Dir::openat(parent, name, no_follow_dir_flags(), Mode::empty())
}

fn no_follow_dir_flags() -> OFlag {
	OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC
}

fn make_dir_no_follow(parent: &Dir, name: &str) -> anyhow::Result<Dir> {
	match mkdirat(parent, name, Mode::S_IRWXU) {
		Ok(()) | Err(Errno::EEXIST) => {}
		Err(error) => return Err(error).with_context(|| format!("create {name}")),
	}
	open_dir_no_follow(parent, name).with_context(|| format!("open {name} without following links"))
}

fn create_job_work_dir(run: &RunDir, record: &JobRecord) -> anyhow::Result<PathBuf> {
	let task_dir = run.task_dir(&record.owner_task);
	fs::create_dir_all(&task_dir)?;
	let task = Dir::open(&task_dir, no_follow_dir_flags(), Mode::empty())
		.with_context(|| format!("open {task_dir:?}"))?;
	let work = make_dir_no_follow(&task, JOB_WORK_DIR)?;
	make_dir_no_follow(&work, &record.id.0)?;
	Ok(run.job_work_dir(&record.owner_task, &record.id))
}

pub fn spawn_job(run: &RunDir, record: &JobRecord, grant: &Grant) -> anyhow::Result<u32> {
	spawn_job_scrubbed(run, record, grant, &[])
}

pub fn spawn_job_scrubbed(
	run: &RunDir,
	record: &JobRecord,
	grant: &Grant,
	removed_env: &[String],
) -> anyhow::Result<u32> {
	anyhow::ensure!(
		grant.id == record.id,
		"grant {grant_id} does not belong to job {job_id}",
		grant_id = grant.id,
		job_id = record.id
	);
	let (program, args) = record.cmd.split_first().context("job command is empty")?;
	let job_dir = run.job_dir(&record.id);
	fs::create_dir_all(&job_dir)?;
	write_lease(run, &grant.id, &Lease::first(grant))?;
	let work = create_job_work_dir(run, record)?;
	let log = File::create(run.job_log_path(&record.id))?;
	let set = cpu_set(&grant.cpus)?;
	let batch = matches!(record.request.class, JobClass::Batch);
	let mut command = Command::new(program);
	for name in removed_env {
		command.env_remove(name);
	}
	command
		.args(args)
		.current_dir(&record.cwd)
		.env("CONTEST_LEASE", run.job_lease_path(&record.id))
		.env("CONTEST_JOB_DIR", &job_dir)
		.env("CONTEST_JOB_WORK", &work)
		.env("CUDA_VISIBLE_DEVICES", if grant.gpu { "0" } else { "" })
		.stdin(Stdio::null())
		.stdout(log.try_clone()?)
		.stderr(log);
	unsafe {
		command.pre_exec(move || {
			setsid().map_err(std::io::Error::from)?;
			set_pdeathsig(Signal::SIGTERM).map_err(std::io::Error::from)?;
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

fn stat_field<T: FromStr>(pid: u32, index: usize) -> Option<T> {
	let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
	let after_name = &stat[stat.rfind(')')? + 1..];
	after_name.split_whitespace().nth(index)?.parse().ok()
}

pub fn process_starttime(pid: u32) -> Option<u64> {
	stat_field(pid, STAT_STARTTIME)
}

pub fn mark_spawned(record: &mut JobRecord, pid: u32) {
	record.pid = Some(pid);
	record.starttime = process_starttime(pid);
}

pub fn signal_if_same_process(record: &JobRecord, signal: Signal) -> anyhow::Result<bool> {
	let (Some(pid), Some(starttime)) = (record.pid, record.starttime) else {
		return Ok(false);
	};
	if pid <= 1 || process_starttime(pid) != Some(starttime) {
		return Ok(false);
	}
	signal_job(pid, record.sandboxed, signal)?;
	Ok(true)
}

pub fn sandbox_job(record: &mut JobRecord, spec: &SandboxSpec) -> anyhow::Result<()> {
	record.cmd = sandbox_command(spec, &record.cmd)?;
	record.sandboxed = true;
	Ok(())
}

pub fn signal_job(pid: u32, sandboxed: bool, signal: Signal) -> anyhow::Result<()> {
	if signal == Signal::SIGKILL {
		return kill_job_tree(pid);
	}
	let group = if sandboxed {
		sandbox_workload_group(pid).unwrap_or(pid)
	} else {
		pid
	};
	signal_group(group, signal)
}

fn signal_group(group: u32, signal: Signal) -> anyhow::Result<()> {
	match killpg(Pid::from_raw(group as i32), signal) {
		Ok(()) | Err(Errno::ESRCH) => Ok(()),
		Err(error) => Err(error.into()),
	}
}

fn kill_job_tree(leader: u32) -> anyhow::Result<()> {
	let tree = process_tree(leader);
	signal_group(leader, Signal::SIGKILL)?;
	for &process in &tree[1..] {
		match kill(Pid::from_raw(process as i32), Signal::SIGKILL) {
			Ok(()) | Err(Errno::ESRCH) => {}
			Err(error) => return Err(error).with_context(|| format!("kill {process}")),
		}
	}
	Ok(())
}

fn only_child(parent: u32) -> Option<u32> {
	match children(&parent_table(), parent)[..] {
		[child] => Some(child),
		_ => None,
	}
}

fn sandbox_workload_group(pasta: u32) -> Option<u32> {
	let bwrap =
		only_child(pasta).filter(|&pid| is_namespace_init(pid) && session_of(pid) != Some(pid))?;
	only_child(bwrap).filter(|&pid| is_namespace_init(pid) && session_of(pid) == Some(pid))
}

fn is_namespace_init(pid: u32) -> bool {
	fs::read_to_string(format!("/proc/{pid}/status")).is_ok_and(|status| {
		status
			.lines()
			.find_map(|line| line.strip_prefix("NSpid:"))
			.and_then(|pids| pids.split_whitespace().last())
			== Some(NAMESPACE_INIT_PID)
	})
}

fn session_of(pid: u32) -> Option<u32> {
	stat_field(pid, STAT_SESSION)
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

fn parent_table() -> Vec<(u32, u32)> {
	all_pids()
		.into_iter()
		.filter_map(|pid| Some((pid, stat_field(pid, STAT_PARENT)?)))
		.collect()
}

fn children(parents: &[(u32, u32)], parent: u32) -> Vec<u32> {
	parents
		.iter()
		.filter(|(_, of)| *of == parent)
		.map(|(pid, _)| *pid)
		.collect()
}

pub fn process_tree(root: u32) -> Vec<u32> {
	let parents = parent_table();
	let mut tree = vec![root];
	let mut next = 0;
	while let Some(&parent) = tree.get(next) {
		tree.extend(children(&parents, parent));
		next += 1;
	}
	tree
}

fn pin_until_no_new_member(
	members: impl Fn() -> Vec<u32>,
	mut pin: impl FnMut(u32) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
	let mut pinned: Vec<u32> = Vec::new();
	for _ in 0..MAX_PIN_ROUNDS {
		let fresh: Vec<u32> = members()
			.into_iter()
			.filter(|member| !pinned.contains(member))
			.collect();
		if fresh.is_empty() {
			return Ok(());
		}
		for &member in &fresh {
			pin(member)?;
		}
		pinned.extend(fresh);
	}
	Ok(())
}

fn pin_threads(process: u32, set: &CpuSet) -> anyhow::Result<()> {
	pin_until_no_new_member(
		|| thread_ids(process),
		|thread| match sched_setaffinity(Pid::from_raw(thread as i32), set) {
			Ok(()) | Err(Errno::ESRCH) => Ok(()),
			Err(error) => Err(error).with_context(|| format!("repin thread {thread}")),
		},
	)
}

pub fn pin_process_threads(pid: u32, cpus: &[usize]) -> anyhow::Result<()> {
	pin_threads(pid, &cpu_set(cpus)?)
}

fn tree_and_session(pid: u32) -> Vec<u32> {
	let session_members = all_pids()
		.into_iter()
		.filter(|&process| session_of(process) == Some(pid));
	let mut processes: Vec<u32> = process_tree(pid)
		.into_iter()
		.chain(session_members)
		.collect();
	processes.sort_unstable();
	processes.dedup();
	processes
}

pub fn repin(pid: u32, cpus: &[usize]) -> anyhow::Result<()> {
	let set = cpu_set(cpus)?;
	pin_until_no_new_member(
		|| tree_and_session(pid),
		|process| pin_threads(process, &set),
	)
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
