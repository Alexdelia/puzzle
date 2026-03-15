use std::{
	collections::HashMap,
	fmt::Display,
	io,
	time::{Duration, Instant},
};

use rand::{Rng, rngs::ThreadRng};

// TODO: must remove initially stuck snakebot before monte-carlo tree search

const MAX_TURN_DURATION: Duration = Duration::from_millis(80);
const MAX_TURN_COUNT: Turn = 200;

type Turn = u8;

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

type SnakebotAction = u8;
const STRAIGHT: SnakebotAction = 0;
const LEFT: SnakebotAction = 1;
const RIGHT: SnakebotAction = 2;

type DecodedAction = Vec<SnakebotAction>;

/// technically fit in a u8, but always cast as a usize
type PlayerActionCount = usize;

type NodeIndex = usize;
type VisitCount = u32;

type HeuristicReward = f32;

struct Env {
	turn: Turn,
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
			turn: 0,

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

	fn read_snakebot(&self) -> (Vec<(SnakebotId, Snakebot)>, Vec<(SnakebotId, Snakebot)>) {
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

			let facing_dir = Dir::from(body.as_slice());
			let snakebot = Snakebot { body, facing_dir };

			if self.my_snakebot_id_list.contains(&snakebot_id) {
				my_snakebot_list.push((snakebot_id, snakebot));
			} else {
				foe_snakebot_list.push((snakebot_id, snakebot));
			}
		}

		my_snakebot_list.shrink_to_fit();
		foe_snakebot_list.shrink_to_fit();

		my_snakebot_list.sort_by_key(|(id, _)| *id);
		foe_snakebot_list.sort_by_key(|(id, _)| *id);

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

struct Mcts<'e, S: GameStateTrait> {
	env: &'e Env,

	node_list: Vec<MctsNode<S>>,

	root: NodeIndex,

	rng: ThreadRng,
}

/// MCTS => Monte Carlo Tree Search
struct MctsNode<S: GameStateTrait> {
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

trait GameStateTrait: Clone {
	fn my_alive_agent_count(&self) -> usize;

	fn my_action_count(&self) -> PlayerActionCount;
	fn foe_action_count(&self) -> PlayerActionCount;

	fn apply(
		&self,
		my_action: PlayerActionReprAsIndex,
		foe_action: PlayerActionReprAsIndex,
	) -> Self;

	fn evaluate(&self) -> HeuristicReward;

	fn is_terminal(&self) -> bool;
	fn terminal_value(&self) -> HeuristicReward;

	/// decode a player action into per‑agent action for the alive agents.
	/// the order corresponds to increasing agent index (first alive agent is most significant digit).
	fn decode_action(&self, action: PlayerActionReprAsIndex) -> DecodedAction {
		let my_alive_agent_count = self.my_alive_agent_count();
		let mut agent_action_list = Vec::with_capacity(my_alive_agent_count);
		let mut rem = action;
		for _ in 0..my_alive_agent_count {
			agent_action_list.push(rem % 3);
			rem /= 3;
		}
		agent_action_list.reverse();
		agent_action_list
	}
}

impl<'e, S: GameStateTrait> Mcts<'e, S> {
	const EXPLORATION_CONSTANT: f32 = 1.4;

	fn new(env: &'e Env, initial_state: S) -> Self {
		let root = MctsNode::new(initial_state);

		Mcts {
			env,

			node_list: vec![root],
			root: 0,

			rng: rand::rng(),
		}
	}

	fn search(&mut self) -> DecodedAction {
		let start = Instant::now();
		while start.elapsed() < MAX_TURN_DURATION {
			self.iterate();
		}
		eprintln!("search took: {:?}", start.elapsed());

		self.my_best_action()
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

			let my_action =
				Self::ucb1_action(&mut self.rng, &node.my_ucb_list, node.node_visit_count);
			let foe_action =
				Self::ucb1_action(&mut self.rng, &node.foe_ucb_list, node.node_visit_count);

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
		rng: &mut ThreadRng,
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
			rng.random_range(0..player_ucb_list.len()) as PlayerActionReprAsIndex
		} else {
			best_action_list[rng.random_range(0..best_action_list.len())] as PlayerActionReprAsIndex
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

	fn my_best_action(&self) -> DecodedAction {
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
		root.state.decode_action(best as PlayerActionReprAsIndex)
	}
}

impl<S: GameStateTrait> MctsNode<S> {
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

#[derive(Clone)]
struct GameState {
	turn: Turn,

	my_snakebot_list: Vec<Snakebot>,
	foe_snakebot_list: Vec<Snakebot>,

	apple_list: AppleList,
}

#[derive(Clone)]
struct Snakebot {
	body: Vec<Coord>,
	facing_dir: Dir,
}

#[derive(Clone, Copy)]
enum Dir {
	U,
	R,
	D,
	L,
}

impl Dir {
	fn turn_left(&self) -> Self {
		match self {
			Self::U => Self::L,
			Self::R => Self::U,
			Self::D => Self::R,
			Self::L => Self::D,
		}
	}

	fn turn_right(&self) -> Self {
		match self {
			Self::U => Self::R,
			Self::R => Self::D,
			Self::D => Self::L,
			Self::L => Self::U,
		}
	}
}

impl Display for Dir {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = match self {
			Self::U => "UP",
			Self::R => "RIGHT",
			Self::D => "DOWN",
			Self::L => "LEFT",
		};
		write!(f, "{s}")
	}
}

impl From<&[Coord]> for Dir {
	fn from(body: &[Coord]) -> Self {
		let (head_x, head_y) = body[0];
		let (neck_x, neck_y) = body[1];

		if head_x == neck_x {
			if head_y < neck_y { Self::U } else { Self::D }
		} else {
			if head_x < neck_x { Self::L } else { Self::R }
		}
	}
}

const ACTION_COUNT_BY_AGENT_COUNT: [PlayerActionCount; 5] = [
	1,  // 0
	3,  // 1
	9,  // 2
	27, // 3
	81, // 4
];

impl GameStateTrait for GameState {
	fn my_alive_agent_count(&self) -> usize {
		self.my_snakebot_list.len()
	}

	fn my_action_count(&self) -> PlayerActionCount {
		ACTION_COUNT_BY_AGENT_COUNT[self.my_alive_agent_count()]
	}
	fn foe_action_count(&self) -> PlayerActionCount {
		ACTION_COUNT_BY_AGENT_COUNT[self.foe_snakebot_list.len()]
	}

	fn apply(
		&self,
		my_action: PlayerActionReprAsIndex,
		foe_action: PlayerActionReprAsIndex,
	) -> Self {
		todo!()
	}

	fn evaluate(&self) -> HeuristicReward {
		todo!()
	}

	fn is_terminal(&self) -> bool {
		todo!()
	}
	fn terminal_value(&self) -> HeuristicReward {
		todo!()
	}
}

fn print_action(list: &[(SnakebotId, SnakebotAction, Dir)]) {
	let s = list
		.iter()
		.map(|(id, action, facing_dir)| {
			let dir = match *action {
				STRAIGHT => *facing_dir,
				LEFT => facing_dir.turn_left(),
				_ => facing_dir.turn_right(),
			};

			format!("{id} {dir}")
		})
		.collect::<Vec<_>>()
		.join(";");
	println!("{s}");
}

fn main() {
	let mut env = Env::read();

	loop {
		let apple_list = Env::read_apple();
		let (my_snakebot_list, foe_snakebot_list) = env.read_snakebot();

		// TODO: remove initially stuck snakebot (if any)
		let stuck_snakebot_id_list = Vec::<(SnakebotId, Snakebot)>::default();

		let state = GameState {
			turn: env.turn,
			my_snakebot_list: my_snakebot_list
				.iter()
				.map(|(_, snakebot)| snakebot.clone())
				.collect(),
			foe_snakebot_list: foe_snakebot_list
				.iter()
				.map(|(_, snakebot)| snakebot.clone())
				.collect(),
			apple_list,
		};

		let mut mcts = Mcts::new(&env, state);
		let best_action = mcts.search();

		let mut grouped_snakebot_action_list = Vec::with_capacity(my_snakebot_list.len());
		for ((id, snakebot), action) in my_snakebot_list.into_iter().zip(best_action.into_iter()) {
			grouped_snakebot_action_list.push((id, action, snakebot.facing_dir));
		}
		for (id, snakebot) in stuck_snakebot_id_list.into_iter() {
			// TODO: compute best action to stay alive instead of always straight
			grouped_snakebot_action_list.push((id, STRAIGHT, snakebot.facing_dir));
		}

		print_action(&grouped_snakebot_action_list);

		env.turn += 1;
	}
}
