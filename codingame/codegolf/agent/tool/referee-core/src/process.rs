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
}

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
		})
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

	pub fn read_line(&self, deadline: Instant) -> Result<String, ReadError> {
		let wait = deadline.saturating_duration_since(Instant::now());
		self.lines.recv_timeout(wait).map_err(|error| match error {
			RecvTimeoutError::Timeout => ReadError::Timeout,
			RecvTimeoutError::Disconnected => ReadError::Closed,
		})
	}

	pub fn kill(&mut self) {
		let Ok(group) = libc::pid_t::try_from(self.child.id()) else {
			return;
		};
		unsafe {
			libc::killpg(group, libc::SIGKILL);
		}
		let _ = self.child.wait();
	}
}

impl Drop for Solution {
	fn drop(&mut self) {
		self.kill();
	}
}

fn cpu_time(who: libc::c_int) -> Duration {
	let mut usage = MaybeUninit::<libc::rusage>::zeroed();
	let usage = unsafe {
		libc::getrusage(who, usage.as_mut_ptr());
		usage.assume_init()
	};
	let seconds = |time: libc::timeval| {
		Duration::from_secs(time.tv_sec.unsigned_abs())
			+ Duration::from_micros(time.tv_usec.unsigned_abs())
	};
	seconds(usage.ru_utime) + seconds(usage.ru_stime)
}

pub fn referee_cpu() -> Duration {
	cpu_time(libc::RUSAGE_SELF)
}

pub fn solutions_cpu() -> Duration {
	cpu_time(libc::RUSAGE_CHILDREN)
}
