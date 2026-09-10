use btk::action::{Action, parse};

fn describe(action: &Action) -> String {
	match action {
		Action::Wait => "WAIT".into(),
		Action::Place { at, .. } => format!("PLACE_TRACKS {} {}", at.x, at.y),
		Action::Autoplace { from, to } => {
			format!("AUTOPLACE {} {} {} {}", from.x, from.y, to.x, to.y)
		}
		Action::Disrupt { zone } => format!("DISRUPT {zone}"),
		Action::DisruptAt { at } => format!("DISRUPT_AT {} {}", at.x, at.y),
	}
}

fn replay(line: &str) -> String {
	let read = parse(line);
	let mut out = format!("line <{line}>\n");
	for intent in &read.intents {
		out.push_str(&format!("  intent {}\n", describe(intent)));
	}
	if let Some(message) = &read.message {
		out.push_str(&format!("  message <{message}>\n"));
	}
	if let Some(why) = &read.rejected {
		out.push_str(&format!("  rejected {why}\n"));
	}
	out
}

#[test]
fn reads_output_lines_like_the_official_parser() {
	let golden = include_str!("golden/commands.txt");
	let mut ours = String::new();
	for line in golden.lines() {
		if let Some(input) = line
			.strip_prefix("line <")
			.and_then(|l| l.strip_suffix('>'))
		{
			ours.push_str(&replay(input));
		}
	}
	assert!(!ours.is_empty(), "golden command file has no lines");
	for (at, (want, got)) in golden.lines().zip(ours.lines()).enumerate() {
		assert_eq!(want, got, "line {}", at + 1);
	}
	assert_eq!(golden.lines().count(), ours.lines().count());
}
