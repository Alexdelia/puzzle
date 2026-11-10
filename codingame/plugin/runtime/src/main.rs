use clap::Parser;
use contest::cli::{Cli, run};

fn main() -> anyhow::Result<std::process::ExitCode> {
	run(Cli::parse())
}
