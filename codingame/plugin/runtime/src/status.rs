use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use nix::errno::Errno;
use nix::fcntl::{Flock, FlockArg};

use crate::atomic::write_atomic;
use crate::config::Config;
use crate::job::{JobState, list_job_records};
use crate::journal::journal_tail;
use crate::paths::RunDir;
use crate::run_info::RunInfo;
use crate::task::{TaskState, TaskStore};
use crate::usage::{
	NO_LAUNCHES_NOTE, five_hour_used_percent, is_fresh_sample, load_usage, seven_day_used_percent,
};

pub const MAX_STATUS_LINES: usize = 40;
const TASKS_SHOWN: usize = 8;
const JOBS_SHOWN: usize = 8;
const JOURNAL_SHOWN: usize = 5;
const LINE_MAX_CHARS: usize = 160;
const REPLIES_TAIL_BYTES: u64 = 16 * 1024;
const SECONDS_PER_HOUR: u64 = 3600;
const SECONDS_PER_MINUTE: u64 = 60;

fn clock(seconds: u64) -> String {
	format!(
		"{hours}h{minutes:02}m",
		hours = seconds / SECONDS_PER_HOUR,
		minutes = seconds % SECONDS_PER_HOUR / SECONDS_PER_MINUTE
	)
}

fn clipped(line: &str) -> String {
	line.chars()
		.map(|c| if c.is_control() { ' ' } else { c })
		.take(LINE_MAX_CHARS)
		.collect()
}

pub fn running_supervisor(run: &RunDir) -> Option<String> {
	let file = File::open(run.supervisor_pid()).ok()?;
	match Flock::lock(file, FlockArg::LockSharedNonblock) {
		Err((_, Errno::EWOULDBLOCK)) => {
			let pid = fs::read_to_string(run.supervisor_pid()).unwrap_or_default();
			Some(pid.trim().to_owned())
		}
		_ => None,
	}
}

fn run_lines(run: &RunDir, now: i64) -> Vec<String> {
	let info = match fs::read(run.run_json())
		.map_err(anyhow::Error::from)
		.and_then(|bytes| Ok(serde_json::from_slice::<RunInfo>(&bytes)?))
	{
		Ok(info) => info,
		Err(error) => return vec![format!("run info unavailable: {error:#}")],
	};
	let budget = info
		.active_budget_seconds
		.map_or("no budget".to_owned(), clock);
	let deadline = info.deadline_unix.map_or("none".to_owned(), |unix| {
		clock(u64::try_from(unix - now).unwrap_or(0))
	});
	vec![
		format!(
			"puzzle: {puzzle} ({mode:?})",
			puzzle = info.puzzle,
			mode = info.mode
		),
		format!("phase: {phase}", phase = info.phase),
		format!(
			"active: {used} of {budget}",
			used = clock(info.active_seconds)
		),
		format!("deadline in: {deadline}"),
	]
}

fn usage_lines(run: &RunDir, now: i64) -> Vec<String> {
	let config = Config::load(&run.config()).unwrap_or_default();
	match load_usage(run) {
		Some((info, recorded_at)) => {
			let mut lines = vec![format!(
				"usage: 5h {five}%, 7d {seven}%, sampled {age}s ago",
				five = five_hour_used_percent(&info, now),
				seven = seven_day_used_percent(&info, now),
				age = now - recorded_at
			)];
			if !is_fresh_sample(recorded_at, now, &config) {
				lines.push(format!("usage sample stale: {NO_LAUNCHES_NOTE}"));
			}
			lines
		}
		None => vec![format!("usage: no sample, {NO_LAUNCHES_NOTE}")],
	}
}

fn section(title: &str, items: Vec<String>) -> Vec<String> {
	let mut lines = vec![format!("## {title}")];
	if items.is_empty() {
		lines.push("(none)".to_owned());
	}
	lines.extend(items);
	lines
}

fn tasks_in(run: &RunDir, state: TaskState) -> Vec<String> {
	TaskStore { run }
		.list()
		.unwrap_or_default()
		.into_iter()
		.filter(|task| task.state == state)
		.take(TASKS_SHOWN)
		.map(|task| format!("- {id} {title}", id = task.id, title = task.title))
		.collect()
}

fn active_jobs(run: &RunDir) -> Vec<String> {
	list_job_records(run)
		.into_iter()
		.filter(|record| {
			matches!(
				record.state,
				JobState::Queued | JobState::Running | JobState::Cancelling
			)
		})
		.take(JOBS_SHOWN)
		.map(|record| {
			format!(
				"- {id} {state:?} for {owner}",
				id = record.id,
				state = record.state,
				owner = record.owner_task
			)
		})
		.collect()
}

fn replies_tail(path: &Path) -> String {
	let Ok(mut file) = File::open(path) else {
		return String::new();
	};
	let length = file.metadata().map_or(0, |meta| meta.len());
	if file
		.seek(SeekFrom::Start(length.saturating_sub(REPLIES_TAIL_BYTES)))
		.is_err()
	{
		return String::new();
	}
	let mut bytes = Vec::new();
	if file.read_to_end(&mut bytes).is_err() {
		return String::new();
	}
	String::from_utf8_lossy(&bytes).into_owned()
}

fn last_reply(run: &RunDir) -> Vec<String> {
	let tail = replies_tail(&run.replies());
	tail.split("\n\n")
		.map(str::trim)
		.filter(|reply| !reply.is_empty())
		.last()
		.map(|reply| reply.lines().map(str::to_owned).collect())
		.unwrap_or_default()
}

fn supervisor_line(run: &RunDir, pid: Option<String>) -> String {
	let stop_requested = run.stop_file().exists();
	match pid {
		Some(pid) if pid.is_empty() => "supervisor: starting".to_owned(),
		Some(pid) if stop_requested => format!("supervisor: stopping, pid {pid}"),
		Some(pid) => format!("supervisor: running, pid {pid}"),
		None if stop_requested => "supervisor: stopped".to_owned(),
		None => "supervisor: not running".to_owned(),
	}
}

pub fn status_text(run: &RunDir, now: i64) -> String {
	status_text_for(run, now, running_supervisor(run))
}

fn block_lines(run: &RunDir) -> Vec<String> {
	fs::read_to_string(run.blocks())
		.unwrap_or_default()
		.lines()
		.filter(|line| !line.trim().is_empty())
		.map(|line| format!("blocked: {line}"))
		.collect()
}

fn status_text_for(run: &RunDir, now: i64, supervisor_pid: Option<String>) -> String {
	let mut lines = vec!["# contest status".to_owned()];
	let supervised = supervisor_pid.is_some();
	lines.push(supervisor_line(run, supervisor_pid));
	if supervised {
		lines.extend(block_lines(run));
	}
	if let Ok(reason) = fs::read_to_string(run.stop_file()) {
		lines.push(format!("stop requested: {reason}", reason = reason.trim()));
	}
	lines.extend(run_lines(run, now));
	lines.extend(usage_lines(run, now));
	lines.extend(section("running tasks", tasks_in(run, TaskState::Running)));
	lines.extend(section("waiting tasks", tasks_in(run, TaskState::Waiting)));
	lines.extend(section("jobs", active_jobs(run)));
	lines.extend(section("journal", journal_tail(run, JOURNAL_SHOWN)));
	lines.extend(section("last reply", last_reply(run)));
	lines
		.iter()
		.take(MAX_STATUS_LINES)
		.map(|line| format!("{line}\n", line = clipped(line)))
		.collect()
}

pub fn write_status(run: &RunDir, now: i64, supervisor_pid: Option<u32>) -> anyhow::Result<()> {
	let text = status_text_for(run, now, supervisor_pid.map(|pid| pid.to_string()));
	write_atomic(&run.status(), text.as_bytes())?;
	Ok(())
}
