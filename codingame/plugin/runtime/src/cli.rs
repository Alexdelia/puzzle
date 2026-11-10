use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use clap::{Parser, Subcommand};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::atomic::write_atomic;
use crate::config::Config;
use crate::hook::{HookVerb, run_hook};
use crate::paths::RunDir;
use crate::run_info::{Mode, RunInfo};
use crate::worker::{WorkerVerb, run_worker};

const RUN_GITIGNORE: &str = "agents/\nrequests/\nclaude-config/\njobs/*/log\n";
const SECONDS_PER_HOUR: u64 = 3600;

#[derive(Debug, Parser)]
#[command(name = "contest")]
pub struct Cli {
	#[arg(long, global = true, default_value_t = 60)]
	pub timeout: u64,
	#[command(subcommand)]
	pub command: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
	Init(InitArgs),
	#[command(subcommand)]
	Hook(HookVerb),
	#[command(flatten)]
	Worker(WorkerVerb),
}

#[derive(Debug, clap::Args)]
pub struct InitArgs {
	pub contest_dir: PathBuf,
	#[arg(long)]
	pub puzzle: String,
	#[arg(long)]
	pub mode: Mode,
	#[arg(long)]
	pub active_budget_hours: Option<u64>,
	#[arg(long)]
	pub deadline: Option<String>,
	#[arg(long)]
	pub plugin_dir: Option<PathBuf>,
}

pub fn run(cli: Cli) -> anyhow::Result<ExitCode> {
	match cli.command {
		Verb::Init(args) => init(args).map(|()| ExitCode::SUCCESS),
		Verb::Hook(verb) => Ok(run_hook(verb)),
		Verb::Worker(verb) => run_worker(verb, cli.timeout).map(|()| ExitCode::SUCCESS),
	}
}

fn init(args: InitArgs) -> anyhow::Result<()> {
	let run = RunDir {
		root: args.contest_dir,
	};
	for dir in [
		run.tasks(),
		run.jobs(),
		run.inbox(),
		run.requests(),
		run.agents(),
	] {
		fs::create_dir_all(&dir).with_context(|| format!("create {dir:?}"))?;
	}
	write_atomic(&run.root.join("run/.gitignore"), RUN_GITIGNORE.as_bytes())?;

	let mut config = Config::default();
	if let Some(plugin_dir) = args.plugin_dir {
		config.plugin_dir =
			fs::canonicalize(&plugin_dir).with_context(|| format!("resolve {plugin_dir:?}"))?;
	}
	let (repo_root, git_common_dir) = pinned_git_dirs(&run.root);
	config.repo_root = repo_root;
	config.git_common_dir = git_common_dir;
	write_atomic(&run.config(), toml::to_string(&config)?.as_bytes())?;

	let deadline_unix = args
		.deadline
		.map(|text| {
			OffsetDateTime::parse(&text, &Rfc3339)
				.map(OffsetDateTime::unix_timestamp)
				.with_context(|| format!("parse deadline {text:?}"))
		})
		.transpose()?;
	let info = RunInfo {
		puzzle: args.puzzle,
		mode: args.mode,
		deadline_unix,
		active_budget_seconds: args
			.active_budget_hours
			.map(|hours| hours * SECONDS_PER_HOUR),
		active_seconds: 0,
		phase: "start".into(),
		plugin_commit: plugin_commit(&config.plugin_dir),
		started_at: now_unix(),
	};
	write_atomic(&run.run_json(), &serde_json::to_vec_pretty(&info)?)?;
	Ok(())
}

fn git_path(dir: &Path, args: &[&str]) -> PathBuf {
	Command::new("git")
		.arg("-C")
		.arg(dir)
		.args(args)
		.output()
		.ok()
		.filter(|output| output.status.success())
		.map(|output| PathBuf::from(String::from_utf8_lossy(&output.stdout).trim()))
		.unwrap_or_default()
}

fn pinned_git_dirs(contest_dir: &Path) -> (PathBuf, PathBuf) {
	(
		git_path(contest_dir, &["rev-parse", "--show-toplevel"]),
		git_path(
			contest_dir,
			&["rev-parse", "--path-format=absolute", "--git-common-dir"],
		),
	)
}

fn plugin_commit(plugin_dir: &Path) -> String {
	Command::new("git")
		.arg("-C")
		.arg(plugin_dir)
		.args(["rev-parse", "HEAD"])
		.output()
		.ok()
		.filter(|output| output.status.success())
		.map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
		.unwrap_or_default()
}

fn now_unix() -> i64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|elapsed| elapsed.as_secs() as i64)
		.unwrap_or_default()
}
