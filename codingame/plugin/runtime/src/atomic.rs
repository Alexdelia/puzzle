use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
	let dir = path.parent().unwrap_or_else(|| Path::new("."));
	let name = path
		.file_name()
		.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no file name"))?
		.to_string_lossy();
	let temp = dir.join(format!(
		".{name}.tmp.{pid}.{seq}",
		pid = std::process::id(),
		seq = NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
	));
	let written = write_and_rename(&temp, path, bytes);
	if written.is_err() {
		let _ = fs::remove_file(&temp);
	}
	written?;
	File::open(dir)?.sync_all()
}

fn write_and_rename(temp: &Path, path: &Path, bytes: &[u8]) -> io::Result<()> {
	let mut file = OpenOptions::new()
		.write(true)
		.create_new(true)
		.custom_flags(libc::O_NOFOLLOW)
		.open(temp)?;
	file.write_all(bytes)?;
	file.sync_all()?;
	fs::rename(temp, path)
}
