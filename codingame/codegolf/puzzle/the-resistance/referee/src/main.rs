use std::path::{Component, Path};
use std::process::ExitCode;
use std::time::Duration;

use referee_core::IoPuzzle;

const DEFAULT_LIMIT_MS: u64 = 6000;
const GROOVY_LIMIT_MS: u64 = 1100;

fn launcher_lang(launcher: &Path) -> Option<String> {
	let names: Vec<_> = launcher
		.components()
		.filter_map(|component| match component {
			Component::Normal(name) => name.to_str(),
			_ => None,
		})
		.collect();
	names
		.windows(2)
		.find(|pair| pair[0] == "lang")
		.map(|pair| pair[1].to_string())
}

fn limit_ms(lang: Option<&str>) -> u64 {
	match lang {
		Some("groovy") => GROOVY_LIMIT_MS,
		_ => DEFAULT_LIMIT_MS,
	}
}

fn main() -> ExitCode {
	let lang = std::env::args()
		.nth(1)
		.and_then(|launcher| launcher_lang(Path::new(&launcher)));
	referee_core::main(IoPuzzle {
		limit: Duration::from_millis(limit_ms(lang.as_deref())),
	})
}
