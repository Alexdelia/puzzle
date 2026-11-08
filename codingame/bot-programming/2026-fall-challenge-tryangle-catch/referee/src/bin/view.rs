use clap::Parser;
use tryangle::arena::{FrameRecord, run_match};
use tryangle::board::{Board, HEIGHT, SIDE_BORDER, TOP_BORDER, WIDTH};
use tryangle::cli::{BotArgs, MapArgs, die};
use tryangle::game::Game;

const COLUMNS: i32 = 150;
const ROWS: i32 = 40;

/// Play one game and draw the board after every input turn in a range: houses as `id:mine/theirs`, owned triangles, both answers.
#[derive(Parser, Debug)]
#[command(name = "view", version, about, long_about = None)]
struct Cli {
	#[command(flatten)]
	bots: BotArgs,

	#[command(flatten)]
	map: MapArgs,

	/// first input turn to draw
	#[arg(long, default_value_t = 1)]
	from: i32,

	/// last input turn to draw
	#[arg(long, default_value_t = 20)]
	to: i32,

	/// list occupied houses and every triangle with its owner and capture rights instead of drawing
	#[arg(long)]
	compact: bool,
}

fn draw(board: &Board) -> String {
	let mut grid = vec![vec![' '; COLUMNS as usize]; ROWS as usize];
	for (id, point) in board.points.iter().enumerate() {
		let column = (point.x - SIDE_BORDER) * (COLUMNS - 12) / WIDTH;
		let row = (point.y - TOP_BORDER) * (ROWS - 1) / HEIGHT;
		let [a, b] = board.units[id];
		let label = match (a, b) {
			(0, 0) => format!("{id}."),
			(a, 0) => format!("{id}:A{a}"),
			(0, b) => format!("{id}:b{b}"),
			(a, b) => format!("{id}:A{a}b{b}"),
		};
		for (offset, ch) in label.chars().enumerate() {
			let x = column as usize + offset;
			if x < COLUMNS as usize {
				grid[row as usize][x] = ch;
			}
		}
	}
	grid.into_iter()
		.map(|line| line.into_iter().collect::<String>().trim_end().to_string())
		.collect::<Vec<_>>()
		.join("\n")
}

fn listing(board: &Board) -> String {
	let houses: Vec<String> = (0..board.len())
		.filter(|&n| board.units[n] != [0, 0])
		.map(|n| {
			let [a, b] = board.units[n];
			format!("{n}:{a}/{b}")
		})
		.collect();
	let triangles: Vec<String> = board
		.triangles
		.iter()
		.map(|&t| {
			let triangle = board.triangle(t);
			let [a, b, c] = triangle.nodes;
			let owner = match triangle.owner {
				Some(0) => "A",
				Some(_) => "b",
				None => "-",
			};
			let rights: String = [0, 1]
				.map(|p| if triangle.can_capture[p] { "+" } else { "x" })
				.concat();
			format!("{a},{b},{c}{owner}{rights}")
		})
		.collect();
	let houses = houses.join(" ");
	let triangles = triangles.join(" ");
	format!("houses {houses}\ntriangles {triangles}")
}

fn main() {
	let cli = Cli::parse();
	let board = cli.map.build(cli.map.seed).unwrap_or_else(die);
	let mut game = Game::new(board, cli.map.league);
	let mut p1 = cli.bots.driver(0).unwrap_or_else(die);
	let mut p2 = cli.bots.driver(1).unwrap_or_else(die);
	let mut asked = 0;
	let mut seen = game.board.clone();
	let mut sink = |game: &Game, record: &FrameRecord| {
		let before = std::mem::replace(&mut seen, game.board.clone());
		if !record.asked.contains(&true) {
			return;
		}
		asked += 1;
		if asked < cli.from || asked > cli.to {
			return;
		}
		let board = &before;
		let owned: Vec<String> = board
			.triangles
			.iter()
			.filter_map(|&t| {
				let triangle = board.triangle(t);
				let [a, b, c] = triangle.nodes;
				let tag = match triangle.owner {
					Some(0) => "A",
					Some(_) => "b",
					None => return None,
				};
				Some(format!("{tag}({a},{b},{c})"))
			})
			.collect();
		let [answer0, answer1] = record.answers.clone().map(|a| a.unwrap_or_default());
		let [score0, score1] = game.scores();
		let [units0, units1] = [0, 1].map(|p| board.units_of(p));
		let picture = if cli.compact {
			listing(board)
		} else {
			draw(board)
		};
		let owned = owned.join(" ");
		println!("==== turn {asked} units {units0}/{units1} score {score0}/{score1}");
		println!("A> {answer0}");
		println!("b> {answer1}");
		println!("{picture}");
		println!("owned {owned}");
		for line in &record.summary {
			println!("  {line}");
		}
	};
	if cli.compact {
		let edges: Vec<String> = game
			.board
			.edges()
			.iter()
			.map(|(a, b)| format!("{a}-{b}"))
			.collect();
		let edges = edges.join(" ");
		println!("edges {edges}");
	}
	run_match(&mut game, [p1.as_mut(), p2.as_mut()], None, Some(&mut sink));
	let [score0, score1] = game.scores();
	println!("final {score0} {score1}");
}
