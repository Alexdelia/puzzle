use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PathConfig {
	pub base_dir: PathBuf,
	pub validator: PathBuf,
	pub output_dir: PathBuf,
	pub score: PathBuf,
	pub solution: PathBuf,
}

impl PathConfig {
	pub fn new(base_dir: PathBuf, validator_name: &str) -> Self {
		let validator = base_dir
			.join("validator")
			.join(format!("{validator_name}.txt"));
		let output_dir = base_dir.join("output").join(validator_name);
		let score = output_dir.join("score.txt");
		let solution = output_dir.join("solution.txt");
		Self {
			base_dir,
			validator,
			output_dir,
			score,
			solution,
		}
	}
}

pub fn cwd() -> Result<PathBuf, String> {
	std::env::current_dir().map_err(|e| format!("cwd: {e}"))
}
