mod connection;
mod point;

use std::collections::BinaryHeap;

use aocd::*;

use crate::{connection::Connection, point::Point};

#[aocd(2025, 8)]
fn parse() -> Result<Vec<Point>, String> {
	let input = input!();
	input.lines().map(|line| line.parse::<Point>()).collect()
}

fn main() -> Result<(), String> {
	let point_list = parse()?;

	dbg!(point_list.len());

	Ok(())
}
