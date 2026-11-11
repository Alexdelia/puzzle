use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use serde_json::Value;

use crate::broker::{JobClass, physical_core_count};
use crate::paths::{JobId, RunDir, TaskId};
use crate::spool::{Outcome, Request, send_request};

const MAX_WAIT_SECONDS: u64 = 100;
const WAIT_ANSWER_MARGIN_SECONDS: u64 = 30;

#[derive(Debug, Subcommand)]
pub enum WorkerVerb {
	Commit(CommitArgs),
	#[command(subcommand)]
	Job(JobVerb),
	#[command(subcommand)]
	Task(TaskVerb),
}

#[derive(Debug, Args)]
pub struct CommitArgs {
	#[arg(long)]
	pub milestone: String,
}

#[derive(Debug, Subcommand)]
pub enum JobVerb {
	Submit(SubmitArgs),
	Status {
		job: String,
	},
	Wait {
		job: String,
		#[arg(long, default_value_t = MAX_WAIT_SECONDS)]
		seconds: u64,
	},
	Cancel {
		job: String,
	},
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ClassArg {
	Timed,
	Batch,
}

#[derive(Debug, Args)]
pub struct SubmitArgs {
	#[arg(long)]
	pub class: ClassArg,
	#[arg(long, default_value_t = 1)]
	pub min_cores: usize,
	#[arg(long)]
	pub max_cores: Option<usize>,
	#[arg(long)]
	pub gpu: bool,
	#[arg(long)]
	pub ram_mb: u64,
	#[arg(long, default_value_t = 0)]
	pub priority: i32,
	#[arg(last = true, required = true)]
	pub cmd: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub enum TaskVerb {
	Done,
	Failed,
	Ask { question: String },
	WaitJobs,
}

fn env_path(name: &str) -> Result<PathBuf> {
	std::env::var_os(name)
		.map(PathBuf::from)
		.with_context(|| format!("{name} is not set"))
}

fn job_class(args: &SubmitArgs) -> JobClass {
	match args.class {
		ClassArg::Batch => JobClass::Batch,
		ClassArg::Timed => JobClass::Timed {
			min_cores: args.min_cores,
			max_cores: args
				.max_cores
				.unwrap_or_else(|| physical_core_count().max(args.min_cores)),
		},
	}
}

fn request_of(verb: WorkerVerb) -> Result<(Request, Duration)> {
	let no_extra_wait = Duration::ZERO;
	Ok(match verb {
		WorkerVerb::Commit(args) => (
			Request::Commit {
				milestone: args.milestone,
			},
			no_extra_wait,
		),
		WorkerVerb::Job(JobVerb::Submit(args)) => (
			Request::JobSubmit {
				class: job_class(&args),
				gpu: args.gpu,
				ram_mb: args.ram_mb,
				priority: args.priority,
				cwd: std::env::current_dir()?,
				cmd: args.cmd,
			},
			no_extra_wait,
		),
		WorkerVerb::Job(JobVerb::Status { job }) => {
			(Request::JobStatus { job: JobId(job) }, no_extra_wait)
		}
		WorkerVerb::Job(JobVerb::Wait { job, seconds }) => {
			let timeout_seconds = seconds.min(MAX_WAIT_SECONDS);
			(
				Request::JobWait {
					job: JobId(job),
					timeout_seconds,
				},
				Duration::from_secs(timeout_seconds + WAIT_ANSWER_MARGIN_SECONDS),
			)
		}
		WorkerVerb::Job(JobVerb::Cancel { job }) => {
			(Request::JobCancel { job: JobId(job) }, no_extra_wait)
		}
		WorkerVerb::Task(TaskVerb::Done) => (finish(Outcome::Done, None), no_extra_wait),
		WorkerVerb::Task(TaskVerb::Failed) => (finish(Outcome::Failed, None), no_extra_wait),
		WorkerVerb::Task(TaskVerb::Ask { question }) => {
			(finish(Outcome::NeedsAnswer, Some(question)), no_extra_wait)
		}
		WorkerVerb::Task(TaskVerb::WaitJobs) => (Request::TaskWaitJobs, no_extra_wait),
	})
}

fn finish(outcome: Outcome, question: Option<String>) -> Request {
	Request::TaskFinish { outcome, question }
}

fn printable(request: &Request, value: &Value) -> String {
	match (request, value["job"].as_str()) {
		(Request::JobSubmit { .. }, Some(job)) => job.to_owned(),
		_ => value.to_string(),
	}
}

pub fn run_worker(verb: WorkerVerb, timeout_seconds: u64) -> Result<()> {
	let run = RunDir {
		root: env_path("CONTEST_RUN")?,
	};
	let task = TaskId(std::env::var("CONTEST_TASK").context("CONTEST_TASK is not set")?);
	let (request, wait_budget) = request_of(verb)?;
	let timeout = Duration::from_secs(timeout_seconds).max(wait_budget);
	let value = send_request(&run, &task, &request, timeout)?;
	println!("{answer}", answer = printable(&request, &value));
	Ok(())
}
