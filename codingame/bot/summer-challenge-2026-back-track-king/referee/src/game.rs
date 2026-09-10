use crate::map::{Map, NO_TOWN};
use std::collections::BinaryHeap;

pub const MAX_TURNS: usize = 100;
pub const PAINT_PER_TURN: u32 = 3;
pub const INK_THRESHOLD: u8 = 4;

pub const NO_TRACK: i8 = -1;
pub const NEUTRAL: i8 = 2;

const UNREACHED: u32 = u32::MAX;
const HOP_SCALE: u32 = 1 << 12;

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overflow {
	Stop,
	Skip,
}

#[derive(Clone, Copy, Debug)]
pub struct Rules {
	pub max_turns: usize,
	pub paint: u32,
	pub ink_threshold: u8,
	pub disruption: bool,
	pub neutral_scores: bool,
	pub strict: bool,
	pub overflow: Overflow,
}

impl Default for Rules {
	fn default() -> Self {
		Rules {
			max_turns: MAX_TURNS,
			paint: PAINT_PER_TURN,
			ink_threshold: INK_THRESHOLD,
			disruption: false,
			neutral_scores: false,
			strict: false,
			overflow: Overflow::Stop,
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
	Place(usize, usize),
	Autoplace(usize, usize, usize, usize),
	DisruptRegion(usize),
	DisruptCell(usize, usize),
	Message(String),
	Wait,
}

#[derive(Clone, Debug)]
pub struct Foul {
	pub player: usize,
	pub reason: String,
}

#[derive(Clone, Debug, Default)]
pub struct TurnReport {
	pub placed: [Vec<usize>; 2],
	pub gained: [u32; 2],
	pub inked: Vec<usize>,
	pub skipped: [Vec<String>; 2],
	pub messages: [Option<String>; 2],
}

#[derive(Clone, Debug)]
pub struct State {
	pub turn: usize,
	pub score: [u32; 2],
	pub owner: Vec<i8>,
	pub instability: Vec<u8>,
	pub inked: Vec<bool>,
}

impl State {
	pub fn new(map: &Map) -> Self {
		State {
			turn: 0,
			score: [0; 2],
			owner: vec![NO_TRACK; map.cells()],
			instability: vec![0; map.region_count],
			inked: vec![false; map.region_count],
		}
	}
}

pub fn parse_actions(line: &str) -> Result<Vec<Action>, String> {
	let mut out = Vec::new();
	for chunk in line.split(';') {
		let text = chunk.trim();
		if text.is_empty() {
			return Err("empty action".into());
		}
		let mut tokens = text.split_whitespace();
		let verb = tokens.next().unwrap();
		let rest: Vec<&str> = tokens.collect();
		let num = |i: usize| -> Result<usize, String> {
			rest[i]
				.parse::<i64>()
				.map_err(|_| format!("{verb}: `{}` is not an integer", rest[i]))
				.and_then(|v| {
					usize::try_from(v).map_err(|_| format!("{verb}: negative coordinate"))
				})
		};
		let action = match (verb, rest.len()) {
			("PLACE_TRACKS", 2) => Action::Place(num(0)?, num(1)?),
			("AUTOPLACE", 4) => Action::Autoplace(num(0)?, num(1)?, num(2)?, num(3)?),
			("DISRUPT", 1) => Action::DisruptRegion(num(0)?),
			("DISRUPT", 2) => Action::DisruptCell(num(0)?, num(1)?),
			("MESSAGE", _) => Action::Message(rest.join(" ")),
			("WAIT", 0) => Action::Wait,
			_ => return Err(format!("unknown action `{text}`")),
		};
		out.push(action);
	}
	validate_actions(&out)?;
	Ok(out)
}

pub fn validate_actions(actions: &[Action]) -> Result<(), String> {
	if actions.is_empty() {
		return Err("no action".into());
	}
	if actions
		.iter()
		.filter(|a| matches!(a, Action::Autoplace(..)))
		.count()
		> 1
	{
		return Err("more than one AUTOPLACE".into());
	}
	Ok(())
}

pub struct Connections {
	pub pairs: Vec<(u8, u8)>,
	pub paths: Vec<Vec<u32>>,
	pub live: Vec<bool>,
	pub cell_pairs: Vec<Vec<u16>>,
	targets: Vec<u8>,
	by_target: Vec<Vec<usize>>,
	dist: Vec<u32>,
	queue: std::collections::VecDeque<u32>,
}

impl Connections {
	pub fn new(map: &Map) -> Self {
		let mut pairs = Vec::new();
		for (src, town) in map.towns.iter().enumerate() {
			for &dst in &town.desired {
				pairs.push((src as u8, dst));
			}
		}
		pairs.sort_unstable();
		let mut targets: Vec<u8> = pairs.iter().map(|p| p.1).collect();
		targets.sort_unstable();
		targets.dedup();
		let by_target = targets
			.iter()
			.map(|&t| (0..pairs.len()).filter(|&i| pairs[i].1 == t).collect())
			.collect();
		Connections {
			paths: vec![Vec::new(); pairs.len()],
			live: vec![false; pairs.len()],
			cell_pairs: vec![Vec::new(); map.cells()],
			pairs,
			targets,
			by_target,
			dist: vec![UNREACHED; map.cells()],
			queue: Default::default(),
		}
	}

	fn is_node(map: &Map, state: &State, cell: usize) -> bool {
		map.town_at[cell] != NO_TOWN || state.owner[cell] != NO_TRACK
	}

	fn bfs(&mut self, map: &Map, state: &State, from: usize) {
		self.dist.fill(UNREACHED);
		self.queue.clear();
		self.dist[from] = 0;
		self.queue.push_back(from as u32);
		while let Some(cell) = self.queue.pop_front() {
			let next = self.dist[cell as usize] + 1;
			for &n in &map.neighbors[cell as usize] {
				if n < 0 {
					continue;
				}
				let n = n as usize;
				if self.dist[n] == UNREACHED && Self::is_node(map, state, n) {
					self.dist[n] = next;
					self.queue.push_back(n as u32);
				}
			}
		}
	}

	pub fn recompute(&mut self, map: &Map, state: &State) {
		for cell in self.cell_pairs.iter_mut() {
			cell.clear();
		}
		self.live.fill(false);
		for ti in 0..self.targets.len() {
			let target = &map.towns[self.targets[ti] as usize];
			let target_cell = map.idx(target.x, target.y);
			self.bfs(map, state, target_cell);
			for k in 0..self.by_target[ti].len() {
				let pi = self.by_target[ti][k];
				let src = &map.towns[self.pairs[pi].0 as usize];
				let mut cur = map.idx(src.x, src.y);
				if self.dist[cur] == UNREACHED {
					continue;
				}
				let path = &mut self.paths[pi];
				path.clear();
				path.push(cur as u32);
				while self.dist[cur] != 0 {
					let want = self.dist[cur] - 1;
					let step = map.neighbors[cur]
						.iter()
						.copied()
						.find(|&n| n >= 0 && self.dist[n as usize] == want)
						.expect("shortest path walk stalled");
					cur = step as usize;
					path.push(cur as u32);
				}
				self.live[pi] = true;
			}
		}
		for pi in 0..self.pairs.len() {
			if self.live[pi] {
				for &cell in &self.paths[pi] {
					self.cell_pairs[cell as usize].push(pi as u16);
				}
			}
		}
	}

	pub fn gains(&self, state: &State, rules: &Rules) -> [u32; 2] {
		let mut gain = [0u32; 2];
		for pi in 0..self.pairs.len() {
			if !self.live[pi] {
				continue;
			}
			for &cell in &self.paths[pi] {
				match state.owner[cell as usize] {
					0 => gain[0] += 1,
					1 => gain[1] += 1,
					NEUTRAL if rules.neutral_scores => {
						gain[0] += 1;
						gain[1] += 1;
					}
					_ => {}
				}
			}
		}
		gain
	}
}

pub struct Pathfinder {
	dist: Vec<u32>,
	heap: BinaryHeap<std::cmp::Reverse<(u32, u32)>>,
	seen: Vec<bool>,
	queue: std::collections::VecDeque<u32>,
}

impl Pathfinder {
	pub fn new(map: &Map) -> Self {
		Pathfinder {
			dist: vec![UNREACHED; map.cells()],
			heap: BinaryHeap::new(),
			seen: vec![false; map.cells()],
			queue: Default::default(),
		}
	}

	fn paint(map: &Map, state: &State, cell: usize) -> u32 {
		if map.town_at[cell] != NO_TOWN || state.owner[cell] != NO_TRACK {
			0
		} else {
			map.cost(cell)
		}
	}

	fn weight(map: &Map, state: &State, cell: usize) -> u32 {
		Self::paint(map, state, cell) * HOP_SCALE + 1
	}

	fn open(map: &Map, state: &State, cell: usize) -> bool {
		!state.inked[map.region_of(cell)]
	}

	pub fn connected(&mut self, map: &Map, state: &State, from: usize, to: usize) -> bool {
		if !Connections::is_node(map, state, from) || !Connections::is_node(map, state, to) {
			return false;
		}
		self.seen.fill(false);
		self.queue.clear();
		self.seen[from] = true;
		self.queue.push_back(from as u32);
		while let Some(cell) = self.queue.pop_front() {
			if cell as usize == to {
				return true;
			}
			for &n in &map.neighbors[cell as usize] {
				if n < 0 {
					continue;
				}
				let n = n as usize;
				if !self.seen[n] && Connections::is_node(map, state, n) {
					self.seen[n] = true;
					self.queue.push_back(n as u32);
				}
			}
		}
		false
	}

	pub fn cheapest(
		&mut self,
		map: &Map,
		state: &State,
		from: usize,
		to: usize,
		out: &mut Vec<usize>,
	) {
		out.clear();
		if !Self::open(map, state, from) || !Self::open(map, state, to) {
			return;
		}
		self.dist.fill(UNREACHED);
		self.heap.clear();
		self.dist[to] = Self::weight(map, state, to);
		self.heap
			.push(std::cmp::Reverse((self.dist[to], to as u32)));
		while let Some(std::cmp::Reverse((d, cell))) = self.heap.pop() {
			let cell = cell as usize;
			if d > self.dist[cell] {
				continue;
			}
			if cell == from {
				break;
			}
			for &n in &map.neighbors[cell] {
				if n < 0 {
					continue;
				}
				let n = n as usize;
				if !Self::open(map, state, n) {
					continue;
				}
				let nd = d + Self::weight(map, state, n);
				if nd < self.dist[n] {
					self.dist[n] = nd;
					self.heap.push(std::cmp::Reverse((nd, n as u32)));
				}
			}
		}
		if self.dist[from] == UNREACHED {
			return;
		}
		let mut cur = from;
		loop {
			if Self::paint(map, state, cur) > 0 {
				out.push(cur);
			}
			if cur == to {
				return;
			}
			let want = self.dist[cur] - Self::weight(map, state, cur);
			let step = map.neighbors[cur]
				.iter()
				.copied()
				.find(|&n| n >= 0 && self.dist[n as usize] == want)
				.expect("cheapest path walk stalled");
			cur = step as usize;
		}
	}
}

pub struct Game {
	pub map: Map,
	pub rules: Rules,
	pub state: State,
	pub conn: Connections,
	pathfinder: Pathfinder,
	scratch: Vec<usize>,
}

impl Game {
	pub fn new(map: Map, rules: Rules) -> Self {
		let state = State::new(&map);
		let mut conn = Connections::new(&map);
		conn.recompute(&map, &state);
		Game {
			pathfinder: Pathfinder::new(&map),
			scratch: Vec::new(),
			conn,
			state,
			map,
			rules,
		}
	}

	pub fn over(&self) -> bool {
		self.state.turn >= self.rules.max_turns
	}

	fn cell_of(&self, x: usize, y: usize) -> Option<usize> {
		(x < self.map.width && y < self.map.height).then(|| self.map.idx(x, y))
	}

	fn place_refusal(&self, cell: usize, claimed: &[usize]) -> Option<&'static str> {
		if self.map.town_at[cell] != NO_TOWN {
			Some("cell holds a town")
		} else if self.state.owner[cell] != NO_TRACK {
			Some("cell already holds a track")
		} else if self.state.inked[self.map.region_of(cell)] {
			Some("region is inked out")
		} else if claimed.contains(&cell) {
			Some("cell already painted this turn")
		} else {
			None
		}
	}

	fn disrupt_refusal(&self, region: usize) -> Option<&'static str> {
		if region >= self.map.region_count {
			Some("no such region")
		} else if self.map.region_has_town[region] {
			Some("region holds a town")
		} else if self.state.inked[region] {
			Some("region is already inked out")
		} else {
			None
		}
	}

	pub fn step(&mut self, actions: [&[Action]; 2]) -> Result<TurnReport, Foul> {
		let mut report = TurnReport::default();
		let mut disrupted = [None; 2];

		for player in 0..2 {
			let mut paint = self.rules.paint;
			let mut broke = false;
			let mut claimed: Vec<usize> = Vec::new();
			let mut wanted: Vec<usize> = Vec::new();
			let strict = self.rules.strict;
			let refuse = |player: usize, why: String| -> Result<(), Foul> {
				if strict {
					Err(Foul {
						player,
						reason: why,
					})
				} else {
					Ok(())
				}
			};

			for action in actions[player] {
				wanted.clear();
				let generated = matches!(action, Action::Autoplace(..));
				if broke && matches!(action, Action::Place(..) | Action::Autoplace(..)) {
					continue;
				}
				match action {
					Action::Wait => continue,
					Action::Message(text) => {
						report.messages[player] = Some(text.clone());
						continue;
					}
					Action::Place(x, y) => match self.cell_of(*x, *y) {
						Some(cell) => wanted.push(cell),
						None => {
							return Err(Foul {
								player,
								reason: format!("PLACE_TRACKS {x} {y} is off the map"),
							});
						}
					},
					Action::Autoplace(fx, fy, tx, ty) => {
						let (Some(from), Some(to)) =
							(self.cell_of(*fx, *fy), self.cell_of(*tx, *ty))
						else {
							return Err(Foul {
								player,
								reason: format!("AUTOPLACE {fx} {fy} {tx} {ty} is off the map"),
							});
						};
						if !self.pathfinder.connected(&self.map, &self.state, from, to) {
							self.pathfinder.cheapest(
								&self.map,
								&self.state,
								from,
								to,
								&mut self.scratch,
							);
							wanted.extend_from_slice(&self.scratch);
						}
					}
					Action::DisruptRegion(_) | Action::DisruptCell(..) => {
						if !self.rules.disruption {
							report.skipped[player].push("DISRUPT: inert in this league".into());
							continue;
						}
						let region = match action {
							Action::DisruptRegion(r) => *r,
							Action::DisruptCell(x, y) => match self.cell_of(*x, *y) {
								Some(cell) => self.map.region_of(cell),
								None => {
									return Err(Foul {
										player,
										reason: format!("DISRUPT {x} {y} is off the map"),
									});
								}
							},
							_ => unreachable!(),
						};
						if disrupted[player].is_some() {
							refuse(player, "more than one DISRUPT".into())?;
							report.skipped[player].push("DISRUPT: point already spent".into());
							continue;
						}
						if let Some(why) = self.disrupt_refusal(region) {
							refuse(player, format!("DISRUPT {region}: {why}"))?;
							report.skipped[player].push(format!("DISRUPT {region}: {why}"));
							continue;
						}
						disrupted[player] = Some(region);
						continue;
					}
				}

				for &cell in &wanted {
					let (x, y) = self.map.xy(cell);
					if let Some(why) = self.place_refusal(cell, &claimed) {
						if !generated {
							refuse(player, format!("PLACE_TRACKS {x} {y}: {why}"))?;
						}
						report.skipped[player].push(format!("PLACE_TRACKS {x} {y}: {why}"));
						continue;
					}
					let cost = self.map.cost(cell);
					if cost > paint {
						report.skipped[player]
							.push(format!("PLACE_TRACKS {x} {y}: not enough paint"));
						if self.rules.overflow == Overflow::Stop {
							broke = true;
							break;
						}
						continue;
					}
					paint -= cost;
					claimed.push(cell);
				}
			}
			report.placed[player] = claimed;
		}

		for player in 0..2 {
			let other = 1 - player;
			for i in 0..report.placed[player].len() {
				let cell = report.placed[player][i];
				self.state.owner[cell] = if report.placed[other].contains(&cell) {
					NEUTRAL
				} else {
					player as i8
				};
			}
		}

		for target in disrupted.into_iter().flatten() {
			let reached = (self.state.instability[target] + 1).min(self.rules.ink_threshold);
			self.state.instability[target] = reached;
			if reached >= self.rules.ink_threshold && !self.state.inked[target] {
				self.state.inked[target] = true;
				report.inked.push(target);
			}
		}
		for &region in &report.inked {
			for cell in 0..self.map.cells() {
				if self.map.region_of(cell) == region {
					self.state.owner[cell] = NO_TRACK;
				}
			}
		}

		if !report.placed[0].is_empty() || !report.placed[1].is_empty() || !report.inked.is_empty()
		{
			self.conn.recompute(&self.map, &self.state);
		}
		report.gained = self.conn.gains(&self.state, &self.rules);
		self.state.score[0] += report.gained[0];
		self.state.score[1] += report.gained[1];
		self.state.turn += 1;
		Ok(report)
	}
}
