use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

use crate::atomic::write_atomic;
use crate::commit::check_write_path;
use crate::journal::journal_append;
use crate::paths::{RunDir, TaskId, is_valid_kind};
use crate::run_info::RunInfo;
use crate::task::{TaskSpec, TaskState, TaskStore};

const MAX_ACTIONS: usize = 50;
const MAX_TITLE_CHARS: usize = 200;
const MAX_BODY_BYTES: usize = 64 * 1024;
const MAX_ANSWER_BYTES: usize = 16 * 1024;
const MAX_REPLY_BYTES: usize = 8 * 1024;
const MAX_JOURNAL_CHARS: usize = 500;
const MAX_PHASE_CHARS: usize = 100;
const MAX_WRITES: usize = 64;
const MAX_MODEL_CHARS: usize = 64;
const MAX_BUDGET_USD: f64 = 100.0;
const JSON_FENCE: &str = "```json";
const FENCE: &str = "```";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case")]
pub enum Action {
	CreateTask {
		kind: String,
		title: String,
		priority: i64,
		body: String,
		model: Option<String>,
		budget_usd: Option<f64>,
		#[serde(default)]
		writes: Vec<PathBuf>,
		parent: Option<TaskId>,
	},
	Cancel {
		task: TaskId,
	},
	Reprioritize {
		task: TaskId,
		priority: i64,
	},
	Answer {
		task: TaskId,
		answer: String,
	},
	SetPhase {
		phase: String,
	},
	Journal {
		line: String,
	},
	Reply {
		text: String,
	},
}

pub fn parse_actions(text: &str) -> Result<Vec<Action>> {
	let mut last_block: Option<Vec<&str>> = None;
	let mut open: Option<Vec<&str>> = None;
	for line in text.lines() {
		let trimmed = line.trim();
		match open.as_mut() {
			Some(_) if trimmed == FENCE => last_block = open.take(),
			Some(block) => block.push(line),
			None if trimmed == JSON_FENCE => open = Some(Vec::new()),
			None => {}
		}
	}
	let block = last_block.context("no fenced json block")?;
	let actions: Vec<Action> =
		serde_json::from_str(&block.join("\n")).context("json block is not an action array")?;
	ensure!(
		actions.len() <= MAX_ACTIONS,
		"{count} actions exceed the limit of {MAX_ACTIONS}",
		count = actions.len()
	);
	Ok(actions)
}

fn is_clean_line(text: &str) -> bool {
	!text.is_empty() && !text.chars().any(char::is_control)
}

fn validated_task(run: &RunDir, id: &TaskId) -> Result<TaskId> {
	let id = TaskId::parse(&id.0).map_err(anyhow::Error::msg)?;
	TaskStore { run }.state(&id)?;
	Ok(id)
}

fn priority_of(priority: i64) -> Result<i32> {
	i32::try_from(priority).with_context(|| format!("priority {priority} out of range"))
}

fn is_valid_model(model: &str) -> bool {
	model.len() <= MAX_MODEL_CHARS
		&& model.starts_with(|c: char| c.is_ascii_alphanumeric())
		&& model
			.chars()
			.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '[' | ']'))
}

fn create_task(run: &RunDir, spec: TaskSpec) -> Result<String> {
	ensure!(
		is_valid_kind(&spec.kind),
		"invalid kind {kind:?}",
		kind = spec.kind
	);
	ensure!(
		is_clean_line(&spec.title) && spec.title.chars().count() <= MAX_TITLE_CHARS,
		"title must be one non-empty line of at most {MAX_TITLE_CHARS} chars"
	);
	ensure!(
		spec.body.len() <= MAX_BODY_BYTES,
		"body exceeds {MAX_BODY_BYTES} bytes"
	);
	ensure!(
		spec.writes.len() <= MAX_WRITES,
		"more than {MAX_WRITES} writes"
	);
	for path in &spec.writes {
		check_write_path(path, None)?;
	}
	if let Some(model) = &spec.model {
		ensure!(is_valid_model(model), "invalid model {model:?}");
	}
	if let Some(budget) = spec.budget_usd {
		ensure!(
			budget.is_finite() && budget > 0.0 && budget <= MAX_BUDGET_USD,
			"budget_usd {budget} out of range"
		);
	}
	if let Some(parent) = &spec.parent {
		validated_task(run, parent)?;
	}
	let title = spec.title.clone();
	let id = TaskStore { run }.create(spec)?;
	Ok(format!("created {id} {title}"))
}

fn ensure_live(store: &TaskStore, id: &TaskId) -> Result<TaskState> {
	let state = store.state(id)?;
	ensure!(
		!matches!(
			state,
			TaskState::Done | TaskState::Failed | TaskState::Cancelled
		),
		"task {id} is already {state:?}"
	);
	Ok(state)
}

fn answer_task(run: &RunDir, task: &TaskId, answer: &str) -> Result<String> {
	let id = validated_task(run, task)?;
	ensure!(
		answer.len() <= MAX_ANSWER_BYTES,
		"answer exceeds {MAX_ANSWER_BYTES} bytes"
	);
	let store = TaskStore { run };
	let state = store.state(&id)?;
	ensure!(
		state == TaskState::NeedsAnswer,
		"task {id} is {state:?}, not waiting for an answer"
	);
	store.append_notes(&id, &format!("\nanswer: {answer}\n"))?;
	store.transition(&id, TaskState::Queued)?;
	Ok(format!("answered {id}"))
}

fn set_phase(run: &RunDir, phase: &str) -> Result<String> {
	ensure!(
		is_clean_line(phase) && phase.chars().count() <= MAX_PHASE_CHARS,
		"phase must be one non-empty line of at most {MAX_PHASE_CHARS} chars"
	);
	let path = run.run_json();
	let mut info: RunInfo = serde_json::from_slice(&fs::read(&path)?)?;
	info.phase = phase.to_owned();
	write_atomic(&path, &serde_json::to_vec(&info)?)?;
	Ok(format!("phase {phase}"))
}

fn append_reply(path: &Path, text: &str) -> Result<()> {
	let mut file = OpenOptions::new().create(true).append(true).open(path)?;
	file.write_all(format!("{text}\n\n").as_bytes())?;
	Ok(())
}

fn reply(run: &RunDir, text: &str) -> Result<String> {
	ensure!(
		!text.trim().is_empty() && text.len() <= MAX_REPLY_BYTES,
		"reply must be non-empty and at most {MAX_REPLY_BYTES} bytes"
	);
	append_reply(&run.replies(), text.trim())?;
	Ok("replied".to_owned())
}

fn journal_note(line: &str) -> Result<String> {
	ensure!(!line.trim().is_empty(), "journal line must not be empty");
	let clean: String = line
		.chars()
		.map(|c| if c.is_control() { ' ' } else { c })
		.take(MAX_JOURNAL_CHARS)
		.collect();
	Ok(format!("note {clean}", clean = clean.trim()))
}

fn apply(run: &RunDir, action: Action) -> Result<String> {
	let store = TaskStore { run };
	match action {
		Action::CreateTask {
			kind,
			title,
			priority,
			body,
			model,
			budget_usd,
			writes,
			parent,
		} => create_task(
			run,
			TaskSpec {
				kind,
				title,
				priority: priority_of(priority)?,
				model,
				budget_usd,
				writes,
				body,
				parent,
			},
		),
		Action::Cancel { task } => {
			let id = validated_task(run, &task)?;
			ensure_live(&store, &id)?;
			store.transition(&id, TaskState::Cancelled)?;
			Ok(format!("cancelled {id}"))
		}
		Action::Reprioritize { task, priority } => {
			let id = validated_task(run, &task)?;
			ensure_live(&store, &id)?;
			let priority = priority_of(priority)?;
			store.set_priority(&id, priority)?;
			Ok(format!("reprioritized {id} to {priority}"))
		}
		Action::Answer { task, answer } => answer_task(run, &task, &answer),
		Action::SetPhase { phase } => set_phase(run, &phase),
		Action::Journal { line } => journal_note(&line),
		Action::Reply { text } => reply(run, &text),
	}
}

pub fn apply_actions(run: &RunDir, actions: Vec<Action>) -> Vec<String> {
	let mut errors = Vec::new();
	for (index, action) in actions.into_iter().enumerate() {
		let outcome =
			apply(run, action).and_then(|line| journal_append(run, &format!("manager: {line}")));
		if let Err(error) = outcome {
			errors.push(format!("action {index}: {error:#}"));
		}
	}
	errors
}
