use btk::arena::{EndReason, TimeLimits, run_match};
use btk::cli::{BotArgs, MapArgs, die};
use btk::game::Game;
use btk::grid::describe;
use btk::trace::{Detail, Trace};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "referee", version, about, long_about = None)]
struct Cli {
	#[command(flatten)]
	bots: BotArgs,

	#[command(flatten)]
	map: MapArgs,

	/// how much of the game to print
	#[arg(long, value_enum, default_value_t = Detail::Digest)]
	trace: Detail,

	/// seat --p1 on side 1 instead of side 0, to replay the mirror of a ladder game
	#[arg(long)]
	swap: bool,

	/// write the trace here instead of to stdout
	#[arg(long, value_name = "FILE")]
	replay: Option<String>,

	/// print the board the seed builds and exit
	#[arg(long)]
	dump_map: bool,
}

fn main() {
	let cli = Cli::parse();
	let grid = cli.map.build(cli.map.seed).unwrap_or_else(die);

	if cli.dump_map {
		print!("seed {}\n{}", cli.map.seed, describe(&grid));
		return;
	}

	let mut game = Game::new(grid, cli.map.league);
	let mut d0 = cli.bots.driver(usize::from(cli.swap)).unwrap_or_else(die);
	let mut d1 = cli.bots.driver(usize::from(!cli.swap)).unwrap_or_else(die);

	let mut trace = Trace::new(cli.trace);
	trace.header(cli.map.seed, &game);
	let limits = cli.bots.enforce_cg_time.then(TimeLimits::default);
	let result = {
		let mut record =
			|game: &Game, report: &_, answers: &_, turn| trace.turn(game, report, answers, turn);
		run_match(
			&mut game,
			[d0.as_mut(), d1.as_mut()],
			limits,
			Some(&mut record),
		)
	};
	trace.footer(&game, &result);

	match &cli.replay {
		Some(path) => {
			std::fs::write(path, &trace.text).unwrap_or_else(|e| die(format!("{path}: {e}")))
		}
		None => print!("{}", trace.text),
	}
	for (side, driver) in [(0, &mut d0), (1, &mut d1)] {
		let errors = driver.stderr();
		if !errors.is_empty() {
			eprintln!("--- side{side} stderr ---\n{errors}");
		}
	}
	println!("{}", btk::trace::result_line(&result, cli.map.seed));
	if result.reason != EndReason::GameOver {
		std::process::exit(1);
	}
}
