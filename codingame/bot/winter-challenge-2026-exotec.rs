use std::{
	collections::{HashMap, HashSet, VecDeque},
	fmt::Display,
	io,
	time::{Duration, Instant, SystemTime},
};

use rand::{Rng, rngs::ThreadRng};

// TODO: must remove initially stuck snakebot before monte-carlo tree search

const MAX_TURN_DURATION: Duration = Duration::from_millis(80);

type SnakebotId = u8;

/// max 45w x 30h (=1350 tile)
/// snakebot can go out of bounds, but we don't expect under -128 or above 127-45=82
type Axis = i8;
type Coord = (Axis, Axis);

// TODO: check time optimization between u16 and usize
/// each snakebot has 3 possible actions (straight, left, right)
/// encode actions as base‑3 number over the alive agents in increasing order of agent index
/// this gives a unique incrementing index representing the permutation of actions for all agents
/// max 4 snakebot per player, and 3^4 = 81 (so < u8::MAX = 255)
type PlayerActionReprAsIndex = u8;
type BothPlayerAction = (PlayerActionReprAsIndex, PlayerActionReprAsIndex);

const ACTION_REPR_STRAIGHT: PlayerActionReprAsIndex = 0;
const ACTION_REPR_LEFT: PlayerActionReprAsIndex = 1;
const ACTION_REPR_RIGHT: PlayerActionReprAsIndex = 2;

/// technically fit in a u8, but always cast as a usize
type PlayerActionCount = usize;

type NodeIndex = usize;
type VisitCount = u32;

type HeuristicReward = f32;

struct Env {
	g: BlockGrid,

	#[allow(dead_code)]
	my_id: SnakebotId,
	my_snakebot_id_list: Vec<SnakebotId>,
	#[allow(dead_code)]
	foe_snakebot_id_list: Vec<SnakebotId>,
}

struct BlockGrid {
	w: Axis,
	h: Axis,
	d: Vec<u64>,
}

// TODO: try HashSet<Coord>
// TODO: try bitset like BlockGrid
type AppleList = Vec<Coord>;

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

impl Env {
	fn read() -> Self {
		let mut s = String::new();

		io::stdin().read_line(&mut s).unwrap();
		let my_id = parse_input!(s, SnakebotId);

		let g = BlockGrid::read();

		s.clear();
		io::stdin().read_line(&mut s).unwrap();
		let snakebot_per_player = parse_input!(s, usize);

		let mut my_snakebot_id_list = Vec::with_capacity(snakebot_per_player);
		let mut foe_snakebot_id_list = Vec::with_capacity(snakebot_per_player);

		for _ in 0..snakebot_per_player {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();
			my_snakebot_id_list.push(parse_input!(s, SnakebotId));
		}
		for _ in 0..snakebot_per_player {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();
			foe_snakebot_id_list.push(parse_input!(s, SnakebotId));
		}

		Env {
			g,

			my_id,
			my_snakebot_id_list,
			foe_snakebot_id_list,
		}
	}

	fn read_apple() -> AppleList {
		let mut s = String::new();

		io::stdin().read_line(&mut s).unwrap();
		let power_source_count = parse_input!(s, usize);

		let mut apple_list = Vec::with_capacity(power_source_count);

		for _ in 0..power_source_count {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();
			let mut input = s.split(" ");
			let x = parse_input!(input.next().unwrap(), Axis);
			let y = parse_input!(input.next().unwrap(), Axis);

			apple_list.push((x, y));
		}

		apple_list
	}

	fn read_snakebot(&self) -> (Vec<(SnakebotId, Vec<Coord>)>, Vec<(SnakebotId, Vec<Coord>)>) {
		let mut s = String::new();

		io::stdin().read_line(&mut s).unwrap();
		let snakebot_count = parse_input!(s, usize);

		let mut my_snakebot_list = Vec::with_capacity(snakebot_count);
		let mut foe_snakebot_list = Vec::with_capacity(snakebot_count);

		for _ in 0..snakebot_count {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();
			let mut input = s.split(" ");
			let snakebot_id = parse_input!(input.next().unwrap(), SnakebotId);
			let body = input
				.next()
				.unwrap()
				.trim()
				.split(":")
				.filter_map(|coord| {
					let mut parts = coord.split(",");
					let x = parse_input!(parts.next().unwrap(), Axis);
					let y = parse_input!(parts.next().unwrap(), Axis);
					Some((x, y))
				})
				.collect::<Vec<_>>();

			if self.my_snakebot_id_list.contains(&snakebot_id) {
				my_snakebot_list.push((snakebot_id, body));
			} else {
				foe_snakebot_list.push((snakebot_id, body));
			}
		}

		my_snakebot_list.shrink_to_fit();
		foe_snakebot_list.shrink_to_fit();

		(my_snakebot_list, foe_snakebot_list)
	}
}

impl BlockGrid {
	fn read() -> Self {
		let mut s = String::new();

		io::stdin().read_line(&mut s).unwrap();
		let w = parse_input!(s, Axis);

		s.clear();
		io::stdin().read_line(&mut s).unwrap();
		let h = parse_input!(s, Axis);

		let mut d = vec![0; ((w as usize * h as usize) + 63) / 64];
		for y in 0..h {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();
			for (x, c) in s.trim_matches('\n').chars().enumerate() {
				if c == '#' {
					let index = (y as usize * w as usize + x) / 64;
					let bit = (y as usize * w as usize + x) % 64;
					d[index] |= 1 << bit;
				}
			}
		}

		BlockGrid { w, h, d }
	}

	fn is_block(&self, x: Axis, y: Axis) -> bool {
		if x < 0 || y < 0 || x >= self.w || y >= self.h {
			return false;
		}

		let index = (y as usize * self.w as usize + x as usize) / 64;
		let bit = (y as usize * self.w as usize + x as usize) % 64;
		(self.d[index] & (1 << bit)) != 0
	}
}

struct Mcts<S: GameState> {
	node_list: Vec<MctsNode<S>>,

	root: NodeIndex,

	rng: ThreadRng,
}

/// MCTS => Monte Carlo Tree Search
struct MctsNode<S: GameState> {
	state: S,

	node_visit_count: VisitCount,
	/// must be indexable by PlayerActionReprAsIndex
	my_ucb_list: Vec<UcbActionStats>,
	foe_ucb_list: Vec<UcbActionStats>,

	children: HashMap<BothPlayerAction, NodeIndex>,
}

/// UCB => Upper Confidence Bound
#[derive(Clone, Copy, Default)]
struct UcbActionStats {
	visit_count: VisitCount,
	total_reward: HeuristicReward,
}

impl Display for Action {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} {}", self.snakebot_id, self.direction)
	}
}

impl Display for Dir {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Dir::U => write!(f, "UP"),
			Dir::D => write!(f, "DOWN"),
			Dir::L => write!(f, "LEFT"),
			Dir::R => write!(f, "RIGHT"),
		}
	}
}

fn find_snakebot_action(
	env: &Env,
	grid: &Grid,
	apple_list: &[Coord],
	snakebot_id: SnakebotId,
	snakebot_body: &[Coord],
	allowed_time: Duration,
) -> (Action, Option<Coord>) {
	if let Some((single_move, apple)) = has_single_depth_move(env, grid, apple_list, snakebot_body)
	{
		return (
			Action {
				snakebot_id,
				direction: single_move,
			},
			apple,
		);
	}

	// TODO: store body more efficiently?
	let mut visited = HashSet::<Vec<Coord>>::new();
	visited.insert(snakebot_body.to_vec());

	let mut queue = VecDeque::<(Dir, Vec<Coord>)>::new();
	initial_visit_neighbor!(env, queue, visited, grid, snakebot_id, snakebot_body);

	let first = queue.clone().pop_front();
	let default_dir = first.map(|(dir, _)| dir).unwrap_or(Dir::U);
	if queue.len() <= 1 {
		return (
			Action {
				snakebot_id,
				direction: default_dir,
			},
			None,
		);
	}

	let start = SystemTime::now();
	let mut i = 0;
	while let Some((initial_dir, body)) = queue.pop_front() {
		visit_neighbor!(env, queue, visited, grid, snakebot_id, initial_dir, body);

		i += 1;
		let elapsed = start.elapsed().unwrap();
		if i % 1000 == 0 && elapsed >= allowed_time {
			eprintln!("timeout: visited {i} states in {elapsed:?}");
			break;
		}
	}

	(
		Action {
			snakebot_id,
			direction: default_dir,
		},
		None,
	)
}

/// The Mcts solver.

/// Trait that a game must implement to be used with Mcts.
trait GameState: Clone {
	fn my_action_count(&self) -> PlayerActionCount;
	fn foe_action_count(&self) -> PlayerActionCount;

	/// Apply joint actions and return the new state.
	/// Actions are encoded as base‑3 numbers over the alive agents in increasing order of agent index.
	fn apply(
		&self,
		my_action: PlayerActionReprAsIndex,
		foe_action: PlayerActionReprAsIndex,
	) -> Self;
	/// Heuristic evaluation from player 1's perspective (positive if player 1 is ahead).
	/// Only called for non‑terminal states.
	fn evaluate(&self) -> HeuristicReward;
	/// Returns true if the game has ended in this state.
	fn is_terminal(&self) -> bool;
	/// Returns the final value from player 1's perspective (+1 win, -1 loss, 0 draw).
	/// Only called when `is_terminal()` is true.
	fn terminal_value(&self) -> HeuristicReward;
}

impl<S: GameState> Mcts<S> {
	const EXPLORATION_CONSTANT: f32 = 1.4;

	fn new(initial_state: S) -> Self {
		let root = MctsNode::new(initial_state);

		Mcts {
			node_list: vec![root],
			root: 0,

			rng: rand::rng(),
		}
	}

	fn search(&mut self, time_limit: Duration) {
		let start = Instant::now();
		while start.elapsed() < time_limit {
			self.iterate();
		}
	}

	fn iterate(&mut self) {
		let mut path = Vec::<(NodeIndex, PlayerActionReprAsIndex, PlayerActionReprAsIndex)>::new();
		let mut node_index = self.root;

		// --- Selection ---
		loop {
			let node = &self.node_list[node_index];
			if node.state.is_terminal() {
				self.backpropagate(path, node.state.terminal_value());
				return;
			}

			let my_action = self.ucb1_action(&node.my_ucb_list, node.node_visit_count);
			let foe_action = self.ucb1_action(&node.foe_ucb_list, node.node_visit_count);

			if let Some(&child_node_index) = node.children.get(&(my_action, foe_action)) {
				path.push((node_index, my_action, foe_action));
				node_index = child_node_index;
				continue;
			}

			// --- Expansion ---
			let new_state = node.state.apply(my_action, foe_action);

			let new_node = MctsNode::new(new_state);

			let new_node_index = self.node_list.len();

			self.node_list.push(new_node);
			self.node_list[node_index]
				.children
				.insert((my_action, foe_action), new_node_index);
			path.push((node_index, my_action, foe_action));

			node_index = new_node_index;
			break;
		}

		// --- Evaluation ---
		let leaf_node = &self.node_list[node_index];
		let value = if leaf_node.state.is_terminal() {
			leaf_node.state.terminal_value()
		} else {
			self.rollout(&leaf_node.state)
		};

		// --- Backpropagation ---
		self.backpropagate(path, value);
	}

	fn ucb1_action(
		&mut self,
		player_ucb_list: &[UcbActionStats],
		parent_visit_count: VisitCount,
	) -> PlayerActionReprAsIndex {
		let mut best_action_list = Vec::new();
		let mut best_ucb = f32::NEG_INFINITY;

		for (i, stat) in player_ucb_list.iter().enumerate() {
			let ucb = if stat.visit_count == 0 {
				f32::INFINITY
			} else {
				let mean = stat.total_reward / stat.visit_count as f32;
				let exploration = Self::EXPLORATION_CONSTANT
					* ((parent_visit_count as f32).ln() / stat.visit_count as f32).sqrt();
				mean + exploration
			};

			// TODO: do not store all best action, and only keep one
			if ucb > best_ucb + 1e-6 {
				best_ucb = ucb;
				best_action_list.clear();
				best_action_list.push(i);
			} else if (ucb - best_ucb).abs() < 1e-6 {
				best_action_list.push(i);
			}
		}
		if best_action_list.is_empty() {
			// fallback, should not happen
			self.rng.random_range(0..player_ucb_list.len()) as PlayerActionReprAsIndex
		} else {
			best_action_list[self.rng.random_range(0..best_action_list.len())]
				as PlayerActionReprAsIndex
		}
	}

	fn rollout(&self, state: &S) -> HeuristicReward {
		state.evaluate()
	}

	fn backpropagate(
		&mut self,
		path: Vec<(NodeIndex, PlayerActionReprAsIndex, PlayerActionReprAsIndex)>,
		value: HeuristicReward,
	) {
		for (node_index, my_action, foe_action) in path.into_iter().rev() {
			let node = &mut self.node_list[node_index];

			node.node_visit_count += 1;

			let my_action = my_action as usize;
			let foe_action = foe_action as usize;
			node.my_ucb_list[my_action].visit_count += 1;
			node.my_ucb_list[my_action].total_reward += value;
			node.foe_ucb_list[foe_action].visit_count += 1;
			node.foe_ucb_list[foe_action].total_reward -= value;
		}
	}

	fn my_best_action(&self) -> PlayerActionReprAsIndex {
		let root = &self.node_list[self.root];
		let mut best = 0;
		let mut best_value = f32::NEG_INFINITY;
		for (i, stat) in root.my_ucb_list.iter().enumerate() {
			if stat.visit_count > 0 {
				let avg = stat.total_reward / stat.visit_count as f32;
				if avg > best_value {
					best_value = avg;
					best = i;
				}
			}
		}
		best as PlayerActionReprAsIndex
	}

	/// decode a player action into per‑agent action for the alive agents.
	/// the order corresponds to increasing agent index (first alive agent is most significant digit).
	fn decode_action(
		&self,
		action: PlayerActionReprAsIndex,
		my_agent_alive_count: usize,
	) -> Vec<PlayerActionReprAsIndex> {
		let mut agent_action_list = Vec::with_capacity(my_agent_alive_count);
		let mut rem = action;
		for _ in 0..my_agent_alive_count {
			agent_action_list.push(rem % 3);
			rem /= 3;
		}
		agent_action_list.reverse();
		agent_action_list
	}
}

impl<S: GameState> MctsNode<S> {
	fn new(state: S) -> Self {
		let my_action_count = state.my_action_count();
		let foe_action_count = state.foe_action_count();

		MctsNode {
			state,

			node_visit_count: 0,
			my_ucb_list: vec![UcbActionStats::default(); my_action_count],
			foe_ucb_list: vec![UcbActionStats::default(); foe_action_count],

			children: HashMap::new(),
		}
	}
}

fn main() {
	let env = Env::read();
	let mut my_snakebot_list = Vec::with_capacity(env.my_snakebot_id_list.len());

	loop {
		let start = SystemTime::now();

		let mut grid = env.base_grid.clone();

		let mut apple_list = Env::read_apple(&mut grid);
		env.read_snakebot(&mut grid, &mut my_snakebot_list);

		let action_list = my_snakebot_list
			.iter()
			.enumerate()
			.map(|(index, (id, body))| {
				let sub_start = SystemTime::now();

				let Some(allowed_time) = (MAX_TURN_DURATION.checked_sub(start.elapsed().unwrap()))
					.and_then(|remaining| {
						remaining.checked_div(env.my_snakebot_id_list.len() as u32 - index as u32)
					})
				else {
					eprintln!("not enough time for snakebot {id}, skipping");
					return Action {
						snakebot_id: *id,
						direction: Dir::U,
					}
					.to_string();
				};
				eprintln!("[{id}] allowed: {allowed_time:?}");

				for &(x, y) in body {
					grid[y][x] = Tile::Empty;
				}

				let (action, apple) =
					find_snakebot_action(&env, &grid, &apple_list, *id, body, allowed_time);

				if let Some(apple) = apple {
					grid[apple.1][apple.0] = Tile::Empty;
					apple_list.retain(|&(x, y)| x != apple.0 || y != apple.1);
				}
				let (nx, ny) = apply_dir(body[0], action.direction);
				grid[ny][nx] = Tile::Block;
				// TODO: test if tail is still block (take care of apple)
				for &(x, y) in body.iter() {
					grid[y][x] = Tile::Block;
				}

				let elapsed = sub_start.elapsed().unwrap();
				eprintln!("[{id}] took: {elapsed:?}");

				action.to_string()
			})
			.collect::<Vec<_>>()
			.join(";");

		println!("{action_list}");
	}
}
