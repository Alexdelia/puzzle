mod building;
mod detail;
mod economy;
mod mimic;
mod overview;
mod paths;
mod play;
mod view;

use btk::cli::die;
use btk::driver::make_driver;
use btk::grid::Grid;
use btk::gridmaker;
use btk::replay::Probe;
use btk::replay::{self, Log, Replay};
use clap::Parser;
use std::time::Duration;
use view::View;

#[derive(Parser, Debug)]
#[command(
	name = "study",
	about = "Read a recorded CodinGame game log and explain it"
)]
struct Cli {
	/// log files, named after the seed that built the map
	#[arg(value_name = "LOG")]
	logs: Vec<String>,

	/// play a fresh game between --p1 and --p2 on --seed and study that instead of a log
	#[arg(long)]
	play: bool,

	/// side 0 bot for --play
	#[arg(long, value_name = "SPEC")]
	p1: Option<String>,

	/// side 1 bot for --play
	#[arg(long, value_name = "SPEC")]
	p2: Option<String>,

	/// seed for the first log, when its name is not the seed
	#[arg(long, allow_negative_numbers = true)]
	seed: Option<i64>,

	/// which side to speak of as "us"; default is the myId the log starts with
	#[arg(long, value_name = "0|1")]
	me: Option<usize>,

	/// force the pairing of the two logged answer streams instead of detecting it
	#[arg(long)]
	swapped: Option<bool>,

	/// print every turn, not just the summary
	#[arg(long)]
	turns: bool,

	/// print the board every N turns (0 for never)
	#[arg(long, value_name = "N", default_value_t = 0)]
	boards: usize,

	/// print the board and the full state of these turns
	#[arg(long, value_name = "N", num_args = 1..)]
	turn: Vec<i32>,

	/// print the region ids of the map
	#[arg(long)]
	regions: bool,

	/// how many rows the ranked tables keep
	#[arg(long, value_name = "N", default_value_t = 12)]
	top: usize,

	/// draw the path of these pairs, as "A-C", at every turn given by --turn
	#[arg(long, value_name = "A-C", num_args = 1..)]
	pair: Vec<String>,

	/// report a pair whose ownership share moves by at least this many points in one turn
	#[arg(long, value_name = "PCT", default_value_t = 25.0)]
	flip: f64,

	/// ask this bot the frames one logged player saw, and score how often it answers the same
	#[arg(long, value_name = "SPEC")]
	against: Option<String>,

	/// which logged player --against imitates; default is the foe
	#[arg(long, value_name = "0|1")]
	against_side: Option<usize>,

	/// how long the --against bot may take per turn
	#[arg(long, value_name = "MS", default_value_t = 10000)]
	against_timeout: u64,

	/// how many disagreeing turns --against prints
	#[arg(long, value_name = "N", default_value_t = 15)]
	against_show: usize,
}

fn main() {
	let cli = Cli::parse();
	if cli.play {
		played(&cli);
		return;
	}
	if cli.logs.is_empty() {
		die::<()>("give a log file, or --play with --p1 and --p2");
	}
	for (index, path) in cli.logs.iter().enumerate() {
		if index > 0 {
			println!();
		}
		study(&cli, path, if index == 0 { cli.seed } else { None });
	}
}

fn study(cli: &Cli, path: &str, seed: Option<i64>) {
	let log = load(path, seed).unwrap_or_else(die);
	let grid = gridmaker::make(log.seed);
	if let Err(why) = log.check_map(&grid) {
		die::<()>(why);
	}

	let (swapped, run) = match cli.swapped {
		Some(swapped) => (swapped, replay::run(grid.clone(), &log.answers(swapped))),
		None => detect_sides(&grid, &log),
	};
	let me = cli.me.unwrap_or(log.my_id);
	let view = View::new(&run, me);
	report(cli, &view, &log, &grid, swapped);

	if let Some(spec) = &cli.against {
		let side = cli.against_side.unwrap_or(1 - me);
		let answers = log.answers(swapped);
		let mut driver =
			make_driver(spec, Duration::from_millis(cli.against_timeout)).unwrap_or_else(die);
		let mut probe = Probe::new(side, driver.as_mut());
		let probed = replay::run_probed(grid, &answers, Some(&mut probe));
		let view = View::new(&probed, me);
		if let Some(why) = &probe.broke {
			println!(
				"\n(the candidate stopped after {} turns: {why})",
				probe.answers.len()
			);
		}
		mimic::agreement(&view, &probe.answers, side, cli.against_show);
	}
}

fn played(cli: &Cli) {
	let seed = cli.seed.unwrap_or(1);
	let grid = gridmaker::make(seed);
	let specs = [
		cli.p1
			.as_deref()
			.unwrap_or_else(|| die("--play needs --p1")),
		cli.p2
			.as_deref()
			.unwrap_or_else(|| die("--play needs --p2")),
	];
	let (answers, result) = play::play(&grid, specs, cli.against_timeout).unwrap_or_else(die);
	let run = replay::run(grid, &answers);
	let view = View::new(&run, cli.me.unwrap_or(0));
	println!("== game ==");
	println!("seed          {seed}");
	println!("p0            {}", specs[0]);
	println!("p1            {}", specs[1]);
	println!("{}", btk::trace::result_line(&result, seed));
	sections(cli, &view);
}

fn load(path: &str, seed: Option<i64>) -> Result<Log, String> {
	match seed {
		Some(seed) => {
			let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
			replay::parse(path, seed, &text)
		}
		None => replay::read(path),
	}
}

fn detect_sides(grid: &Grid, log: &Log) -> (bool, Replay) {
	let straight = replay::run(grid.clone(), &log.answers(false));
	let Some(want) = log.final_score else {
		return (false, straight);
	};
	if straight.score == want {
		return (false, straight);
	}
	let crossed = replay::run(grid.clone(), &log.answers(true));
	if crossed.score == want {
		return (true, crossed);
	}
	(false, straight)
}

fn report(cli: &Cli, view: &View, log: &Log, grid: &Grid, swapped: bool) {
	overview::header(view, log, swapped);
	overview::map(grid, cli.regions);
	sections(cli, view);
}

fn sections(cli: &Cli, view: &View) {
	if cli.turns {
		overview::timeline(view);
	}
	economy::paint(view);
	economy::disruption(view);
	economy::ink_effect(view);
	paths::connections(view);
	paths::paid_cells(view, cli.top);
	building::gateways(view);
	building::placements(view);
	building::network(view);
	paths::openings(view);
	paths::takeovers(view, cli.flip);
	economy::income(view);
	for &turn in &cli.turn {
		detail::turn(view, turn);
		for wanted in &cli.pair {
			paths::one_pair(view, turn, wanted);
		}
	}
	if cli.boards > 0 {
		detail::boards(view, cli.boards);
	}
}
