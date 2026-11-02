use clap::Parser;
use tryangle::arena::{FrameRecord, MatchResult, TimeLimits, run_match};
use tryangle::cglog::CgTrace;
use tryangle::cli::{BotArgs, MapArgs, die};
use tryangle::game::Game;
use tryangle::mapgen::options;

/// Play one TryAngle-Catch game between two bots, with the rules of the official referee.
#[derive(Parser, Debug)]
#[command(name = "referee", version, about, long_about = None)]
struct Cli {
	#[command(flatten)]
	bots: BotArgs,

	#[command(flatten)]
	map: MapArgs,

	/// seat --p1 on side 1 instead of side 0
	#[arg(long)]
	swap: bool,

	/// write the game in the CodinGame text log layout here
	#[arg(long, value_name = "FILE")]
	log: Option<String>,

	/// print the board the seed builds, as CodinGame game options, and exit
	#[arg(long)]
	dump_map: bool,
}

fn result_line(result: &MatchResult, seed: i64) -> String {
	let winner = result
		.winner
		.map_or("draw".to_string(), |side| format!("side{side}"));
	let mut line = format!(
		"seed {seed} winner {winner} score {} {} turns {} frames {} reason {:?}",
		result.score[0], result.score[1], result.turns, result.frames, result.reason
	);
	if let Some(note) = &result.note {
		line.push_str(&format!(" note {note:?}"));
	}
	line
}

fn main() {
	let cli = Cli::parse();
	let board = cli.map.build(cli.map.seed).unwrap_or_else(die);

	if cli.dump_map {
		let map = options(&board);
		println!(
			"seed={}\nnodes={}\nedges={}\nunits={}",
			cli.map.seed, map.nodes, map.edges, map.units
		);
		return;
	}

	let mut game = Game::new(board, cli.map.league);
	let mut d0 = cli.bots.driver(usize::from(cli.swap)).unwrap_or_else(die);
	let mut d1 = cli.bots.driver(usize::from(!cli.swap)).unwrap_or_else(die);
	let limits = cli.bots.enforce_cg_time.then(TimeLimits::default);

	let mut trace = CgTrace::default();
	let result = {
		let mut record = |_: &Game, frame: &FrameRecord| trace.push(frame);
		run_match(
			&mut game,
			[d0.as_mut(), d1.as_mut()],
			limits,
			Some(&mut record),
		)
	};

	if let Some(path) = &cli.log {
		let mut text = format!("options: seed={}\n", cli.map.seed);
		text.push_str(&trace.frames_text());
		text.push_str(&format!(
			"result\n  scores: 0={} 1={}\n",
			result.score[0], result.score[1]
		));
		if let Some(tooltips) = trace.tooltips_line() {
			text.push_str(&format!("  tooltips: {tooltips}\n"));
		}
		std::fs::write(path, text).unwrap_or_else(|e| die(format!("{path}: {e}")));
	}
	for (side, driver) in [(0, &mut d0), (1, &mut d1)] {
		let errors = driver.stderr();
		if !errors.is_empty() {
			eprintln!("--- side{side} stderr ---\n{errors}");
		}
	}
	println!("{}", result_line(&result, cli.map.seed));
}
