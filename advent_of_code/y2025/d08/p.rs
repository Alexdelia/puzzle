mod connection;
mod point;

use aocd::*;

#[aocd(2025, 8)]
fn main() {
	let lines: Vec<String> = input!().lines().map(|s| s.to_string()).collect();

	dbg!(lines.len());
}
