use std::path::{Component, Path};

pub struct Limits {
	pub memory_kb: u64,
	pub io_cpu_over_baseline: Option<f64>,
}

const CG_MEMORY_KB: u64 = 768 * 1024;
const GROOVY_IO_CPU_OVER_BASELINE: f64 = 2.05;

pub fn limits(lang: Option<&str>) -> Limits {
	Limits {
		memory_kb: CG_MEMORY_KB,
		io_cpu_over_baseline: match lang {
			Some("groovy") => Some(GROOVY_IO_CPU_OVER_BASELINE),
			_ => None,
		},
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
