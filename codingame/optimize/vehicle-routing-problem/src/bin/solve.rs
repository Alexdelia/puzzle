use clap::Parser;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::thread;
use vehicle_routing_problem::{
	Instance, Tour, load_instance, score_line, score_tours, test_paths, write_tours,
};

#[derive(Parser)]
#[command(about = "Run HGS-CVRP on each test, keep the best checked answer per test")]
struct Cli {
	#[arg(value_name = "TEST|DIR", default_values = ["validator/test", "validator/submit"])]
	tests: Vec<PathBuf>,
	#[arg(long, default_value_t = 60.0)]
	seconds: f64,
	#[arg(long, default_value_t = 8)]
	seeds: u32,
	#[arg(long, default_value_t = 1)]
	first_seed: u32,
	#[arg(long, default_value_t = thread::available_parallelism().map_or(1, |n| n.get()))]
	jobs: usize,
	#[arg(long, default_value = "hgs")]
	hgs: PathBuf,
	#[arg(long, default_value = "target/hgs")]
	work: PathBuf,
	#[arg(long, default_value = "solution")]
	solutions: PathBuf,
}

struct Job {
	test: PathBuf,
	seed: u32,
}

fn cvrplib(instance: &Instance, name: &str) -> String {
	let mut text = format!(
		"NAME : {name}\nCOMMENT : codingame\nTYPE : CVRP\nDIMENSION : {dimension}\nEDGE_WEIGHT_TYPE : EUC_2D\nCAPACITY : {capacity}\nNODE_COORD_SECTION\n",
		dimension = instance.site_count(),
		capacity = instance.capacity,
	);
	for (index, site) in instance.sites.iter().enumerate() {
		writeln!(
			text,
			"{node} {x} {y}",
			node = index + 1,
			x = site.x,
			y = site.y
		)
		.unwrap();
	}
	text.push_str("DEMAND_SECTION\n");
	for (index, site) in instance.sites.iter().enumerate() {
		writeln!(
			text,
			"{node} {demand}",
			node = index + 1,
			demand = site.demand
		)
		.unwrap();
	}
	text.push_str("DEPOT_SECTION\n1\n-1\nEOF\n");
	text
}

fn parse_hgs_solution(text: &str) -> Vec<Tour> {
	text.lines()
		.filter_map(|line| line.strip_prefix("Route #"))
		.map(|route| {
			let (_, stops) = route.split_once(':').unwrap();
			stops
				.split_whitespace()
				.map(|stop| stop.parse().unwrap())
				.collect()
		})
		.collect()
}

fn set_and_name(test: &Path) -> (String, String) {
	let name = test.file_name().unwrap().to_string_lossy().into_owned();
	let set = test
		.parent()
		.and_then(Path::file_name)
		.map_or(String::new(), |set| set.to_string_lossy().into_owned());
	(set, name)
}

fn stored_score(instance: &Instance, path: &Path) -> Option<u32> {
	let text = fs::read_to_string(path).ok()?;
	score_line(instance, text.lines().next()?).ok()
}

fn work_path(cli: &Cli, test: &Path, extension: &str) -> PathBuf {
	let (set, name) = set_and_name(test);
	let stem = name.trim_end_matches(".txt");
	cli.work.join(set).join(format!("{stem}.{extension}"))
}

fn write_problem(cli: &Cli, test: &Path) {
	let problem = work_path(cli, test, "vrp");
	fs::create_dir_all(problem.parent().unwrap()).unwrap();
	let name = problem.file_stem().unwrap().to_string_lossy();
	fs::write(&problem, cvrplib(&load_instance(test), &name)).unwrap();
}

fn run(cli: &Cli, job: &Job) -> Result<(Vec<Tour>, u32), String> {
	let instance = load_instance(&job.test);
	let problem = work_path(cli, &job.test, "vrp");
	let answer = work_path(cli, &job.test, &format!("seed{seed}.sol", seed = job.seed));
	let status = Command::new(&cli.hgs)
		.arg(&problem)
		.arg(&answer)
		.args(["-t", &cli.seconds.to_string()])
		.args(["-seed", &job.seed.to_string()])
		.args(["-round", "1", "-log", "0"])
		.stdout(Stdio::null())
		.status()
		.map_err(|e| format!("cannot start {hgs:?}: {e}", hgs = cli.hgs))?;
	if !status.success() {
		return Err(format!("hgs exited with {status}"));
	}
	let text = fs::read_to_string(&answer).map_err(|e| format!("{answer:?}: {e}"))?;
	let tours = parse_hgs_solution(&text);
	let score = score_tours(&instance, &tours).map_err(|fault| fault.to_string())?;
	Ok((tours, score))
}

fn keep_if_better(cli: &Cli, job: &Job, tours: &[Tour], score: u32) {
	let instance = load_instance(&job.test);
	let (set, name) = set_and_name(&job.test);
	let dir = cli.solutions.join(&set);
	fs::create_dir_all(&dir).unwrap();
	let path = dir.join(&name);
	let previous = stored_score(&instance, &path);
	let verdict = match previous {
		Some(best) if best <= score => format!("kept {best}"),
		_ => {
			fs::write(&path, write_tours(tours) + "\n").unwrap();
			format!("NEW (was {previous:?})")
		}
	};
	println!(
		"{set}/{name:<45} seed {seed:>3} {score:>7} {verdict}",
		seed = job.seed,
	);
}

fn main() {
	let cli = Cli::parse();
	let tests = test_paths(&cli.tests);
	for test in &tests {
		write_problem(&cli, test);
	}
	let jobs = tests
		.into_iter()
		.flat_map(|test| {
			(cli.first_seed..cli.first_seed + cli.seeds).map(move |seed| Job {
				test: test.clone(),
				seed,
			})
		})
		.collect::<Vec<_>>();
	let queue = Mutex::new(jobs.into_iter());
	let next_job = || queue.lock().unwrap().next();
	let store = Mutex::new(());
	thread::scope(|scope| {
		for _ in 0..cli.jobs {
			scope.spawn(|| {
				while let Some(job) = next_job() {
					match run(&cli, &job) {
						Ok((tours, score)) => {
							let _guard = store.lock().unwrap();
							keep_if_better(&cli, &job, &tours, score);
						}
						Err(e) => eprintln!(
							"{test} seed {seed}: {e}",
							test = job.test.display(),
							seed = job.seed,
						),
					}
				}
			});
		}
	});
}
