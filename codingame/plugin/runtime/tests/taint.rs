use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;

use contest::taint::remove_planted_files;
use tempfile::TempDir;

fn touch(path: &Path) {
	fs::create_dir_all(path.parent().unwrap()).unwrap();
	fs::write(path, "x").unwrap();
}

fn contest() -> (TempDir, PathBuf) {
	let dir = TempDir::new().unwrap();
	let root = dir.path().join("contest");
	fs::create_dir_all(&root).unwrap();
	(dir, root)
}

fn sorted(mut paths: Vec<PathBuf>) -> Vec<PathBuf> {
	paths.sort();
	paths
}

#[test]
fn removes_planted_names_below_top_level() {
	let (_dir, root) = contest();
	for planted in [
		"sub/.git/config",
		"sub/.envrc",
		"sub/deep/.direnv/bin/nix-direnv",
		"sub/deep/.claude/settings.json",
		"sub/.mcp.json",
	] {
		touch(&root.join(planted));
	}
	touch(&root.join("other/.git"));
	touch(&root.join("sub/main.rs"));
	let removed = remove_planted_files(&root).unwrap();
	assert_eq!(
		sorted(removed),
		sorted(
			[
				"other/.git",
				"sub/.envrc",
				"sub/.git",
				"sub/.mcp.json",
				"sub/deep/.claude",
				"sub/deep/.direnv",
			]
			.map(|relative| root.join(relative))
			.to_vec()
		)
	);
	for gone in [
		"other/.git",
		"sub/.envrc",
		"sub/.git",
		"sub/.mcp.json",
		"sub/deep/.claude",
		"sub/deep/.direnv",
	] {
		assert!(fs::symlink_metadata(root.join(gone)).is_err(), "{gone}");
	}
	assert!(root.join("sub/main.rs").is_file());
}

#[test]
fn top_level_keeps_host_files_and_drops_git_and_mcp() {
	let (_dir, root) = contest();
	touch(&root.join(".envrc"));
	touch(&root.join(".direnv/bin/x"));
	touch(&root.join(".claude/settings.json"));
	touch(&root.join(".git/config"));
	touch(&root.join(".mcp.json"));
	let removed = remove_planted_files(&root).unwrap();
	assert_eq!(
		sorted(removed),
		vec![root.join(".git"), root.join(".mcp.json")]
	);
	assert!(root.join(".envrc").is_file());
	assert!(root.join(".direnv/bin/x").is_file());
	assert!(root.join(".claude/settings.json").is_file());
}

#[test]
fn run_subtree_is_skipped() {
	let (_dir, root) = contest();
	touch(&root.join("run/tasks/t0001-a/.git/config"));
	touch(&root.join("run/.claude/settings.json"));
	assert_eq!(remove_planted_files(&root).unwrap(), Vec::<PathBuf>::new());
	assert!(root.join("run/tasks/t0001-a/.git/config").is_file());
}

#[test]
fn symlinks_are_never_followed() {
	let (dir, root) = contest();
	let outside = dir.path().join("outside");
	touch(&outside.join(".git/config"));
	touch(&outside.join("keep"));
	symlink(&outside, root.join("linked")).unwrap();
	symlink(&outside, root.join("sub-claude")).unwrap();
	fs::create_dir_all(root.join("sub")).unwrap();
	symlink(&outside, root.join("sub/.claude")).unwrap();
	let removed = remove_planted_files(&root).unwrap();
	assert_eq!(removed, vec![root.join("sub/.claude")]);
	assert!(fs::symlink_metadata(root.join("sub/.claude")).is_err());
	assert!(outside.join(".git/config").is_file());
	assert!(outside.join("keep").is_file());
	assert!(root.join("linked").exists());
}

#[test]
fn planted_nested_repo_with_fsmonitor_is_removed() {
	let (dir, root) = contest();
	let nested = root.join("sub");
	fs::create_dir_all(&nested).unwrap();
	let status = Command::new("git")
		.arg("-C")
		.arg(&nested)
		.args(["init", "-q"])
		.status()
		.unwrap();
	assert!(status.success());
	let marker = dir.path().join("fsmonitor-ran");
	let hook = dir.path().join("fsmonitor.sh");
	fs::write(
		&hook,
		format!("#!/bin/sh\ntouch '{marker}'\n", marker = marker.display()),
	)
	.unwrap();
	let config = nested.join(".git/config");
	let mut text = fs::read_to_string(&config).unwrap();
	text.push_str(&format!("\tfsmonitor = {hook}\n", hook = hook.display()));
	fs::write(&config, text).unwrap();
	assert_eq!(
		remove_planted_files(&root).unwrap(),
		vec![nested.join(".git")]
	);
	assert!(!nested.join(".git").exists());
	assert!(!marker.exists());
}

#[test]
fn symlinked_contest_root_is_refused() {
	let (dir, root) = contest();
	let link = dir.path().join("link");
	symlink(&root, &link).unwrap();
	assert!(remove_planted_files(&link).is_err());
}
