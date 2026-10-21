use std::path::{Component, Path};
use std::process::ExitCode;
use std::time::Duration;

use anyhow::Result;
use referee_core::IoPuzzle;
use referee_core::process::solutions_cpu;
use referee_core::suite::{Case, Played, RunOptions, Runner, Verdict};

const GROOVY_CPU_LIMIT_MS: u64 = 2300;

struct CpuCapped {
	io: IoPuzzle,
	cpu_limit: Option<Duration>,
}

impl Runner for CpuCapped {
	const EXTENSION: &'static str = IoPuzzle::EXTENSION;

	fn run(&self, case: &Case, launcher: &Path, options: &RunOptions) -> Result<Played> {
		let cpu_before = solutions_cpu();
		let played = self.io.run(case, launcher, options)?;
		let cpu = solutions_cpu().saturating_sub(cpu_before);
		Ok(match (played.verdict, self.cpu_limit) {
			(Verdict::Pass, Some(limit)) if cpu > limit => {
				let cpu_ms = cpu.as_millis();
				let limit_ms = limit.as_millis();
				Played {
					verdict: Verdict::Timeout(format!(
						"solution cpu {cpu_ms} ms over {limit_ms} ms"
					)),
					..played
				}
			}
			(verdict, _) => Played { verdict, ..played },
		})
	}
}

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

fn cpu_limit(lang: Option<&str>) -> Option<Duration> {
	match lang {
		Some("groovy") => Some(Duration::from_millis(GROOVY_CPU_LIMIT_MS)),
		_ => None,
	}
}

fn main() -> ExitCode {
	let lang = std::env::args()
		.nth(1)
		.and_then(|launcher| launcher_lang(Path::new(&launcher)));
	referee_core::main(CpuCapped {
		io: IoPuzzle::default(),
		cpu_limit: cpu_limit(lang.as_deref()),
	})
}
