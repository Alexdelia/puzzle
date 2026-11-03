use clap::Parser;
use tryangle::arena::{FrameRecord, run_match};
use tryangle::cglog::CgLog;
use tryangle::cli::die;
use tryangle::driver::Driver;
use tryangle::game::Game;
use tryangle::mapgen;

const ACTION_KINDS: [&str; 4] = ["SPAWN", "ATTACK", "ADD_PATH", "REMOVE_PATH"];

/// Replay CodinGame logs from both sides' recorded answers, and print per game the units, triangles and actions of the player seen as "me".
#[derive(Parser, Debug)]
#[command(name = "replay", version, about, long_about = None)]
struct Cli {
	/// CodinGame text logs (`GET /games/<id>`)
	logs: Vec<String>,

	/// nickname whose side is reported first
	#[arg(long)]
	me: String,

	/// league level of the official referee
	#[arg(long, default_value_t = 3)]
	league: i32,

	/// print a line every this many turns, 0 for only the game line
	#[arg(long, default_value_t = 0)]
	every: i32,
}

struct Scripted {
	answers: std::vec::IntoIter<String>,
}

impl Driver for Scripted {
	fn send(&mut self, _: &str) -> Result<(), String> {
		Ok(())
	}

	fn answer(&mut self) -> Result<String, String> {
		self.answers
			.next()
			.ok_or_else(|| "the recorded game has no more answers".into())
	}
}

fn agent_names(text: &str) -> Vec<String> {
	text.lines()
		.skip_while(|line| *line != "agents:")
		.skip(1)
		.take_while(|line| line.starts_with("  "))
		.filter_map(|line| line.split_whitespace().nth(1).map(str::to_string))
		.collect()
}

fn main() {
	let cli = Cli::parse();
	println!(
		"game result opponent turns score_me score_opp end | units_me units_opp tri_me tri_opp | spawn attack add remove (me) | (opp)"
	);
	for path in &cli.logs {
		let text = std::fs::read_to_string(path).unwrap_or_else(die);
		let log = CgLog::parse(&text).unwrap_or_else(die);
		let names = agent_names(&text);
		let Some(me) = names.iter().position(|n| *n == cli.me) else {
			eprintln!("{path}: no agent named {}", cli.me);
			continue;
		};
		let opponent = 1 - me;
		let field = |key: &str| {
			log.option(key)
				.unwrap_or_else(|| die(format!("{path}: no {key}")))
		};
		let board = mapgen::from_options(field("nodes"), field("edges"), field("units"))
			.unwrap_or_else(die);
		let mut game = Game::new(board, cli.league);
		let mut sides = [0, 1].map(|player| Scripted {
			answers: log.answers(player).into_iter(),
		});
		let mut counts = [[0usize; 4]; 2];
		let mut peak = [0i32; 2];
		let mut last = (0, [0; 2], [0; 2]);
		let every = cli.every;
		let mut sink = |game: &Game, record: &FrameRecord| {
			if !record.asked.contains(&true) {
				return;
			}
			for player in 0..2 {
				let line = record.answers[player].clone().unwrap_or_default();
				for (k, kind) in ACTION_KINDS.iter().enumerate() {
					counts[player][k] += line.matches(&format!("{kind} ")).count();
				}
			}
			let board = &game.board;
			let units = [0, 1].map(|p| board.units_of(p) as i32);
			let triangles = [0, 1].map(|p| board.triangles_of(p) as i32);
			for p in 0..2 {
				peak[p] = peak[p].max(units[p]);
			}
			last = (record.turn, units, triangles);
			if every > 0 && record.turn % every == 0 {
				let [s0, s1] = game.scores();
				let score = [s0, s1];
				println!(
					"  t{} units {}-{} tri {}-{} score {}-{}",
					record.turn,
					units[me],
					units[opponent],
					triangles[me],
					triangles[opponent],
					score[me],
					score[opponent]
				);
			}
		};
		let result = {
			let [a, b] = sides.each_mut();
			run_match(&mut game, [a, b], None, Some(&mut sink))
		};
		let outcome = match result.winner {
			Some(w) if w == me => "WIN",
			Some(_) => "LOSS",
			None => "DRAW",
		};
		let reason = match result.culprit {
			Some(_) => result.note.clone().unwrap_or_default().replace(' ', "_"),
			None => "turns".to_string(),
		};
		let (_, units, triangles) = last;
		let show = |c: [usize; 4]| c.map(|n| n.to_string()).join(" ");
		println!(
			"{} {outcome} {} {} {} {} {reason} | {} {} {} {} | {} | {} | peak {}-{}",
			log.game_id,
			names[opponent],
			result.turns,
			result.score[me],
			result.score[opponent],
			units[me],
			units[opponent],
			triangles[me],
			triangles[opponent],
			show(counts[me]),
			show(counts[opponent]),
			peak[me],
			peak[opponent]
		);
	}
}
