use mpr::geom::{V2, v2};
use mpr::maps::{Rng, agade_map, random_map};
use mpr::sim::{Cmd, CmdKind, RAD_TO_DEG, State, expand_cps, init_state, step};
use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

struct Config {
	games: usize,
	seed: u64,
	turns: usize,
	referee: String,
	verbose: bool,
}

fn parse_args() -> Config {
	let mut cfg = Config {
        games: 200,
        seed: 1,
        turns: 150,
        referee: "/tmp/claude-1000/-home-alex-p-puzzle-codingame-bot-mad-pod-racing/4517d08f-1912-4737-9acb-dcaecf1d55d3/scratchpad/csbref".to_string(),
        verbose: false,
    };
	let mut args = std::env::args().skip(1);
	while let Some(a) = args.next() {
		match a.as_str() {
			"--games" => cfg.games = args.next().unwrap().parse().unwrap(),
			"--seed" => cfg.seed = args.next().unwrap().parse().unwrap(),
			"--turns" => cfg.turns = args.next().unwrap().parse().unwrap(),
			"--ref" => cfg.referee = args.next().unwrap(),
			"--verbose" => cfg.verbose = true,
			other => panic!("unknown arg: {other}"),
		}
	}
	cfg
}

fn cmd_to_line(cmd: &Cmd) -> String {
	let t = match cmd.kind {
		CmdKind::Thrust(v) => format!("{}", v as i64),
		CmdKind::Boost => "BOOST".to_string(),
		CmdKind::Shield => "SHIELD".to_string(),
	};
	format!("{} {} {}", cmd.target.x as i64, cmd.target.y as i64, t)
}

fn random_cmd(rng: &mut Rng, state: &State, cps: &[V2], pod: usize) -> Cmd {
	let me = &state.pods[pod];
	let target = match rng.below(100) {
		0..=54 => {
			let cp = cps[me.next];
			v2(
				(cp.x as i64 + rng.range(-1200, 1200)) as f64,
				(cp.y as i64 + rng.range(-1200, 1200)) as f64,
			)
		}
		55..=69 => {
			let enemy = &state.pods[(pod + 2) % 4];
			v2(
				((enemy.pos.x + enemy.vel.x) as i64 + rng.range(-300, 300)) as f64,
				((enemy.pos.y + enemy.vel.y) as i64 + rng.range(-300, 300)) as f64,
			)
		}
		70..=74 => v2(me.pos.x, me.pos.y),
		_ => v2(
			rng.range(-2000, 18000) as f64,
			rng.range(-2000, 11000) as f64,
		),
	};
	let kind = match rng.below(100) {
		0..=59 => CmdKind::Thrust([0, 30, 70, 100, 150, 200][rng.below(6) as usize] as f64),
		60..=74 => CmdKind::Thrust(100.0),
		75..=82 => CmdKind::Shield,
		83..=89 => CmdKind::Boost,
		_ => CmdKind::Thrust(0.0),
	};
	Cmd { target, kind }
}

struct PodLine {
	x: i64,
	y: i64,
	vx: i64,
	vy: i64,
	angle_deg: f64,
	next: usize,
	shield: u8,
	boosted: bool,
}

fn my_pod_line(state: &State, pod: usize) -> PodLine {
	let p = &state.pods[pod];
	PodLine {
		x: p.pos.x as i64,
		y: p.pos.y as i64,
		vx: p.vel.x as i64,
		vy: p.vel.y as i64,
		angle_deg: p.angle * RAD_TO_DEG,
		next: p.next,
		shield: p.shield,
		boosted: p.boosted,
	}
}

fn parse_go_line(line: &str) -> PodLine {
	let t: Vec<&str> = line.split_whitespace().collect();
	PodLine {
		x: t[0].parse().unwrap(),
		y: t[1].parse().unwrap(),
		vx: t[2].parse().unwrap(),
		vy: t[3].parse().unwrap(),
		angle_deg: t[4].parse().unwrap(),
		next: t[5].parse().unwrap(),
		shield: t[6].parse().unwrap(),
		boosted: t[7] != "0",
	}
}

fn run_game(cfg: &Config, game: usize) -> (usize, usize, u32) {
	let mut rng = Rng::new(
		cfg.seed
			.wrapping_add(game as u64)
			.wrapping_mul(0x9E3779B97F4A7C15)
			| 1,
	);
	let raw = if game % 2 == 0 {
		agade_map(&mut rng)
	} else {
		random_map(&mut rng)
	};
	let cps = expand_cps(&raw, 3);
	let mut state = init_state(&cps);

	let mut input = String::new();
	writeln!(input, "{}", cps.len()).unwrap();
	for c in &cps {
		writeln!(input, "{} {}", c.x as i64, c.y as i64).unwrap();
	}
	writeln!(input, "{}", cfg.turns).unwrap();

	let mut my_lines: Vec<PodLine> = Vec::with_capacity(cfg.turns * 4);
	let mut all_cmds: Vec<String> = Vec::with_capacity(cfg.turns * 4);
	let mut bounces = 0u32;

	for _ in 0..cfg.turns {
		for i in 0..4 {
			writeln!(input, "ignored {i}").unwrap();
		}
		let cmds = [
			random_cmd(&mut rng, &state, &cps, 0),
			random_cmd(&mut rng, &state, &cps, 1),
			random_cmd(&mut rng, &state, &cps, 2),
			random_cmd(&mut rng, &state, &cps, 3),
		];
		for c in &cmds {
			let line = cmd_to_line(c);
			writeln!(input, "{line}").unwrap();
			all_cmds.push(line);
		}
		let events = step(&cps, &mut state, &cmds);
		bounces += events.bounces;
		for i in 0..4 {
			my_lines.push(my_pod_line(&state, i));
		}
	}

	let mut child = Command::new(&cfg.referee)
		.arg("-test")
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::null())
		.spawn()
		.expect("spawn go referee");
	let mut stdin = child.stdin.take().unwrap();
	let writer = std::thread::spawn(move || {
		stdin.write_all(input.as_bytes()).ok();
	});
	let stdout = BufReader::new(child.stdout.take().unwrap());
	let go_lines: Vec<PodLine> = stdout.lines().map(|l| parse_go_line(&l.unwrap())).collect();
	writer.join().unwrap();
	child.wait().unwrap();

	assert_eq!(
		go_lines.len(),
		my_lines.len(),
		"game {game}: go produced {} lines, expected {}",
		go_lines.len(),
		my_lines.len()
	);

	let mut mismatches = 0;
	for (idx, (mine, go)) in my_lines.iter().zip(go_lines.iter()).enumerate() {
		let turn = idx / 4;
		let pod = idx % 4;
		let mut bad: Vec<String> = Vec::new();
		if mine.x != go.x || mine.y != go.y {
			bad.push(format!("pos {},{} vs {},{}", mine.x, mine.y, go.x, go.y));
		}
		if mine.vx != go.vx || mine.vy != go.vy {
			bad.push(format!(
				"vel {},{} vs {},{}",
				mine.vx, mine.vy, go.vx, go.vy
			));
		}
		if (mine.angle_deg - go.angle_deg).abs() > 1e-4 {
			bad.push(format!("angle {} vs {}", mine.angle_deg, go.angle_deg));
		}
		if mine.next != go.next {
			bad.push(format!("next {} vs {}", mine.next, go.next));
		}
		if mine.shield != go.shield {
			bad.push(format!("shield {} vs {}", mine.shield, go.shield));
		}
		if mine.boosted != go.boosted {
			bad.push(format!("boosted {} vs {}", mine.boosted, go.boosted));
		}
		if !bad.is_empty() {
			mismatches += 1;
			if mismatches <= 5 || cfg.verbose {
				eprintln!(
					"game {game} turn {turn} pod {pod}: {} | cmd: {}",
					bad.join("; "),
					all_cmds[turn * 4 + pod]
				);
			}
		}
	}

	(my_lines.len(), mismatches, bounces)
}

fn main() {
	let cfg = parse_args();
	let mut total = 0usize;
	let mut bad = 0usize;
	let mut bounces = 0u32;
	for game in 0..cfg.games {
		let (states, mismatches, game_bounces) = run_game(&cfg, game);
		total += states;
		bad += mismatches;
		bounces += game_bounces;
		if game % 50 == 49 {
			eprintln!(
				"[{}/{}] states={total} mismatches={bad} bounces={bounces}",
				game + 1,
				cfg.games
			);
		}
	}
	println!(
		"checked {} pod-states over {} games ({} bounces): {} mismatches",
		total, cfg.games, bounces, bad
	);
	if bad > 0 {
		std::process::exit(1);
	}
}
