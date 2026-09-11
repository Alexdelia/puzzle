use super::Pair;
use crate::game::PASSIVE_INCOME;
use crate::grid::{Coord, Grid};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct ZoneLook {
	pub instability: i32,
	pub inked: bool,
	pub has_town: bool,
	pub cells: usize,
	pub tracks: [i32; 3],
	pub path_cells: [i32; 3],
	pub pairs: Vec<Pair>,
}

#[derive(Clone, Copy, Debug)]
pub struct Claim {
	pub at: Coord,
	pub turn: i32,
	pub owner: i8,
	pub touching: [bool; 3],
	pub near_town: bool,
}

impl Claim {
	pub fn player(&self) -> Option<usize> {
		(self.owner == 0 || self.owner == 1).then_some(self.owner as usize)
	}
}

#[derive(Clone, Debug, Default)]
pub struct PairPay {
	pub turns: i32,
	pub length_sum: i64,
	pub paid: [i64; 2],
}

impl PairPay {
	pub fn mean_length(&self) -> f64 {
		self.length_sum as f64 / self.turns.max(1) as f64
	}

	pub fn total(&self) -> i64 {
		self.paid[0] + self.paid[1]
	}
}

#[derive(Clone, Debug)]
pub struct ActivePair {
	pub pair: Pair,
	pub length: usize,
	pub owned: [i32; 2],
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
	pub pairs: Vec<ActivePair>,
	pub paths: BTreeMap<Pair, Vec<Coord>>,
	pub active_cells: [i32; 4],
	pub before: BTreeMap<usize, ZoneLook>,
	pub board: String,
	pub tracks: Vec<i8>,
	pub width: i32,
}

impl TurnRecord {
	pub fn owner(&self, at: Coord) -> i8 {
		let cell = at.y * self.width + at.x;
		self.tracks.get(cell as usize).copied().unwrap_or(-1)
	}

	pub fn owners_of(&self, path: &[Coord]) -> [i32; 2] {
		let mut owned = [0i32; 2];
		for &at in path {
			match self.owner(at) {
				0 => owned[0] += 1,
				1 => owned[1] += 1,
				_ => {}
			}
		}
		owned
	}

	pub fn paint_spent(&self, player: usize) -> i32 {
		PASSIVE_INCOME - self.paint_left[player]
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
	pub pay: BTreeMap<Pair, PairPay>,
	pub claims: Vec<Claim>,
	pub cell_pay: BTreeMap<Coord, [i64; 2]>,
}

impl Totals {
	pub fn paid_for(&self, at: Coord) -> [i64; 2] {
		self.cell_pay.get(&at).copied().unwrap_or([0, 0])
	}

	pub fn claim_of(&self, at: Coord) -> Option<&Claim> {
		self.claims.iter().find(|claim| claim.at == at)
	}
}

pub struct Replay {
	pub grid: Grid,
	pub turns: Vec<TurnRecord>,
	pub totals: Totals,
	pub score: [i32; 2],
	pub ended: bool,
	pub disqualified: [Option<String>; 2],
}

impl Replay {
	pub fn last_turn(&self) -> i32 {
		self.turns.last().map_or(0, |record| record.turn)
	}

	pub fn at_turn(&self, turn: i32) -> Option<&TurnRecord> {
		self.turns.iter().find(|record| record.turn == turn)
	}

	pub fn tracks_left(&self, owner: i8) -> i32 {
		self.grid
			.tiles
			.iter()
			.filter(|tile| tile.track == owner)
			.count() as i32
	}
}
