use crate::action::{Action, Commands};
use crate::grid::{Coord, Grid, TRACK_NEUTRAL, TRACK_NONE};
use crate::pathfind::{autobuild, terrain_reachable, train_path};
use std::collections::BTreeMap;

pub const PASSIVE_INCOME: i32 = 3;
pub const STARTING_DOSH: i32 = 0;
pub const BLOT_POINTS_PER_TURN: i32 = 1;
pub const MAX_TURNS: i32 = 100;
pub const INSTABILITY_THRESHOLD_BASE: i32 = 4;
pub const INSTABILITY_THRESHOLD_INCREASE: i32 = 0;
/// `gameManager.setMaxTurns(400)`: the engine's own cap, which the 100-turn rule reaches first.
pub const ENGINE_MAX_TURNS: i32 = 400;
/// League 1 and 2 are solo tutorials; the real game starts at 3.
pub const DEFAULT_LEAGUE: i32 = 3;
/// `Player.getExpectedOutputLines()`.
pub const EXPECTED_OUTPUT_LINES: i32 = 1;

#[derive(Clone, Debug, Default)]
pub struct Player {
	pub score: i32,
	pub dosh: i32,
	pub blot_points: i32,
	pub message: Option<String>,
	pub intents: Vec<Action>,
	pub active: bool,
	pub deactivated_by: Option<String>,
}

impl Player {
	fn new() -> Self {
		Player {
			dosh: STARTING_DOSH,
			active: true,
			..Player::default()
		}
	}
}

/// The counters the official referee reports as game metadata.
#[derive(Clone, Debug, Default)]
pub struct Stats {
	pub placed_tracks: [i32; 2],
	pub on_plains: [i32; 2],
	pub on_river: [i32; 2],
	pub on_mountains: [i32; 2],
	pub zones_inked: [i32; 2],
	pub own_tracks_inked_out: [i32; 2],
	pub enemy_tracks_inked_out: [i32; 2],
	pub extra_tiles_in_connection: [i32; 2],
	pub autobuild_called: [i32; 2],
	pub ownership_sum: [f32; 2],
	pub ownership_count: [i32; 2],
}

impl Stats {
	pub fn average_ownership(&self, player: usize) -> f32 {
		if self.ownership_count[player] == 0 {
			0.0
		} else {
			self.ownership_sum[player] / self.ownership_count[player] as f32
		}
	}
}

/// What one turn's update did, for traces and the viewer-less arena.
#[derive(Clone, Debug, Default)]
pub struct TurnReport {
	pub built: Vec<(Coord, i8)>,
	pub disrupted: [Option<usize>; 2],
	pub inked: Vec<usize>,
	pub gained: [i32; 2],
	pub errors: Vec<String>,
}

pub struct Game {
	pub grid: Grid,
	pub players: [Player; 2],
	pub turn: i32,
	pub league: i32,
	pub instability_threshold: i32,
	pub in_tutorial: bool,
	pub stats: Stats,
	pub summary: Vec<String>,
	pub ended: bool,
	tutorial_inked_enemy_track: bool,
	successful_blots: BTreeMap<usize, usize>,
}

impl Game {
	pub fn new(grid: Grid, league: i32) -> Self {
		Game {
			grid,
			players: [Player::new(), Player::new()],
			turn: 0,
			league,
			instability_threshold: INSTABILITY_THRESHOLD_BASE,
			in_tutorial: league == 1 || league == 2,
			stats: Stats::default(),
			summary: Vec::new(),
			ended: false,
			tutorial_inked_enemy_track: false,
			successful_blots: BTreeMap::new(),
		}
	}

	pub fn active_players(&self) -> usize {
		self.players.iter().filter(|player| player.active).count()
	}

	/// `Referee.gameTurn`'s first move: last turn's intents and messages are dropped.
	pub fn reset_turn_data(&mut self) {
		for player in &mut self.players {
			player.intents.clear();
			player.message = None;
		}
	}

	/// Hands one player's output line to the game, disqualifying them if it does not parse.
	pub fn take_commands(&mut self, player: usize, line: &str) {
		let Commands {
			intents,
			message,
			rejected,
		} = crate::action::parse(line);
		self.players[player].intents = intents;
		if let Some(text) = message {
			self.players[player].message = Some(text);
		}
		if let Some(why) = rejected {
			self.disqualify(player, why);
		}
	}

	pub fn timed_out(&mut self, player: usize) {
		self.players[player].active = false;
		self.players[player].deactivated_by = Some("Timeout!".into());
		self.summary.push(format!(
			"${player} has not provided {EXPECTED_OUTPUT_LINES} lines in time"
		));
	}

	fn disqualify(&mut self, player: usize, why: String) {
		self.players[player].active = false;
		self.players[player].deactivated_by = Some(why.clone());
		self.players[player].score = -1;
		self.summary.push(why);
		self.summary
			.push(error_message(&format!("${player}: disqualified!")));
	}

	pub fn perform_update(&mut self) -> TurnReport {
		let mut report = TurnReport::default();
		self.turn += 1;

		self.do_income();
		self.compute_autobuilds(&mut report);
		self.do_actions(&mut report);
		self.do_instability_check(&mut report);
		self.move_trains(&mut report);
		self.compute_tile_states();

		if self.is_game_over() {
			self.ended = true;
		}
		report
	}

	fn do_income(&mut self) {
		for player in &mut self.players {
			player.dosh = PASSIVE_INCOME;
			player.blot_points = BLOT_POINTS_PER_TURN;
		}
	}

	fn report_error(&mut self, report: &mut TurnReport, player: usize, message: String) {
		let line = error_message(&format!("${player} {message}"));
		report.errors.push(line.clone());
		self.summary.push(line);
	}

	fn compute_autobuilds(&mut self, report: &mut TurnReport) {
		for player in 0..2 {
			let mut used = false;
			let mut resolved: Vec<Action> = Vec::new();
			for intent in std::mem::take(&mut self.players[player].intents) {
				let Action::Autoplace { from, to } = intent else {
					resolved.push(intent);
					continue;
				};
				if used {
					self.report_error(
						report,
						player,
						"Only one autobuild action allowed per turn.".into(),
					);
					continue;
				}
				resolved.extend(autobuild(&self.grid, from, to).into_iter().map(|at| {
					Action::Place {
						at,
						autobuilt: true,
					}
				}));
				used = true;
				self.stats.autobuild_called[player] += 1;
			}
			self.players[player].intents = resolved;
		}
	}

	fn do_actions(&mut self, report: &mut TurnReport) {
		let mut claims: BTreeMap<Coord, Vec<usize>> = BTreeMap::new();
		let mut order: Vec<Coord> = Vec::new();

		for player in 0..2 {
			let mut autobuild_interrupted = false;
			for intent in self.players[player].intents.clone() {
				if autobuild_interrupted && intent.is_autobuilt() {
					continue;
				}
				let Action::Place { at, autobuilt } = intent else {
					continue;
				};
				let Some(tile) = self.grid.get(at) else {
					self.report_error(report, player, format!("Not part of grid: {at}"));
					continue;
				};
				if tile.is_town() {
					self.report_error(
						report,
						player,
						format!("Cannot place tracks on a town at {at}"),
					);
					continue;
				}
				let zone = self.grid.zone_of(at);
				if zone.inked {
					let id = zone.id;
					self.report_error(report, player, format!("Cannot build in region {id}"));
					continue;
				}
				let taken = claims.get(&at).is_some_and(|by| by.contains(&player));
				if self.grid.tile(at).track != TRACK_NONE || taken {
					self.report_error(
						report,
						player,
						format!("Cannot place tracks on existing tracks at {at}"),
					);
					continue;
				}
				let cost = self.grid.tile(at).rail_cost();
				if self.players[player].dosh < cost {
					if autobuilt {
						self.report_error(
							report,
							player,
							format!(
								"Autobuild interrupted: not enough track points to build a track at {at}."
							),
						);
						autobuild_interrupted = true;
					} else {
						self.report_error(
							report,
							player,
							format!("Not enough track points to build a track at {at}."),
						);
					}
					continue;
				}

				claims.entry(at).or_default().push(player);
				order.retain(|&other| other != at);
				order.push(at);

				self.players[player].dosh -= cost;
				self.stats.placed_tracks[player] += 1;
				let tile = self.grid.tile(at);
				if tile.is_mountain() {
					self.stats.on_mountains[player] += 1;
				} else if tile.is_water() {
					self.stats.on_river[player] += 1;
				} else if tile.is_plains() {
					self.stats.on_plains[player] += 1;
				}
			}
		}

		for at in order {
			let by = &claims[&at];
			let owner = if by.len() == 1 {
				by[0] as i8
			} else {
				TRACK_NEUTRAL
			};
			self.grid.tile_mut(at).track = owner;
			report.built.push((at, owner));
		}

		for player in 0..2 {
			for intent in self.players[player].intents.clone() {
				let zone = match intent {
					Action::Disrupt { zone } => zone,
					Action::DisruptAt { at } => match self.grid.get(at) {
						Some(tile) => tile.zone as i32,
						None => {
							self.report_error(report, player, format!("Not part of grid: {at}"));
							continue;
						}
					},
					_ => continue,
				};
				if self.players[player].blot_points <= 0 {
					self.report_error(report, player, "Not enough disruption points.".into());
					continue;
				}
				if zone < 0 || zone as usize >= self.grid.zones.len() {
					self.report_error(report, player, format!("Invalid region id: {zone}"));
					continue;
				}
				let zone = zone as usize;
				if self.grid.zones[zone].inked {
					self.report_error(
						report,
						player,
						format!("Cannot disrupt region{zone}. Already inked out."),
					);
					continue;
				}
				if !self.grid.zones[zone].towns.is_empty() {
					self.report_error(
						report,
						player,
						format!("Cannot disrupt region{zone}. It contains a town."),
					);
					continue;
				}
				self.players[player].blot_points -= 1;
				self.grid.zones[zone].instability += 1;
				self.successful_blots.insert(player, zone);
				report.disrupted[player] = Some(zone);
			}
		}
	}

	fn do_instability_check(&mut self, report: &mut TurnReport) {
		let to_ink: Vec<usize> = self
			.grid
			.zones
			.iter()
			.filter(|zone| {
				!zone.inked
					&& zone.towns.is_empty()
					&& zone.instability >= self.instability_threshold
			})
			.map(|zone| zone.id)
			.collect();

		for zone in to_ink {
			self.instability_threshold += INSTABILITY_THRESHOLD_INCREASE;
			self.grid.zones[zone].inked = true;
			report.inked.push(zone);

			let mut wiped = [0i32; 3];
			for at in self.grid.zones[zone].coords.clone() {
				let tile = self.grid.tile_mut(at);
				if !tile.is_track() {
					continue;
				}
				if tile.track > -1 {
					wiped[tile.track as usize] += 1;
				}
				tile.track = TRACK_NONE;
			}

			if self.successful_blots.get(&0) == Some(&zone) && wiped[1] > 0 {
				self.tutorial_inked_enemy_track = true;
			}
			for player in 0..2 {
				if self.successful_blots.get(&player) == Some(&zone) {
					self.stats.zones_inked[player] += 1;
					self.stats.own_tracks_inked_out[player] += wiped[player];
					self.stats.enemy_tracks_inked_out[player] += wiped[1 - player];
				}
			}
		}
	}

	fn move_trains(&mut self, report: &mut TurnReport) {
		for id in 0..self.grid.towns.len() {
			let from = self.grid.towns[id].coord;
			let desired = self.grid.towns[id].desired.clone();
			let mut active = Vec::new();
			let mut paths: BTreeMap<usize, Vec<Coord>> = BTreeMap::new();

			for other in desired {
				let to = self.grid.towns[other].coord;
				let path = train_path(&self.grid, from, to);
				if path.is_empty() {
					continue;
				}
				active.push(other);

				let mut points = [0i32; 2];
				for &at in &path {
					match self.grid.tile(at).track {
						0 => points[0] += 1,
						1 => points[1] += 1,
						_ => {}
					}
				}
				for (player, &gained) in points.iter().enumerate() {
					if gained == 0 {
						continue;
					}
					self.players[player].score += gained;
					report.gained[player] += gained;
					self.stats.extra_tiles_in_connection[player] +=
						path.len() as i32 - from.manhattan_to(to);
					self.stats.ownership_sum[player] += gained as f32 / path.len() as f32;
					self.stats.ownership_count[player] += 1;
				}
				paths.insert(other, path);
			}

			self.grid.towns[id].active = active;
			self.grid.towns[id].paths = paths;
		}
	}

	fn compute_tile_states(&mut self) {
		for tile in &mut self.grid.tiles {
			tile.connections.clear();
		}
		for id in 0..self.grid.towns.len() {
			for (&other, path) in &self.grid.towns[id].paths.clone() {
				for &at in path {
					self.grid
						.tile_mut(at)
						.connections
						.push((id as u8, other as u8));
				}
			}
		}
	}

	pub fn is_game_over(&self) -> bool {
		if self.in_tutorial {
			return self.tutorial_objective_complete() || self.turn >= MAX_TURNS;
		}
		!self.any_connection_still_possible() || self.turn >= MAX_TURNS
	}

	fn any_connection_still_possible(&self) -> bool {
		self.grid.towns.iter().any(|town| {
			town.desired.iter().any(|&other| {
				terrain_reachable(&self.grid, town.coord, self.grid.towns[other].coord)
			})
		})
	}

	fn tutorial_objective_complete(&self) -> bool {
		match self.league {
			1 => self.players[0].score >= 1,
			2 => self.tutorial_inked_enemy_track,
			_ => false,
		}
	}

	/// `Game.onEnd`: the scores the endscreen reports, and the texts beside them.
	pub fn on_end(&mut self) -> [String; 2] {
		if self.in_tutorial {
			let win = self.tutorial_objective_complete();
			self.players[0].score = if win { 0 } else { -1 };
			self.players[1].score = if win { -1 } else { 0 };
			return [
				if win {
					"objective complete".into()
				} else {
					"objective failed".into()
				},
				"-".into(),
			];
		}
		let mut texts = [String::new(), String::new()];
		for (player, text) in texts.iter_mut().enumerate() {
			if !self.players[player].active {
				self.players[player].score = -1;
				*text = "-".into();
			} else {
				let score = self.players[player].score;
				*text = format!("{score} point{}", if score > 1 { "s" } else { "" });
			}
		}
		texts
	}
}

/// `GameManager.formatErrorMessage`.
pub fn error_message(message: &str) -> String {
	format!("¤RED¤{message}§RED§")
}
