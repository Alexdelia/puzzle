use crate::game::{Connections, State};
use crate::map::Map;
use std::fmt::Write;

pub fn init_text(map: &Map, player: usize) -> String {
	let mut out = String::with_capacity(16 * map.cells());
	let _ = writeln!(out, "{player}\n{}\n{}", map.width, map.height);
	for cell in 0..map.cells() {
		let _ = writeln!(out, "{} {}", map.region[cell], map.terrain[cell]);
	}
	let _ = writeln!(out, "{}", map.towns.len());
	for (id, town) in map.towns.iter().enumerate() {
		let _ = writeln!(
			out,
			"{id} {} {} {}",
			town.x,
			town.y,
			desired_text(&town.desired)
		);
	}
	out
}

pub fn turn_text(map: &Map, state: &State, conn: &Connections, player: usize, out: &mut String) {
	out.clear();
	let _ = writeln!(out, "{}\n{}", state.score[player], state.score[1 - player]);
	for cell in 0..map.cells() {
		let region = map.region_of(cell);
		out.push_str(OWNER_TEXT[(state.owner[cell] + 1) as usize]);
		out.push(' ');
		push_num(out, state.instability[region] as u32);
		out.push(' ');
		out.push(if state.inked[region] { '1' } else { '0' });
		out.push(' ');
		write_connections(conn, cell, out);
		out.push('\n');
	}
}

const OWNER_TEXT: [&str; 4] = ["-1", "0", "1", "2"];

fn push_num(out: &mut String, value: u32) {
	let mut digits = [0u8; 10];
	let mut at = digits.len();
	let mut left = value;
	loop {
		at -= 1;
		digits[at] = b'0' + (left % 10) as u8;
		left /= 10;
		if left == 0 {
			break;
		}
	}
	out.push_str(str::from_utf8(&digits[at..]).unwrap());
}

fn desired_text(desired: &[u8]) -> String {
	if desired.is_empty() {
		return "x".into();
	}
	desired
		.iter()
		.map(|d| d.to_string())
		.collect::<Vec<_>>()
		.join(",")
}

fn write_connections(conn: &Connections, cell: usize, out: &mut String) {
	let pairs = &conn.cell_pairs[cell];
	if pairs.is_empty() {
		out.push('x');
		return;
	}
	for (i, &pi) in pairs.iter().enumerate() {
		if i > 0 {
			out.push(',');
		}
		let (src, dst) = conn.pairs[pi as usize];
		push_num(out, src as u32);
		out.push('-');
		push_num(out, dst as u32);
	}
}
