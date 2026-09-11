use crate::game::{DEFAULT_LEAGUE, Game};
use crate::grid::{Coord, Grid, TERRAIN_CHAR, coord};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Log {
	pub path: String,
	pub seed: i64,
	pub my_id: usize,
	pub width: i32,
	pub height: i32,
	pub cells: Vec<(i16, u8)>,
	pub towns: Vec<(usize, i32, i32, Vec<usize>)>,
	pub outputs: Vec<String>,
	pub marked_turns: usize,
	pub marked_total: Option<usize>,
	pub final_score: Option<[i32; 2]>,
}

const STREAM_MARKER: &str = "Standard Output Stream:";

pub fn read(path: &str) -> Result<Log, String> {
	let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
	let seed = std::path::Path::new(path)
		.file_stem()
		.and_then(|stem| stem.to_str())
		.and_then(|stem| stem.parse::<i64>().ok())
		.ok_or_else(|| format!("{path}: file name is not a seed, pass --seed"))?;
	parse(path, seed, &text)
}

pub fn parse(path: &str, seed: i64, text: &str) -> Result<Log, String> {
	let mut lines = text.lines();
	let mut next = |what: &str| -> Result<String, String> {
		lines
			.next()
			.map(str::to_string)
			.ok_or_else(|| format!("{path}: log ends where {what} was expected"))
	};
	let number = |line: &str, what: &str| -> Result<i64, String> {
		line.trim()
			.parse::<i64>()
			.map_err(|_| format!("{path}: {what} is not a number: {line:?}"))
	};

	let my_id = number(&next("myId")?, "myId")? as usize;
	let width = number(&next("width")?, "width")? as i32;
	let height = number(&next("height")?, "height")? as i32;

	let mut cells = Vec::with_capacity((width * height) as usize);
	for cell in 0..width * height {
		let line = next("a grid cell")?;
		let mut parts = line.split_whitespace();
		let zone = parts
			.next()
			.and_then(|word| word.parse::<i16>().ok())
			.ok_or_else(|| format!("{path}: cell {cell} has no region id: {line:?}"))?;
		let kind = parts
			.next()
			.and_then(|word| word.parse::<u8>().ok())
			.ok_or_else(|| format!("{path}: cell {cell} has no terrain: {line:?}"))?;
		cells.push((zone, kind));
	}

	let town_count = number(&next("townCount")?, "townCount")? as usize;
	let mut towns = Vec::with_capacity(town_count);
	for _ in 0..town_count {
		let line = next("a town")?;
		let mut parts = line.split_whitespace();
		let mut field = |what: &str| -> Result<i32, String> {
			parts
				.next()
				.and_then(|word| word.parse::<i32>().ok())
				.ok_or_else(|| format!("{path}: town {what} missing in {line:?}"))
		};
		let id = field("id")? as usize;
		let x = field("x")?;
		let y = field("y")?;
		let desired = match parts.next() {
			None | Some("x") => Vec::new(),
			Some(list) => list
				.split(',')
				.map(|word| {
					word.parse::<usize>()
						.map_err(|_| format!("{path}: bad desiredConnections in {line:?}"))
				})
				.collect::<Result<_, _>>()?,
		};
		towns.push((id, x, y, desired));
	}

	let mut outputs = Vec::new();
	let mut trailing: Vec<i64> = Vec::new();
	let mut markers: Vec<i64> = Vec::new();
	let mut awaiting_output = false;
	for line in lines {
		let line = line.trim_end();
		if line == STREAM_MARKER {
			awaiting_output = true;
			continue;
		}
		if awaiting_output {
			outputs.push(line.to_string());
			awaiting_output = false;
			trailing.clear();
			continue;
		}
		if line.trim().is_empty() {
			continue;
		}
		match line.trim().parse::<i64>() {
			Ok(value) => {
				trailing.push(value);
				markers.push(value);
			}
			Err(_) => return Err(format!("{path}: unexpected line {line:?}")),
		}
	}

	let final_score = (trailing.len() >= 2).then(|| {
		let tail = &trailing[trailing.len() - 2..];
		[tail[0] as i32, tail[1] as i32]
	});
	let totals: Vec<i64> = markers
		.chunks(2)
		.filter_map(|pair| pair.get(1))
		.copied()
		.collect();
	let marked_total = totals
		.iter()
		.max()
		.filter(|_| final_score.is_some())
		.map(|&value| value as usize);

	Ok(Log {
		path: path.to_string(),
		seed,
		my_id,
		width,
		height,
		cells,
		towns,
		marked_turns: outputs.len() / 2,
		marked_total,
		final_score,
		outputs,
	})
}

impl Log {
	pub fn check_map(&self, grid: &Grid) -> Result<(), String> {
		if grid.width != self.width || grid.height != self.height {
			return Err(format!(
				"seed {} builds {}x{}, the log says {}x{}",
				self.seed, grid.width, grid.height, self.width, self.height
			));
		}
		let wrong = (0..grid.cells())
			.filter(|&cell| {
				let tile = &grid.tiles[cell];
				(tile.zone, tile.kind) != self.cells[cell]
			})
			.count();
		if wrong > 0 {
			return Err(format!(
				"seed {} disagrees with the log on {wrong} cells",
				self.seed
			));
		}
		if grid.towns.len() != self.towns.len() {
			return Err(format!(
				"seed {} places {} towns, the log has {}",
				self.seed,
				grid.towns.len(),
				self.towns.len()
			));
		}
		for (town, (id, x, y, desired)) in grid.towns.iter().zip(&self.towns) {
			if town.id != *id || town.coord != coord(*x, *y) || &town.desired != desired {
				return Err(format!(
					"seed {} disagrees with the log on town {id}",
					self.seed
				));
			}
		}
		Ok(())
	}

	pub fn answers(&self, swapped: bool) -> Vec<[String; 2]> {
		self.outputs
			.chunks(2)
			.filter(|pair| pair.len() == 2)
			.map(|pair| {
				if swapped {
					[pair[1].clone(), pair[0].clone()]
				} else {
					[pair[0].clone(), pair[1].clone()]
				}
			})
			.collect()
	}
}

#[derive(Clone, Debug, Default)]
pub struct ZoneLook {
	pub instability: i32,
	pub inked: bool,
	pub has_town: bool,
	pub cells: usize,
	pub tracks: [i32; 3],
	pub path_cells: [i32; 3],
	pub pairs: Vec<(usize, usize)>,
}

#[derive(Clone, Copy, Debug)]
pub struct Claim {
	pub at: Coord,
	pub turn: i32,
	pub owner: i8,
	pub touching: [bool; 3],
	pub near_town: bool,
}

#[derive(Clone, Debug, Default)]
pub struct PairPay {
	pub turns: i32,
	pub length_sum: i64,
	pub paid: [i64; 2],
}

#[derive(Clone, Debug)]
pub struct TurnRecord {
	pub turn: i32,
	pub answers: [String; 2],
	pub placed: [i32; 2],
	pub paint_left: [i32; 2],
	pub disrupted: [Option<usize>; 2],
	pub inked: Vec<usize>,
	pub wiped: BTreeMap<usize, [i32; 3]>,
	pub gained: [i32; 2],
	pub score: [i32; 2],
	pub errors: Vec<String>,
	pub pairs: Vec<(usize, usize, usize, [i32; 2])>,
	pub paths: BTreeMap<(usize, usize), Vec<Coord>>,
	pub active_cells: [i32; 4],
	pub before: BTreeMap<usize, ZoneLook>,
	pub board: String,
}

impl TurnRecord {
	pub fn board_owner(&self, at: Coord) -> i8 {
		let Some(line) = self.board.lines().nth(at.y as usize) else {
			return -1;
		};
		match line.chars().nth(at.x as usize) {
			Some('o') => 0,
			Some('x') => 1,
			Some('*') => 2,
			_ => -1,
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct Totals {
	pub placed: [i32; 2],
	pub plains: [i32; 2],
	pub river: [i32; 2],
	pub mountain: [i32; 2],
	pub paint_spent: [i32; 2],
	pub paint_lost: [i32; 2],
	pub disrupts: [i32; 2],
	pub blots_lost: [i32; 2],
	pub inked_by: [i32; 2],
	pub wiped_own: [i32; 2],
	pub wiped_foe: [i32; 2],
	pub pay: BTreeMap<(usize, usize), PairPay>,
	pub claims: Vec<Claim>,
	pub cell_pay: BTreeMap<Coord, [i64; 2]>,
}

pub struct Replay {
	pub grid: Grid,
	pub turns: Vec<TurnRecord>,
	pub totals: Totals,
	pub score: [i32; 2],
	pub ended: bool,
	pub disqualified: [Option<String>; 2],
}

pub fn run(grid: Grid, answers: &[[String; 2]]) -> Replay {
	let mut game = Game::new(grid, DEFAULT_LEAGUE);
	let mut turns = Vec::with_capacity(answers.len());
	let mut totals = Totals::default();
	let mut seen_errors = 0;
	let mut last = game.stats.clone();

	for pair in answers {
		if game.ended || game.active_players() < 2 {
			break;
		}
		let before = look_at_zones(&game);
		let tracks_before: Vec<i8> = game.grid.tiles.iter().map(|tile| tile.track).collect();

		game.reset_turn_data();
		for (player, line) in pair.iter().enumerate() {
			game.take_commands(player, line);
		}
		let report = game.perform_update();

		let mut wiped: BTreeMap<usize, [i32; 3]> = BTreeMap::new();
		for &zone in &report.inked {
			let mut counts = [0i32; 3];
			for &at in &game.grid.zones[zone].coords {
				let owner = tracks_before[game.grid.index(at)];
				if (0..=2).contains(&owner) {
					counts[owner as usize] += 1;
				}
			}
			wiped.insert(zone, counts);
		}

		let mut pairs = Vec::new();
		let mut paths: BTreeMap<(usize, usize), Vec<Coord>> = BTreeMap::new();
		let mut on_path: BTreeMap<Coord, i8> = BTreeMap::new();
		for town in &game.grid.towns {
			for (&other, path) in &town.paths {
				let mut owned = [0i32; 2];
				for &at in path {
					match game.grid.tile(at).track {
						0 => owned[0] += 1,
						1 => owned[1] += 1,
						_ => {}
					}
				}
				pairs.push((town.id, other, path.len(), owned));
				paths.insert((town.id, other), path.clone());
				for &at in path {
					on_path.insert(at, game.grid.tile(at).track);
				}
				let entry = totals.pay.entry((town.id, other)).or_default();
				entry.turns += 1;
				entry.length_sum += path.len() as i64;
				entry.paid[0] += owned[0] as i64;
				entry.paid[1] += owned[1] as i64;
				for &at in path {
					let owner = game.grid.tile(at).track;
					if owner == 0 || owner == 1 {
						totals.cell_pay.entry(at).or_default()[owner as usize] += 1;
					}
				}
			}
		}

		for &(at, owner) in &report.built {
			let mut touching = [false; 3];
			let mut near_town = false;
			for next in game.grid.neighbours(at) {
				let was = tracks_before[game.grid.index(next)];
				if (0..=2).contains(&was) {
					touching[was as usize] = true;
				}
				near_town |= game.grid.tile(next).is_town();
			}
			totals.claims.push(Claim {
				at,
				turn: game.turn,
				owner,
				touching,
				near_town,
			});
		}

		let stats = game.stats.clone();
		let placed = [
			stats.placed_tracks[0] - last.placed_tracks[0],
			stats.placed_tracks[1] - last.placed_tracks[1],
		];
		for (player, &count) in placed.iter().enumerate() {
			totals.plains[player] += stats.on_plains[player] - last.on_plains[player];
			totals.river[player] += stats.on_river[player] - last.on_river[player];
			totals.mountain[player] += stats.on_mountains[player] - last.on_mountains[player];
			totals.placed[player] += count;
			totals.paint_spent[player] += 3 - game.players[player].dosh;
			totals.paint_lost[player] += game.players[player].dosh;
			if report.disrupted[player].is_some() {
				totals.disrupts[player] += 1;
			} else {
				totals.blots_lost[player] += 1;
			}
		}
		for (&zone, counts) in &wiped {
			for player in 0..2 {
				if report.disrupted[player] == Some(zone) {
					totals.inked_by[player] += 1;
					totals.wiped_own[player] += counts[player];
					totals.wiped_foe[player] += counts[1 - player];
				}
			}
		}
		last = stats;

		turns.push(TurnRecord {
			turn: game.turn,
			answers: pair.clone(),
			placed,
			paint_left: [game.players[0].dosh, game.players[1].dosh],
			disrupted: report.disrupted,
			inked: report.inked.clone(),
			wiped,
			gained: report.gained,
			score: [game.players[0].score, game.players[1].score],
			errors: game.summary[seen_errors..].to_vec(),
			pairs,
			paths,
			active_cells: {
				let mut counts = [0i32; 4];
				for owner in on_path.values() {
					counts[match owner {
						0 => 0,
						1 => 1,
						2 => 2,
						_ => 3,
					}] += 1;
				}
				counts
			},
			before,
			board: render(&game),
		});
		seen_errors = game.summary.len();
	}

	Replay {
		score: [game.players[0].score, game.players[1].score],
		ended: game.ended,
		disqualified: [
			game.players[0].deactivated_by.clone(),
			game.players[1].deactivated_by.clone(),
		],
		grid: game.grid,
		turns,
		totals,
	}
}

fn look_at_zones(game: &Game) -> BTreeMap<usize, ZoneLook> {
	let mut looks = BTreeMap::new();
	for zone in &game.grid.zones {
		let mut look = ZoneLook {
			instability: zone.instability,
			inked: zone.inked,
			has_town: !zone.towns.is_empty(),
			cells: zone.coords.len(),
			..ZoneLook::default()
		};
		for &at in &zone.coords {
			let tile = game.grid.tile(at);
			if (0..=2).contains(&tile.track) {
				look.tracks[tile.track as usize] += 1;
			}
			for &(from, to) in &tile.connections {
				let pair = (from as usize, to as usize);
				if !look.pairs.contains(&pair) {
					look.pairs.push(pair);
				}
				if (0..=2).contains(&tile.track) {
					look.path_cells[tile.track as usize] += 1;
				}
			}
		}
		looks.insert(zone.id, look);
	}
	looks
}

pub fn town_char(id: usize) -> char {
	if id < 26 {
		(b'A' + id as u8) as char
	} else {
		'#'
	}
}

pub fn render(game: &Game) -> String {
	let grid = &game.grid;
	let mut out = String::new();
	for y in 0..grid.height {
		for x in 0..grid.width {
			let tile = grid.tile(coord(x, y));
			out.push(if tile.is_town() {
				town_char(tile.town as usize)
			} else if grid.zones[tile.zone as usize].inked {
				':'
			} else {
				match tile.track {
					0 => 'o',
					1 => 'x',
					2 => '*',
					_ => TERRAIN_CHAR[tile.kind as usize] as char,
				}
			});
		}
		out.push('\n');
	}
	out
}
