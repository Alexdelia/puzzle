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
