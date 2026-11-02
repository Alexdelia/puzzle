use crate::board::Board;
use crate::jhash::CountMap;
use crate::proto::input;
use crate::task::{
	Applied, Kind, Task, TaskManager, group_errors, java_split, java_trim, parse_task,
};
use std::collections::VecDeque;

pub const MAX_TURNS: i32 = 200;
pub const DOMINATION_SHARE: f64 = 0.8;
pub const DEFAULT_LEAGUE: i32 = 1;
pub const MESSAGE_LIMIT: usize = 50;

#[derive(Clone, Debug)]
pub struct Player {
	pub score: i32,
	pub active: bool,
	pub message: String,
	pub deactivated_by: Option<String>,
}

impl Player {
	fn new() -> Self {
		Player {
			score: 0,
			active: true,
			message: String::new(),
			deactivated_by: None,
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase {
	Quiet,
	Ask([Option<String>; 2]),
}

impl Phase {
	pub fn asked(&self) -> [bool; 2] {
		match self {
			Phase::Quiet => [false; 2],
			Phase::Ask(inputs) => [0, 1].map(|p| inputs[p].is_some()),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
	Line(String),
	Timeout,
}

#[derive(Clone, Debug)]
pub struct Game {
	pub board: Board,
	pub league: i32,
	pub players: [Player; 2],
	pub summary: Vec<String>,
	pub tooltips: Vec<String>,
	pub max_turns: i32,
	pub turn: i32,
	pub ended: bool,
	tasks: TaskManager,
	killed_surrounded: bool,
	turn_summary: CountMap,
}

pub fn token(player: usize) -> String {
	format!("${player}")
}

impl Game {
	pub fn new(board: Board, league: i32) -> Self {
		let mut game = Game {
			board,
			league,
			players: [Player::new(), Player::new()],
			summary: Vec::new(),
			tooltips: Vec::new(),
			max_turns: MAX_TURNS,
			turn: 0,
			ended: false,
			tasks: TaskManager::default(),
			killed_surrounded: false,
			turn_summary: CountMap::default(),
		};
		while game.finalize_turn(false) {}
		game
	}

	pub fn scores(&self) -> [i32; 2] {
		[self.players[0].score, self.players[1].score]
	}

	pub fn pending_tasks(&self) -> usize {
		self.tasks.len()
	}

	pub fn is_over(&self) -> bool {
		self.ended || self.turn >= self.max_turns
	}

	pub fn begin_frame(&mut self) -> Phase {
		self.turn += 1;
		self.summary.clear();
		self.tooltips.clear();
		if self.tasks.has_tasks(&self.board) {
			self.max_turns += 1;
			self.apply_actions();
			return Phase::Quiet;
		}
		if self.finalize_turn(true) {
			self.max_turns += 1;
			return Phase::Quiet;
		}
		Phase::Ask(std::array::from_fn(|player| self.check_and_ask(player)))
	}

	fn check_and_ask(&mut self, player: usize) -> Option<String> {
		if !self.players[player].active {
			return None;
		}
		let units = self.board.units_of(player);
		let triangles = self.board.triangles_of(player);
		let opponent = 1 - player;
		if units == 0 && triangles == 0 {
			self.kill(player, "no units or triangles left");
			return None;
		}
		if units == 1 && triangles == 0 && self.players[player].score < self.players[opponent].score
		{
			self.kill(player, "can't score any more points");
			return None;
		}
		let dominated = self.board.triangles.len() as f64 * DOMINATION_SHARE;
		if self.board.triangles_of(opponent) as f64 >= dominated {
			self.kill(player, "is dominated");
			return None;
		}
		Some(input(self, player, self.turn == 1))
	}

	pub fn end_frame(&mut self, asked: [bool; 2], answers: [Option<Answer>; 2]) {
		let responding: Vec<usize> = (0..2)
			.filter(|&p| asked[p] && self.players[p].active)
			.collect();
		for player in responding {
			let mut errors = Vec::new();
			match &answers[player] {
				Some(Answer::Line(line)) => self.take_commands(player, line, &mut errors),
				Some(Answer::Timeout) | None => self.kill(player, "timeout"),
			}
			for error in group_errors(&mut errors) {
				if error.critical {
					self.kill(player, &error.message);
				} else {
					self.summary
						.push(format!("{}: {}", token(player), error.message));
				}
			}
		}
		if self.tasks.has_debug(&self.board) {
			self.tasks.pop(&self.board);
		}
		self.apply_actions();
	}

	fn take_commands(
		&mut self,
		player: usize,
		line: &str,
		errors: &mut Vec<crate::task::InputError>,
	) {
		self.players[player].message.clear();
		for command in java_split(line, ';') {
			if command.to_uppercase().starts_with("MSG ") {
				let text: String = command.chars().skip(4).collect();
				self.players[player].message =
					java_trim(&text).chars().take(MESSAGE_LIMIT).collect();
				continue;
			}
			let id = self.tasks.next_id();
			let parsed = parse_task(&self.board, player, command, self.league, id);
			errors.extend(parsed.errors);
			if let Some(task) = parsed.task {
				self.tasks.push(task);
			}
		}
	}

	fn kill(&mut self, player: usize, message: &str) {
		if !self.players[player].active {
			return;
		}
		self.summary.push(format!("{}: {message}", token(player)));
		let tooltip = format!("{} {message}!", token(player));
		self.tooltips.push(tooltip.clone());
		let state = &mut self.players[player];
		state.active = false;
		state.deactivated_by = Some(tooltip);
		state.score = -1;
		self.ended = true;
	}

	fn apply_actions(&mut self) {
		self.killed_surrounded = false;
		let mut queues = self.tasks.pop(&self.board).map(VecDeque::from);
		let mut to_apply: Vec<Task> = Vec::new();
		for queue in &mut queues {
			while let Some(task) = queue.pop_front() {
				if !task.can_apply(&self.board, true) {
					continue;
				}
				let many = task.allows_many_per_frame();
				to_apply.push(task);
				if !many {
					break;
				}
			}
		}
		to_apply.sort_by_key(|task| task.cost(&self.board));
		for mut task in to_apply {
			if !task.can_apply(&self.board, false) {
				continue;
			}
			if task.allows_many_per_frame() && !task.can_apply(&self.board, true) {
				continue;
			}
			let who = token(task.player);
			match task.kind {
				Kind::Spawn { .. } => self.turn_summary.add(format!("{who} spawned %d unit"), 1),
				Kind::AddPath { .. } => self.turn_summary.add(format!("{who} added %d path"), 1),
				Kind::RemovePath { .. } => {
					self.turn_summary.add(format!("{who} removed %d path"), 1)
				}
				_ => {}
			}
			if let Applied::Attacked(killed) = task.apply(&mut self.board) {
				self.turn_summary
					.add(format!("{who} attacked %d unit"), killed);
			}
		}
		self.tasks.give_back(queues.map(Vec::from));
		if !self.tasks.has_tasks(&self.board) {
			self.summarize(true);
		}
	}

	fn finalize_turn(&mut self, report: bool) -> bool {
		self.board.remaining = self.board.units.clone();
		for t in self.board.triangles.clone() {
			self.board.update_allowed_captures(t);
		}
		if self.surround_nodes() || self.capture_triangles() {
			self.summarize(report);
			return true;
		}
		for &t in &self.board.triangles {
			let triangle = &mut self.board.arena[t];
			if let Some(owner) = triangle.owner {
				self.players[owner].score += 1;
			}
			triangle.initial_owner = triangle.owner;
		}
		self.summarize(report);
		false
	}

	fn surround_nodes(&mut self) -> bool {
		if self.killed_surrounded {
			return false;
		}
		self.killed_surrounded = true;
		let board = &self.board;
		let advantage: [Vec<bool>; 2] = [0, 1].map(|player| {
			board
				.units
				.iter()
				.map(|u| u[player] > u[1 - player])
				.collect()
		});
		let mut surrounded = false;
		for player in 0..2 {
			let opponent = 1 - player;
			for node in 0..self.board.len() {
				let around = &self.board.neighbors[node];
				let units = self.board.units[node][player];
				if units > 0 && !around.is_empty() && around.iter().all(|&n| advantage[opponent][n])
				{
					self.turn_summary
						.add(format!("{} surrounded %d unit", token(opponent)), units);
					self.board.units[node][player] = 0;
					surrounded = true;
				}
			}
		}
		surrounded
	}

	fn capture_triangles(&mut self) -> bool {
		let mut changed = false;
		let owners = self.board.owners();
		for t in self.board.triangles.clone() {
			if self.board.capture(t, &owners) {
				changed = true;
				let owner = self.board.arena[t].owner.expect("a capture sets an owner");
				self.turn_summary
					.add(format!("{} captured %d triangle", token(owner)), 1);
			}
		}
		changed
	}

	fn summarize(&mut self, report: bool) {
		if report {
			for (key, count) in self.turn_summary.in_java_order() {
				let mut line = key.replace("%d", &count.to_string());
				if count != 1 {
					line.push('s');
				}
				self.summary.push(line);
			}
		}
		self.turn_summary.clear();
	}
}
