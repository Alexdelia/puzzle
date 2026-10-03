use crate::instance::Instance;
use std::fs;
use std::path::{Path, PathBuf};

pub fn load_instance(path: &Path) -> Instance {
	let shown = path.display();
	let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{shown}: {e}"));
	Instance::parse(&text).unwrap_or_else(|e| panic!("{shown}: {e}"))
}

fn txt_files(dir: &Path) -> Vec<PathBuf> {
	let mut files = fs::read_dir(dir)
		.unwrap_or_else(|e| panic!("{dir}: {e}", dir = dir.display()))
		.map(|entry| entry.unwrap().path())
		.filter(|path| path.extension().is_some_and(|ext| ext == "txt"))
		.collect::<Vec<_>>();
	files.sort();
	files
}

pub fn test_paths(paths: &[PathBuf]) -> Vec<PathBuf> {
	paths
		.iter()
		.flat_map(|path| {
			if path.is_dir() {
				txt_files(path)
			} else {
				vec![path.clone()]
			}
		})
		.collect()
}
