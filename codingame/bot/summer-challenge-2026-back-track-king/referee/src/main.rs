use btk::arena::{EndReason, MatchResult, TimeLimits, run_match};
use btk::cli::{BotArgs, MapArgs, RuleArgs, die};
use btk::game::Game;
use btk::trace::{Trace, render, replay_text};
use clap::Parser;

/// Play one Back Track King game and report the outcome.
#[derive(Parser, Debug)]
#[command(name = "referee", version, about, long_about = None)]
struct Cli {
	#[command(flatten)]
	bots: BotArgs,

	#[command(flatten)]
	map: MapArgs,

	#[command(flatten)]
	rules: RuleArgs,

	/// seat --p1 on side 1 instead of side 0, to replay the mirror of a ladder game
	#[arg(long)]
	swap: bool,

	/// print the board after every turn
	#[arg(long)]
	board: bool,

	/// write the full replay trace here
	#[arg(long, value_name = "FILE")]
	replay: Option<String>,

	/// print the map and exit without playing
	#[arg(long)]
	dump_map: bool,

	/// print only the result line
	#[arg(long)]
	quiet: bool,
}

fn main() {
	let cli = Cli::parse();
	if cli.dump_map {
		print!("{}", cli.map.dump().unwrap_or_else(die));
		return;
	}

	let map = cli.map.build(cli.map.seed).unwrap_or_else(die);
	let mut game = Game::new(map, cli.rules.rules());
	let mut d0 = cli.bots.driver(usize::from(cli.swap)).unwrap_or_else(die);
	let mut d1 = cli.bots.driver(usize::from(!cli.swap)).unwrap_or_else(die);

	let want_trace = cli.replay.is_some() || cli.board || !cli.quiet;
	let mut trace = Trace::with_board(cli.board);
	let result = if want_trace {
		let mut record = |game: &Game, report: &_| trace.record(game, report);
		run_match(
			&mut game,
			[d0.as_mut(), d1.as_mut()],
			limits(&cli),
			Some(&mut record),
		)
	} else {
		run_match(&mut game, [d0.as_mut(), d1.as_mut()], limits(&cli), None)
	};

	let verdict = describe(&cli, &result);
	if let Some(path) = &cli.replay {
		let header = [
			("seed", cli.map.seed.to_string()),
			("swapped", cli.swap.to_string()),
			("p1", cli.bots.spec(0).to_string()),
			("p2", cli.bots.spec(1).to_string()),
			("rules", format!("{:?}", cli.rules.rules())),
		];
		let text = replay_text(&header, &game, &trace, verdict.clone());
		std::fs::write(path, text).unwrap_or_else(|e| die(format!("{path}: {e}")));
	}
	if !cli.quiet {
		print!("{}", trace.text);
		if !cli.board {
			print!("{}", render(&game));
		}
		for (side, driver) in [(0, &mut d0), (1, &mut d1)] {
			let errors = driver.stderr();
			if !errors.is_empty() {
				eprintln!("--- side{side} stderr ---\n{errors}");
			}
		}
	}
	println!("{verdict}");
	if result.reason != EndReason::TurnLimit {
		std::process::exit(1);
	}
}

fn limits(cli: &Cli) -> Option<TimeLimits> {
	cli.bots.enforce_cg_time.then(TimeLimits::default)
}

fn bot_of(cli: &Cli, side: Option<usize>) -> Option<usize> {
	side.map(|side| if cli.swap { 1 - side } else { side })
}

fn describe(cli: &Cli, result: &MatchResult) -> String {
	let (a, b) = if cli.swap {
		(result.score[1], result.score[0])
	} else {
		(result.score[0], result.score[1])
	};
	let outcome = match bot_of(cli, result.winner) {
		Some(bot) => format!("p{} wins", bot + 1),
		None => "draw".into(),
	};
	let mut line = format!(
		"seed {} swapped {} score p1 {a} p2 {b} turns {} {outcome} ({:?})",
		cli.map.seed, cli.swap, result.turns, result.reason
	);
	if result.reason != EndReason::TurnLimit
		&& let (Some(side), Some(note)) = (result.culprit, &result.note)
	{
		let bot = if cli.swap { 1 - side } else { side };
		line.push_str(&format!(": p{} {note}", bot + 1));
	}
	line
}
