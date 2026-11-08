use crate::arena::FrameRecord;
use std::collections::BTreeMap;
use std::fmt::Write;

const FIELD: &str = "  ";
const CONTINUATION: &str = "      ";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CgFrame {
	pub index: usize,
	pub agent: Option<usize>,
	pub summary: Vec<String>,
	pub stderr: Vec<String>,
	pub stdout: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct CgLog {
	pub game_id: String,
	pub options: BTreeMap<String, String>,
	pub frames: Vec<CgFrame>,
	pub scores: Option<[i32; 2]>,
	pub tooltips: Option<String>,
}

fn parse_scores(text: &str) -> Option<[i32; 2]> {
	let mut scores = [0; 2];
	for part in text.split_whitespace() {
		let (agent, score) = part.split_once('=')?;
		scores[agent.parse::<usize>().ok()?] = score.parse().ok()?;
	}
	Some(scores)
}

impl CgLog {
	pub fn parse(text: &str) -> Result<CgLog, String> {
		let mut log = CgLog::default();
		let mut section = "";
		let mut field: Option<&str> = None;
		for line in text.lines() {
			if let Some(id) = line.strip_prefix("game ") {
				log.game_id = id.trim().to_string();
				continue;
			}
			if let Some(rest) = line.strip_prefix("options: ") {
				section = "options";
				let (key, value) = rest.split_once('=').ok_or("options line without `=`")?;
				log.options.insert(key.to_string(), value.to_string());
				continue;
			}
			if line == "agents:" || line == "result" {
				section = if line == "result" { "result" } else { "agents" };
				continue;
			}
			if let Some(rest) = line.strip_prefix("frame ") {
				section = "frame";
				field = None;
				let mut words = rest.split_whitespace();
				let index = words
					.next()
					.and_then(|w| w.parse().ok())
					.ok_or_else(|| format!("bad frame line `{line}`"))?;
				let agent = match (words.next(), words.next()) {
					(Some("agent"), Some(a)) => {
						Some(a.parse().map_err(|_| format!("bad agent in `{line}`"))?)
					}
					_ => None,
				};
				log.frames.push(CgFrame {
					index,
					agent,
					..CgFrame::default()
				});
				continue;
			}
			match section {
				"options" => {
					let (key, value) = line
						.trim()
						.split_once('=')
						.ok_or_else(|| format!("bad option line `{line}`"))?;
					log.options.insert(key.to_string(), value.to_string());
				}
				"result" => {
					if let Some(scores) = line.trim().strip_prefix("scores: ") {
						log.scores = parse_scores(scores);
					}
					if let Some(tooltips) = line.trim().strip_prefix("tooltips: ") {
						log.tooltips = Some(tooltips.to_string());
					}
				}
				"frame" => {
					let frame = log.frames.last_mut().unwrap();
					if let Some(more) = line.strip_prefix(CONTINUATION) {
						match field {
							Some("summary") => frame.summary.push(more.to_string()),
							Some("stderr") => frame.stderr.push(more.to_string()),
							_ => return Err(format!("continuation outside a field: `{line}`")),
						}
						continue;
					}
					let body = line
						.strip_prefix(FIELD)
						.ok_or_else(|| format!("unexpected frame line `{line}`"))?;
					let (name, value) = body
						.split_once(": ")
						.unwrap_or((body.trim_end_matches(':'), ""));
					field = Some(match name {
						"summary" => {
							frame.summary.push(value.to_string());
							"summary"
						}
						"stderr" => {
							frame.stderr.push(value.to_string());
							"stderr"
						}
						"stdout" => {
							frame.stdout = Some(value.to_string());
							"stdout"
						}
						_ => "other",
					});
				}
				_ => {}
			}
		}
		Ok(log)
	}

	pub fn option(&self, key: &str) -> Option<&str> {
		self.options.get(key).map(String::as_str)
	}

	pub fn answers(&self, player: usize) -> Vec<String> {
		self.frames
			.iter()
			.filter(|f| f.agent == Some(player))
			.filter_map(|f| f.stdout.clone())
			.collect()
	}

	pub fn stdin_of(&self, player: usize) -> Vec<Vec<String>> {
		self.frames
			.iter()
			.filter(|f| f.agent == Some(player) && (f.stdout.is_some() || !f.stderr.is_empty()))
			.map(|f| f.stderr.clone())
			.collect()
	}

	pub fn frames_text(&self) -> String {
		let mut out = String::new();
		for frame in &self.frames {
			render_frame(&mut out, frame);
		}
		out
	}
}

fn render_frame(out: &mut String, frame: &CgFrame) {
	match frame.agent {
		Some(agent) => writeln!(out, "frame {} agent {agent}", frame.index).unwrap(),
		None => writeln!(out, "frame {}", frame.index).unwrap(),
	}
	for (i, line) in frame.summary.iter().enumerate() {
		let lead = if i == 0 { "  summary: " } else { CONTINUATION };
		writeln!(out, "{lead}{line}").unwrap();
	}
	if let Some(stdout) = &frame.stdout {
		writeln!(out, "  stdout: {stdout}").unwrap();
	}
}

#[derive(Clone, Debug)]
pub struct CgTrace {
	pub frames: Vec<CgFrame>,
	pub tooltips: Vec<String>,
}

impl Default for CgTrace {
	fn default() -> Self {
		CgTrace {
			frames: vec![CgFrame::default()],
			tooltips: Vec::new(),
		}
	}
}

impl CgTrace {
	pub fn push(&mut self, record: &FrameRecord) {
		let asked: Vec<usize> = (0..2).filter(|&p| record.asked[p]).collect();
		let agents = if asked.is_empty() { vec![0] } else { asked };
		let last = agents.len() - 1;
		for (i, &agent) in agents.iter().enumerate() {
			let index = self.frames.len();
			let summary = if i == last {
				record.summary.clone()
			} else {
				Vec::new()
			};
			if i == last {
				for tooltip in &record.tooltips {
					self.tooltips.push(format!("frame {index}: {tooltip}"));
				}
			}
			self.frames.push(CgFrame {
				index,
				agent: Some(agent),
				summary,
				stderr: Vec::new(),
				stdout: record.answers[agent].clone(),
			});
		}
	}

	pub fn frames_text(&self) -> String {
		let mut out = String::new();
		for frame in &self.frames {
			render_frame(&mut out, frame);
		}
		out
	}

	pub fn tooltips_line(&self) -> Option<String> {
		(!self.tooltips.is_empty()).then(|| self.tooltips.join(", "))
	}
}
