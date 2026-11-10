use std::fs::{self, OpenOptions};
use std::io::Write;

use time::OffsetDateTime;
use time::format_description::parse_borrowed;

use crate::paths::RunDir;

const TIMESTAMP_FORMAT: &str = "[year]-[month]-[day] [hour]:[minute]:[second]";

pub fn journal_append(run: &RunDir, line: &str) {
	let Ok(format) = parse_borrowed::<2>(TIMESTAMP_FORMAT) else {
		return;
	};
	let Ok(stamp) = OffsetDateTime::now_utc().format(&format) else {
		return;
	};
	let path = run.journal();
	if let Some(dir) = path.parent() {
		let _ = fs::create_dir_all(dir);
	}
	if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
		let _ = file.write_all(format!("{stamp} {line}\n").as_bytes());
	}
}

pub fn journal_tail(run: &RunDir, n: usize) -> Vec<String> {
	let text = fs::read_to_string(run.journal()).unwrap_or_default();
	let lines: Vec<String> = text.lines().map(str::to_owned).collect();
	let skip = lines.len().saturating_sub(n);
	lines.into_iter().skip(skip).collect()
}
