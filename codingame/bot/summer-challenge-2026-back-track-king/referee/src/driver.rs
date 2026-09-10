use crate::game::{Action, Connections, State, parse_actions};
use crate::map::Map;
use crate::proto::{init_text, turn_text};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel, sync_channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const STDERR_KEPT: usize = 200;
const UNREAD_TURNS: usize = 2;

pub struct View<'a> {
	pub map: &'a Map,
	pub state: &'a State,
	pub conn: &'a Connections,
	pub player: usize,
}

pub trait Driver {
	fn init(&mut self, map: &Map, player: usize) -> Result<(), String>;

	fn observe(&mut self, view: &View) -> Result<(), String> {
		let _ = view;
		Ok(())
	}

	fn decide(&mut self, view: &View) -> Result<Vec<Action>, String>;

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
	recycled: Receiver<String>,
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
		let (spent, recycled) = sync_channel::<String>(2);
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
				let _ = spent.try_send(text);
			}
		});

		Ok(ProcessDriver {
			lines: pump(child.stdout.take().unwrap()),
			errors,
			child,
			feed: Some(feed),
			backlog,
			recycled,
			timeout,
			sent: Instant::now(),
			elapsed: Duration::ZERO,
			shell,
			command: command.to_string(),
		})
	}

	fn send(&mut self, text: String) -> Result<(), String> {
		let feed = self.feed.as_ref().ok_or("input already closed")?;
		let queued = self.backlog.fetch_add(1, Ordering::AcqRel) + 1;
		feed.send(text)
			.map_err(|_| format!("`{}` closed its input", self.command))?;
		if queued > UNREAD_TURNS {
			return Err(format!("`{}` stopped reading its input", self.command));
		}
		Ok(())
	}

	fn buffer(&mut self) -> String {
		self.recycled.try_recv().unwrap_or_default()
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
	fn init(&mut self, map: &Map, player: usize) -> Result<(), String> {
		let text = init_text(map, player);
		self.send(text)?;
		self.sent = Instant::now();
		Ok(())
	}

	fn observe(&mut self, view: &View) -> Result<(), String> {
		let mut text = self.buffer();
		turn_text(view.map, view.state, view.conn, view.player, &mut text);
		self.send(text)?;
		if view.state.turn > 0 {
			self.sent = Instant::now();
		}
		Ok(())
	}

	fn decide(&mut self, _view: &View) -> Result<Vec<Action>, String> {
		let (arrived, line) = self.lines.recv_timeout(self.timeout).map_err(|_| {
			format!(
				"`{}` gave no answer within {:?}",
				self.command, self.timeout
			)
		})?;
		self.elapsed = arrived.saturating_duration_since(self.sent);
		match self.lines.try_recv() {
			Err(TryRecvError::Empty) => {}
			_ => {
				return Err(format!(
					"`{}` wrote more than one line this turn",
					self.command
				));
			}
		}
		parse_actions(&line)
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
