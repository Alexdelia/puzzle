use btk::arena::{EndReason, MatchResult, TimeLimits, elo_diff, run_match, wilson_ci};
use btk::cli::{BotArgs, MapArgs, RuleArgs, die};
use btk::game::Game;
use btk::trace::{Trace, replay_text};
use clap::Parser;
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// Play many Back Track King games and report how the two bots compare.
#[derive(Parser, Debug)]
#[command(name = "arena", version, about, long_about = None)]
struct Cli {
	#[command(flatten)]
	bots: BotArgs,

	#[command(flatten)]
	map: MapArgs,

	#[command(flatten)]
	rules: RuleArgs,

	/// number of games to play
	#[arg(long, value_name = "N", default_value_t = 100, value_parser = clap::value_parser!(u64).range(1..))]
	games: u64,

	/// games to run at once
	#[arg(long, value_name = "T", default_value_t = default_threads(), value_parser = clap::value_parser!(u64).range(1..))]
	threads: u64,

	/// play each map once instead of twice with the sides swapped
	#[arg(long)]
	no_swap: bool,

	/// write one replay trace per game into this directory
	#[arg(long, value_name = "DIR")]
	replay_dir: Option<String>,

	/// leave measured turn times out of the summary, so two runs compare byte for byte
	#[arg(long)]
	no_timings: bool,

	/// suppress progress output
	#[arg(long)]
	quiet: bool,
}

impl Cli {
	fn swap(&self) -> bool {
		!self.no_swap
	}
}

fn default_threads() -> u64 {
	std::thread::available_parallelism().map_or(4, |n| n.get()) as u64
}

struct Pairing {
	index: usize,
	seed: u64,
	swapped: bool,
}

struct Played {
	pairing: Pairing,
	result: MatchResult,
}

fn main() {
	let cli = Cli::parse();
	let games = cli.games as usize;
	if let Some(dir) = &cli.replay_dir {
		std::fs::create_dir_all(dir).unwrap_or_else(|e| die(format!("{dir}: {e}")));
	}
	if cli.map.map.is_some() && games > 1 {
		eprintln!(
			"note: --map pins every game to one board, so the interval below covers bot noise only"
		);
	}

	let pairings: Vec<Pairing> = (0..games)
		.map(|index| Pairing {
			index,
			seed: cli.map.seed
				+ if cli.swap() {
					(index / 2) as u64
				} else {
					index as u64
				},
			swapped: cli.swap() && index % 2 == 1,
		})
		.collect();

	let next = AtomicUsize::new(0);
	let done = AtomicUsize::new(0);
	let played: Mutex<Vec<Played>> = Mutex::new(Vec::with_capacity(games));
	let crashed: Mutex<Vec<u64>> = Mutex::new(Vec::new());
	std::thread::scope(|scope| {
		for _ in 0..(cli.threads as usize).min(games) {
			scope.spawn(|| {
				loop {
					let index = next.fetch_add(1, Ordering::Relaxed);
					if index >= pairings.len() {
						return;
					}
					let pairing = &pairings[index];
					let attempt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
						play(&cli, pairing)
					}));
					match attempt {
						Ok(result) => played.lock().unwrap().push(Played {
							pairing: Pairing {
								index: pairing.index,
								seed: pairing.seed,
								swapped: pairing.swapped,
							},
							result,
						}),
						Err(_) => crashed.lock().unwrap().push(pairing.seed),
					}
					let count = done.fetch_add(1, Ordering::Relaxed) + 1;
					if !cli.quiet && games > 20 && count.is_multiple_of(20) {
						eprintln!("[{count}/{games}]");
					}
				}
			});
		}
	});

	let mut played = played.into_inner().unwrap();
	played.sort_by_key(|p| p.pairing.index);
	report(&cli, &played);

	let mut crashed = crashed.into_inner().unwrap();
	if !crashed.is_empty() {
		crashed.sort_unstable();
		println!("referee crashed on {} seeds: {crashed:?}", crashed.len());
		std::process::exit(1);
	}
}

fn play(cli: &Cli, pairing: &Pairing) -> MatchResult {
	let map = cli.map.build(pairing.seed).unwrap_or_else(die);
	let mut game = Game::new(map, cli.rules.rules());
	let mut d0 = cli
		.bots
		.driver(usize::from(pairing.swapped))
		.unwrap_or_else(die);
	let mut d1 = cli
		.bots
		.driver(usize::from(!pairing.swapped))
		.unwrap_or_else(die);
	let limits = cli.bots.enforce_cg_time.then(TimeLimits::default);

	let mut trace = Trace::default();
	let mut result = if cli.replay_dir.is_some() {
		let mut record = |game: &Game, report: &_| trace.record(game, report);
		run_match(
			&mut game,
			[d0.as_mut(), d1.as_mut()],
			limits,
			Some(&mut record),
		)
	} else {
		run_match(&mut game, [d0.as_mut(), d1.as_mut()], limits, None)
	};

	if let Some(dir) = &cli.replay_dir {
		let path = format!(
			"{dir}/game{:05}_seed{}_{}.txt",
			pairing.index,
			pairing.seed,
			if pairing.swapped { "swap" } else { "norm" }
		);
		let header = [
			("seed", pairing.seed.to_string()),
			("swapped", pairing.swapped.to_string()),
			("p1", cli.bots.spec(0).to_string()),
			("p2", cli.bots.spec(1).to_string()),
			("rules", format!("{:?}", cli.rules.rules())),
		];
		let verdict = format!("{:?} winner {:?}", result.reason, result.winner);
		let _ = std::fs::write(path, replay_text(&header, &game, &trace, verdict));
	}

	if pairing.swapped {
		result.score.swap(0, 1);
		result.winner = result.winner.map(|side| 1 - side);
		result.culprit = result.culprit.map(|side| 1 - side);
		result.slowest.swap(0, 1);
	}
	result
}

fn replay_command(cli: &Cli, pairing: &Pairing) -> String {
	let mut line = format!(
		"referee --p1 {} --p2 {} --seed {}",
		shell_quote(cli.bots.spec(0)),
		shell_quote(cli.bots.spec(1)),
		pairing.seed
	);
	if pairing.swapped {
		line.push_str(" --swap");
	}
	for (flag, on) in [
		("--disruption", cli.rules.disruption),
		("--neutral-scores", cli.rules.neutral_scores),
		("--strict", cli.rules.strict),
	] {
		if on {
			line.push(' ');
			line.push_str(flag);
		}
	}
	line
}

fn shell_quote(spec: &str) -> String {
	if spec
		.bytes()
		.all(|b| b.is_ascii_alphanumeric() || b"-_./:".contains(&b))
	{
		spec.to_string()
	} else {
		format!("'{}'", spec.replace('\'', r"'\''"))
	}
}

fn report(cli: &Cli, played: &[Played]) {
	let n = played.len();
	let mut wins = [0usize; 2];
	let mut draws = 0usize;
	let mut points = [0u64; 2];
	let mut side = [[[0usize; 3]; 2]; 2];
	let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
	let mut slowest = [Duration::ZERO; 2];
	let mut incidents: Vec<String> = Vec::new();

	for Played { pairing, result } in played {
		match result.winner {
			Some(bot) => wins[bot] += 1,
			None => draws += 1,
		}
		points[0] += result.score[0] as u64;
		points[1] += result.score[1] as u64;
		*reasons.entry(format!("{:?}", result.reason)).or_default() += 1;
		for bot in 0..2 {
			let seat = if pairing.swapped { 1 - bot } else { bot };
			let outcome = match result.winner {
				Some(winner) if winner == bot => 0,
				Some(_) => 1,
				None => 2,
			};
			side[bot][seat][outcome] += 1;
			slowest[bot] = slowest[bot].max(result.slowest[bot]);
		}
		if result.reason != EndReason::TurnLimit
			&& let (Some(bot), Some(note)) = (result.culprit, &result.note)
		{
			incidents.push(format!(
				"p{} {note}\n    {}",
				bot + 1,
				replay_command(cli, pairing)
			));
		}
	}

	let scored = wins[0] as f64 + 0.5 * draws as f64;
	let rate = scored / n.max(1) as f64;
	let (lo, hi) = wilson_ci(scored, n as f64);
	println!();
	println!("p1: {}", cli.bots.spec(0));
	println!("p2: {}", cli.bots.spec(1));
	println!(
		"games: {n} (seeds {}..{}, swap {})",
		cli.map.seed,
		cli.map.seed
			+ if cli.swap() {
				(n as u64).div_ceil(2)
			} else {
				n as u64
			} - 1,
		cli.swap()
	);
	println!(
		"p1 wins: {} ({:.1}%)  p2 wins: {} ({:.1}%)  draws: {draws}",
		wins[0],
		100.0 * wins[0] as f64 / n.max(1) as f64,
		wins[1],
		100.0 * wins[1] as f64 / n.max(1) as f64
	);
	println!(
		"p1 score rate CI95: [{:.1}%, {:.1}%]  elo: {:+.0} [{:+.0}, {:+.0}]",
		100.0 * lo,
		100.0 * hi,
		elo_diff(rate),
		elo_diff(lo),
		elo_diff(hi)
	);
	println!(
		"p1 as side0: {}-{}-{}  as side1: {}-{}-{}  (w-l-d)",
		side[0][0][0], side[0][0][1], side[0][0][2], side[0][1][0], side[0][1][1], side[0][1][2]
	);
	println!(
		"mean points: p1 {:.1}  p2 {:.1}",
		points[0] as f64 / n.max(1) as f64,
		points[1] as f64 / n.max(1) as f64
	);
	if !cli.no_timings {
		println!("slowest turn: p1 {:?}  p2 {:?}", slowest[0], slowest[1]);
	}
	let tally: Vec<String> = reasons.iter().map(|(k, v)| format!("{k}: {v}")).collect();
	println!("end reasons: {}", tally.join(", "));
	for incident in incidents.iter().take(5) {
		println!("  {incident}");
	}
	if incidents.len() > 5 {
		println!("  ... and {} more", incidents.len() - 5);
	}
}
