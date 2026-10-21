use std::path::{Component, Path};
use std::time::Duration;

pub struct Limits {
	pub memory_kb: u64,
	pub io_cpu: Option<Duration>,
	pub first_turn_cpu: Option<Duration>,
}

const CG_MEMORY_KB: u64 = 768 * 1024;

pub fn limits(lang: Option<&str>) -> Limits {
	let unbounded = Limits {
		memory_kb: CG_MEMORY_KB,
		io_cpu: None,
		first_turn_cpu: None,
	};
	match lang {
		_ => unbounded,
	}
}

pub fn of_launcher(launcher: &Path) -> Option<String> {
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
