use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::mem::MaybeUninit;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

pub enum ReadError {
	Timeout,
	Closed,
}

pub struct Solution {
	child: Child,
	stdin: Option<ChildStdin>,
	lines: Receiver<String>,
	reaped: Option<Usage>,
	orphan_grace: Option<Duration>,
	main_exit: Option<Instant>,
	orphans_killed: bool,
}

#[derive(Clone, Copy, Default)]
pub struct Usage {
	pub cpu: Duration,
	pub peak_rss_kb: u64,
}

impl Usage {
	fn max(self, other: Self) -> Self {
		Self {
			cpu: self.cpu.max(other.cpu),
			peak_rss_kb: self.peak_rss_kb.max(other.peak_rss_kb),
		}
	}
}

const EXIT_GRACE: Duration = Duration::from_millis(300);
const EXIT_POLL: Duration = Duration::from_millis(1);
const READ_SLICE: Duration = Duration::from_millis(5);

impl Solution {
	pub fn spawn(launcher: &Path, show_stderr: bool) -> Result<Self> {
		let stderr = if show_stderr {
			Stdio::inherit()
		} else {
			Stdio::null()
		};
		let mut child = Command::new(launcher)
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.stderr(stderr)
			.process_group(0)
			.spawn()
			.with_context(|| format!("spawn {launcher}", launcher = launcher.display()))?;
		let stdout = child.stdout.take().context("solution stdout not piped")?;
		let (sender, lines) = mpsc::channel();
		thread::spawn(move || {
			let mut reader = BufReader::new(stdout);
			let mut buffer = Vec::new();
			loop {
				buffer.clear();
				match reader.read_until(b'\n', &mut buffer) {
					Ok(0) | Err(_) => break,
					Ok(_) => {
						if buffer.last() == Some(&b'\n') {
							buffer.pop();
						}
						let line = String::from_utf8_lossy(&buffer).into_owned();
						if sender.send(line).is_err() {
							break;
						}
					}
				}
			}
		});
		Ok(Self {
			stdin: child.stdin.take(),
			child,
			lines,
			reaped: None,
			orphan_grace: None,
			main_exit: None,
			orphans_killed: false,
		})
	}

	pub fn kill_orphans_after_main_exit(mut self, grace: Duration) -> Self {
		self.orphan_grace = Some(grace);
		self
	}

	pub fn write(&mut self, text: &str) -> bool {
		let Some(stdin) = self.stdin.as_mut() else {
			return false;
		};
		stdin.write_all(text.as_bytes()).is_ok() && stdin.flush().is_ok()
	}

	pub fn write_all_then_close(&mut self, text: String) {
		if let Some(mut stdin) = self.stdin.take() {
			thread::spawn(move || {
				let _ = stdin.write_all(text.as_bytes());
			});
		}
	}

	pub fn read_line(&mut self, deadline: Instant) -> Result<String, ReadError> {
		loop {
			let now = Instant::now();
			if now >= deadline {
				return Err(ReadError::Timeout);
			}
			let wake = self
				.orphan_watch(now)
				.map_or(deadline, |wake| wake.min(deadline));
			match self.lines.recv_timeout(wake - now) {
				Ok(line) => return Ok(line),
				Err(RecvTimeoutError::Disconnected) => return Err(ReadError::Closed),
				Err(RecvTimeoutError::Timeout) => {}
			}
		}
	}

	fn orphan_watch(&mut self, now: Instant) -> Option<Instant> {
		let grace = self.orphan_grace?;
		if self.main_exit.is_none() && self.reap(libc::WNOHANG).is_some() {
			self.main_exit = Some(now);
		}
		let Some(main_exit) = self.main_exit else {
			return Some(now + READ_SLICE);
		};
		let kill_at = main_exit + grace;
		if now < kill_at {
			return Some(kill_at);
		}
		if !self.orphans_killed {
			self.kill_group();
			self.orphans_killed = true;
		}
		None
	}

	fn pid(&self) -> libc::pid_t {
		libc::pid_t::try_from(self.child.id()).unwrap_or(libc::pid_t::MAX)
	}

	pub fn live_usage(&self) -> Usage {
		tree_usage(self.pid())
	}

	fn reap(&mut self, flags: libc::c_int) -> Option<Usage> {
		if self.reaped.is_none() {
			let mut status = 0;
			let mut usage = MaybeUninit::<libc::rusage>::zeroed();
			let reaped = unsafe { libc::wait4(self.pid(), &mut status, flags, usage.as_mut_ptr()) };
			if reaped == self.pid() {
				let usage = unsafe { usage.assume_init() };
				self.reaped = Some(Usage {
					cpu: seconds(usage.ru_utime) + seconds(usage.ru_stime),
					peak_rss_kb: u64::try_from(usage.ru_maxrss).unwrap_or(0),
				});
			}
		}
		self.reaped
	}

	pub fn finish(&mut self) -> Usage {
		self.finish_within(EXIT_GRACE)
	}

	pub fn finish_within(&mut self, grace: Duration) -> Usage {
		let grace = Instant::now() + grace;
		while Instant::now() < grace {
			if let Some(usage) = self.reap(libc::WNOHANG) {
				return usage;
			}
			thread::sleep(EXIT_POLL);
		}
		self.stop()
	}

	pub fn stop(&mut self) -> Usage {
		let live = self.live_usage();
		self.kill();
		self.reaped.unwrap_or_default().max(live)
	}

	pub fn kill(&mut self) {
		self.kill_group();
		self.reap(0);
	}

	fn kill_group(&self) {
		unsafe {
			libc::killpg(self.pid(), libc::SIGKILL);
		}
	}
}

fn children(pid: libc::pid_t) -> Vec<libc::pid_t> {
	let Ok(tasks) = fs::read_dir(format!("/proc/{pid}/task")) else {
		return Vec::new();
	};
	tasks
		.filter_map(Result::ok)
		.filter_map(|task| fs::read_to_string(task.path().join("children")).ok())
		.flat_map(|text| {
			text.split_whitespace()
				.filter_map(|child| child.parse().ok())
				.collect::<Vec<_>>()
		})
		.collect()
}

fn process_usage(pid: libc::pid_t) -> Usage {
	let ticks_per_second = u64::try_from(unsafe { libc::sysconf(libc::_SC_CLK_TCK) })
		.unwrap_or(100)
		.max(1);
	let ticks: u64 = fs::read_to_string(format!("/proc/{pid}/stat"))
		.ok()
		.and_then(|stat| {
			let fields: Vec<u64> = stat
				.rsplit_once(')')?
				.1
				.split_whitespace()
				.skip(11)
				.take(4)
				.filter_map(|field| field.parse().ok())
				.collect();
			Some(fields.iter().sum())
		})
		.unwrap_or(0);
	let peak_rss_kb = fs::read_to_string(format!("/proc/{pid}/status"))
		.ok()
		.and_then(|status| {
			status
				.lines()
				.find_map(|line| line.strip_prefix("VmHWM:"))?
				.split_whitespace()
				.next()?
				.parse()
				.ok()
		})
		.unwrap_or(0);
	Usage {
		cpu: Duration::from_millis(ticks * 1000 / ticks_per_second),
		peak_rss_kb,
	}
}

fn tree_usage(root: libc::pid_t) -> Usage {
	let mut pending = vec![root];
	let mut total = Usage::default();
	while let Some(pid) = pending.pop() {
		let usage = process_usage(pid);
		total.cpu += usage.cpu;
		total.peak_rss_kb = total.peak_rss_kb.max(usage.peak_rss_kb);
		pending.extend(children(pid));
	}
	total
}

impl Drop for Solution {
	fn drop(&mut self) {
		self.kill();
	}
}

fn seconds(time: libc::timeval) -> Duration {
	Duration::from_secs(time.tv_sec.unsigned_abs())
		+ Duration::from_micros(time.tv_usec.unsigned_abs())
}

fn cpu_time(who: libc::c_int) -> Duration {
	let mut usage = MaybeUninit::<libc::rusage>::zeroed();
	let usage = unsafe {
		libc::getrusage(who, usage.as_mut_ptr());
		usage.assume_init()
	};
	seconds(usage.ru_utime) + seconds(usage.ru_stime)
}

pub fn referee_cpu() -> Duration {
	cpu_time(libc::RUSAGE_SELF)
}

pub fn solutions_cpu() -> Duration {
	cpu_time(libc::RUSAGE_CHILDREN)
}
