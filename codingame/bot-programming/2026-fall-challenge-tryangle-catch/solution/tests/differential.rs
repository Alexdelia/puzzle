use engine::{Command, Map, Point, State};
use std::collections::BTreeMap;
use tryangle::board::{Board, is_crossing};
use tryangle::game::{Answer, Game, Phase};
use tryangle::mapgen;

const LEAGUE_WITH_PATHS: i32 = 3;
const SEEDS: std::ops::Range<i64> = 0..200;
const CROWDED_TURNS: usize = 40;
const CROWDED_STACK_LIMIT: usize = 4;
const FAVORED_OWNER_PERCENTS: [usize; 3] = [50, 70, 78];
const OWNED_PERCENT: usize = 95;
const TRIANGLE_USE_RATES: [usize; 4] = [2, 5, 12, 80];
const RULE_EVENTS: [&str; 9] = [
	"spawned",
	"attacked",
	"added",
	"removed",
	"surrounded",
	"captured",
	"is dominated",
	"no units or triangles left",
	"can't score any more points",
];

#[derive(Default)]
struct Coverage {
	turns: usize,
	events: BTreeMap<&'static str, usize>,
}

impl Coverage {
	fn record(&mut self, summary: &[String]) {
		for line in summary {
			for event in RULE_EVENTS.iter().filter(|&&e| line.contains(e)) {
				*self.events.entry(event).or_default() += 1;
			}
		}
	}
}

struct Rng(u64);

impl Rng {
	fn new(seed: u64) -> Self {
		Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
	}

	fn below(&mut self, bound: usize) -> usize {
		self.0 ^= self.0 << 13;
		self.0 ^= self.0 >> 7;
		self.0 ^= self.0 << 17;
		(self.0 % bound as u64) as usize
	}

	fn pick<T: Copy>(&mut self, items: &[T]) -> Option<T> {
		(!items.is_empty()).then(|| items[self.below(items.len())])
	}
}

struct Wanderer {
	rng: Rng,
	goal: Option<[usize; 3]>,
	triangle_use_one_in: usize,
}

struct Order {
	text: String,
	command: Command,
}

impl Wanderer {
	fn new(seed: u64) -> Self {
		let mut rng = Rng::new(seed);
		let triangle_use_one_in = TRIANGLE_USE_RATES[rng.below(TRIANGLE_USE_RATES.len())];
		Wanderer {
			rng,
			goal: None,
			triangle_use_one_in,
		}
	}

	fn retarget(&mut self, board: &Board, player: usize) {
		let capturable: Vec<[usize; 3]> = board
			.triangles
			.iter()
			.map(|&t| board.triangle(t))
			.filter(|t| t.can_capture[player] && t.owner != Some(player))
			.map(|t| t.nodes)
			.collect();
		let stale = self.goal.is_none_or(|goal| !capturable.contains(&goal));
		if stale || self.rng.below(8) == 0 {
			self.goal = self.rng.pick(&capturable);
		}
	}

	fn moves(&mut self, board: &Board, player: usize) -> Vec<Order> {
		let guarded = owned_corners(board, player);
		let mut orders = Vec::new();
		for from in 0..board.len() {
			let units = board.units[from][player];
			let holding = guarded.contains(&from) && self.rng.below(3) != 0;
			if units == 0 || holding || self.rng.below(4) == 0 {
				continue;
			}
			let to = match (self.rng.below(10), self.goal) {
				(0, _) => self.rng.below(board.len()),
				(1..=5, Some(goal)) => goal[self.rng.below(3)],
				_ => self.rng.pick(&board.neighbors[from]).unwrap_or(from),
			};
			let amount = self.rng.below(units as usize + 2) as i32;
			orders.push(Order {
				text: format!("MOVE {from} {to} {amount}"),
				command: Command::Move { from, to, amount },
			});
		}
		orders
	}

	fn triangle_use(
		&mut self,
		map: &Map,
		board: &Board,
		player: usize,
		corners: [usize; 3],
	) -> Option<Order> {
		let first = self.rng.below(3);
		let [a, b, c] = [0, 1, 2].map(|i| corners[(first + i) % 3]);
		let triangle = map
			.triangle(corners)
			.expect("live triangles are candidates");
		let around: Vec<usize> = (0..board.len())
			.filter(|&n| corners.iter().any(|&corner| board.connected(corner, n)))
			.collect();
		let manned = corners.iter().all(|&c| board.units[c][player] > 0);
		let action = if manned || self.rng.below(4) == 0 {
			self.rng.below(4)
		} else {
			0
		};
		match action {
			0 => Some(Order {
				text: format!("SPAWN {a} {b} {c}"),
				command: Command::Spawn { triangle, node: a },
			}),
			1 => {
				let enemies: Vec<usize> = around
					.iter()
					.copied()
					.filter(|&n| board.units[n][1 - player] > 0)
					.collect();
				let target = match self.rng.below(10) {
					0 => self.rng.below(board.len()),
					1..=3 => self.rng.pick(&around)?,
					_ => self.rng.pick(&enemies).or_else(|| self.rng.pick(&around))?,
				};
				Some(Order {
					text: format!("ATTACK {a} {b} {c} {target}"),
					command: Command::Attack { triangle, target },
				})
			}
			2 => {
				let reachable: Vec<usize> = (0..board.len())
					.filter(|&to| to != a && !board.connected(a, to) && board.can_connect(a, to))
					.collect();
				let to = match self.rng.below(6) {
					0 => (a + 1 + self.rng.below(board.len() - 1)) % board.len(),
					_ => self.rng.pick(&reachable)?,
				};
				Some(Order {
					text: format!("ADD_PATH {a} {b} {c} {to}"),
					command: Command::AddPath {
						triangle,
						from: a,
						to,
					},
				})
			}
			_ => {
				let outward: Vec<usize> = board.neighbors[a]
					.iter()
					.copied()
					.filter(|n| !corners.contains(n))
					.collect();
				let to = self.rng.pick(&outward)?;
				Some(Order {
					text: format!("REMOVE_PATH {a} {b} {c} {to}"),
					command: Command::RemovePath {
						triangle,
						from: a,
						to,
					},
				})
			}
		}
	}

	fn orders(&mut self, map: &Map, game: &Game, player: usize) -> Vec<Order> {
		let board = &game.board;
		self.retarget(board, player);
		let mut orders = self.moves(board, player);
		for &t in &board.triangles {
			let triangle = board.triangle(t);
			let probing_foreign = self.rng.below(40) == 0;
			if triangle.owner != Some(player) && !probing_foreign {
				continue;
			}
			let attempts = 1 + usize::from(self.rng.below(6) == 0);
			for _ in 0..attempts {
				if self.rng.below(self.triangle_use_one_in) != 0 {
					continue;
				}
				orders.extend(self.triangle_use(map, board, player, triangle.nodes));
			}
		}
		let shuffles = orders.len();
		for i in 0..shuffles {
			let j = i + self.rng.below(shuffles - i);
			orders.swap(i, j);
		}
		orders
	}
}

fn owned_corners(board: &Board, player: usize) -> Vec<usize> {
	board
		.triangles
		.iter()
		.map(|&t| board.triangle(t))
		.filter(|t| t.owner == Some(player))
		.flat_map(|t| t.nodes)
		.collect()
}

fn engine_map(board: &Board) -> Map {
	Map::new(
		board
			.points
			.iter()
			.map(|p| Point { x: p.x, y: p.y })
			.collect(),
	)
}

fn snapshot(map: &Map, game: &Game) -> State {
	let board = &game.board;
	let mut state = State::new(map, &board.units, &board.edges());
	for &t in &board.triangles {
		let triangle = board.triangle(t);
		assert_eq!(
			triangle.initial_owner, triangle.owner,
			"owner settles before players are asked"
		);
		let id = map
			.triangle(triangle.nodes)
			.expect("live triangles are candidates");
		state.set_triangle(id, triangle.owner, triangle.can_capture);
	}
	state.score = game.scores();
	state
}

fn first_difference(map: &Map, predicted: &State, actual: &State) -> Option<String> {
	for node in 0..map.len() {
		if predicted.units[node] != actual.units[node] {
			return Some(format!(
				"units at {node}: predicted {:?}, actual {:?}",
				predicted.units[node], actual.units[node]
			));
		}
		if predicted.adjacency[node] != actual.adjacency[node] {
			return Some(format!("paths around {node} differ"));
		}
	}
	if predicted.edges != actual.edges {
		return Some("edge sets differ".into());
	}
	for t in 0..map.triangles.len() {
		let view = |s: &State| {
			(
				s.live.has(t),
				[0, 1].map(|p| s.owner[p].has(t)),
				[0, 1].map(|p| s.initial_owner[p].has(t)),
				[0, 1].map(|p| s.can_capture[p].has(t)),
			)
		};
		if view(predicted) != view(actual) {
			return Some(format!(
				"triangle {:?} (live, owner, initial owner, can capture): predicted {:?}, actual {:?}",
				map.triangles[t],
				view(predicted),
				view(actual)
			));
		}
	}
	if predicted.score != actual.score {
		return Some(format!(
			"scores: predicted {:?}, actual {:?}",
			predicted.score, actual.score
		));
	}
	(predicted != actual).then(|| "states differ outside compared fields".into())
}

fn crowd(game: &mut Game, seed: i64) {
	let mut rng = Rng::new(!(seed as u64));
	let board = &mut game.board;
	for node in 0..board.len() {
		board.units[node] = [0, 1].map(|_| rng.below(CROWDED_STACK_LIMIT) as i32);
	}
	board.remaining = board.units.clone();
	let favored_percent = FAVORED_OWNER_PERCENTS[rng.below(FAVORED_OWNER_PERCENTS.len())];
	for t in board.triangles.clone() {
		let roll = rng.below(100);
		let owner = match roll {
			_ if roll < favored_percent => Some(0),
			_ if roll < OWNED_PERCENT => Some(1),
			_ => None,
		};
		let triangle = &mut board.arena[t];
		triangle.owner = owner;
		triangle.initial_owner = owner;
		triangle.can_capture = [0, 1].map(|p| owner != Some(p));
	}
}

fn replay(mut game: Game, seed: i64, turn_limit: usize, coverage: &mut Coverage) {
	let map = engine_map(&game.board);
	let mut wanderers = [0, 1].map(|p| Wanderer::new(seed as u64 * 2 + p));
	let mut predicted: Option<(State, Vec<String>)> = None;
	let mut turns = 0;
	while !game.is_over() && turns <= turn_limit {
		let phase = game.begin_frame();
		coverage.record(&game.summary);
		let Phase::Ask(_) = phase else { continue };
		turns += 1;
		coverage.turns += 1;
		let killed = [0, 1].map(|p| !game.players[p].active);
		let mut actual = snapshot(&map, &game);
		if let Some((expected, answers)) = &predicted {
			assert_eq!(
				expected.eliminated(),
				killed,
				"seed {seed} turn {turns}: eliminations after {answers:?}"
			);
			for player in (0..2).filter(|&p| killed[p]) {
				actual.score[player] = expected.score[player];
			}
			if let Some(difference) = first_difference(&map, expected, &actual) {
				panic!("seed {seed} turn {turns}: {difference}\nanswers: {answers:?}");
			}
		}
		if killed.contains(&true) {
			break;
		}
		let orders = [0, 1].map(|p| wanderers[p].orders(&map, &game, p));
		let answers: Vec<String> = orders
			.iter()
			.map(|list| {
				let texts: Vec<&str> = list.iter().map(|o| o.text.as_str()).collect();
				if texts.is_empty() {
					"WAIT".into()
				} else {
					texts.join(";")
				}
			})
			.collect();
		let commands = orders.map(|list| list.into_iter().map(|o| o.command).collect::<Vec<_>>());
		let mut next = actual;
		next.step(&map, [&commands[0], &commands[1]]);
		predicted = Some((next, answers.clone()));
		let [first, second] = [0, 1].map(|p| Some(Answer::Line(answers[p].clone())));
		game.end_frame(phase.asked(), [first, second]);
		coverage.record(&game.summary);
	}
}

#[test]
fn engine_step_matches_referee_on_random_games() {
	let mut coverage = Coverage::default();
	for seed in SEEDS {
		let fresh = || Game::new(mapgen::generate(seed), LEAGUE_WITH_PATHS);
		replay(fresh(), seed, usize::MAX, &mut coverage);
		let mut crowded = fresh();
		crowd(&mut crowded, seed);
		replay(crowded, seed, CROWDED_TURNS, &mut coverage);
	}
	eprintln!(
		"{} turns, rule events {:?}",
		coverage.turns, coverage.events
	);
	for event in RULE_EVENTS {
		assert!(
			coverage.events.contains_key(event),
			"no game exercised `{event}`"
		);
	}
}

struct PathClaim {
	corners: [usize; 3],
	to: usize,
}

impl PathClaim {
	fn text(&self) -> String {
		let [a, b, c] = self.corners;
		format!("ADD_PATH {a} {b} {c} {}", self.to)
	}

	fn command(&self, map: &Map) -> Command {
		Command::AddPath {
			triangle: map
				.triangle(self.corners)
				.expect("live triangles are candidates"),
			from: self.corners[0],
			to: self.to,
		}
	}

	fn length(&self, board: &Board) -> i32 {
		board.points[self.corners[0]].dist2(board.points[self.to])
	}
}

fn path_claims(board: &Board, player: usize) -> Vec<PathClaim> {
	let mut claims = Vec::new();
	for &t in &board.triangles {
		let triangle = board.triangle(t);
		if triangle.owner != Some(player) || triangle.initial_owner != Some(player) {
			continue;
		}
		for first in 0..3 {
			let corners = [0, 1, 2].map(|i| triangle.nodes[(first + i) % 3]);
			for to in 0..board.len() {
				if to != corners[0]
					&& !board.connected(corners[0], to)
					&& board.can_connect(corners[0], to)
				{
					claims.push(PathClaim { corners, to });
				}
			}
		}
	}
	claims
}

fn crossing_duel(board: &Board) -> Option<[PathClaim; 2]> {
	let theirs = path_claims(board, 1);
	for mine in path_claims(board, 0) {
		let rival = theirs.iter().position(|other| {
			is_crossing(
				&board.points,
				(mine.corners[0], mine.to),
				(other.corners[0], other.to),
			) && other.length(board) != mine.length(board)
		});
		if let Some(rival) = rival {
			let other = &theirs[rival];
			let other = PathClaim {
				corners: other.corners,
				to: other.to,
			};
			return Some([mine, other]);
		}
	}
	None
}

fn settle(game: &mut Game) -> Phase {
	loop {
		let phase = game.begin_frame();
		if matches!(phase, Phase::Ask(_)) || game.is_over() {
			return phase;
		}
	}
}

#[test]
fn simultaneous_crossing_paths_keep_only_the_shortest() {
	let mut shorter_wins = [0; 2];
	for seed in SEEDS {
		let mut game = Game::new(mapgen::generate(seed), LEAGUE_WITH_PATHS);
		crowd(&mut game, seed);
		let phase = settle(&mut game);
		if game.is_over() || game.players.iter().any(|p| !p.active) {
			continue;
		}
		let Some(duel) = crossing_duel(&game.board) else {
			continue;
		};
		for (player, claim) in duel.iter().enumerate() {
			for corner in claim.corners {
				let units = &mut game.board.units[corner][player];
				*units = (*units).max(1);
			}
		}
		game.board.remaining = game.board.units.clone();
		let map = engine_map(&game.board);
		let mut predicted = snapshot(&map, &game);
		let commands = duel.each_ref().map(|claim| vec![claim.command(&map)]);
		predicted.step(&map, [&commands[0], &commands[1]]);
		let answers = duel
			.each_ref()
			.map(|claim| Some(Answer::Line(claim.text())));
		game.end_frame(phase.asked(), answers);
		settle(&mut game);
		if game.players.iter().any(|p| !p.active) {
			continue;
		}
		let actual = snapshot(&map, &game);
		if let Some(difference) = first_difference(&map, &predicted, &actual) {
			panic!("seed {seed}: {difference}");
		}
		let shorter = usize::from(duel[1].length(&game.board) < duel[0].length(&game.board));
		shorter_wins[shorter] += 1;
	}
	assert!(
		shorter_wins.iter().all(|&count| count > 0),
		"duels where each player had the shorter path: {shorter_wins:?}"
	);
}
