use super::geom::{V2, v2};

pub const WIDTH: f64 = 16000.0;
pub const HEIGHT: f64 = 9000.0;
pub const POD_RADIUS: f64 = 400.0;
pub const POD_RSQ: f64 = 800.0 * 800.0;
pub const CP_RADIUS: f64 = 600.0;
pub const CP_RSQ: f64 = 600.0 * 600.0;
pub const FRICTION: f64 = 0.85;
pub const MAX_ROTATE: f64 = std::f64::consts::PI / 10.0;
pub const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;
pub const RAD_TO_DEG: f64 = 180.0 / std::f64::consts::PI;
pub const TAU: f64 = std::f64::consts::TAU;
pub const SEPARATION_EPSILON: f64 = 0.00001;
pub const MIN_IMPULSE: f64 = 120.0;
pub const BOOST_THRUST: f64 = 650.0;
pub const USED_BOOST_THRUST: f64 = 200.0;
pub const MAX_THRUST: i64 = 200;
pub const SHIELD_TIMER: u8 = 4;
pub const TIMEOUT_TURNS: i32 = 100;
pub const MAX_TURNS: usize = 500;
pub const NO_COLLISION: f64 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pod {
	pub pos: V2,
	pub vel: V2,
	pub angle: f64,
	pub next: usize,
	pub shield: u8,
	pub boosted: bool,
	pub won: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CmdKind {
	Thrust(f64),
	Boost,
	Shield,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cmd {
	pub target: V2,
	pub kind: CmdKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
	pub pods: [Pod; 4],
	pub timeout: [i32; 2],
	pub turn: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StepEvents {
	pub bounces: u32,
}

pub fn player_of(pod: usize) -> usize {
	pod / 2
}

pub fn round_cg(x: f64) -> f64 {
	(x + 0.5).floor()
}

pub fn get_angle(from: V2, to: V2) -> f64 {
	(to.y - from.y).atan2(to.x - from.x)
}

impl Pod {
	pub fn diff_angle(&self, target: V2) -> f64 {
		let a = get_angle(self.pos, target);
		let da = (a - self.angle) % TAU;
		(2.0 * da) % TAU - da
	}

	pub fn rotate_toward(&mut self, target: V2) {
		let mut a = get_angle(self.pos, target);
		let d = self.diff_angle(target);
		if d < -MAX_ROTATE {
			a = self.angle - MAX_ROTATE;
		}
		if d > MAX_ROTATE {
			a = self.angle + MAX_ROTATE;
		}
		self.angle = a;
	}

	pub fn rotate_free(&mut self, target: V2) {
		self.angle = 0.0;
		let mut a = self.diff_angle(target);
		while a < 0.0 {
			a += TAU;
		}
		while a > TAU {
			a -= TAU;
		}
		self.angle = a;
	}

	pub fn apply_thrust(&mut self, thrust: f64) {
		let (sin, cos) = self.angle.sin_cos();
		self.vel.x += cos * thrust;
		self.vel.y += sin * thrust;
	}

	fn end_turn(&mut self) {
		self.vel.x = (self.vel.x * FRICTION).trunc();
		self.vel.y = (self.vel.y * FRICTION).trunc();
		self.pos.x = round_cg(self.pos.x);
		self.pos.y = round_cg(self.pos.y);
		if self.shield > 0 {
			self.shield -= 1;
		}
	}
}

pub fn collide_time(a: &Pod, b: &Pod, rsq: f64) -> f64 {
	let p = b.pos - a.pos;
	let p_len2 = p.dot(p);
	if p_len2 <= rsq {
		return 0.0;
	}
	let v = b.vel - a.vel;
	let dot = p.dot(v);
	if dot > 0.0 {
		return NO_COLLISION;
	}
	let v_len2 = v.dot(v);
	let disc = dot * dot - v_len2 * (p_len2 - rsq);
	if disc < 0.0 {
		return NO_COLLISION;
	}
	(-dot - disc.sqrt()) / v_len2
}

pub fn cp_hit(seg_start: V2, seg_end: V2, cp: V2) -> bool {
	let d = seg_end - seg_start;
	let pd2 = d.dot(d);
	let mut pp = seg_start;
	if pd2 != 0.0 {
		let u = (cp - seg_start).dot(d) / pd2;
		if u > 1.0 {
			pp = seg_end;
		} else if u > 0.0 {
			pp = v2(seg_start.x + u * d.x, seg_start.y + u * d.y);
		}
	}
	let dx = pp.x - cp.x;
	let dy = pp.y - cp.y;
	dx * dx + dy * dy < CP_RSQ
}

fn bounce(pods: &mut [Pod; 4], i: usize, j: usize) {
	let mut a = pods[i];
	let mut b = pods[j];

	let mut normal = b.pos - a.pos;
	let dd = normal.norm();
	normal.x /= dd;
	normal.y /= dd;

	let relv = a.vel - b.vel;

	let m1 = if a.shield == SHIELD_TIMER { 0.1 } else { 1.0 };
	let m2 = if b.shield == SHIELD_TIMER { 0.1 } else { 1.0 };

	let mut force = normal.dot(relv) / (m1 + m2);
	if force < MIN_IMPULSE {
		force += MIN_IMPULSE;
	} else {
		force += force;
	}

	let impulse = normal * -force;
	a.vel.x += impulse.x * m1;
	a.vel.y += impulse.y * m1;
	b.vel.x += -impulse.x * m2;
	b.vel.y += -impulse.y * m2;

	if dd <= 800.0 {
		let gap = dd - 800.0;
		a.pos.x += normal.x * -(-gap / 2.0 + SEPARATION_EPSILON);
		a.pos.y += normal.y * -(-gap / 2.0 + SEPARATION_EPSILON);
		b.pos.x += normal.x * (-gap / 2.0 + SEPARATION_EPSILON);
		b.pos.y += normal.y * (-gap / 2.0 + SEPARATION_EPSILON);
	}

	pods[i] = a;
	pods[j] = b;
}

fn forward(pods: &mut [Pod; 4], t: f64) {
	for p in pods {
		p.pos.x += p.vel.x * t;
		p.pos.y += p.vel.y * t;
	}
}

fn pass_checkpoint(state: &mut State, total_cps: usize, pod: usize) {
	let p = &mut state.pods[pod];
	p.next += 1;
	if p.next >= total_cps {
		p.next = total_cps - 1;
		p.won = true;
	}
	state.timeout[player_of(pod)] = TIMEOUT_TURNS;
}

fn move_pods(cps: &[V2], state: &mut State) -> StepEvents {
	let mut events = StepEvents::default();
	let mut t = 1.0;
	let mut seg_start = [
		state.pods[0].pos,
		state.pods[1].pos,
		state.pods[2].pos,
		state.pods[3].pos,
	];
	while t > 0.0 {
		let mut first = t;
		let mut cli = 0usize;
		let mut clj = 0usize;
		for i in (1..4).rev() {
			for j in (0..i).rev() {
				let tx = collide_time(&state.pods[i], &state.pods[j], POD_RSQ);
				if tx <= first {
					first = tx;
					cli = i;
					clj = j;
				}
			}
		}
		forward(&mut state.pods, first);
		t -= first;
		if cli != clj {
			bounce(&mut state.pods, cli, clj);
			events.bounces += 1;
		}
		if t > 0.0 {
			for i in 0..4 {
				if cp_hit(seg_start[i], state.pods[i].pos, cps[state.pods[i].next]) {
					pass_checkpoint(state, cps.len(), i);
				}
			}
			seg_start = [
				state.pods[0].pos,
				state.pods[1].pos,
				state.pods[2].pos,
				state.pods[3].pos,
			];
		}
	}
	for i in 0..4 {
		state.pods[i].end_turn();
		if cp_hit(seg_start[i], state.pods[i].pos, cps[state.pods[i].next]) {
			pass_checkpoint(state, cps.len(), i);
		}
	}
	state.timeout[0] -= 1;
	state.timeout[1] -= 1;
	events
}

pub fn apply_cmd(turn: usize, pod: &mut Pod, cmd: &Cmd) {
	let mut thrust = match cmd.kind {
		CmdKind::Thrust(v) => v,
		CmdKind::Boost => {
			if !pod.boosted {
				pod.boosted = true;
				BOOST_THRUST
			} else {
				USED_BOOST_THRUST
			}
		}
		CmdKind::Shield => {
			pod.shield = SHIELD_TIMER;
			0.0
		}
	};
	if pod.shield > 0 {
		thrust = 0.0;
	}
	if cmd.target == pod.pos {
		return;
	}
	if turn == 0 {
		pod.rotate_free(cmd.target);
	} else {
		pod.rotate_toward(cmd.target);
	}
	pod.apply_thrust(thrust);
}

pub fn step(cps: &[V2], state: &mut State, cmds: &[Cmd; 4]) -> StepEvents {
	for i in 0..4 {
		apply_cmd(state.turn, &mut state.pods[i], &cmds[i]);
	}
	let events = move_pods(cps, state);
	state.turn += 1;
	events
}

pub const START_OFFSETS: [V2; 4] = [
	v2(500.0, -500.0),
	v2(-500.0, 500.0),
	v2(1500.0, -1500.0),
	v2(-1500.0, 1500.0),
];

pub fn init_state(cps: &[V2]) -> State {
	let mut dir = cps[1] - cps[0];
	let dd = cps[1].dist(cps[0]);
	dir.x /= dd;
	dir.y /= dd;

	let make_pod = |i: usize| Pod {
		pos: v2(
			round_cg(cps[0].x + dir.y * START_OFFSETS[i].x),
			round_cg(cps[0].y + dir.x * START_OFFSETS[i].y),
		),
		vel: v2(0.0, 0.0),
		angle: -DEG_TO_RAD,
		next: 1,
		shield: 0,
		boosted: false,
		won: false,
	};

	State {
		pods: [make_pod(0), make_pod(1), make_pod(2), make_pod(3)],
		timeout: [TIMEOUT_TURNS, TIMEOUT_TURNS],
		turn: 0,
	}
}

pub fn expand_cps(raw: &[V2], laps: usize) -> Vec<V2> {
	let mut cps = Vec::with_capacity(raw.len() * laps + 1);
	for _ in 0..laps {
		cps.extend_from_slice(raw);
	}
	cps.push(raw[0]);
	cps
}

pub fn progress_score(cps: &[V2], pod: &Pod) -> f64 {
	pod.next as f64 * 1000000.0 - pod.pos.dist(cps[pod.next])
}

pub fn progress_winner(cps: &[V2], state: &State) -> usize {
	let mut winner = 0;
	let mut best = 0.0;
	for i in 0..4 {
		let score = progress_score(cps, &state.pods[i]);
		if score > best {
			best = score;
			winner = i;
		}
	}
	player_of(winner)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn round_cg_matches_go() {
		assert_eq!(round_cg(2.5), 3.0);
		assert_eq!(round_cg(-2.5), -2.0);
		assert_eq!(round_cg(-2.51), -3.0);
		assert_eq!(round_cg(2.49), 2.0);
		assert_eq!(round_cg(-0.3), 0.0);
	}

	#[test]
	fn trunc_matches_go() {
		assert_eq!((3.7f64).trunc(), 3.0);
		assert_eq!((-3.7f64).trunc(), -3.0);
	}

	fn track4() -> Vec<V2> {
		expand_cps(
			&[
				v2(12460.0, 1350.0),
				v2(10540.0, 5980.0),
				v2(3580.0, 5180.0),
				v2(13580.0, 7600.0),
			],
			3,
		)
	}

	#[test]
	fn straight_drive_accumulates_speed() {
		let cps = track4();
		let mut state = init_state(&cps);
		let target = cps[1];
		let cmds = [Cmd {
			target,
			kind: CmdKind::Thrust(100.0),
		}; 4];
		let start = state.pods[0].pos;
		for _ in 0..5 {
			step(&cps, &mut state, &cmds);
		}
		assert!(state.pods[0].pos.dist(cps[1]) < start.dist(cps[1]));
		assert!(state.pods[0].vel.norm() > 200.0);
		assert_eq!(state.timeout, [95, 95]);
	}

	#[test]
	fn head_on_collision_bounces() {
		let cps = track4();
		let mut state = init_state(&cps);
		state.pods[0].pos = v2(1000.0, 1000.0);
		state.pods[0].vel = v2(300.0, 0.0);
		state.pods[2].pos = v2(2000.0, 1000.0);
		state.pods[2].vel = v2(-300.0, 0.0);
		state.pods[1].pos = v2(8000.0, 8000.0);
		state.pods[3].pos = v2(1000.0, 8000.0);
		state.turn = 5;
		let cmds = [Cmd {
			target: v2(8000.0, 4500.0),
			kind: CmdKind::Thrust(0.0),
		}; 4];
		let events = step(&cps, &mut state, &cmds);
		assert_eq!(events.bounces, 1);
		assert!(state.pods[0].vel.x < 0.0);
		assert!(state.pods[2].vel.x > 0.0);
		assert!(state.pods[0].pos.dist(state.pods[2].pos) >= 800.0);
	}

	#[test]
	fn shield_makes_heavy() {
		let cps = track4();
		let make = |shielded: bool| {
			let mut state = init_state(&cps);
			state.pods[0].pos = v2(1000.0, 1000.0);
			state.pods[0].vel = v2(0.0, 0.0);
			state.pods[2].pos = v2(1900.0, 1000.0);
			state.pods[2].vel = v2(-400.0, 0.0);
			state.pods[1].pos = v2(8000.0, 8000.0);
			state.pods[3].pos = v2(1000.0, 8000.0);
			state.turn = 5;
			let mut cmds = [Cmd {
				target: v2(1000.0, 900.0),
				kind: CmdKind::Thrust(0.0),
			}; 4];
			if shielded {
				cmds[0].kind = CmdKind::Shield;
			}
			step(&cps, &mut state, &cmds);
			state
		};
		let normal = make(false);
		let shielded = make(true);
		assert!(shielded.pods[0].vel.x.abs() < normal.pods[0].vel.x.abs());
		assert_eq!(shielded.pods[0].shield, 3);
	}

	#[test]
	fn boost_once_per_pod() {
		let cps = track4();
		let mut state = init_state(&cps);
		state.turn = 5;
		state.pods[0].angle = 0.0;
		state.pods[0].pos = v2(2000.0, 4500.0);
		let target = v2(15000.0, 4500.0);
		let boost = [Cmd {
			target,
			kind: CmdKind::Boost,
		}; 4];
		step(&cps, &mut state, &boost);
		let v_first = state.pods[0].vel.norm();
		assert!(v_first > 500.0);
		let mut state2 = state;
		state2.pods[0].vel = v2(0.0, 0.0);
		step(&cps, &mut state2, &boost);
		assert!(state2.pods[0].vel.norm() < 200.0);
	}

	#[test]
	fn checkpoint_pass_resets_timeout_then_decrements() {
		let cps = track4();
		let mut state = init_state(&cps);
		state.turn = 5;
		state.pods[0].pos = cps[1] - v2(700.0, 0.0);
		state.pods[0].vel = v2(300.0, 0.0);
		let cmds = [Cmd {
			target: v2(15000.0, 1.0),
			kind: CmdKind::Thrust(0.0),
		}; 4];
		state.timeout = [50, 50];
		step(&cps, &mut state, &cmds);
		assert_eq!(state.pods[0].next, 2);
		assert_eq!(state.timeout, [99, 49]);
	}

	#[test]
	fn finishing_pins_last_checkpoint() {
		let raw = [v2(1000.0, 1000.0), v2(5000.0, 1000.0)];
		let cps = expand_cps(&raw, 1);
		let mut state = init_state(&cps);
		state.turn = 5;
		state.pods[1].pos = v2(9000.0, 8000.0);
		state.pods[2].pos = v2(12000.0, 8000.0);
		state.pods[3].pos = v2(15000.0, 8000.0);
		state.pods[0].next = cps.len() - 1;
		state.pods[0].pos = v2(1000.0, 2500.0);
		state.pods[0].vel = v2(0.0, -1000.0);
		state.pods[0].angle = -std::f64::consts::FRAC_PI_2;
		let cmds = [Cmd {
			target: v2(1000.0, 0.0),
			kind: CmdKind::Thrust(0.0),
		}; 4];
		step(&cps, &mut state, &cmds);
		assert!(state.pods[0].won);
		assert_eq!(state.pods[0].next, cps.len() - 1);
	}
}
