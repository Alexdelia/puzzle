#[allow(dead_code)]
#[path = "../../solution/nova.rs"]
pub mod nova;

use engine::{Command, MAX_NODES, Map, Point, State, bit};
use nova::{
	ATTACK_BASE, Features, GLOBAL_FEATURES, Input, MAX_LIVE, NODE_FEATURES, Net, Rng, SPAWN_BASE,
	TRIANGLE_FEATURES, answer, commands, decide, features, read_state,
};
use pyo3::buffer::{Element, PyBuffer};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::io::BufRead;
use std::sync::Arc;
use std::time::Duration;
use tryangle::driver::{Driver, ProcessDriver};
use tryangle::game::{Answer, Game, Phase};
use tryangle::mapgen;

const LEAGUE: i32 = 3;
const BOT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
enum Opponent {
	SelfPlay,
	Frozen(Arc<Net>),
	Bot(String),
}

impl Opponent {
	fn parse(text: &str) -> Result<Opponent, String> {
		if text == "self" {
			return Ok(Opponent::SelfPlay);
		}
		if let Some(path) = text.strip_prefix("frozen:") {
			let bytes = std::fs::read(path).map_err(|e| format!("cannot read {path}: {e}"))?;
			return Ok(Opponent::Frozen(Arc::new(Net::load(&bytes))));
		}
		if let Some(command) = text.strip_prefix("bot:") {
			return Ok(Opponent::Bot(command.to_string()));
		}
		Err(format!(
			"unknown opponent {text}: use self, frozen:<weights.bin> or bot:<command>"
		))
	}
}

enum Seat {
	Python,
	Frozen(Arc<Net>, Rng),
	Bot(ProcessDriver),
}

#[derive(Default)]
struct Viewer {
	map: Option<Map>,
	me: Option<usize>,
	turn: i32,
	state: Option<State>,
	features: Option<Features>,
}

impl Viewer {
	fn observe(&mut self, text: &str) {
		let mut input = Input {
			lines: text.as_bytes().lines(),
		};
		if self.map.is_none() {
			let count = input.int() as usize;
			let mut points = vec![Point { x: 0, y: 0 }; count];
			for _ in 0..count {
				let row = input.ints();
				points[row[0] as usize] = Point {
					x: row[1] as i32,
					y: row[2] as i32,
				};
			}
			self.map = Some(Map::new(points));
		}
		let map = self.map.as_ref().expect("the map was read");
		let (state, me) = read_state(&mut input, map, self.me);
		self.me = Some(me);
		self.turn += 1;
		self.features = Some(features(map, &state, me, self.turn));
		self.state = Some(state);
	}
}

struct Match {
	game: Game,
	seats: [Seat; 2],
	viewers: [Viewer; 2],
	asked: [bool; 2],
	scores: [i32; 2],
	units: [i32; 2],
	unit_reward: f32,
	seed: i64,
}

fn seat_for(opponent: &Opponent, seed: i64) -> Result<Seat, String> {
	Ok(match opponent {
		Opponent::SelfPlay => Seat::Python,
		Opponent::Frozen(net) => Seat::Frozen(
			net.clone(),
			Rng((seed as u64).wrapping_mul(0x9E3779B97F4A7C15) | 1),
		),
		Opponent::Bot(command) => Seat::Bot(ProcessDriver::spawn(command, BOT_TIMEOUT)?),
	})
}

impl Match {
	fn start(seed: i64, opponent: &Opponent, python_seat: usize) -> Result<Match, String> {
		let board = mapgen::generate(seed);
		let other = seat_for(opponent, seed)?;
		let seats = if python_seat == 0 {
			[Seat::Python, other]
		} else {
			[other, Seat::Python]
		};
		let mut game = Match {
			game: Game::new(board, LEAGUE),
			seats,
			viewers: [Viewer::default(), Viewer::default()],
			asked: [false; 2],
			scores: [0; 2],
			units: [0; 2],
			unit_reward: std::env::var("NOVA_UNIT_REWARD")
				.ok()
				.map(|w| w.parse::<f32>().expect("NOVA_UNIT_REWARD is a number"))
				.unwrap_or(0.0),
			seed,
		};
		game.units = [0, 1].map(|p| game.game.board.units_of(p));
		game.advance();
		Ok(game)
	}

	fn finished(&self) -> bool {
		self.game.is_over() || self.game.players.iter().any(|p| !p.active)
	}

	fn advance(&mut self) {
		self.asked = [false; 2];
		while !self.finished() {
			if let Phase::Ask(inputs) = self.game.begin_frame() {
				for (player, text) in inputs.iter().enumerate() {
					let Some(text) = text else { continue };
					self.asked[player] = true;
					match &mut self.seats[player] {
						Seat::Bot(driver) => {
							let _ = driver.send(text);
						}
						_ => self.viewers[player].observe(text),
					}
				}
				if self.asked.contains(&true) {
					return;
				}
			}
		}
	}

	fn respond(&mut self, player: usize, moves: &[i32], triangles: &[i32]) -> Option<Answer> {
		if !self.asked[player] {
			return None;
		}
		let viewer = &self.viewers[player];
		Some(match &mut self.seats[player] {
			Seat::Bot(driver) => match driver.answer() {
				Ok(line) => Answer::Line(line),
				Err(_) => Answer::Timeout,
			},
			Seat::Frozen(net, rng) => {
				let (map, state, me, seen) = viewer_parts(viewer);
				let decision = decide(net, map, state, me, seen);
				Answer::Line(answer(
					map,
					&commands(map, state, me, seen, &decision, Some(rng)),
				))
			}
			Seat::Python => {
				let (map, state, me, seen) = viewer_parts(viewer);
				Answer::Line(command_line(map, state, me, seen, moves, triangles))
			}
		})
	}

	fn play(&mut self, moves: [&[i32]; 2], triangles: [&[i32]; 2]) -> [f32; 2] {
		let answers = [0, 1].map(|p| self.respond(p, moves[p], triangles[p]));
		self.game.end_frame(self.asked, answers);
		self.advance();
		let scores = self.game.scores();
		let gained = [scores[0] - self.scores[0], scores[1] - self.scores[1]];
		self.scores = scores;
		let units = [0, 1].map(|p| self.game.board.units_of(p));
		let grown = [units[0] - self.units[0], units[1] - self.units[1]];
		self.units = units;
		let shaped = self.unit_reward * (grown[0] - grown[1]) as f32;
		[
			(gained[0] - gained[1]) as f32 + shaped,
			(gained[1] - gained[0]) as f32 - shaped,
		]
	}
}

fn viewer_parts(viewer: &Viewer) -> (&Map, &State, usize, &Features) {
	(
		viewer.map.as_ref().expect("an asked seat has a map"),
		viewer.state.as_ref().expect("an asked seat has a state"),
		viewer.me.expect("an asked seat knows its side"),
		viewer
			.features
			.as_ref()
			.expect("an asked seat has features"),
	)
}

fn command_line(
	map: &Map,
	state: &State,
	me: usize,
	seen: &Features,
	moves: &[i32],
	triangles: &[i32],
) -> String {
	let mut reserved = [0i32; MAX_NODES];
	let mut orders = Vec::new();
	for (index, &t) in seen.live.iter().enumerate() {
		if !seen.usable[index] {
			continue;
		}
		let action = triangles[index];
		let corners = map.triangles[t];
		if action >= ATTACK_BASE {
			let target = (action - ATTACK_BASE) as usize;
			let affordable = corners
				.iter()
				.all(|&c| state.units[c][me] - reserved[c] >= 1);
			if target < MAX_NODES && seen.attack_targets[index] & bit(target) != 0 && affordable {
				for &c in &corners {
					reserved[c] += 1;
				}
				orders.push(Command::Attack {
					triangle: t,
					target,
				});
			}
		} else if action >= SPAWN_BASE {
			orders.push(Command::Spawn {
				triangle: t,
				node: corners[(action - SPAWN_BASE) as usize],
			});
		}
	}
	for v in 0..map.len() {
		let mut left = state.units[v][me] - reserved[v];
		for w in 0..map.len() {
			let amount = moves[v * MAX_NODES + w].min(left);
			if w != v && amount > 0 && state.connected(v, w) {
				orders.push(Command::Move {
					from: v,
					to: w,
					amount,
				});
				left -= amount;
			}
		}
	}
	answer(map, &orders)
}

struct SlotObservation {
	nodes: Vec<f32>,
	adjacency: Vec<i64>,
	units: Vec<i32>,
	globals: Vec<f32>,
	corners: Vec<i32>,
	triangles: Vec<f32>,
	attack: Vec<i64>,
	count: i32,
	controlled: u8,
}

impl SlotObservation {
	fn empty() -> SlotObservation {
		SlotObservation {
			nodes: vec![0.0; MAX_NODES * NODE_FEATURES],
			adjacency: vec![0; MAX_NODES],
			units: vec![0; MAX_NODES],
			globals: vec![0.0; GLOBAL_FEATURES],
			corners: vec![-1; MAX_LIVE * 3],
			triangles: vec![0.0; MAX_LIVE * TRIANGLE_FEATURES],
			attack: vec![0; MAX_LIVE],
			count: 0,
			controlled: 0,
		}
	}

	fn of(game: &Match, player: usize) -> SlotObservation {
		let mut slot = SlotObservation::empty();
		if !game.asked[player] || !matches!(game.seats[player], Seat::Python) {
			return slot;
		}
		let (map, state, me, seen) = viewer_parts(&game.viewers[player]);
		slot.controlled = 1;
		slot.count = map.len() as i32;
		for (n, row) in seen.nodes.iter().enumerate() {
			slot.nodes[n * NODE_FEATURES..(n + 1) * NODE_FEATURES].copy_from_slice(row);
			slot.adjacency[n] = state.adjacency[n] as i64;
			slot.units[n] = state.units[n][me];
		}
		slot.globals.copy_from_slice(&seen.globals);
		for (index, &t) in seen.live.iter().enumerate() {
			let corners = map.triangles[t];
			for k in 0..3 {
				slot.corners[index * 3 + k] = corners[k] as i32;
			}
			slot.triangles[index * TRIANGLE_FEATURES..(index + 1) * TRIANGLE_FEATURES]
				.copy_from_slice(&seen.triangles[index]);
			slot.attack[index] = seen.attack_targets[index] as i64;
		}
		slot
	}
}

struct StepResult {
	slots: [SlotObservation; 2],
	reward: [f32; 2],
	done: bool,
	scores: [i32; 2],
	error: Option<String>,
}

#[pyclass(unsendable)]
pub struct VecEnv {
	games: Vec<Match>,
	opponents: Vec<Opponent>,
	assignment: Vec<usize>,
	next_seed: i64,
	threads: usize,
	finished: u64,
}

fn input<T: Element + Copy>(
	buffers: &Bound<'_, PyDict>,
	key: &str,
	want: usize,
) -> PyResult<Vec<T>> {
	let item = buffers
		.get_item(key)?
		.ok_or_else(|| PyValueError::new_err(format!("missing buffer {key}")))?;
	let buffer: PyBuffer<T> = PyBuffer::get(&item)?;
	if buffer.item_count() != want {
		return Err(PyValueError::new_err(format!(
			"buffer {key} holds {} items, expected {want}",
			buffer.item_count()
		)));
	}
	buffer.to_vec(buffers.py())
}

fn output<T: Element + Copy>(buffers: &Bound<'_, PyDict>, key: &str, values: &[T]) -> PyResult<()> {
	let item = buffers
		.get_item(key)?
		.ok_or_else(|| PyValueError::new_err(format!("missing buffer {key}")))?;
	let buffer: PyBuffer<T> = PyBuffer::get(&item)?;
	if buffer.item_count() != values.len() {
		return Err(PyValueError::new_err(format!(
			"buffer {key} holds {} items, expected {}",
			buffer.item_count(),
			values.len()
		)));
	}
	buffer.copy_from_slice(buffers.py(), values)
}

fn publish(buffers: &Bound<'_, PyDict>, results: &[StepResult]) -> PyResult<()> {
	let slots: Vec<&SlotObservation> = results.iter().flat_map(|r| r.slots.iter()).collect();
	let gather_f32 = |pick: fn(&SlotObservation) -> &Vec<f32>| -> Vec<f32> {
		slots.iter().flat_map(|s| pick(s).iter().copied()).collect()
	};
	let gather_i64 = |pick: fn(&SlotObservation) -> &Vec<i64>| -> Vec<i64> {
		slots.iter().flat_map(|s| pick(s).iter().copied()).collect()
	};
	let gather_i32 = |pick: fn(&SlotObservation) -> &Vec<i32>| -> Vec<i32> {
		slots.iter().flat_map(|s| pick(s).iter().copied()).collect()
	};
	output(buffers, "nodes", &gather_f32(|s| &s.nodes))?;
	output(buffers, "adjacency", &gather_i64(|s| &s.adjacency))?;
	output(buffers, "units", &gather_i32(|s| &s.units))?;
	output(buffers, "globals", &gather_f32(|s| &s.globals))?;
	output(buffers, "corners", &gather_i32(|s| &s.corners))?;
	output(buffers, "triangles", &gather_f32(|s| &s.triangles))?;
	output(buffers, "attack", &gather_i64(|s| &s.attack))?;
	output(
		buffers,
		"count",
		&slots.iter().map(|s| s.count).collect::<Vec<i32>>(),
	)?;
	output(
		buffers,
		"controlled",
		&slots.iter().map(|s| s.controlled).collect::<Vec<u8>>(),
	)?;
	output(
		buffers,
		"reward",
		&results.iter().flat_map(|r| r.reward).collect::<Vec<f32>>(),
	)?;
	output(
		buffers,
		"done",
		&results
			.iter()
			.flat_map(|r| [u8::from(r.done); 2])
			.collect::<Vec<u8>>(),
	)?;
	let scores: Vec<i32> = results
		.iter()
		.flat_map(|r| [r.scores[0], r.scores[1], r.scores[1], r.scores[0]])
		.collect();
	output(buffers, "scores", &scores)
}

#[pymethods]
impl VecEnv {
	#[new]
	#[pyo3(signature = (games, seed, opponents, threads = 8))]
	fn new(games: usize, seed: i64, opponents: Vec<String>, threads: usize) -> PyResult<VecEnv> {
		let opponents: Vec<Opponent> = opponents
			.iter()
			.map(|text| Opponent::parse(text).map_err(PyValueError::new_err))
			.collect::<PyResult<_>>()?;
		if opponents.is_empty() {
			return Err(PyValueError::new_err("give at least one opponent"));
		}
		let assignment: Vec<usize> = (0..games).map(|g| g % opponents.len()).collect();
		let started: Vec<Match> = (0..games)
			.map(|g| Match::start(seed + g as i64, &opponents[assignment[g]], g % 2))
			.collect::<Result<_, String>>()
			.map_err(PyValueError::new_err)?;
		Ok(VecEnv {
			games: started,
			opponents,
			assignment,
			next_seed: seed + games as i64,
			threads,
			finished: 0,
		})
	}

	#[getter]
	fn slots(&self) -> usize {
		self.games.len() * 2
	}

	#[getter]
	fn finished(&self) -> u64 {
		self.finished
	}

	fn seeds(&self) -> Vec<i64> {
		self.games.iter().map(|g| g.seed).collect()
	}

	fn opponent_of(&self) -> Vec<usize> {
		self.assignment.clone()
	}

	fn observe(&self, buffers: &Bound<'_, PyDict>) -> PyResult<()> {
		let results: Vec<StepResult> = self
			.games
			.iter()
			.map(|game| StepResult {
				slots: [SlotObservation::of(game, 0), SlotObservation::of(game, 1)],
				reward: [0.0; 2],
				done: false,
				scores: game.scores,
				error: None,
			})
			.collect();
		publish(buffers, &results)
	}

	fn step(&mut self, py: Python<'_>, buffers: &Bound<'_, PyDict>) -> PyResult<()> {
		let slots = self.slots();
		let moves: Vec<i32> = input(buffers, "moves", slots * MAX_NODES * MAX_NODES)?;
		let triangles: Vec<i32> = input(buffers, "actions", slots * MAX_LIVE)?;
		let per = self.games.len().div_ceil(self.threads.max(1)).max(1);
		let base_seed = self.next_seed;
		let opponents = &self.opponents;
		let assignment = &self.assignment;
		let games = &mut self.games;
		let results: Vec<StepResult> = py.allow_threads(|| {
			std::thread::scope(|scope| {
				let handles: Vec<_> = games
					.chunks_mut(per)
					.enumerate()
					.map(|(chunk, part)| {
						let moves = &moves;
						let triangles = &triangles;
						scope.spawn(move || {
							part.iter_mut()
								.enumerate()
								.map(|(offset, game)| {
									let index = chunk * per + offset;
									let slot_moves = [0, 1].map(|p| {
										let slot = index * 2 + p;
										&moves[slot * MAX_NODES * MAX_NODES
											..(slot + 1) * MAX_NODES * MAX_NODES]
									});
									let slot_triangles = [0, 1].map(|p| {
										let slot = index * 2 + p;
										&triangles[slot * MAX_LIVE..(slot + 1) * MAX_LIVE]
									});
									let reward = game.play(slot_moves, slot_triangles);
									let done = game.finished();
									let scores = game.scores;
									let mut error = None;
									if done {
										let seed = base_seed + index as i64;
										let python_seat = usize::from(
											matches!(game.seats[1], Seat::Python)
												&& !matches!(game.seats[0], Seat::Python),
										);
										match Match::start(
											seed,
											&opponents[assignment[index]],
											python_seat ^ 1,
										) {
											Ok(fresh) => *game = fresh,
											Err(why) => error = Some(why),
										}
									}
									StepResult {
										slots: [
											SlotObservation::of(game, 0),
											SlotObservation::of(game, 1),
										],
										reward,
										done,
										scores,
										error,
									}
								})
								.collect::<Vec<_>>()
						})
					})
					.collect();
				handles
					.into_iter()
					.flat_map(|h| h.join().expect("a game thread panicked"))
					.collect()
			})
		});
		if let Some(why) = results.iter().find_map(|r| r.error.clone()) {
			return Err(PyValueError::new_err(why));
		}
		let finished = results.iter().filter(|r| r.done).count() as u64;
		self.finished += finished;
		if finished > 0 {
			self.next_seed += self.games.len() as i64;
		}
		publish(buffers, &results)
	}
}

#[pymodule]
fn novaenv(module: &Bound<'_, PyModule>) -> PyResult<()> {
	module.add_class::<VecEnv>()?;
	Ok(())
}
