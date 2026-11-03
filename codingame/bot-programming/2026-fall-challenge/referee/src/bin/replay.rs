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

	/// print the units on every house, the triangles and both answers for frames in this range, e.g. 15..22
	#[arg(long, value_name = "FIRST..LAST")]
	dump: Option<String>,

	/// write each player's stdin (`<game>-<nick>.in`) and recorded answers (`<game>-<nick>.out`) to this directory
	#[arg(long, value_name = "DIR")]
	streams: Option<String>,
}

struct Scripted {
	answers: std::vec::IntoIter<String>,
	received: String,
}

impl Driver for Scripted {
	fn send(&mut self, text: &str) -> Result<(), String> {
		self.received.push_str(text);
		if !text.ends_with('\n') {
			self.received.push('\n');
		}
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
		let home = home_sides(&board, me);
		let mut game = Game::new(board, cli.league);
		let mut sides = [0, 1].map(|player| Scripted {
			answers: log.answers(player).into_iter(),
			received: String::new(),
		});
		let mut counts = [[0usize; 4]; 2];
		let mut peak = [0i32; 2];
		let mut last = (0, [0; 2], [0; 2]);
		let every = cli.every;
		let dumped = cli.dump.as_deref().map(|range| {
			let (first, last) = range
				.split_once("..")
				.unwrap_or_else(|| die("--dump takes FIRST..LAST"));
			let bound = |text: &str| text.parse::<i32>().unwrap_or_else(die);
			bound(first)..=bound(last)
		});
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
			if dumped
				.as_ref()
				.is_some_and(|range| range.contains(&record.turn))
			{
				let turn = record.turn;
				let [answer_me, answer_opp] =
					[me, opponent].map(|p| record.answers[p].clone().unwrap_or_default());
				println!("  frame {turn}");
				println!("    me:  {answer_me}");
				println!("    opp: {answer_opp}");
				let houses: Vec<String> = (0..board.units.len())
					.filter(|&n| board.units[n] != [0, 0])
					.map(|n| {
						let [mine, theirs] = [board.units[n][me], board.units[n][opponent]];
						format!("{n}:{mine}/{theirs}")
					})
					.collect();
				let houses = houses.join(" ");
				println!("    houses me/opp: {houses}");
				for (label, player) in [
					("mine", Some(me)),
					("theirs", Some(opponent)),
					("neutral", None),
				] {
					let listed: Vec<String> = board
						.triangles
						.iter()
						.map(|&t| &board.arena[t])
						.filter(|t| t.owner == player)
						.map(|t| {
							let [a, b, c] = t.nodes;
							let flags = [t.can_capture[me], t.can_capture[opponent]]
								.map(|f| if f { 'c' } else { '-' });
							let [flag_me, flag_opp] = flags;
							format!("{a}-{b}-{c}{flag_me}{flag_opp}")
						})
						.collect();
					let listed = listed.join(" ");
					println!("    {label}: {listed}");
				}
			}
			if every > 0 && record.turn % every == 0 {
				let [s0, s1] = game.scores();
				let score = [s0, s1];
				let locked = [me, opponent].map(|p| {
					board
						.triangles
						.iter()
						.filter(|&&t| {
							let triangle = &board.arena[t];
							triangle.owner.is_none() && !triangle.can_capture[p]
						})
						.count()
				});
				let neutral = board
					.triangles
					.iter()
					.filter(|&&t| board.arena[t].owner.is_none())
					.count();
				let turn = record.turn;
				let [units_me, units_opp] = [units[me], units[opponent]];
				let [tri_me, tri_opp] = [triangles[me], triangles[opponent]];
				let [score_me, score_opp] = [score[me], score[opponent]];
				let [locked_me, locked_opp] = locked;
				let idle = [me, opponent].map(|p| {
					(0..board.units.len())
						.filter(|&n| {
							board.triangles.iter().all(|&t| {
								let triangle = &board.arena[t];
								!triangle.nodes.contains(&n)
									|| triangle.owner == Some(1 - p)
									|| (triangle.owner.is_none() && !triangle.can_capture[p])
							})
						})
						.map(|n| board.units[n][p])
						.sum::<i32>()
				});
				let stacked = [me, opponent]
					.map(|p| board.units.iter().map(|u| (u[p] - 1).max(0)).sum::<i32>());
				let [idle_me, idle_opp] = idle;
				let held = |player: usize, mine_half: bool| {
					board
						.triangles
						.iter()
						.filter(|&&t| {
							let triangle = &board.arena[t];
							let on_my_half =
								triangle.nodes.iter().filter(|&&n| home[n]).count() >= 2;
							triangle.owner == Some(player) && on_my_half == mine_half
						})
						.count()
				};
				let at_home = |player: usize, own_half: bool| -> i32 {
					(0..board.units.len())
						.filter(|&n| home[n] == own_half)
						.map(|n| board.units[n][player])
						.sum()
				};
				let [units_home_me, units_home_opp] = [at_home(me, true), at_home(opponent, false)];
				let [mine_home, mine_away, theirs_home, theirs_away] = [
					held(me, true),
					held(me, false),
					held(opponent, false),
					held(opponent, true),
				];
				let [stacked_me, stacked_opp] = stacked;
				println!(
					"  t{turn} units {units_me}-{units_opp} tri {tri_me}-{tri_opp} score {score_me}-{score_opp} neutral {neutral} locked {locked_me}-{locked_opp} idle {idle_me}-{idle_opp} stacked {stacked_me}-{stacked_opp} home {mine_home}-{theirs_home} away {mine_away}-{theirs_away} athome {units_home_me}-{units_home_opp}"
				);
			}
		};
		let result = {
			let [a, b] = sides.each_mut();
			run_match(&mut game, [a, b], None, Some(&mut sink))
		};
		if let Some(dir) = &cli.streams {
			for (player, side) in sides.iter().enumerate() {
				let stem = format!("{dir}/{}-{}", log.game_id, names[player]);
				let answers = log.answers(player).join("\n") + "\n";
				std::fs::write(format!("{stem}.in"), &side.received).unwrap_or_else(die);
				std::fs::write(format!("{stem}.out"), answers).unwrap_or_else(die);
			}
		}
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

fn home_sides(board: &tryangle::board::Board, me: usize) -> Vec<bool> {
	let count = board.units.len();
	let distances = |player: usize| {
		let mut distance = vec![usize::MAX; count];
		let mut frontier: Vec<usize> = (0..count).filter(|&n| board.units[n][player] > 0).collect();
		for &n in &frontier {
			distance[n] = 0;
		}
		let mut depth = 0;
		while !frontier.is_empty() {
			depth += 1;
			let mut next = Vec::new();
			for &n in &frontier {
				for &m in &board.neighbors[n] {
					if distance[m] == usize::MAX {
						distance[m] = depth;
						next.push(m);
					}
				}
			}
			frontier = next;
		}
		distance
	};
	let [mine, theirs] = [distances(me), distances(1 - me)];
	(0..count).map(|n| mine[n] < theirs[n]).collect()
}
