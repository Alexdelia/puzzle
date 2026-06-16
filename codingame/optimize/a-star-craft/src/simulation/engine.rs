pub const MAP_WIDTH: usize = 19;
pub const MAP_HEIGHT: usize = 10;
pub const MAP_AREA: usize = MAP_WIDTH * MAP_HEIGHT;
pub const MAX_ROBOTS: usize = 10;

pub type Cell = u8;
pub type Tile = u8;
pub type RobotCount = u8;
pub type SolutionCount = u32;
pub type Score = u16;
pub type Turn = u16;

pub const UP: Tile = 0;
pub const RIGHT: Tile = 1;
pub const DOWN: Tile = 2;
pub const LEFT: Tile = 3;
pub const NONE: Tile = 4;
pub const VOID: Tile = 5;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Robot {
	pub cell: Cell,
	pub heading: Tile,
}

#[derive(Clone, Debug)]
pub struct Engine {
	pub base: [Tile; MAP_AREA],
	pub robot_list: Vec<Robot>,
}

impl Engine {
	#[inline]
	pub fn robot_count(&self) -> RobotCount {
		self.robot_list.len() as RobotCount
	}
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Solution {
	pub arrow: [Tile; MAP_AREA],
}

impl Solution {
	#[inline]
	pub fn empty() -> Self {
		Self {
			arrow: [NONE; MAP_AREA],
		}
	}
}

impl Default for Solution {
	fn default() -> Self {
		Self::empty()
	}
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct SimOutput {
	pub score: Score,
	pub game_length: Turn,
}

pub fn build_next_table() -> Vec<Cell> {
	let mut next = vec![0 as Cell; MAP_AREA * 4];
	for cell in 0..MAP_AREA {
		let x = cell % MAP_WIDTH;
		let y = cell / MAP_WIDTH;
		next[cell * 4 + UP as usize] =
			(((y + MAP_HEIGHT - 1) % MAP_HEIGHT) * MAP_WIDTH + x) as Cell;
		next[cell * 4 + RIGHT as usize] = (y * MAP_WIDTH + (x + 1) % MAP_WIDTH) as Cell;
		next[cell * 4 + DOWN as usize] = (((y + 1) % MAP_HEIGHT) * MAP_WIDTH + x) as Cell;
		next[cell * 4 + LEFT as usize] = (y * MAP_WIDTH + (x + MAP_WIDTH - 1) % MAP_WIDTH) as Cell;
	}
	next
}

#[derive(Clone, Copy, Debug)]
pub struct Spot {
	pub cell: Cell,
	pub candidate: [Tile; 5],
	pub alive_count: u8,
}

pub fn placeable_spot_list(base: &[Tile], next: &[Cell], robot_list: &[Robot]) -> Vec<Spot> {
	control_cell_list(base, next, robot_list)
		.into_iter()
		.filter_map(|cell| {
			let mut candidate = [NONE; 5];
			let mut alive_count = 0usize;
			for direction in [UP, RIGHT, DOWN, LEFT] {
				let neighbor = next[cell as usize * 4 + direction as usize] as usize;
				if base[neighbor] != VOID {
					candidate[alive_count] = direction;
					alive_count += 1;
				}
			}
			if alive_count == 0 {
				return None;
			}
			candidate[alive_count] = NONE;
			Some(Spot {
				cell,
				candidate,
				alive_count: alive_count as u8,
			})
		})
		.collect()
}

pub fn control_cell_list(base: &[Tile], next: &[Cell], robot_list: &[Robot]) -> Vec<Cell> {
	let passable = |cell: usize| base[cell] != VOID;
	let neighbor = |cell: usize, direction: Tile| next[cell * 4 + direction as usize] as usize;

	let mut in_cycle = [false; MAP_AREA];
	for (cell, slot) in in_cycle.iter_mut().enumerate() {
		*slot = passable(cell);
	}
	loop {
		let mut peeled = false;
		for cell in 0..MAP_AREA {
			if in_cycle[cell]
				&& [UP, RIGHT, DOWN, LEFT]
					.iter()
					.filter(|&&direction| in_cycle[neighbor(cell, direction)])
					.count() <= 1
			{
				in_cycle[cell] = false;
				peeled = true;
			}
		}
		if !peeled {
			break;
		}
	}

	let mut on_robot = [false; MAP_AREA];
	for robot in robot_list {
		on_robot[robot.cell as usize] = true;
	}

	(0..MAP_AREA)
		.filter(|&cell| base[cell] == NONE)
		.filter(|&cell| {
			if on_robot[cell] {
				return true;
			}
			let up = passable(neighbor(cell, UP));
			let right = passable(neighbor(cell, RIGHT));
			let down = passable(neighbor(cell, DOWN));
			let left = passable(neighbor(cell, LEFT));
			match up as u8 + right as u8 + down as u8 + left as u8 {
				0 => false,
				2 => !((up && down) || (left && right)) || in_cycle[cell],
				_ => true,
			}
		})
		.map(|cell| cell as Cell)
		.collect()
}

pub fn char_to_tile(c: char) -> Option<Tile> {
	match c.to_ascii_lowercase() {
		'u' => Some(UP),
		'r' => Some(RIGHT),
		'd' => Some(DOWN),
		'l' => Some(LEFT),
		'.' => Some(NONE),
		'#' => Some(VOID),
		_ => None,
	}
}

pub fn tile_to_char(tile: Tile) -> Option<char> {
	match tile {
		UP => Some('U'),
		RIGHT => Some('R'),
		DOWN => Some('D'),
		LEFT => Some('L'),
		_ => None,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::parse::parse_map;

	fn control_cell(map: &str) -> Vec<Cell> {
		let engine = parse_map(map).unwrap();
		let next = build_next_table();
		control_cell_list(&engine.base, &next, &engine.robot_list)
	}

	fn placeable_spot(map: &str) -> Vec<Spot> {
		let engine = parse_map(map).unwrap();
		let next = build_next_table();
		placeable_spot_list(&engine.base, &next, &engine.robot_list)
	}

	fn alive_direction(spot: &Spot) -> Vec<Tile> {
		spot.candidate[..spot.alive_count as usize].to_vec()
	}

	const CORRIDOR: &str = concat!(
		"###################\n",
		"###################\n",
		"###################\n",
		"###################\n",
		"###R...........####\n",
		"###################\n",
		"###################\n",
		"###################\n",
		"###################\n",
		"###################",
	);

	const RING: &str = concat!(
		"###################\n",
		"###################\n",
		"###################\n",
		"###################\n",
		"########R..########\n",
		"########.#.########\n",
		"########...########\n",
		"###################\n",
		"###################\n",
		"###################",
	);

	#[test]
	fn corridor_keeps_only_its_ends() {
		assert_eq!(control_cell(CORRIDOR), vec![4 * 19 + 3, 4 * 19 + 14]);
	}

	#[test]
	fn corridor_end_offers_only_inward_direction() {
		let spot = placeable_spot(CORRIDOR);
		assert_eq!(spot.len(), 2);
		assert_eq!(spot[0].cell, 4 * 19 + 3);
		assert_eq!(alive_direction(&spot[0]), vec![RIGHT]);
		assert_eq!(spot[1].cell, 4 * 19 + 14);
		assert_eq!(alive_direction(&spot[1]), vec![LEFT]);
	}

	#[test]
	fn ring_keeps_every_cell() {
		assert_eq!(
			control_cell(RING),
			vec![
				4 * 19 + 8,
				4 * 19 + 9,
				4 * 19 + 10,
				5 * 19 + 8,
				5 * 19 + 10,
				6 * 19 + 8,
				6 * 19 + 9,
				6 * 19 + 10,
			]
		);
	}
}
