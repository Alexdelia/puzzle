use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use vehicle_routing_problem::{DEPOT, Instance, load_instance, score_line, test_paths};

const MAX_CUSTOMERS: usize = 22;
const UNREACHABLE: u32 = u32::MAX / 2;

#[derive(Parser)]
#[command(about = "Prove optimal distance on small tests (Held-Karp + set partition)")]
struct Cli {
	#[arg(value_name = "TEST|DIR", default_values = ["validator/test", "validator/submit"])]
	tests: Vec<PathBuf>,
	#[arg(long, default_value = "solution")]
	solutions: PathBuf,
}

fn single_tour_costs(instance: &Instance) -> Vec<u32> {
	let customers = instance.site_count() - 1;
	let site = |customer: usize| customer + 1;
	let subsets = 1usize << customers;
	let mut ending_at = vec![UNREACHABLE; subsets * customers];
	for last in 0..customers {
		ending_at[(1 << last) * customers + last] = instance.distance(DEPOT, site(last));
	}
	let mut closed = vec![UNREACHABLE; subsets];
	closed[0] = 0;
	for subset in 1..subsets {
		for last in 0..customers {
			let cost = ending_at[subset * customers + last];
			if cost >= UNREACHABLE {
				continue;
			}
			let back = cost + instance.distance(site(last), DEPOT);
			closed[subset] = closed[subset].min(back);
			for next in (0..customers).filter(|next| subset & (1 << next) == 0) {
				let extended = (subset | (1 << next)) * customers + next;
				let step = cost + instance.distance(site(last), site(next));
				ending_at[extended] = ending_at[extended].min(step);
			}
		}
	}
	closed
}

fn optimum(instance: &Instance) -> u32 {
	let customers = instance.site_count() - 1;
	let subsets = 1usize << customers;
	let mut load = vec![0u32; subsets];
	for subset in 1..subsets {
		let lowest = subset.trailing_zeros() as usize;
		load[subset] = load[subset & (subset - 1)] + instance.sites[lowest + 1].demand;
	}
	let tour = single_tour_costs(instance);
	let mut best = vec![UNREACHABLE; subsets];
	best[0] = 0;
	for subset in 1..subsets {
		let lowest = subset & subset.wrapping_neg();
		let rest = subset ^ lowest;
		let mut others = rest;
		loop {
			let route = others | lowest;
			if load[route] <= instance.capacity {
				best[subset] = best[subset].min(tour[route] + best[subset ^ route]);
			}
			if others == 0 {
				break;
			}
			others = (others - 1) & rest;
		}
	}
	best[subsets - 1]
}

fn stored_score(instance: &Instance, path: &Path) -> Option<u32> {
	let text = fs::read_to_string(path).ok()?;
	score_line(instance, text.lines().next()?).ok()
}

fn main() {
	let cli = Cli::parse();
	for test in test_paths(&cli.tests) {
		let instance = load_instance(&test);
		if instance.site_count() - 1 > MAX_CUSTOMERS {
			continue;
		}
		let set = test.parent().and_then(Path::file_name).unwrap_or_default();
		let name = test.file_name().unwrap();
		let stored = stored_score(&instance, &cli.solutions.join(set).join(name));
		println!(
			"{set}/{name:<45} optimum {optimum:>6} stored {stored:?}",
			set = set.to_string_lossy(),
			name = name.to_string_lossy(),
			optimum = optimum(&instance),
		);
	}
}
