use std::fs;
use std::os::unix::fs::symlink;

use contest::atomic::write_atomic;
use tempfile::TempDir;

const PLANTED_TEMP_NAMES: u64 = 64;

#[test]
fn planted_temp_symlink_is_never_written_through() {
	let dir = TempDir::new().unwrap();
	let victim = dir.path().join("victim");
	fs::write(&victim, "original").unwrap();
	let worker_dir = dir.path().join("requests");
	fs::create_dir(&worker_dir).unwrap();
	let pid = std::process::id();
	for seq in 0..PLANTED_TEMP_NAMES {
		symlink(
			&victim,
			worker_dir.join(format!(".answer.json.tmp.{pid}.{seq}")),
		)
		.unwrap();
	}
	let target = worker_dir.join("answer.json");
	let written = write_atomic(&target, b"supervisor data");
	assert_eq!(fs::read_to_string(&victim).unwrap(), "original");
	if written.is_ok() {
		assert_eq!(fs::read_to_string(&target).unwrap(), "supervisor data");
	}
}

#[test]
fn writes_and_replaces() {
	let dir = TempDir::new().unwrap();
	let target = dir.path().join("f");
	write_atomic(&target, b"one").unwrap();
	write_atomic(&target, b"two").unwrap();
	assert_eq!(fs::read_to_string(&target).unwrap(), "two");
	let leftovers = fs::read_dir(dir.path()).unwrap().count();
	assert_eq!(leftovers, 1);
}
