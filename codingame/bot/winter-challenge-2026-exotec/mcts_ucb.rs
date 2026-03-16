use std::{
	fmt::Display,
	io,
	time::{Duration, Instant},
};

// TODO: must remove initially stuck snakebot before monte-carlo tree search

const MAX_TURN_DURATION: Duration = Duration::from_millis(45);
const MAX_TURN_COUNT: Turn = 200;
const MIN_SNAKEBOT_LEN: usize = 3;
const MAX_SNAKEBOT_PER_PLAYER: usize = 4;

type Turn = u8;

type SnakebotId = u8;

/// max 45w x 30h (=1350 tile)
/// snakebot can go out of bounds, but we don't expect under -128 or above 127-45=82
type Axis = i8;
type Coord = (Axis, Axis);

// TODO: check time optimization between u8 and usize
/// each snakebot has 3 possible actions (straight, left, right)
/// encode actions as base‑3 number over the alive agents in increasing order of agent index
/// this gives a unique incrementing index representing the permutation of actions for all agents
/// max 4 snakebot per player, and 3^4 = 81 (so < u8::MAX = 255)
type PlayerActionReprAsIndex = u8;
// type BothPlayerAction = (PlayerActionReprAsIndex, PlayerActionReprAsIndex);

type SnakebotAction = u8;
const STRAIGHT: SnakebotAction = 0;
const LEFT: SnakebotAction = 1;
const RIGHT: SnakebotAction = 2;

type DecodedAction = [SnakebotAction; MAX_SNAKEBOT_PER_PLAYER];

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

#[derive(Clone)]
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

	#[allow(clippy::type_complexity)]
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
				.map(|coord| {
					let mut parts = coord.split(",");
					let x = parse_input!(parts.next().unwrap(), Axis);
					let y = parse_input!(parts.next().unwrap(), Axis);
					(x, y)
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
	#[inline]
	fn index(x: Axis, y: Axis, w: Axis) -> (usize, usize) {
		let index = (y as usize * w as usize + x as usize) / 64;
		let bit = (y as usize * w as usize + x as usize) % 64;
		(index, bit)
	}

	fn read() -> Self {
		let mut s = String::new();

		io::stdin().read_line(&mut s).unwrap();
		let w = parse_input!(s, Axis);

		s.clear();
		io::stdin().read_line(&mut s).unwrap();
		let h = parse_input!(s, Axis);

		let d = vec![0; (w as usize * h as usize).div_ceil(64)];

		let mut g = BlockGrid { w, h, d };

		for y in 0..h {
			s.clear();
			io::stdin().read_line(&mut s).unwrap();
			for (x, c) in s.trim_matches('\n').chars().enumerate() {
				if c == '#' {
					g.set(x as Axis, y);
				}
			}
		}

		g
	}

	#[inline]
	fn is_block(&self, x: Axis, y: Axis) -> bool {
		if x < 0 || y < 0 || x >= self.w || y >= self.h {
			return false;
		}

		let (index, bit) = Self::index(x, y, self.w);
		(self.d[index] & (1 << bit)) != 0
	}

	#[inline]
	fn safe_set(&mut self, x: Axis, y: Axis) {
		if x < 0 || y < 0 || x >= self.w || y >= self.h {
			return;
		}

		self.set(x, y);
	}

	#[inline]
	fn set(&mut self, x: Axis, y: Axis) {
		let (index, bit) = Self::index(x, y, self.w);
		self.d[index] |= 1 << bit;
	}
}

struct Mcts<'e, S: GameStateTrait> {
	env: &'e Env,

	node_list: Vec<MctsNode<S>>,

	root: NodeIndex,
}

/// MCTS => Monte Carlo Tree Search
struct MctsNode<S: GameStateTrait> {
	state: S,

	node_visit_count: VisitCount,
	/// must be indexable by PlayerActionReprAsIndex
	my_ucb_list: Vec<UcbActionStats>,
	foe_ucb_list: Vec<UcbActionStats>,
	// children: HashMap<BothPlayerAction, NodeIndex>,
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
		env: &Env,
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
		let mut agent_action_list = [0; MAX_SNAKEBOT_PER_PLAYER];
		let mut rem = action;
		for i in 0..my_alive_agent_count {
			agent_action_list[my_alive_agent_count - 1 - i] = rem % 3;
			rem /= 3;
		}
		agent_action_list
	}
}

impl<'e, S: GameStateTrait> Mcts<'e, S> {
	const EXPLORATION_CONSTANT: f32 = 1.1;

	fn new(env: &'e Env, initial_state: S) -> Self {
		let root = MctsNode::new(initial_state);

		Mcts {
			env,

			node_list: vec![root],
			root: 0,
		}
	}

	fn search(&mut self) -> DecodedAction {
		let start = Instant::now();
		let mut iteration_count = 0;
		loop {
			self.iterate();

			iteration_count += 1;
			if iteration_count % 50 == 0 {
				let elapsed = start.elapsed();
				if elapsed >= MAX_TURN_DURATION {
					eprintln!("search took: {elapsed:?}\niteration count: {iteration_count}");
					break;
				}
			}
		}

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

			let my_action = Self::ucb1_action(&node.my_ucb_list, node.node_visit_count);
			let foe_action = Self::ucb1_action(&node.foe_ucb_list, node.node_visit_count);

			/*
			if let Some(&child_node_index) = node.children.get(&(my_action, foe_action)) {
				path.push((node_index, my_action, foe_action));
				node_index = child_node_index;
				continue;
			}
			*/

			// --- Expansion ---
			let new_state = node.state.apply(self.env, my_action, foe_action);

			let new_node = MctsNode::new(new_state);

			let new_node_index = self.node_list.len();

			self.node_list.push(new_node);
			/*
			self.node_list[node_index]
				.children
				.insert((my_action, foe_action), new_node_index);
			*/
			path.push((node_index, my_action, foe_action));

			node_index = new_node_index;
			break;
		}

		// --- Evaluation ---
		let leaf_node = &self.node_list[node_index];
		let value = if leaf_node.state.is_terminal() {
			leaf_node.state.terminal_value()
		} else {
			leaf_node.state.evaluate()
		};

		// --- Backpropagation ---
		self.backpropagate(path, value);
	}

	fn ucb1_action(
		player_ucb_list: &[UcbActionStats],
		parent_visit_count: VisitCount,
	) -> PlayerActionReprAsIndex {
		let mut best = (PlayerActionReprAsIndex::default(), f32::NEG_INFINITY);

		for (i, stat) in player_ucb_list.iter().enumerate() {
			// TODO: validate if we want to force exploration
			if stat.visit_count == 0 {
				return i as PlayerActionReprAsIndex;
			}

			let mean = stat.total_reward / stat.visit_count as f32;
			let exploration = Self::EXPLORATION_CONSTANT
				* ((parent_visit_count as f32).ln() / stat.visit_count as f32).sqrt();
			let ucb = mean + exploration;

			if ucb > best.1 {
				best = (i as PlayerActionReprAsIndex, ucb);
			}
		}

		best.0
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
			// children: HashMap::new(),
		}
	}
}

#[derive(Clone)]
struct GameState {
	turn: Turn,

	// TODO: store as [Snakebot; MAX_SNAKEBOT_PER_PLAYER]
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
		} else if head_x < neck_x {
			Self::L
		} else {
			Self::R
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

impl GameState {
	fn my_raw_score(&self) -> usize {
		self.my_snakebot_list
			.iter()
			.map(|snakebot| snakebot.body.len())
			.sum()
	}

	fn foe_raw_score(&self) -> usize {
		self.foe_snakebot_list
			.iter()
			.map(|snakebot| snakebot.body.len())
			.sum()
	}
}

fn move_and_eat(
	snakebot_list: &mut [Snakebot],
	action_list: DecodedAction,
	apple_list: &AppleList,
	eaten_apple_index_list: &mut Vec<usize>,
) {
	for (snakebot, action) in snakebot_list.iter_mut().zip(action_list.into_iter()) {
		let new_dir = match action {
			STRAIGHT => snakebot.facing_dir,
			LEFT => snakebot.facing_dir.turn_left(),
			RIGHT => snakebot.facing_dir.turn_right(),
			_ => unreachable!(),
		};

		let (head_x, head_y) = snakebot.body[0];
		let new_head = match new_dir {
			Dir::U => (head_x, head_y - 1),
			Dir::R => (head_x + 1, head_y),
			Dir::D => (head_x, head_y + 1),
			Dir::L => (head_x - 1, head_y),
		};

		snakebot.body.insert(0, new_head);
		snakebot.facing_dir = new_dir;

		if let Some(apple_index) = apple_list.iter().position(|&apple| apple == new_head) {
			eaten_apple_index_list.push(apple_index);
		} else {
			snakebot.body.pop();
		}
	}
}

macro_rules! pop_from_collision {
	($dead_snakebot_index_list:expr, $dead_head_index_list:expr, $snakebot:expr, $index:expr) => {{
		if $snakebot.body.len() <= MIN_SNAKEBOT_LEN {
			$dead_snakebot_index_list.push($index);
		} else {
			$dead_head_index_list.push($index);
		}
	}};
}

fn apply_collision(
	snakebot_list: &mut [Snakebot],
	dead_snakebot_index_list: &mut Vec<usize>,
	dead_head_index_list: &mut Vec<usize>,
	grid: &BlockGrid,
	other_snakebot_list: &[Snakebot],
) {
	for (index, snakebot) in snakebot_list.iter().enumerate() {
		let (head_x, head_y) = snakebot.body[0];

		if grid.is_block(head_x, head_y) {
			pop_from_collision!(
				dead_snakebot_index_list,
				dead_head_index_list,
				snakebot,
				index
			);
			continue;
		}

		for other_snakebot in other_snakebot_list.iter() {
			if other_snakebot.body.contains(&(head_x, head_y)) {
				pop_from_collision!(
					dead_snakebot_index_list,
					dead_head_index_list,
					snakebot,
					index
				);
				break;
			}
		}

		for (ally_index, ally_snakebot) in snakebot_list.iter().enumerate() {
			if ally_index == index {
				if ally_snakebot.body[1..].contains(&(head_x, head_y)) {
					pop_from_collision!(
						dead_snakebot_index_list,
						dead_head_index_list,
						snakebot,
						index
					);
					break;
				}
			} else if ally_snakebot.body.contains(&(head_x, head_y)) {
				pop_from_collision!(
					dead_snakebot_index_list,
					dead_head_index_list,
					snakebot,
					index
				);
				break;
			}
		}
	}
}

// TODO: refactor
fn apply_gravity(
	my_snakebot_list: &mut Vec<Snakebot>,
	foe_snake_bot_list: &mut Vec<Snakebot>,
	grid: &BlockGrid,
	apple_list: &AppleList,
) {
	let mut extra_block = grid.clone();
	for apple in apple_list.iter() {
		extra_block.set(apple.0, apple.1);
	}

	let mut snakebot_fall_flag_list = vec![true; my_snakebot_list.len() + foe_snake_bot_list.len()];

	let mut remaining_fall_distance = grid.h;
	while snakebot_fall_flag_list.iter().any(|flag| *flag) && remaining_fall_distance > 0 {
		for snakebot_index in 0..my_snakebot_list.len() {
			if !snakebot_fall_flag_list[snakebot_index] {
				continue;
			}

			for body_part_index in 0..my_snakebot_list[snakebot_index].body.len() {
				my_snakebot_list[snakebot_index].body[body_part_index].1 += 1;
				if grid.is_block(
					my_snakebot_list[snakebot_index].body[body_part_index].0,
					my_snakebot_list[snakebot_index].body[body_part_index].1,
				) || extra_block.is_block(
					my_snakebot_list[snakebot_index].body[body_part_index].0,
					my_snakebot_list[snakebot_index].body[body_part_index].1,
				) {
					for i in 0..=body_part_index {
						my_snakebot_list[snakebot_index].body[i].1 -= 1;
					}

					extra_block.safe_set(
						my_snakebot_list[snakebot_index].body[body_part_index].0,
						my_snakebot_list[snakebot_index].body[body_part_index].1,
					);

					snakebot_fall_flag_list[snakebot_index] = false;

					for previous_snakebot_index in 0..snakebot_index {
						if !snakebot_fall_flag_list[previous_snakebot_index] {
							continue;
						}

						for body_part_index in
							0..my_snakebot_list[previous_snakebot_index].body.len()
						{
							my_snakebot_list[previous_snakebot_index].body[body_part_index].1 -= 1;
						}
					}
				}
			}
		}

		for snakebot_index in 0..foe_snake_bot_list.len() {
			if !snakebot_fall_flag_list[my_snakebot_list.len() + snakebot_index] {
				continue;
			}

			for body_part_index in 0..foe_snake_bot_list[snakebot_index].body.len() {
				foe_snake_bot_list[snakebot_index].body[body_part_index].1 += 1;
				if grid.is_block(
					foe_snake_bot_list[snakebot_index].body[body_part_index].0,
					foe_snake_bot_list[snakebot_index].body[body_part_index].1,
				) || extra_block.is_block(
					foe_snake_bot_list[snakebot_index].body[body_part_index].0,
					foe_snake_bot_list[snakebot_index].body[body_part_index].1,
				) {
					for i in 0..=body_part_index {
						foe_snake_bot_list[snakebot_index].body[i].1 -= 1;
					}

					extra_block.safe_set(
						foe_snake_bot_list[snakebot_index].body[body_part_index].0,
						foe_snake_bot_list[snakebot_index].body[body_part_index].1,
					);

					snakebot_fall_flag_list[my_snakebot_list.len() + snakebot_index] = false;

					for previous_snakebot_index in 0..snakebot_index {
						if !snakebot_fall_flag_list
							[my_snakebot_list.len() + previous_snakebot_index]
						{
							continue;
						}

						for body_part_index in
							0..foe_snake_bot_list[previous_snakebot_index].body.len()
						{
							foe_snake_bot_list[previous_snakebot_index].body[body_part_index].1 -=
								1;
						}
					}

					for my_snakebot_index in 0..my_snakebot_list.len() {
						if !snakebot_fall_flag_list[my_snakebot_index] {
							continue;
						}

						for body_part_index in 0..my_snakebot_list[my_snakebot_index].body.len() {
							my_snakebot_list[my_snakebot_index].body[body_part_index].1 -= 1;
						}
					}
				}
			}
		}

		remaining_fall_distance -= 1;
	}

	let my_snakebot_count = my_snakebot_list.len();
	for index in (0..my_snakebot_count + foe_snake_bot_list.len()).rev() {
		if !snakebot_fall_flag_list[index] {
			continue;
		}

		if index < my_snakebot_count {
			my_snakebot_list.remove(index);
		} else {
			foe_snake_bot_list.remove(index - my_snakebot_count);
		}
	}
}

fn remove_indices<T>(vec: &mut Vec<T>, mut indices: Vec<usize>) {
	if indices.is_empty() {
		return;
	}

	indices.sort_unstable();

	let mut write = 0;
	let mut read = 0;
	let mut next_remove = 0;

	while read < vec.len() {
		if next_remove < indices.len() && read == indices[next_remove] {
			next_remove += 1;
		} else {
			if write != read {
				vec.swap(write, read);
			}
			write += 1;
		}
		read += 1;
	}

	vec.truncate(write);
}

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
		env: &Env,
		my_action: PlayerActionReprAsIndex,
		foe_action: PlayerActionReprAsIndex,
	) -> Self {
		let mut my_snakebot_list = self.my_snakebot_list.clone();
		let mut foe_snakebot_list = self.foe_snakebot_list.clone();

		let my_decoded_action = self.decode_action(my_action);
		let foe_decoded_action = self.decode_action(foe_action);

		// TODO: try to set as global
		let mut eaten_apple_index_list = Vec::<usize>::new();

		move_and_eat(
			&mut my_snakebot_list,
			my_decoded_action,
			&self.apple_list,
			&mut eaten_apple_index_list,
		);
		move_and_eat(
			&mut foe_snakebot_list,
			foe_decoded_action,
			&self.apple_list,
			&mut eaten_apple_index_list,
		);

		let mut apple_list = self.apple_list.clone();
		if !eaten_apple_index_list.is_empty() {
			eaten_apple_index_list.sort_unstable_by(|a, b| b.cmp(a));
			eaten_apple_index_list.dedup();
			for i in eaten_apple_index_list.into_iter() {
				apple_list.swap_remove(i);
			}
		}

		let mut my_dead_snakebot_index_list = Vec::<usize>::new();
		let mut foe_dead_snakebot_index_list = Vec::<usize>::new();
		let mut my_snakebot_dead_head_list = Vec::<usize>::new();
		let mut foe_snakebot_dead_head_list = Vec::<usize>::new();

		apply_collision(
			&mut my_snakebot_list,
			&mut my_dead_snakebot_index_list,
			&mut my_snakebot_dead_head_list,
			&env.g,
			&foe_snakebot_list,
		);
		apply_collision(
			&mut foe_snakebot_list,
			&mut foe_dead_snakebot_index_list,
			&mut foe_snakebot_dead_head_list,
			&env.g,
			&my_snakebot_list,
		);

		if !my_snakebot_dead_head_list.is_empty() {
			for i in my_snakebot_dead_head_list.into_iter() {
				my_snakebot_list[i].body.remove(0);
			}
		}
		if !foe_snakebot_dead_head_list.is_empty() {
			for i in foe_snakebot_dead_head_list.into_iter() {
				foe_snakebot_list[i].body.remove(0);
			}
		}
		remove_indices(&mut my_snakebot_list, my_dead_snakebot_index_list);
		remove_indices(&mut foe_snakebot_list, foe_dead_snakebot_index_list);

		apply_gravity(
			&mut my_snakebot_list,
			&mut foe_snakebot_list,
			&env.g,
			&apple_list,
		);

		GameState {
			turn: self.turn + 1,
			my_snakebot_list,
			foe_snakebot_list,
			apple_list,
		}
	}

	fn evaluate(&self) -> HeuristicReward {
		let my_raw_score = self.my_raw_score();
		let foe_raw_score = self.foe_raw_score();

		let raw_score = if my_raw_score > foe_raw_score {
			(my_raw_score - foe_raw_score).pow(2) as f32
		} else if my_raw_score < foe_raw_score {
			-((foe_raw_score - my_raw_score).pow(2) as f32)
		} else {
			0.0
		};

		let my_avg_distance_to_apple = if self.my_snakebot_list.is_empty() {
			0.0
		} else {
			self.my_snakebot_list
				.iter()
				.map(|snakebot| {
					let head = snakebot.body[0];

					self.apple_list
						.iter()
						.map(|&apple| manhattan_distance(head, apple) as u16)
						.min()
						.unwrap_or(0)
				})
				.sum::<u16>() as f32
				/ self.my_snakebot_list.len() as f32
		};

		(raw_score * 10.0) + (1.0 / (my_avg_distance_to_apple + 1.0))
	}

	fn is_terminal(&self) -> bool {
		self.apple_list.is_empty()
			|| self.turn >= MAX_TURN_COUNT
			|| self.my_snakebot_list.is_empty()
			|| self.foe_snakebot_list.is_empty()
	}
	fn terminal_value(&self) -> HeuristicReward {
		let my_raw_score = self.my_raw_score();
		let foe_raw_score = self.foe_raw_score();

		if my_raw_score > foe_raw_score {
			(my_raw_score - foe_raw_score).pow(3) as f32
		} else if my_raw_score < foe_raw_score {
			-((foe_raw_score - my_raw_score).pow(3) as f32)
		} else {
			0.0
		}
	}
}

fn manhattan_distance(a: Coord, b: Coord) -> u8 {
	a.0.abs_diff(b.0) + a.1.abs_diff(b.1)
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_bench_mcts() {
		let env = Env {
			turn: 4,
			g: BlockGrid {
				w: 22,
				h: 12,
				d: vec![
					0,
					7734932360009220480,
					17149567210541469696,
					18446742012124758015,
					255,
				],
			},
			my_id: 1,
			my_snakebot_id_list: vec![3, 4, 5],
			foe_snakebot_id_list: vec![0, 1, 2],
		};

		let state = GameState {
			turn: 4,
			my_snakebot_list: vec![
				Snakebot {
					body: vec![(19, 3), (19, 4), (19, 5), (20, 5)],
					facing_dir: Dir::U,
				},
				Snakebot {
					body: vec![(20, 7), (19, 7), (19, 6), (18, 6), (17, 6)],
					facing_dir: Dir::R,
				},
				Snakebot {
					body: vec![(9, 3), (9, 2), (8, 2), (8, 3), (8, 4)],
					facing_dir: Dir::D,
				},
			],
			foe_snakebot_list: vec![
				Snakebot {
					body: vec![(0, 5), (0, 6), (1, 6), (1, 7)],
					facing_dir: Dir::U,
				},
				Snakebot {
					body: vec![(3, 6), (2, 6), (2, 7), (3, 7), (4, 7)],
					facing_dir: Dir::R,
				},
				Snakebot {
					body: vec![(12, 4), (12, 3), (12, 2), (13, 2), (13, 3)],
					facing_dir: Dir::D,
				},
			],
			apple_list: vec![
				(7, 4),
				(14, 4),
				(9, 5),
				(12, 5),
				(0, 0),
				(21, 0),
				(3, 1),
				(18, 1),
				(0, 3),
				(21, 3),
			],
		};

		let mut mcts = Mcts::new(&env, state);
		let _ = mcts.search();
	}

	#[test]
	fn test_decode_action() {
		let state = GameState {
			turn: 0,
			my_snakebot_list: vec![
				Snakebot {
					body: Vec::default(),
					facing_dir: Dir::U,
				};
				3
			],
			foe_snakebot_list: vec![],
			apple_list: vec![],
		};

		assert_eq!(
			state.decode_action(0),
			[STRAIGHT, STRAIGHT, STRAIGHT, STRAIGHT]
		);

		assert_eq!(state.decode_action(1), [STRAIGHT, STRAIGHT, LEFT, STRAIGHT]);

		assert_eq!(
			state.decode_action(2),
			[STRAIGHT, STRAIGHT, RIGHT, STRAIGHT]
		);

		assert_eq!(state.decode_action(3), [STRAIGHT, LEFT, STRAIGHT, STRAIGHT]);
	}
}
