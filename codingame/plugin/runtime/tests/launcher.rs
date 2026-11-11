use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const FAKE_BINARY: &str = "#!/bin/sh\necho \"bin $*\" >> \"$LAUNCH_LOG\"\nif [ \"$1\" = status ] && [ -n \"${FAKE_RUNNING:-}\" ]; then echo 'supervisor: running, pid 1'; fi\n";
const FAKE_CARGO: &str = "#!/bin/sh\necho cargo >> \"$LAUNCH_LOG\"\nif [ -n \"${CARGO_FAILS:-}\" ]; then exit 1; fi\nmkdir -p \"$(dirname \"$FAKE_TARGET\")\"\ncp \"$FAKE_SOURCE\" \"$FAKE_TARGET\"\n";

struct Plugin {
	dir: TempDir,
	launcher: PathBuf,
	binary: PathBuf,
	source: PathBuf,
	log: PathBuf,
	path: std::ffi::OsString,
}

fn executable(path: &Path, text: &str) {
	fs::create_dir_all(path.parent().unwrap()).unwrap();
	fs::write(path, text).unwrap();
	fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn plugin(with_binary: bool) -> Plugin {
	let dir = TempDir::new().unwrap();
	let launcher = dir.path().join("plugin/bin/contest");
	fs::create_dir_all(launcher.parent().unwrap()).unwrap();
	fs::copy(
		Path::new(env!("CARGO_MANIFEST_DIR")).join("../bin/contest"),
		&launcher,
	)
	.unwrap();
	let binary = dir.path().join("plugin/runtime/target/release/contest");
	let source = dir.path().join("fake-binary");
	executable(&source, FAKE_BINARY);
	if with_binary {
		executable(&binary, FAKE_BINARY);
	}
	executable(&dir.path().join("fake-path/cargo"), FAKE_CARGO);
	let mut entries = vec![dir.path().join("fake-path")];
	entries.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
	Plugin {
		log: dir.path().join("launch.log"),
		path: std::env::join_paths(entries).unwrap(),
		launcher,
		binary,
		source,
		dir,
	}
}

impl Plugin {
	fn run(&self, args: &[&str], extra_env: &[(&str, &str)]) -> Output {
		let mut command = Command::new(&self.launcher);
		command
			.args(args)
			.env("PATH", &self.path)
			.env("LAUNCH_LOG", &self.log)
			.env("FAKE_TARGET", &self.binary)
			.env("FAKE_SOURCE", &self.source)
			.env_remove("FAKE_RUNNING")
			.env_remove("CARGO_FAILS");
		for (name, value) in extra_env {
			command.env(name, value);
		}
		command.output().unwrap()
	}

	fn calls(&self) -> Vec<String> {
		let calls = fs::read_to_string(&self.log)
			.unwrap_or_default()
			.lines()
			.map(str::to_owned)
			.collect();
		fs::remove_file(&self.log).ok();
		calls
	}
}

#[test]
fn ordinary_verbs_never_build() {
	let plugin = plugin(true);
	for verb in ["status", "say", "stop", "hook", "extend"] {
		assert!(plugin.run(&[verb, "/c"], &[]).status.success());
		assert_eq!(plugin.calls(), [format!("bin {verb} /c")]);
	}
	assert!(plugin.dir.path().exists());
}

#[test]
fn init_builds_first() {
	let plugin = plugin(true);
	assert!(
		plugin
			.run(&["init", "/c", "--puzzle", "p"], &[])
			.status
			.success()
	);
	assert_eq!(plugin.calls(), ["cargo", "bin init /c --puzzle p"]);
}

#[test]
fn start_builds_only_without_a_running_supervisor() {
	let plugin = plugin(true);
	assert!(plugin.run(&["start", "/c"], &[]).status.success());
	assert_eq!(plugin.calls(), ["bin status /c", "cargo", "bin start /c"]);
	assert!(
		plugin
			.run(&["--timeout", "5", "start", "/c"], &[("FAKE_RUNNING", "1")])
			.status
			.success()
	);
	assert_eq!(
		plugin.calls(),
		["bin status /c", "bin --timeout 5 start /c"]
	);
}

#[test]
fn missing_binary_is_built_once() {
	let plugin = plugin(false);
	assert!(plugin.run(&["status", "/c"], &[]).status.success());
	assert_eq!(plugin.calls(), ["cargo", "bin status /c"]);
	assert!(plugin.run(&["status", "/c"], &[]).status.success());
	assert_eq!(plugin.calls(), ["bin status /c"]);
}

#[test]
fn failed_build_runs_the_existing_binary_with_a_warning() {
	let plugin = plugin(true);
	let output = plugin.run(&["init", "/c"], &[("CARGO_FAILS", "1")]);
	assert!(output.status.success());
	assert!(String::from_utf8_lossy(&output.stderr).contains("cargo build failed"));
	assert_eq!(plugin.calls(), ["cargo", "bin init /c"]);
	let missing = self::plugin(false);
	assert!(
		!missing
			.run(&["init", "/c"], &[("CARGO_FAILS", "1")])
			.status
			.success()
	);
}
