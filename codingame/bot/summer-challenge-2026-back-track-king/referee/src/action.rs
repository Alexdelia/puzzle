use crate::grid::{Coord, coord};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
	Autoplace { from: Coord, to: Coord },
	Place { at: Coord, autobuilt: bool },
	Disrupt { zone: i32 },
	DisruptAt { at: Coord },
	Wait,
}

impl Action {
	pub fn is_autobuilt(&self) -> bool {
		matches!(
			self,
			Action::Place {
				autobuilt: true,
				..
			}
		)
	}
}

#[derive(Debug, Default)]
pub struct Commands {
	pub intents: Vec<Action>,
	pub message: Option<String>,
	pub rejected: Option<String>,
}

pub fn parse(line: &str) -> Commands {
	let mut out = Commands::default();
	for chunk in split(line.trim()) {
		let command = chunk.trim();
		match read(command) {
			Ok(Some(Read::Intent(action))) => out.intents.push(action),
			Ok(Some(Read::Message(text))) => out.message = Some(text),
			Ok(None) => {
				out.rejected = Some(format!(
					"Invalid Input: Expected {} but got '{command}'",
					expected(command)
				));
				return out;
			}
			Err(unparsable) => {
				out.rejected = Some(format!("For input string: \"{unparsable}\""));
				return out;
			}
		}
	}
	out
}

enum Read {
	Intent(Action),
	Message(String),
}

fn split(line: &str) -> Vec<&str> {
	if !line.contains(';') {
		return vec![line];
	}
	let mut commands: Vec<&str> = line.split(';').collect();
	while commands.last().is_some_and(|command| command.is_empty()) {
		commands.pop();
	}
	commands
}

fn read(command: &str) -> Result<Option<Read>, String> {
	if let Some(rest) = strip_verb(command, "AUTOPLACE")
		&& let Some(fields) = numbers(rest, 4, &[0, 2, 1, 3])
	{
		let fields = fields?;
		return Ok(Some(Read::Intent(Action::Autoplace {
			from: coord(fields[0], fields[1]),
			to: coord(fields[2], fields[3]),
		})));
	}
	if let Some(rest) = strip_verb(command, "PLACE_TRACKS")
		&& let Some(fields) = numbers(rest, 2, &[0, 1])
	{
		let fields = fields?;
		return Ok(Some(Read::Intent(Action::Place {
			at: coord(fields[0], fields[1]),
			autobuilt: false,
		})));
	}
	if let Some(rest) = strip_verb(command, "DISRUPT")
		&& let Some(fields) = numbers(rest, 1, &[0])
	{
		return Ok(Some(Read::Intent(Action::Disrupt { zone: fields?[0] })));
	}
	if let Some(rest) = strip_verb(command, "DISRUPT")
		&& let Some(fields) = numbers(rest, 2, &[0, 1])
	{
		let fields = fields?;
		return Ok(Some(Read::Intent(Action::DisruptAt {
			at: coord(fields[0], fields[1]),
		})));
	}
	if let Some(rest) = strip_verb(command, "MESSAGE") {
		return Ok(Some(Read::Message(rest.to_string())));
	}
	if command.eq_ignore_ascii_case("WAIT") {
		return Ok(Some(Read::Intent(Action::Wait)));
	}
	Ok(None)
}

fn strip_verb<'a>(command: &'a str, verb: &str) -> Option<&'a str> {
	let (head, rest) = command.split_at_checked(verb.len())?;
	if !head.eq_ignore_ascii_case(verb) {
		return None;
	}
	rest.strip_prefix(' ')
}

fn numbers(text: &str, wanted: usize, reading: &[usize]) -> Option<Result<Vec<i32>, String>> {
	let fields: Vec<&str> = text.split(' ').collect();
	if fields.len() != wanted {
		return None;
	}
	if !fields
		.iter()
		.all(|field| !field.is_empty() && field.bytes().all(|byte| byte.is_ascii_digit()))
	{
		return None;
	}
	let mut values = vec![0i32; wanted];
	for &field in reading {
		match fields[field].parse::<i32>() {
			Ok(value) => values[field] = value,
			Err(_) => return Some(Err(fields[field].to_string())),
		}
	}
	Some(Ok(values))
}

pub fn expected(command: &str) -> &'static str {
	if command.starts_with("AUTOPLACE") {
		"AUTOPLACE x1 y1 x2 y2"
	} else if command.starts_with("PLACE_TRACK") {
		"PLACE_TRACK x y"
	} else if command.starts_with("DISRUPT") {
		"DISRUPT zoneId"
	} else if command.starts_with("MESSAGE") {
		"MESSAGE text"
	} else if command.starts_with("WAIT") {
		"WAIT"
	} else {
		"AUTOPLACE | PLACE_TRACK | DISRUPT | MESSAGE | WAIT"
	}
}
