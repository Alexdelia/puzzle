use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel, sync_channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const STDERR_KEPT: usize = 200;
const UNREAD_TURNS: usize = 2;

pub trait Driver {
	fn send(&mut self, text: &str) -> Result<(), String>;

	fn answer(&mut self) -> Result<String, String>;

	fn elapsed(&self) -> Duration {
		Duration::ZERO
	}

	fn stderr(&mut self) -> String {
		String::new()
	}
}

pub struct ProcessDriver {
	child: Child,
	feed: Option<Sender<String>>,
	backlog: Arc<AtomicUsize>,
	lines: Receiver<(Instant, String)>,
	errors: Arc<Mutex<VecDeque<String>>>,
	timeout: Duration,
	sent: Instant,
	elapsed: Duration,
	shell: bool,
	command: String,
}

fn needs_a_shell(command: &str) -> bool {
	command.contains(|c| ";&|<>()$`\n*?[]{}~#!'\"\\".contains(c))
}

impl ProcessDriver {
	pub fn spawn(command: &str, timeout: Duration) -> Result<Self, String> {
		let shell = needs_a_shell(command);
		let script = if shell {
			command.to_string()
		} else {
			format!("exec {command}")
		};
		let mut child = Command::new("sh")
			.arg("-c")
			.arg(&script)
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.process_group(0)
			.spawn()
			.map_err(|e| format!("spawn `{command}`: {e}"))?;

		let errors: Arc<Mutex<VecDeque<String>>> = Default::default();
		let sink = errors.clone();
		let stderr = child.stderr.take().unwrap();
		std::thread::spawn(move || {
			for line in BufReader::new(stderr).lines().map_while(Result::ok) {
				let mut kept = sink.lock().unwrap();
				if kept.len() == STDERR_KEPT {
					kept.pop_front();
				}
				kept.push_back(line);
			}
		});

		let mut stdin = child.stdin.take().unwrap();
		let (feed, outbox) = channel::<String>();
		let backlog = Arc::new(AtomicUsize::new(0));
		let pending = backlog.clone();
		std::thread::spawn(move || {
			for text in outbox {
				if stdin
					.write_all(text.as_bytes())
					.and_then(|_| stdin.flush())
					.is_err()
				{
					return;
				}
				pending.fetch_sub(1, Ordering::Release);
			}
		});

		Ok(ProcessDriver {
			lines: pump(child.stdout.take().unwrap()),
			errors,
			child,
			feed: Some(feed),
			backlog,
			timeout,
			sent: Instant::now(),
			elapsed: Duration::ZERO,
			shell,
			command: command.to_string(),
		})
	}
}

fn pump<R: std::io::Read + Send + 'static>(source: R) -> Receiver<(Instant, String)> {
	let (tx, rx) = sync_channel(256);
	std::thread::spawn(move || {
		for line in BufReader::new(source).lines() {
			let Ok(line) = line else { return };
			if tx.send((Instant::now(), line)).is_err() {
				return;
			}
		}
	});
	rx
}

impl Driver for ProcessDriver {
	fn send(&mut self, text: &str) -> Result<(), String> {
		let feed = self.feed.as_ref().ok_or("input already closed")?;
		let queued = self.backlog.fetch_add(1, Ordering::AcqRel) + 1;
		feed.send(text.to_string())
			.map_err(|_| format!("`{}` closed its input", self.command))?;
		if queued > UNREAD_TURNS {
			return Err(format!("`{}` stopped reading its input", self.command));
		}
		self.sent = Instant::now();
		Ok(())
	}

	fn answer(&mut self) -> Result<String, String> {
		let (arrived, line) = self
			.lines
			.recv_timeout(self.timeout)
			.map_err(|why| match why {
				RecvTimeoutError::Timeout => format!(
					"`{}` gave no answer within {:?}",
					self.command, self.timeout
				),
				RecvTimeoutError::Disconnected => {
					format!("`{}` closed its output", self.command)
				}
			})?;
		self.elapsed = arrived.saturating_duration_since(self.sent);
		match self.lines.try_recv() {
			Ok(_) => Err(format!(
				"`{}` wrote more than one line this turn",
				self.command
			)),
			Err(_) => Ok(line),
		}
	}

	fn elapsed(&self) -> Duration {
		self.elapsed
	}

	fn stderr(&mut self) -> String {
		let mut kept = self.errors.lock().unwrap();
		let text = kept.iter().cloned().collect::<Vec<_>>().join("\n");
		kept.clear();
		text
	}
}

impl Drop for ProcessDriver {
	fn drop(&mut self) {
		drop(self.feed.take());
		let _ = self.child.kill();
		let _ = self.child.wait();
		if self.shell {
			let group = format!("-{}", self.child.id());
			let _ = Command::new("kill")
				.arg("-9")
				.arg(group)
				.stdout(Stdio::null())
				.stderr(Stdio::null())
				.status();
		}
	}
}

pub fn make_driver(command: &str, timeout: Duration) -> Result<Box<dyn Driver>, String> {
	if command.trim().is_empty() {
		return Err("empty bot command".into());
	}
	ProcessDriver::spawn(command, timeout).map(|driver| Box::new(driver) as Box<dyn Driver>)
}
