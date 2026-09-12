use crate::grid::{Grid, coord};

#[derive(Clone, Debug)]
pub struct Log {
	pub path: String,
	pub seed: i64,
	pub my_id: usize,
	pub width: i32,
	pub height: i32,
	pub cells: Vec<(i16, u8)>,
	pub towns: Vec<LoggedTown>,
	pub outputs: Vec<String>,
	pub marked_turns: usize,
	pub marked_total: Option<usize>,
	pub final_score: Option<[i32; 2]>,
	pub players: Option<[String; 2]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoggedTown {
	pub id: usize,
	pub x: i32,
	pub y: i32,
	pub desired: Vec<usize>,
}

const STREAM_MARKER: &str = "Standard Output Stream:";
const SUMMARY_MARKER: &str = "Game Summary:";

pub fn read(path: &str) -> Result<Log, String> {
	let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
	let spot = std::path::Path::new(path);
	let beside = spot.with_file_name(INIT_FILE);
	if seed_of(spot).is_some() {
		let seed = seed_of(spot).unwrap();
		return parse(path, seed, &text);
	}
	let folder = spot
		.parent()
		.ok_or_else(|| format!("{path}: no folder to take the seed from"))?;
	let seed = seed_of(folder).ok_or_else(|| {
		format!("{path}: neither the file nor its folder is named after a seed, pass --seed")
	})?;
	let head =
		std::fs::read_to_string(&beside).map_err(|e| format!("{}: {e}", beside.display()))?;
	let mut log = parse(path, seed, &format!("{head}\n{text}"))?;
	log.players = players_of(spot);
	Ok(log)
}

const INIT_FILE: &str = "init";

fn seed_of(spot: &std::path::Path) -> Option<i64> {
	spot.file_stem()
		.and_then(|stem| stem.to_str())
		.and_then(|stem| stem.parse::<i64>().ok())
}

fn players_of(spot: &std::path::Path) -> Option<[String; 2]> {
	let stem = spot.file_stem()?.to_str()?;
	let (first, second) = stem.split_once('-')?;
	(!first.is_empty() && !second.is_empty()).then(|| [first.to_string(), second.to_string()])
}

pub fn parse(path: &str, seed: i64, text: &str) -> Result<Log, String> {
	let mut reader = Reader {
		path,
		lines: text.lines(),
	};

	let my_id = reader.number("myId")? as usize;
	let width = reader.number("width")? as i32;
	let height = reader.number("height")? as i32;

	let mut cells = Vec::with_capacity((width * height) as usize);
	for cell in 0..width * height {
		cells.push(reader.cell(cell)?);
	}

	let town_count = reader.number("townCount")? as usize;
	let mut towns = Vec::with_capacity(town_count);
	for _ in 0..town_count {
		towns.push(reader.town()?);
	}

	let Tail {
		outputs,
		final_score,
		marked_total,
	} = reader.tail()?;

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
		players: None,
	})
}

impl Log {
	pub fn check_map(&self, grid: &Grid) -> Result<(), String> {
		let seed = self.seed;
		if grid.width != self.width || grid.height != self.height {
			return Err(format!(
				"seed {seed} builds {}x{}, the log says {}x{}",
				grid.width, grid.height, self.width, self.height
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
				"seed {seed} disagrees with the log on {wrong} cells"
			));
		}
		if grid.towns.len() != self.towns.len() {
			return Err(format!(
				"seed {seed} places {} towns, the log has {}",
				grid.towns.len(),
				self.towns.len()
			));
		}
		for (town, logged) in grid.towns.iter().zip(&self.towns) {
			if town.id != logged.id
				|| town.coord != coord(logged.x, logged.y)
				|| town.desired != logged.desired
			{
				return Err(format!(
					"seed {seed} disagrees with the log on town {}",
					logged.id
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
				let (first, second) = (pair[0].clone(), pair[1].clone());
				if swapped {
					[second, first]
				} else {
					[first, second]
				}
			})
			.collect()
	}
}

struct Reader<'a> {
	path: &'a str,
	lines: std::str::Lines<'a>,
}

struct Tail {
	outputs: Vec<String>,
	final_score: Option<[i32; 2]>,
	marked_total: Option<usize>,
}

impl Reader<'_> {
	fn line(&mut self, what: &str) -> Result<String, String> {
		self.lines
			.next()
			.map(str::to_string)
			.ok_or_else(|| format!("{}: log ends where {what} was expected", self.path))
	}

	fn number(&mut self, what: &str) -> Result<i64, String> {
		let line = self.line(what)?;
		line.trim()
			.parse::<i64>()
			.map_err(|_| format!("{}: {what} is not a number: {line:?}", self.path))
	}

	fn cell(&mut self, cell: i32) -> Result<(i16, u8), String> {
		let line = self.line("a grid cell")?;
		let mut parts = line.split_whitespace();
		let zone = parts
			.next()
			.and_then(|word| word.parse::<i16>().ok())
			.ok_or_else(|| format!("{}: cell {cell} has no region id: {line:?}", self.path))?;
		let kind = parts
			.next()
			.and_then(|word| word.parse::<u8>().ok())
			.ok_or_else(|| format!("{}: cell {cell} has no terrain: {line:?}", self.path))?;
		Ok((zone, kind))
	}

	fn town(&mut self) -> Result<LoggedTown, String> {
		let line = self.line("a town")?;
		let mut parts = line.split_whitespace();
		let mut field = |what: &str| -> Result<i32, String> {
			parts
				.next()
				.and_then(|word| word.parse::<i32>().ok())
				.ok_or_else(|| format!("{}: town {what} missing in {line:?}", self.path))
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
						.map_err(|_| format!("{}: bad desiredConnections in {line:?}", self.path))
				})
				.collect::<Result<_, _>>()?,
		};
		Ok(LoggedTown { id, x, y, desired })
	}

	fn tail(&mut self) -> Result<Tail, String> {
		let mut outputs = Vec::new();
		let mut trailing: Vec<i64> = Vec::new();
		let mut markers: Vec<i64> = Vec::new();
		let mut awaiting_output = false;
		let mut inside_summary = false;
		for line in self.lines.by_ref() {
			let line = line.trim_end();
			if line == STREAM_MARKER {
				awaiting_output = true;
				inside_summary = false;
				continue;
			}
			if line == SUMMARY_MARKER {
				inside_summary = true;
				continue;
			}
			if inside_summary {
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
			let Ok(value) = line.trim().parse::<i64>() else {
				return Err(format!("{}: unexpected line {line:?}", self.path));
			};
			trailing.push(value);
			markers.push(value);
		}

		let final_score = (trailing.len() >= 2).then(|| {
			let tail = &trailing[trailing.len() - 2..];
			[tail[0] as i32, tail[1] as i32]
		});
		let marked_total = markers
			.chunks(2)
			.filter_map(|pair| pair.get(1))
			.max()
			.filter(|_| final_score.is_some())
			.map(|&value| value as usize);

		Ok(Tail {
			outputs,
			final_score,
			marked_total,
		})
	}
}
