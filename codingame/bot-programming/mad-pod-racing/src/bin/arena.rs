use mpr::bot_search::{Budget, SearchBot};
use mpr::bot_simple::SimpleBot;
use mpr::geom::V2;
use mpr::maps::{Rng, agade_map, format_map, parse_map, random_map};
use mpr::race::{Action, BotAction, Driver, Obs, RaceResult, Track, run_race};
use mpr::sim::{Cmd, CmdKind, State};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, sync_channel};
use std::time::Duration;

#[derive(Clone)]
enum MapSource {
	AgadePool,
	Random,
	Fixed(Vec<V2>),
}

#[derive(Clone)]
struct Config {
	p: [String; 2],
	games: usize,
	seed: u64,
	threads: usize,
	map_source: MapSource,
	laps: usize,
	swap: bool,
	replay_dir: Option<String>,
	bot_timeout: Duration,
	quiet: bool,
}

fn usage() -> ! {
	eprintln!(
		"usage: arena --p1 SPEC --p2 SPEC [options]
  SPEC                 simple | cmd:<shell command>
  --games N            number of games (default 100)
  --seed S             base seed (default 1)
  --threads T          parallel games (default: available cores)
  --pool agade|random  map generator (default agade)
  --map \"x,y;x,y;...\"  fixed checkpoint list
  --laps L             laps (default 3)
  --no-swap            do not mirror each seed with sides swapped
  --replay-dir DIR     write per-game replay traces
  --bot-timeout MS     external bot response timeout (default 10000)
  --quiet              suppress per-game progress"
	);
	std::process::exit(2)
}

fn parse_args() -> Config {
	let mut cfg = Config {
		p: [String::new(), String::new()],
		games: 100,
		seed: 1,
		threads: std::thread::available_parallelism().map_or(4, |n| n.get()),
		map_source: MapSource::AgadePool,
		laps: 3,
		swap: true,
		replay_dir: None,
		bot_timeout: Duration::from_millis(10000),
		quiet: false,
	};
	let mut args = std::env::args().skip(1);
	while let Some(a) = args.next() {
		let mut val = || args.next().unwrap_or_else(|| usage());
		match a.as_str() {
			"--p1" => cfg.p[0] = val(),
			"--p2" => cfg.p[1] = val(),
			"--games" => cfg.games = val().parse().unwrap_or_else(|_| usage()),
			"--seed" => cfg.seed = val().parse().unwrap_or_else(|_| usage()),
			"--threads" => cfg.threads = val().parse().unwrap_or_else(|_| usage()),
			"--pool" => {
				cfg.map_source = match val().as_str() {
					"agade" => MapSource::AgadePool,
					"random" => MapSource::Random,
					_ => usage(),
				}
			}
			"--map" => {
				cfg.map_source = MapSource::Fixed(parse_map(&val()).unwrap_or_else(|| usage()))
			}
			"--laps" => cfg.laps = val().parse().unwrap_or_else(|_| usage()),
			"--no-swap" => cfg.swap = false,
			"--replay-dir" => cfg.replay_dir = Some(val()),
			"--bot-timeout" => {
				cfg.bot_timeout = Duration::from_millis(val().parse().unwrap_or_else(|_| usage()))
			}
			"--quiet" => cfg.quiet = true,
			_ => usage(),
		}
	}
	if cfg.p[0].is_empty() || cfg.p[1].is_empty() {
		usage();
	}
	cfg
}

struct ProcessDriver {
	child: Child,
	stdin: std::process::ChildStdin,
	lines: Receiver<String>,
	timeout: Duration,
}

impl ProcessDriver {
	fn spawn(cmd: &str, timeout: Duration) -> std::io::Result<Self> {
		let mut child = Command::new("sh")
			.arg("-c")
			.arg(cmd)
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.stderr(Stdio::null())
			.spawn()?;
		let stdin = child.stdin.take().unwrap();
		let stdout = child.stdout.take().unwrap();
		let (tx, rx) = sync_channel(64);
		std::thread::spawn(move || {
			for line in BufReader::new(stdout).lines() {
				match line {
					Ok(l) => {
						if tx.send(l).is_err() {
							return;
						}
					}
					Err(_) => return,
				}
			}
		});
		Ok(ProcessDriver {
			child,
			stdin,
			lines: rx,
			timeout,
		})
	}

	fn read_action(&mut self) -> Option<Action> {
		let line = self.lines.recv_timeout(self.timeout).ok()?;
		let mut tokens = line.split_whitespace();
		let x: i64 = tokens.next()?.parse().ok()?;
		let y: i64 = tokens.next()?.parse().ok()?;
		let act = match tokens.next()? {
			"BOOST" => BotAction::Boost,
			"SHIELD" => BotAction::Shield,
			t => BotAction::Thrust(t.parse().ok()?),
		};
		Some(Action { x, y, act })
	}
}

impl Driver for ProcessDriver {
	fn init(&mut self, track: &Track) {
		let mut msg = format!("{}\n{}\n", track.laps, track.cps.len());
		for c in &track.cps {
			msg.push_str(&format!("{} {}\n", c.x as i64, c.y as i64));
		}
		let _ = self.stdin.write_all(msg.as_bytes());
		let _ = self.stdin.flush();
	}

	fn act(&mut self, obs: &Obs) -> Option<[Action; 2]> {
		let mut msg = String::new();
		for p in &obs.pods {
			msg.push_str(&format!(
				"{} {} {} {} {} {}\n",
				p.x, p.y, p.vx, p.vy, p.angle, p.next_cp
			));
		}
		self.stdin.write_all(msg.as_bytes()).ok()?;
		self.stdin.flush().ok()?;
		Some([self.read_action()?, self.read_action()?])
	}
}

impl Drop for ProcessDriver {
	fn drop(&mut self) {
		let _ = self.child.kill();
		let _ = self.child.wait();
	}
}

fn parse_search_spec(spec: &str) -> Box<dyn Driver> {
	let mut budget = Budget::Rollouts(1500);
	let mut seed = 0x5EEDu64;
	if let Some(params) = spec.strip_prefix("search:") {
		for kv in params.split(',') {
			match kv.split_once('=') {
				Some(("r", v)) => budget = Budget::Rollouts(v.parse().expect("search r=")),
				Some(("t", v)) => budget = Budget::TimeMs(v.parse().expect("search t=")),
				Some(("seed", v)) => seed = v.parse().expect("search seed="),
				_ => {
					eprintln!("bad search param: {kv}");
					std::process::exit(2)
				}
			}
		}
	}
	Box::new(SearchBot::new(budget, seed))
}

fn make_driver(spec: &str, timeout: Duration) -> Box<dyn Driver> {
	match spec {
		"simple" => Box::new(SimpleBot::new()),
		s if s == "search" || s.starts_with("search:") => parse_search_spec(s),
		s if s.starts_with("cmd:") => {
			Box::new(ProcessDriver::spawn(&s[4..], timeout).expect("failed to spawn bot process"))
		}
		_ => {
			eprintln!("unknown bot spec: {spec}");
			std::process::exit(2)
		}
	}
}

#[derive(Clone)]
struct GameSpec {
	index: usize,
	pair_seed: u64,
	swapped: bool,
}

struct GameRecord {
	spec: GameSpec,
	winner_bot: usize,
	result: RaceResult,
}

fn game_map(cfg: &Config, pair_seed: u64) -> Vec<V2> {
	let mut rng = Rng::new(
		pair_seed
			.wrapping_mul(0x2545F4914F6CDD1D)
			.wrapping_add(0x1234567),
	);
	match &cfg.map_source {
		MapSource::AgadePool => agade_map(&mut rng),
		MapSource::Random => random_map(&mut rng),
		MapSource::Fixed(cps) => cps.clone(),
	}
}

fn write_replay(
	dir: &str,
	spec: &GameSpec,
	track: &Track,
	turns: &[(State, [Cmd; 4])],
	result: &RaceResult,
	winner_bot: usize,
) {
	use std::fmt::Write;
	let path = format!(
		"{dir}/game{:05}_seed{}_{}.txt",
		spec.index,
		spec.pair_seed,
		if spec.swapped { "swap" } else { "norm" }
	);
	let mut out = String::new();
	let _ = writeln!(out, "map {}", format_map(&track.cps));
	let _ = writeln!(out, "laps {}", track.laps);
	let _ = writeln!(out, "swapped {}", spec.swapped);
	for (state, cmds) in turns {
		let _ = writeln!(out, "turn {}", state.turn);
		for (i, p) in state.pods.iter().enumerate() {
			let kind = match cmds[i].kind {
				CmdKind::Thrust(t) => format!("{t}"),
				CmdKind::Boost => "BOOST".into(),
				CmdKind::Shield => "SHIELD".into(),
			};
			let _ = writeln!(
				out,
				"pod {i} {} {} {} {} {:.3} {} {} {} cmd {} {} {}",
				p.pos.x as i64,
				p.pos.y as i64,
				p.vel.x as i64,
				p.vel.y as i64,
				p.angle,
				p.next,
				p.shield,
				p.boosted as u8,
				cmds[i].target.x as i64,
				cmds[i].target.y as i64,
				kind,
			);
		}
	}
	let _ = writeln!(
		out,
		"result winner_bot {} reason {:?} turns {}",
		winner_bot, result.reason, result.turns
	);
	let _ = std::fs::write(path, out);
}

fn play_game(cfg: &Config, spec: &GameSpec) -> GameRecord {
	let track = Track::new(game_map(cfg, spec.pair_seed), cfg.laps);
	let (spec0, spec1) = if spec.swapped {
		(&cfg.p[1], &cfg.p[0])
	} else {
		(&cfg.p[0], &cfg.p[1])
	};
	let mut d0 = make_driver(spec0, cfg.bot_timeout);
	let mut d1 = make_driver(spec1, cfg.bot_timeout);

	let mut turns: Vec<(State, [Cmd; 4])> = Vec::new();
	let result = if cfg.replay_dir.is_some() {
		let mut sink = |s: &State, c: &[Cmd; 4]| turns.push((*s, *c));
		run_race(&track, [d0.as_mut(), d1.as_mut()], Some(&mut sink))
	} else {
		run_race(&track, [d0.as_mut(), d1.as_mut()], None)
	};

	let winner_bot = if spec.swapped {
		1 - result.winner
	} else {
		result.winner
	};
	if let Some(dir) = &cfg.replay_dir {
		write_replay(dir, spec, &track, &turns, &result, winner_bot);
	}
	GameRecord {
		spec: spec.clone(),
		winner_bot,
		result,
	}
}

fn wilson_ci(wins: f64, n: f64) -> (f64, f64) {
	if n == 0.0 {
		return (0.0, 1.0);
	}
	let z = 1.96;
	let p = wins / n;
	let z2 = z * z;
	let denom = 1.0 + z2 / n;
	let center = (p + z2 / (2.0 * n)) / denom;
	let half = z * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt() / denom;
	(center - half, center + half)
}

fn elo_diff(p: f64) -> f64 {
	let p = p.clamp(0.001, 0.999);
	-400.0 * (1.0 / p - 1.0).log10()
}

fn main() {
	let cfg = parse_args();
	if let Some(dir) = &cfg.replay_dir {
		std::fs::create_dir_all(dir).unwrap();
	}

	let specs: Vec<GameSpec> = (0..cfg.games)
		.map(|g| GameSpec {
			index: g,
			pair_seed: cfg.seed + if cfg.swap { (g / 2) as u64 } else { g as u64 },
			swapped: cfg.swap && g % 2 == 1,
		})
		.collect();

	let next = Mutex::new(0usize);
	let records: Mutex<Vec<GameRecord>> = Mutex::new(Vec::with_capacity(cfg.games));
	let done = std::sync::atomic::AtomicUsize::new(0);

	std::thread::scope(|scope| {
		for _ in 0..cfg.threads.min(cfg.games) {
			scope.spawn(|| {
				loop {
					let idx = {
						let mut n = next.lock().unwrap();
						if *n >= specs.len() {
							return;
						}
						let i = *n;
						*n += 1;
						i
					};
					let rec = play_game(&cfg, &specs[idx]);
					records.lock().unwrap().push(rec);
					let d = done.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
					if !cfg.quiet && d % 20 == 0 {
						eprintln!("[{d}/{}]", cfg.games);
					}
				}
			});
		}
	});

	let records = records.into_inner().unwrap();
	let n = records.len();
	let mut wins = [0usize; 2];
	let mut wins_as_side = [[0usize; 2]; 2];
	let mut games_as_side = [[0usize; 2]; 2];
	let mut reasons: std::collections::BTreeMap<String, usize> = Default::default();
	for r in &records {
		wins[r.winner_bot] += 1;
		for bot in 0..2 {
			let pos = if r.spec.swapped { 1 - bot } else { bot };
			games_as_side[bot][pos] += 1;
			if r.winner_bot == bot {
				wins_as_side[bot][pos] += 1;
			}
		}
		*reasons.entry(format!("{:?}", r.result.reason)).or_default() += 1;
	}

	let p = wins[0] as f64 / n.max(1) as f64;
	let (lo, hi) = wilson_ci(wins[0] as f64, n as f64);
	println!();
	println!("p1: {}", cfg.p[0]);
	println!("p2: {}", cfg.p[1]);
	println!("games: {n} (seed {}, swap {})", cfg.seed, cfg.swap);
	println!(
		"p1 wins: {} ({:.1}%)  p2 wins: {} ({:.1}%)",
		wins[0],
		100.0 * p,
		wins[1],
		100.0 * wins[1] as f64 / n.max(1) as f64
	);
	println!(
		"p1 winrate CI95: [{:.1}%, {:.1}%]  elo: {:+.0} [{:+.0}, {:+.0}]",
		100.0 * lo,
		100.0 * hi,
		elo_diff(p),
		elo_diff(lo),
		elo_diff(hi)
	);
	println!(
		"p1 as side0: {}/{}  as side1: {}/{}",
		wins_as_side[0][0], games_as_side[0][0], wins_as_side[0][1], games_as_side[0][1]
	);
	let reason_str: Vec<String> = reasons.iter().map(|(k, v)| format!("{k}: {v}")).collect();
	println!("end reasons: {}", reason_str.join(", "));
}
