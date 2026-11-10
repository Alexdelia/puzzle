use clap::Parser;
use contest::cli::{Cli, run};

fn main() -> anyhow::Result<()> {
	run(Cli::parse())
}
