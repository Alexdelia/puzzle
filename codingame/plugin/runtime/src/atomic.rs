use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
	let dir = path.parent().unwrap_or_else(|| Path::new("."));
	let name = path
		.file_name()
		.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no file name"))?
		.to_string_lossy();
	let temp = dir.join(format!(".{name}.tmp.{pid}", pid = std::process::id()));
	let mut file = File::create(&temp)?;
	file.write_all(bytes)?;
	file.sync_all()?;
	fs::rename(&temp, path)?;
	File::open(dir)?.sync_all()
}
