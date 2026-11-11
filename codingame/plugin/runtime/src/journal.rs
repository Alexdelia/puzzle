use std::fs::{self, OpenOptions};
use std::io::Write;

use time::OffsetDateTime;
use time::format_description::parse_borrowed;

use crate::paths::RunDir;

const TIMESTAMP_FORMAT: &str = "[year]-[month]-[day] [hour]:[minute]:[second]";

fn single_line(line: &str) -> String {
	let mut escaped = String::with_capacity(line.len());
	for char in line.chars() {
		if char.is_control() {
			escaped.extend(char.escape_debug());
		} else {
			escaped.push(char);
		}
	}
	escaped
}

pub fn journal_append(run: &RunDir, line: &str) -> anyhow::Result<()> {
	let format = parse_borrowed::<2>(TIMESTAMP_FORMAT)?;
	let stamp = OffsetDateTime::now_utc().format(&format)?;
	let path = run.journal();
	if let Some(dir) = path.parent() {
		fs::create_dir_all(dir)?;
	}
	let mut file = OpenOptions::new().create(true).append(true).open(path)?;
	let line = single_line(line);
	file.write_all(format!("{stamp} {line}\n").as_bytes())?;
	Ok(())
}

pub fn journal_tail(run: &RunDir, n: usize) -> Vec<String> {
	let text = fs::read_to_string(run.journal()).unwrap_or_default();
	let lines: Vec<String> = text.lines().map(str::to_owned).collect();
	let skip = lines.len().saturating_sub(n);
	lines.into_iter().skip(skip).collect()
}
