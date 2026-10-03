use crate::game::score_plan;
use crate::map::Map;
use crate::plan::{read_cuts, write_cuts};
use crate::rng::Rng;
use std::time::{Duration, Instant};
use std::{env, fs, thread};

const COLD: f64 = 0.3;

pub struct Schedule {
	began: Instant,
	budget: Duration,
	hot: f64,
}

impl Schedule {
	pub fn has_time(&self) -> bool {
		self.began.elapsed() < self.budget
	}

	pub fn temperature(&self) -> Option<f64> {
		let progress = self.began.elapsed().as_secs_f64() / self.budget.as_secs_f64();
		(progress < 1.0).then(|| self.hot * (COLD / self.hot).powf(progress))
	}
}

pub fn improve_plan<F>(name: &str, hot: f64, search: F)
where
	F: Fn(&Map, &[usize], &mut Rng, &Schedule) -> Vec<usize> + Sync,
{
	let args: Vec<String> = env::args().collect();
	let [_, map_path, plan_path, seconds] = &args[..] else {
		panic!("usage: {name} <map> <plan> <seconds>");
	};
	let map = Map::parse(&fs::read_to_string(map_path).unwrap()).unwrap();
	let budget = Duration::from_secs_f64(seconds.parse().unwrap());
	let start = read_cuts(&map, plan_path);
	let start_score = score_plan(&map, &start).unwrap();
	let threads = thread::available_parallelism().map_or(1, |n| n.get() as u64);
	let candidates: Vec<Vec<usize>> = thread::scope(|scope| {
		let workers: Vec<_> = (1..=threads)
			.map(|seed| {
				let (map, start, search) = (&map, &start, &search);
				scope.spawn(move || {
					let schedule = Schedule {
						began: Instant::now(),
						budget,
						hot,
					};
					search(map, start, &mut Rng::seeded(seed), &schedule)
				})
			})
			.collect();
		workers
			.into_iter()
			.map(|worker| worker.join().unwrap())
			.collect()
	});
	let (score, best) = candidates
		.into_iter()
		.filter_map(|cuts| Some((score_plan(&map, &cuts).ok()?, cuts)))
		.max_by_key(|(score, _)| *score)
		.unwrap_or((start_score, start));
	if score > start_score {
		write_cuts(&map, plan_path, &best);
	}
	println!("{map_path} start {start_score} best {score}");
}
