use crate::board::{Board, NodeId, TriangleId};

pub const UNKNOWN_COMMAND: i32 = 1;
pub const NOT_AVAILABLE: i32 = 2;
pub const INVALID_NODE: i32 = 3;
pub const INVALID_TRIANGLE: i32 = 4;
pub const CANT_BE_APPLIED: i32 = 5;
pub const CANT_ADD_PATH: i32 = 6;
pub const POSITIVE_AMOUNT: i32 = 7;
pub const NOT_NEAR_TRIANGLE: i32 = 8;

const MAX_ERRORS_SHOWN: usize = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputError {
	pub message: String,
	pub code: i32,
	pub critical: bool,
}

pub fn group_errors(errors: &mut Vec<InputError>) -> Vec<InputError> {
	let mut shown = Vec::new();
	while let Some(first) = errors.first().cloned() {
		let (same, rest): (Vec<_>, Vec<_>) = errors.drain(..).partition(|e| e.code == first.code);
		*errors = rest;
		if same.len() <= MAX_ERRORS_SHOWN {
			shown.extend(same);
		} else {
			shown.push(same[0].clone());
			shown.push(same[1].clone());
			shown.push(InputError {
				message: format!("{} more errors of that type", same.len() - 2),
				code: first.code,
				critical: first.critical,
			});
		}
	}
	shown
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
	Move {
		from: Option<NodeId>,
		to: Option<NodeId>,
		amount: i32,
	},
	Spawn {
		node: Option<NodeId>,
		triangle: Option<TriangleId>,
	},
	Attack {
		target: Option<NodeId>,
		triangle: Option<TriangleId>,
		killed: i32,
	},
	AddPath {
		from: Option<NodeId>,
		to: Option<NodeId>,
		triangle: Option<TriangleId>,
	},
	RemovePath {
		from: Option<NodeId>,
		to: Option<NodeId>,
		triangle: Option<TriangleId>,
	},
	Debug,
}

#[derive(Clone, Debug)]
pub struct Task {
	pub id: u64,
	pub player: usize,
	pub kind: Kind,
	pub failed: bool,
}

pub enum Applied {
	Moved,
	Spawned,
	Attacked(i32),
	AddedPath,
	RemovedPath,
	Drawn,
}

impl Task {
	pub fn priority(&self) -> i32 {
		match self.kind {
			Kind::Debug => -1,
			Kind::Attack { .. } => 0,
			Kind::Move { .. } => 1,
			Kind::Spawn { .. } => 2,
			Kind::AddPath { .. } => 3,
			Kind::RemovePath { .. } => 4,
		}
	}

	pub fn allows_many_per_frame(&self) -> bool {
		matches!(
			self.kind,
			Kind::Debug | Kind::Move { .. } | Kind::Spawn { .. }
		)
	}

	pub fn required_league(&self) -> i32 {
		match self.kind {
			Kind::Debug | Kind::Move { .. } | Kind::Spawn { .. } => 1,
			Kind::Attack { .. } => 2,
			Kind::AddPath { .. } | Kind::RemovePath { .. } => 3,
		}
	}

	pub fn cost(&self, board: &Board) -> i32 {
		match self.kind {
			Kind::AddPath {
				from: Some(from),
				to: Some(to),
				..
			} => board.points[from].dist2(board.points[to]),
			_ => 0,
		}
	}

	pub fn can_apply(&self, board: &Board, strict: bool) -> bool {
		let player = self.player;
		match self.kind {
			Kind::Debug => true,
			Kind::Move { from, to, .. } => {
				if !strict {
					return true;
				}
				match (self.failed, from, to) {
					(false, Some(from), Some(to)) => {
						from != to && board.remaining[from][player] > 0
					}
					_ => false,
				}
			}
			Kind::Spawn { triangle, .. } => match (self.failed, triangle) {
				(false, Some(t)) => board.can_use(t, player, 0),
				_ => false,
			},
			Kind::Attack {
				target, triangle, ..
			} => {
				if !strict {
					return true;
				}
				match (self.failed, target, triangle) {
					(false, Some(target), Some(t)) => {
						board.can_use(t, player, 1) && board.has_neighbor(t, target)
					}
					_ => false,
				}
			}
			Kind::AddPath { from, to, triangle } => {
				let (Some(from), Some(to)) = (from, to) else {
					return false;
				};
				let valid = !strict
					|| match (self.failed, triangle) {
						(false, Some(t)) => {
							board.can_use(t, player, 1) && !board.connected(from, to)
						}
						_ => false,
					};
				valid && board.can_connect(from, to)
			}
			Kind::RemovePath { from, to, triangle } => {
				if !strict {
					return true;
				}
				match (self.failed, from, to, triangle) {
					(false, Some(from), Some(to), Some(t)) => {
						board.can_use(t, player, 1)
							&& board.connected(from, to)
							&& !board.triangle(t).has_node(to)
					}
					_ => false,
				}
			}
		}
	}

	pub fn apply(&mut self, board: &mut Board) -> Applied {
		let player = self.player;
		match &mut self.kind {
			Kind::Debug => Applied::Drawn,
			Kind::Move { from, to, amount } => {
				let (from, to) = (from.unwrap(), to.unwrap());
				*amount = (*amount).min(board.remaining[from][player]);
				move_units(board, player, from, to, *amount);
				Applied::Moved
			}
			Kind::Spawn { node, triangle } => {
				board.use_triangle(triangle.unwrap(), 0);
				board.units[node.unwrap()][player] += 1;
				Applied::Spawned
			}
			Kind::Attack {
				target,
				triangle,
				killed,
			} => {
				board.use_triangle(triangle.unwrap(), 1);
				let target = target.unwrap();
				let opponent = 1 - player;
				*killed = board.units[target][opponent];
				board.units[target][opponent] = 0;
				board.remaining[target][opponent] = 0;
				Applied::Attacked(*killed)
			}
			Kind::AddPath { from, to, triangle } => {
				board.use_triangle(triangle.unwrap(), 1);
				board.connect(from.unwrap(), to.unwrap());
				Applied::AddedPath
			}
			Kind::RemovePath { from, to, triangle } => {
				board.use_triangle(triangle.unwrap(), 1);
				board.disconnect(from.unwrap(), to.unwrap());
				Applied::RemovedPath
			}
		}
	}
}

fn move_units(board: &mut Board, player: usize, from: NodeId, to: NodeId, amount: i32) {
	if from == to || amount <= 0 {
		return;
	}
	let dist = board.bfs(to, None);
	let step = board.neighbors[from]
		.iter()
		.copied()
		.find(|&next| dist[next] < dist[from]);
	if let Some(next) = step {
		board.units[from][player] -= amount;
		board.remaining[from][player] -= amount;
		board.units[next][player] += amount;
	}
}

pub fn java_trim(text: &str) -> &str {
	text.trim_matches(|c: char| c <= ' ')
}

pub fn java_split(text: &str, separator: char) -> Vec<&str> {
	let mut parts: Vec<&str> = text.split(separator).collect();
	while parts.len() > 1 && parts.last() == Some(&"") {
		parts.pop();
	}
	if parts == [""] && !text.is_empty() {
		parts.clear();
	}
	parts
}

fn is_java_space(c: char) -> bool {
	matches!(c, ' ' | '\t' | '\n' | '\x0B' | '\x0C' | '\r')
}

fn is_line_terminator(c: char) -> bool {
	matches!(c, '\n' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}')
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Arg {
	Natural,
	Integer,
	Rest,
}

fn skip_spaces(text: &str) -> (&str, usize) {
	let trimmed = text.trim_start_matches(is_java_space);
	(trimmed, text.len() - trimmed.len())
}

fn match_command<'a>(command: &'a str, keyword: &str, args: &[Arg]) -> Option<Vec<&'a str>> {
	let (rest, _) = skip_spaces(command);
	let head = rest.get(..keyword.len())?;
	if !head.eq_ignore_ascii_case(keyword) {
		return None;
	}
	let mut rest = &rest[keyword.len()..];
	let mut groups = Vec::with_capacity(args.len());
	for &arg in args {
		let (after, skipped) = skip_spaces(rest);
		if skipped == 0 {
			return None;
		}
		rest = after;
		match arg {
			Arg::Rest => {
				let end = rest.find(is_line_terminator).unwrap_or(rest.len());
				if end == 0 || !rest[end..].chars().all(is_java_space) {
					return None;
				}
				groups.push(&rest[..end]);
				return Some(groups);
			}
			Arg::Natural | Arg::Integer => {
				let sign = usize::from(arg == Arg::Integer && rest.starts_with('-'));
				let digits = rest[sign..].bytes().take_while(u8::is_ascii_digit).count();
				if digits == 0 {
					return None;
				}
				groups.push(&rest[..sign + digits]);
				rest = &rest[sign + digits..];
			}
		}
	}
	let (rest, _) = skip_spaces(rest);
	rest.is_empty().then_some(groups)
}

struct Parser<'a> {
	board: &'a Board,
	player: usize,
	failed: bool,
	errors: Vec<InputError>,
}

struct Overflow;

impl Parser<'_> {
	fn fail(&mut self, message: String, code: i32) {
		if self.failed {
			return;
		}
		self.failed = true;
		self.errors.push(InputError {
			message,
			code,
			critical: false,
		});
	}

	fn int(&self, text: &str) -> Result<i32, Overflow> {
		text.parse::<i32>().map_err(|_| Overflow)
	}

	fn node(&mut self, text: &str) -> Result<Option<NodeId>, Overflow> {
		let id = self.int(text)?;
		if id < 0 || id as usize >= self.board.len() {
			self.fail(format!("Invalid node: {text}"), INVALID_NODE);
			return Ok(None);
		}
		Ok(Some(id as usize))
	}

	fn triangle(&mut self, corners: [Option<NodeId>; 3]) -> Option<TriangleId> {
		let [Some(a), Some(b), Some(c)] = corners else {
			return None;
		};
		let found = self.board.triangles.iter().copied().find(|&t| {
			let triangle = self.board.triangle(t);
			triangle.has_node(a) && triangle.has_node(b) && triangle.has_node(c)
		});
		if found.is_none() {
			self.fail(format!("Invalid triangle: {a} {b} {c}"), INVALID_TRIANGLE);
		}
		found
	}

	fn build(&mut self, command: &str) -> Result<Option<Kind>, Overflow> {
		use Arg::{Integer, Natural, Rest};
		if let Some(g) = match_command(command, "MOVE", &[Natural, Natural, Integer]) {
			let from = self.node(g[0])?;
			let to = self.node(g[1])?;
			let amount = self.int(g[2])?;
			if amount <= 0 {
				self.fail(
					"Amount of units to move must be positive".into(),
					POSITIVE_AMOUNT,
				);
			}
			return Ok(Some(Kind::Move { from, to, amount }));
		}
		if let Some(g) = match_command(command, "SPAWN", &[Natural, Natural, Natural]) {
			let node = self.node(g[0])?;
			let second = self.node(g[1])?;
			let third = self.node(g[2])?;
			let triangle = self.triangle([node, second, third]);
			return Ok(Some(Kind::Spawn { node, triangle }));
		}
		if let Some(g) = match_command(command, "ATTACK", &[Natural; 4]) {
			let corners = [self.node(g[0])?, self.node(g[1])?, self.node(g[2])?];
			let target = self.node(g[3])?;
			let triangle = self.triangle(corners);
			if let (false, Some(t), Some(target)) = (self.failed, triangle, target)
				&& !self.board.has_neighbor(t, target)
			{
				self.fail(
					format!("Can't attack house {target} from given triangle"),
					NOT_NEAR_TRIANGLE,
				);
			}
			return Ok(Some(Kind::Attack {
				target,
				triangle,
				killed: 0,
			}));
		}
		for (keyword, adding) in [("ADD_PATH", true), ("REMOVE_PATH", false)] {
			let Some(g) = match_command(command, keyword, &[Natural; 4]) else {
				continue;
			};
			let from = self.node(g[0])?;
			let second = self.node(g[1])?;
			let third = self.node(g[2])?;
			let to = self.node(g[3])?;
			let triangle = self.triangle([from, second, third]);
			if let (false, Some(t), Some(from), Some(to)) = (self.failed, triangle, from, to) {
				if adding && !self.board.can_connect(from, to) {
					self.fail(
						format!("Can't connect houses {from} and {to}"),
						CANT_ADD_PATH,
					);
				}
				if !adding && !self.board.has_neighbor(t, to) {
					self.fail(
						format!("House {to} isn't connected to given triangle"),
						NOT_NEAR_TRIANGLE,
					);
				}
			}
			return Ok(Some(if adding {
				Kind::AddPath { from, to, triangle }
			} else {
				Kind::RemovePath { from, to, triangle }
			}));
		}
		let drawings: [(&str, &[Arg]); 4] = [
			("LINE", &[Integer; 4]),
			("RECT", &[Integer; 4]),
			("CIRCLE", &[Integer; 3]),
			("TEXT", &[Integer, Integer, Rest]),
		];
		for (keyword, args) in drawings {
			if let Some(g) = match_command(command, keyword, args) {
				for (group, arg) in g.iter().zip(args) {
					if *arg != Rest {
						self.int(group)?;
					}
				}
				return Ok(Some(Kind::Debug));
			}
		}
		Ok(None)
	}
}

pub struct Parsed {
	pub task: Option<Task>,
	pub errors: Vec<InputError>,
}

pub fn parse_task(board: &Board, player: usize, command: &str, league: i32, id: u64) -> Parsed {
	let unknown = || InputError {
		message: format!("Unknown command: {command}"),
		code: UNKNOWN_COMMAND,
		critical: true,
	};
	let trimmed = java_trim(command);
	if trimmed.is_empty() || trimmed.to_uppercase() == "WAIT" {
		return Parsed {
			task: None,
			errors: Vec::new(),
		};
	}
	let mut parser = Parser {
		board,
		player,
		failed: false,
		errors: Vec::new(),
	};
	let kind = match parser.build(command) {
		Ok(Some(kind)) => kind,
		Ok(None) | Err(Overflow) => {
			parser.errors.push(unknown());
			return Parsed {
				task: None,
				errors: parser.errors,
			};
		}
	};
	let task = Task {
		id,
		player: parser.player,
		kind,
		failed: parser.failed,
	};
	if task.required_league() > league {
		parser.fail(
			format!("Command not available in your league: {command}"),
			NOT_AVAILABLE,
		);
		return Parsed {
			task: None,
			errors: parser.errors,
		};
	}
	let applicable = task.can_apply(board, true);
	let mut task = task;
	if !applicable {
		parser.fail(format!("Task can't be applied: {command}"), CANT_BE_APPLIED);
		task.failed = parser.failed;
	}
	Parsed {
		task: Some(task),
		errors: parser.errors,
	}
}

#[derive(Clone, Debug, Default)]
pub struct TaskManager {
	tasks: Vec<Task>,
	next_id: u64,
}

impl TaskManager {
	pub fn next_id(&mut self) -> u64 {
		self.next_id += 1;
		self.next_id
	}

	pub fn push(&mut self, task: Task) {
		self.tasks.push(task);
	}

	fn peek(&mut self, board: &Board) -> Vec<usize> {
		self.tasks.retain(|task| task.can_apply(board, true));
		let lowest = self.tasks.iter().map(Task::priority).min();
		let mut chosen: Vec<usize> = (0..self.tasks.len())
			.filter(|&i| Some(self.tasks[i].priority()) == lowest)
			.collect();
		chosen.sort_by_key(|&i| self.tasks[i].cost(board));
		chosen
	}

	pub fn has_tasks(&mut self, board: &Board) -> bool {
		!self.peek(board).is_empty()
	}

	pub fn has_debug(&mut self, board: &Board) -> bool {
		let chosen = self.peek(board);
		chosen
			.first()
			.is_some_and(|&i| self.tasks[i].priority() == -1)
	}

	pub fn pop(&mut self, board: &Board) -> [Vec<Task>; 2] {
		let chosen = self.peek(board);
		let mut popped = [Vec::new(), Vec::new()];
		for &i in &chosen {
			let task = self.tasks[i].clone();
			popped[task.player].push(task);
		}
		let mut index = 0;
		self.tasks.retain(|_| {
			index += 1;
			!chosen.contains(&(index - 1))
		});
		popped
	}

	pub fn give_back(&mut self, tasks: [Vec<Task>; 2]) {
		for list in tasks {
			self.tasks.extend(list);
		}
	}

	pub fn len(&self) -> usize {
		self.tasks.len()
	}

	pub fn is_empty(&self) -> bool {
		self.tasks.is_empty()
	}
}
