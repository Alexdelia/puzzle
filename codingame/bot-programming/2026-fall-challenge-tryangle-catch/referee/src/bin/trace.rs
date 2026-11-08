use clap::Parser;
use tryangle::arena::{FrameRecord, run_match};
use tryangle::cli::{BotArgs, MapArgs, die};
use tryangle::game::Game;

/// Play one game and print, after every input turn, the units and triangles each side holds.
#[derive(Parser, Debug)]
#[command(name = "trace", version, about, long_about = None)]
struct Cli {
	#[command(flatten)]
	bots: BotArgs,

	#[command(flatten)]
	map: MapArgs,
}

fn main() {
	let cli = Cli::parse();
	let board = cli.map.build(cli.map.seed).unwrap_or_else(die);
	let mut game = Game::new(board, cli.map.league);
	let mut p1 = cli.bots.driver(0).unwrap_or_else(die);
	let mut p2 = cli.bots.driver(1).unwrap_or_else(die);
	println!("turn units0 units1 triangles0 triangles1 neutral score0 score1 actions0 | actions1");
	let mut sink = |game: &Game, record: &FrameRecord| {
		if !record.asked.contains(&true) {
			return;
		}
		let board = &game.board;
		let [triangles0, triangles1] = [0, 1].map(|p| board.triangles_of(p));
		let neutral = board.triangles.len() - triangles0 - triangles1;
		let actions = record.answers.clone().map(|answer| {
			let line = answer.unwrap_or_default();
			let kinds = ["SPAWN", "ATTACK", "ADD_PATH", "REMOVE_PATH"];
			kinds
				.map(|kind| {
					let count = line.matches(kind).count();
					format!("{kind}={count}")
				})
				.join(" ")
		});
		let [actions0, actions1] = actions;
		let [units0, units1] = [0, 1].map(|p| board.units_of(p));
		let [score0, score1] = game.scores();
		let turn = record.turn;
		println!(
			"{turn} {units0} {units1} {triangles0} {triangles1} {neutral} {score0} {score1} {actions0} | {actions1}"
		);
	};
	let result = run_match(&mut game, [p1.as_mut(), p2.as_mut()], None, Some(&mut sink));
	let [score0, score1] = result.score;
	let note = result.note.unwrap_or_default();
	println!("final {score0} {score1} {note}");
}
