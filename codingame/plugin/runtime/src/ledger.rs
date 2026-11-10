use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::agent::StreamOutcome;
use crate::paths::RunDir;

const UNKNOWN_MODEL: &str = "-";

pub fn ledger_append(run: &RunDir, who: &str, outcome: &StreamOutcome, wall_seconds: u64) {
	let at = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_or(0, |elapsed| elapsed.as_secs());
	let line = format!(
		"{at}\t{who}\t{UNKNOWN_MODEL}\t{input}\t{output}\t{cache_read}\t{cache_write}\t{cost}\t{wall_seconds}\n",
		input = outcome.input_tokens,
		output = outcome.output_tokens,
		cache_read = outcome.cache_read_tokens,
		cache_write = outcome.cache_write_tokens,
		cost = outcome.cost_usd,
	);
	let path = run.ledger();
	if let Some(dir) = path.parent() {
		let _ = fs::create_dir_all(dir);
	}
	if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
		let _ = file.write_all(line.as_bytes());
	}
}
