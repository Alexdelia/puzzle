use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use contest::taint::{
	MAX_SCAN_DEPTH, MAX_SCANNED_ENTRIES, ScanReport, TopLevelExempt, remove_planted_files,
	remove_planted_files_within,
};
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

fn scan(root: &Path) -> ScanReport {
	remove_planted_files(root, TopLevelExempt { envrc: true }).unwrap()
}

fn sorted(mut paths: Vec<PathBuf>) -> Vec<PathBuf> {
	paths.sort();
	paths
}

fn under(root: &Path, relatives: &[&str]) -> Vec<PathBuf> {
	sorted(
		relatives
			.iter()
			.map(|relative| root.join(relative))
			.collect(),
	)
}

fn failed_paths(report: &ScanReport) -> Vec<PathBuf> {
	sorted(
		report
			.failures
			.iter()
			.map(|(path, _)| path.clone())
			.collect(),
	)
}

fn absent(path: &Path) -> bool {
	fs::symlink_metadata(path).is_err()
}

fn set_mode(path: &Path, mode: u32) {
	fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

const BELOW_TOP: [&str; 6] = [
	"other/.git",
	"sub/.envrc",
	"sub/.git",
	"sub/.mcp.json",
	"sub/deep/.claude",
	"sub/deep/.direnv",
];

#[test]
fn removes_planted_names_below_top_level() {
	let (_dir, root) = contest();
	for planted in [
		"sub/.git/config",
		"sub/.envrc",
		"sub/deep/.direnv/bin/nix-direnv",
		"sub/deep/.claude/settings.json",
		"sub/.mcp.json",
		"other/.git",
	] {
		touch(&root.join(planted));
	}
	touch(&root.join("sub/main.rs"));
	let report = scan(&root);
	assert_eq!(report.failures, []);
	assert!(!report.must_fail_closed());
	assert_eq!(sorted(report.removed), under(&root, &BELOW_TOP));
	for gone in BELOW_TOP {
		assert!(absent(&root.join(gone)), "{gone}");
	}
	assert!(root.join("sub/main.rs").is_file());
}

#[test]
fn top_level_exempts_ro_bound_dirs_mcp_and_launch_envrc() {
	let (_dir, root) = contest();
	touch(&root.join(".envrc"));
	touch(&root.join(".direnv/bin/x"));
	touch(&root.join(".claude/settings.json"));
	touch(&root.join(".git/config"));
	touch(&root.join(".mcp.json"));
	assert_eq!(
		TopLevelExempt::pin_at_init(&root).unwrap(),
		TopLevelExempt { envrc: true }
	);
	let report = scan(&root);
	assert_eq!(report.removed, vec![root.join(".git")]);
	for kept in [
		".envrc",
		".direnv/bin/x",
		".claude/settings.json",
		".mcp.json",
	] {
		assert!(root.join(kept).is_file(), "{kept}");
	}
}

#[test]
fn scan_never_enters_exempt_top_level_dirs() {
	let (_dir, root) = contest();
	let user_files = [
		".claude/skills/tool/.git/config",
		".claude/projects/.mcp.json",
		".direnv/flake/.envrc",
		".direnv/cache/.claude/x",
	];
	for path in user_files {
		touch(&root.join(path));
	}
	touch(&root.join("sub/.claude/x"));
	let report = scan(&root);
	assert_eq!(report.removed, vec![root.join("sub/.claude")]);
	for path in user_files {
		assert!(root.join(path).is_file(), "{path}");
	}
}

#[test]
fn top_level_envrc_planted_after_launch_is_removed() {
	let (_dir, root) = contest();
	let exempt = TopLevelExempt::pin_at_init(&root).unwrap();
	assert_eq!(exempt, TopLevelExempt { envrc: false });
	touch(&root.join(".envrc"));
	let report = remove_planted_files(&root, exempt).unwrap();
	assert_eq!(report.removed, vec![root.join(".envrc")]);
	assert!(absent(&root.join(".envrc")));
}

#[test]
fn envrc_pin_refuses_non_regular_envrc() {
	let (_dir, root) = contest();
	symlink("/etc/hostname", root.join(".envrc")).unwrap();
	let error = TopLevelExempt::pin_at_init(&root).unwrap_err();
	assert!(format!("{error:#}").contains(".envrc"), "{error:#}");
	fs::remove_file(root.join(".envrc")).unwrap();
	fs::create_dir(root.join(".envrc")).unwrap();
	assert!(TopLevelExempt::pin_at_init(&root).is_err());
}

#[test]
fn worker_writable_run_dirs_are_scanned_and_meta_skipped() {
	let (_dir, root) = contest();
	for path in [
		"run/tasks/t0001-a/.git/config",
		"run/tasks/t0001-a/work/.envrc",
		"run/tasks/t0001-a/meta/.git",
		"run/requests/t0001-a/.claude/settings.json",
		"run/.claude/settings.json",
		"run/agents/manager/.git",
	] {
		touch(&root.join(path));
	}
	touch(&root.join("run/tasks/next-id"));
	let report = scan(&root);
	assert_eq!(report.failures, []);
	assert_eq!(
		sorted(report.removed),
		under(
			&root,
			&[
				"run/requests/t0001-a/.claude",
				"run/tasks/t0001-a/.git",
				"run/tasks/t0001-a/work/.envrc",
			]
		)
	);
	assert!(root.join("run/tasks/t0001-a/meta/.git").is_file());
	assert!(root.join("run/.claude/settings.json").is_file());
	assert!(root.join("run/agents/manager/.git").is_file());
}

#[test]
fn symlinks_are_never_followed() {
	let (dir, root) = contest();
	let outside = dir.path().join("outside");
	touch(&outside.join(".git/config"));
	touch(&outside.join("keep"));
	symlink(&outside, root.join("linked")).unwrap();
	fs::create_dir_all(root.join("sub")).unwrap();
	symlink(&outside, root.join("sub/.claude")).unwrap();
	let report = scan(&root);
	assert_eq!(report.removed, vec![root.join("sub/.claude")]);
	assert!(absent(&root.join("sub/.claude")));
	assert!(outside.join(".git/config").is_file());
	assert!(outside.join("keep").is_file());
	assert!(root.join("linked").exists());
}

#[test]
fn planted_nested_repo_with_fsmonitor_is_removed() {
	let (_dir, root) = contest();
	let nested = root.join("sub");
	fs::create_dir_all(&nested).unwrap();
	let status = Command::new("git")
		.arg("-C")
		.arg(&nested)
		.args(["init", "-q"])
		.status()
		.unwrap();
	assert!(status.success());
	let config = nested.join(".git/config");
	let mut text = fs::read_to_string(&config).unwrap();
	text.push_str("[core]\n\tfsmonitor = /bin/false\n");
	fs::write(&config, text).unwrap();
	assert_eq!(scan(&root).removed, vec![nested.join(".git")]);
	assert!(absent(&nested.join(".git")));
}

#[test]
fn symlinked_contest_root_is_refused() {
	let (dir, root) = contest();
	let link = dir.path().join("link");
	symlink(&root, &link).unwrap();
	assert!(remove_planted_files(&link, TopLevelExempt::default()).is_err());
}

#[test]
fn unreadable_dirs_are_opened_after_owner_chmod() {
	let (_dir, root) = contest();
	touch(&root.join("locked/.git/config"));
	touch(&root.join("locked/inner/.claude"));
	touch(&root.join("a/.envrc"));
	touch(&root.join("z/.mcp.json"));
	set_mode(&root.join("locked/.git"), 0o000);
	set_mode(&root.join("locked/inner"), 0o000);
	set_mode(&root.join("locked"), 0o000);
	let report = scan(&root);
	assert_eq!(report.failures, []);
	assert_eq!(
		sorted(report.removed),
		under(
			&root,
			&[
				"a/.envrc",
				"locked/.git",
				"locked/inner/.claude",
				"z/.mcp.json"
			]
		)
	);
}

#[test]
fn unremovable_plant_is_reported_and_siblings_still_removed() {
	let (_dir, root) = contest();
	touch(&root.join("frozen/.git"));
	touch(&root.join("a/.envrc"));
	touch(&root.join("z/.claude/settings.json"));
	set_mode(&root.join("frozen"), 0o500);
	let report = scan(&root);
	set_mode(&root.join("frozen"), 0o700);
	assert!(report.must_fail_closed());
	assert_eq!(failed_paths(&report), under(&root, &["frozen/.git"]));
	assert_eq!(
		sorted(report.removed),
		under(&root, &["a/.envrc", "z/.claude"])
	);
	assert!(root.join("frozen/.git").is_file());
}

fn nested_dirs(base: &Path, depth: usize) -> PathBuf {
	let relative: PathBuf = std::iter::repeat_n("d", depth).collect();
	let path = base.join(relative);
	fs::create_dir_all(&path).unwrap();
	path
}

#[test]
fn tree_deeper_than_cap_is_reported_without_crash() {
	let (_dir, root) = contest();
	let bottom = nested_dirs(&root.join("deep"), MAX_SCAN_DEPTH * 3);
	touch(&bottom.join(".git"));
	touch(&root.join("a/.envrc"));
	touch(&root.join("z/.git"));
	let report = scan(&root);
	assert!(report.must_fail_closed());
	assert_eq!(
		report.failures.len(),
		1,
		"{failures:?}",
		failures = report.failures
	);
	assert!(report.failures[0].0.starts_with(root.join("deep")));
	assert!(report.failures[0].1.contains("deeper"));
	assert_eq!(
		sorted(report.removed),
		under(&root, &["a/.envrc", "z/.git"])
	);
}

#[test]
fn planted_tree_deeper_than_cap_is_reported_without_crash() {
	let (_dir, root) = contest();
	nested_dirs(&root.join("sub/.git"), MAX_SCAN_DEPTH * 3);
	touch(&root.join("z/.git"));
	let report = scan(&root);
	assert_eq!(failed_paths(&report), under(&root, &["sub/.git"]));
	assert_eq!(report.removed, vec![root.join("z/.git")]);
}

#[test]
fn concurrent_writes_into_planted_dir_are_tolerated() {
	let (_dir, root) = contest();
	let planted = root.join("sub/.git");
	fs::create_dir_all(&planted).unwrap();
	touch(&root.join("z/.envrc"));
	let stop = Arc::new(AtomicBool::new(false));
	let writer = {
		let stop = Arc::clone(&stop);
		let planted = planted.clone();
		thread::spawn(move || {
			let mut written = 0u64;
			while !stop.load(Ordering::Relaxed) {
				let _ = fs::write(planted.join(format!("object-{written}")), "x");
				written += 1;
			}
		})
	};
	while fs::read_dir(&planted).map_or(0, Iterator::count) < 100 {
		thread::yield_now();
	}
	let report = scan(&root);
	stop.store(true, Ordering::Relaxed);
	writer.join().unwrap();
	assert!(report.removed.contains(&root.join("z/.envrc")));
	let planted_removed = report.removed.contains(&planted);
	let planted_failed = failed_paths(&report) == [planted.clone()];
	assert!(planted_removed != planted_failed, "{report:?}");
	assert_eq!(planted_removed, absent(&planted));
}

#[test]
fn over_budget_scan_is_its_own_state_not_a_failure() {
	let (_dir, root) = contest();
	for index in 0..50 {
		touch(&root.join(format!("run/tasks/t0001-x/f{index}")));
	}
	let report = remove_planted_files_within(&root, TopLevelExempt::default(), 10).unwrap();
	assert!(report.over_budget);
	assert!(
		report.failures.is_empty(),
		"{failures:?}",
		failures = report.failures
	);
	assert!(!report.must_fail_closed());
	let roomy = remove_planted_files_within(&root, TopLevelExempt::default(), 1000).unwrap();
	assert!(!roomy.over_budget);
	assert_eq!(MAX_SCANNED_ENTRIES, 5_000_000);
}
