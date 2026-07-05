use super::geom::v2;
use super::race::{Driver, Obs, PodObs, Track};
use std::io::{BufRead, Write};

fn read_line(input: &mut impl BufRead) -> Option<String> {
	let mut line = String::new();
	if input.read_line(&mut line).ok()? == 0 {
		return None;
	}
	Some(line)
}

fn read_ints(input: &mut impl BufRead) -> Option<Vec<i64>> {
	let line = read_line(input)?;
	Some(
		line.split_whitespace()
			.filter_map(|t| t.parse().ok())
			.collect(),
	)
}

pub fn read_track(input: &mut impl BufRead) -> Option<Track> {
	let laps = read_ints(input)?[0] as usize;
	let ncp = read_ints(input)?[0] as usize;
	let mut cps = Vec::with_capacity(ncp);
	for _ in 0..ncp {
		let c = read_ints(input)?;
		cps.push(v2(c[0] as f64, c[1] as f64));
	}
	Some(Track::new(cps, laps))
}

pub fn read_obs(input: &mut impl BufRead, turn: usize) -> Option<Obs> {
	let mut pods = [PodObs {
		x: 0,
		y: 0,
		vx: 0,
		vy: 0,
		angle: 0,
		next_cp: 0,
	}; 4];
	for pod in &mut pods {
		let v = read_ints(input)?;
		if v.len() < 6 {
			return None;
		}
		*pod = PodObs {
			x: v[0],
			y: v[1],
			vx: v[2],
			vy: v[3],
			angle: v[4],
			next_cp: v[5] as usize,
		};
	}
	Some(Obs { turn, pods })
}

pub fn run_cg_bot(mut driver: impl Driver) {
	let stdin = std::io::stdin();
	let mut input = stdin.lock();
	let stdout = std::io::stdout();
	let mut out = stdout.lock();

	let track = read_track(&mut input).expect("init input");
	driver.init(&track);

	let mut turn = 0;
	while let Some(obs) = read_obs(&mut input, turn) {
		let actions = driver.act(&obs).expect("driver action");
		writeln!(out, "{}\n{}", actions[0], actions[1]).unwrap();
		out.flush().unwrap();
		turn += 1;
	}
}
