use clap::Parser;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tryangle::arena::{EndReason, MatchResult, TimeLimits, elo_diff, run_match, wilson_ci};
use tryangle::cli::{BotArgs, MapArgs, die};
use tryangle::game::Game;

/// Play many TryAngle-Catch games and report how the two bots compare.
#[derive(Parser, Debug)]
#[command(name = "arena", version, about, long_about = None)]
struct Cli {
	#[command(flatten)]
	bots: BotArgs,

	#[command(flatten)]
	map: MapArgs,

	/// number of games to play
	#[arg(long, value_name = "N", default_value_t = 100, value_parser = clap::value_parser!(u64).range(1..))]
	games: u64,

	/// games to run at once
	#[arg(long, value_name = "T", default_value_t = default_threads(), value_parser = clap::value_parser!(u64).range(1..))]
	threads: u64,

	/// play each board once instead of twice with the sides swapped
	#[arg(long)]
	no_swap: bool,

	/// print one line per game
	#[arg(long)]
	verbose: bool,
}

fn default_threads() -> u64 {
	std::thread::available_parallelism().map_or(4, |n| n.get()) as u64
}

struct Pairing {
	index: usize,
	seed: i64,
	swapped: bool,
}

fn play(cli: &Cli, pairing: &Pairing) -> MatchResult {
	let board = cli.map.build(pairing.seed).unwrap_or_else(die);
	let mut game = Game::new(board, cli.map.league);
	let mut d0 = cli
		.bots
		.tagged_driver(usize::from(pairing.swapped), pairing.index)
		.unwrap_or_else(die);
	let mut d1 = cli
		.bots
		.tagged_driver(usize::from(!pairing.swapped), pairing.index)
		.unwrap_or_else(die);
	let limits = cli.bots.enforce_cg_time.then(TimeLimits::default);
	let mut result = run_match(&mut game, [d0.as_mut(), d1.as_mut()], limits, None);
	if pairing.swapped {
		result.score.swap(0, 1);
		result.winner = result.winner.map(|side| 1 - side);
		result.culprit = result.culprit.map(|side| 1 - side);
		result.slowest.swap(0, 1);
	}
	result
}

fn main() {
	let cli = Cli::parse();
	let games = cli.games as usize;
	let swap = !cli.no_swap;
	let pairings: Vec<Pairing> = (0..games)
		.map(|index| Pairing {
			index,
			seed: cli.map.seed
				+ if swap {
					(index / 2) as i64
				} else {
					index as i64
				},
			swapped: swap && index % 2 == 1,
		})
		.collect();

	let next = AtomicUsize::new(0);
	let played: Mutex<Vec<(usize, i64, bool, MatchResult)>> = Mutex::new(Vec::with_capacity(games));
	std::thread::scope(|scope| {
		for _ in 0..(cli.threads as usize).min(games) {
			scope.spawn(|| {
				loop {
					let index = next.fetch_add(1, Ordering::Relaxed);
					let Some(pairing) = pairings.get(index) else {
						return;
					};
					let result = play(&cli, pairing);
					played.lock().unwrap().push((
						pairing.index,
						pairing.seed,
						pairing.swapped,
						result,
					));
				}
			});
		}
	});

	let mut played = played.into_inner().unwrap();
	played.sort_by_key(|(index, ..)| *index);
	let mut wins = [0usize; 2];
	let mut draws = 0usize;
	let mut faults = [0usize; 2];
	let mut slowest = [Duration::ZERO; 2];
	let mut score_sum = [0i64; 2];
	for (index, seed, swapped, result) in &played {
		match result.winner {
			Some(side) => wins[side] += 1,
			None => draws += 1,
		}
		if let (EndReason::Deactivated, Some(culprit), Some(note)) =
			(result.reason, result.culprit, &result.note)
			&& (note.contains("timeout") || note.contains("Unknown command"))
		{
			faults[culprit] += 1;
		}
		for side in 0..2 {
			slowest[side] = slowest[side].max(result.slowest[side]);
			score_sum[side] += i64::from(result.score[side]);
		}
		if cli.verbose {
			println!(
				"game {index} seed {seed} swapped {swapped} winner {:?} score {} {} turns {} reason {:?} {}",
				result.winner,
				result.score[0],
				result.score[1],
				result.turns,
				result.reason,
				result.note.clone().unwrap_or_default()
			);
		}
	}
	let n = played.len() as f64;
	let points = wins[0] as f64 + 0.5 * draws as f64;
	let (low, high) = wilson_ci(points, n);
	println!("p1 {}\np2 {}", cli.bots.spec(0), cli.bots.spec(1));
	println!(
		"games {} p1 wins {} p2 wins {} draws {} | p1 score {:.1}% [{:.1}%, {:.1}%] elo {:+.0}",
		played.len(),
		wins[0],
		wins[1],
		draws,
		100.0 * points / n,
		100.0 * low,
		100.0 * high,
		elo_diff(points / n)
	);
	println!(
		"mean score p1 {:.1} p2 {:.1} | timeouts or crashes p1 {} p2 {} | slowest answer p1 {:?} p2 {:?}",
		score_sum[0] as f64 / n,
		score_sum[1] as f64 / n,
		faults[0],
		faults[1],
		slowest[0],
		slowest[1]
	);
}
