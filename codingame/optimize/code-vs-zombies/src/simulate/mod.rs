pub mod referee;

pub use referee::*;

pub type Axis = u16;

pub const MAX_W: Axis = 16000;
pub const MAX_H: Axis = 9000;

pub const PLAYER_MOVE: f64 = 1000.0;
pub const ZOMBIE_MOVE: f64 = 400.0;
pub const PLAYER_RANGE_SQ: f64 = 4_000_000.0;

#[derive(Clone, Debug)]
pub struct State {
	pub player: (f64, f64),
	pub human_list: Vec<(f64, f64)>,
	pub human_alive_list: Vec<bool>,
	pub zombie_list: Vec<(f64, f64)>,
	pub zombie_alive_list: Vec<bool>,
	pub alive_h: usize,
	pub alive_z: usize,
	pub score: Score,
	pub over: bool,
}

impl State {
	pub fn from_initial(init: &InitialState) -> Self {
		let nh = init.human_list.len();
		let nz = init.zombie_list.len();
		Self {
			player: (init.player.0 as f64, init.player.1 as f64),
			human_list: init
				.human_list
				.iter()
				.map(|&(x, y)| (x as f64, y as f64))
				.collect(),
			human_alive_list: vec![true; nh],
			zombie_list: init
				.zombie_list
				.iter()
				.map(|&(x, y)| (x as f64, y as f64))
				.collect(),
			zombie_alive_list: vec![true; nz],
			alive_h: nh,
			alive_z: nz,
			score: 0,
			over: false,
		}
	}

	pub fn step(&mut self, target: Coord) {
		if self.over {
			return;
		}
		let tx = target.0 as f64;
		let ty = target.1 as f64;

		let nz = self.zombie_list.len();
		let mut new_zombie_list = self.zombie_list.clone();
		for (z, slot) in new_zombie_list.iter_mut().enumerate() {
			if !self.zombie_alive_list[z] {
				continue;
			}
			let (pzx, pzy) = self.zombie_list[z];
			let mut best_d2 = f64::INFINITY;
			let mut zt = (0.0, 0.0);
			for (h, &(hx, hy)) in self.human_list.iter().enumerate() {
				if !self.human_alive_list[h] {
					continue;
				}
				let dx = hx - pzx;
				let dy = hy - pzy;
				let d2 = dx * dx + dy * dy;
				if d2 < best_d2 {
					best_d2 = d2;
					zt = (hx, hy);
				}
			}
			let (px, py) = self.player;
			let dx = px - pzx;
			let dy = py - pzy;
			let d2 = dx * dx + dy * dy;
			if d2 < best_d2 {
				best_d2 = d2;
				zt = (px, py);
			}
			let dist = best_d2.sqrt();
			if dist <= ZOMBIE_MOVE {
				*slot = zt;
			} else {
				let nx = (pzx + (zt.0 - pzx) * ZOMBIE_MOVE / dist).trunc();
				let ny = (pzy + (zt.1 - pzy) * ZOMBIE_MOVE / dist).trunc();
				*slot = (nx, ny);
			}
		}
		self.zombie_list = new_zombie_list;

		let (px, py) = self.player;
		let adx = tx - px;
		let ady = ty - py;
		let adist = (adx * adx + ady * ady).sqrt();
		if adist <= PLAYER_MOVE {
			self.player = (tx, ty);
		} else {
			let nx = (px + adx * PLAYER_MOVE / adist).trunc();
			let ny = (py + ady * PLAYER_MOVE / adist).trunc();
			self.player = (nx, ny);
		}

		let base_score = (self.alive_h as Score) * (self.alive_h as Score) * 10;
		let mut fib_prev: Score = 1;
		let mut fib_cur: Score = 1;
		for z in 0..nz {
			if !self.zombie_alive_list[z] {
				continue;
			}
			let (zx, zy) = self.zombie_list[z];
			let kdx = zx - self.player.0;
			let kdy = zy - self.player.1;
			let kd2 = kdx * kdx + kdy * kdy;
			if kd2 <= PLAYER_RANGE_SQ {
				self.zombie_alive_list[z] = false;
				self.alive_z -= 1;
				self.score += base_score * fib_cur;
				let fib_next = fib_prev + fib_cur;
				fib_prev = fib_cur;
				fib_cur = fib_next;
			}
		}

		for z in 0..nz {
			if !self.zombie_alive_list[z] {
				continue;
			}
			let (zx, zy) = self.zombie_list[z];
			for h in 0..self.human_list.len() {
				if !self.human_alive_list[h] {
					continue;
				}
				let (hx, hy) = self.human_list[h];
				if zx == hx && zy == hy {
					self.human_alive_list[h] = false;
					self.alive_h -= 1;
					break;
				}
			}
		}

		if self.alive_h == 0 {
			self.score = 0;
			self.over = true;
		} else if self.alive_z == 0 {
			self.over = true;
		}
	}
}

pub fn simulate(init: &InitialState, move_list: &[Coord]) -> (Score, usize) {
	let mut state = State::from_initial(init);
	let mut end_turn = move_list.len();
	for (i, &m) in move_list.iter().enumerate() {
		state.step(m);
		if state.over {
			end_turn = i + 1;
			break;
		}
	}
	(state.score, end_turn)
}
