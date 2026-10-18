use super::geom::{V2, v2};
use super::race::{Action, BotAction, Driver, Obs, PodObs, Track};

pub const DRIFT_COMPENSATION: f64 = 3.0;
pub const BRAKE_DIST: f64 = 1200.0;
pub const BRAKE_MIN_FACTOR: f64 = 0.3;
pub const GIVE_UP_ANGLE: f64 = 90.0;
pub const BOOST_MIN_DIST: f64 = 4500.0;
pub const BOOST_MAX_ANGLE: f64 = 3.0;
pub const SHIELD_DIST: f64 = 820.0;
pub const SHIELD_MIN_CLOSING: f64 = 250.0;

pub struct SimpleBot {
	cps: Vec<V2>,
}

impl SimpleBot {
	pub fn new() -> Self {
		SimpleBot { cps: Vec::new() }
	}
}

impl Default for SimpleBot {
	fn default() -> Self {
		Self::new()
	}
}

fn norm_deg180(mut a: f64) -> f64 {
	while a > 180.0 {
		a -= 360.0;
	}
	while a < -180.0 {
		a += 360.0;
	}
	a
}

fn pos_of(p: &PodObs) -> V2 {
	v2(p.x as f64, p.y as f64)
}

fn vel_of(p: &PodObs) -> V2 {
	v2(p.vx as f64, p.vy as f64)
}

fn next_turn_pos(p: &PodObs) -> V2 {
	pos_of(p) + vel_of(p)
}

impl SimpleBot {
	fn pod_action(&self, obs: &Obs, idx: usize) -> Action {
		let me = &obs.pods[idx];
		let pos = pos_of(me);
		let vel = vel_of(me);
		let cp = self.cps[me.next_cp];

		let target = cp - vel * DRIFT_COMPENSATION;
		let dist_cp = pos.dist(cp);

		let desired_deg = (target.y - pos.y).atan2(target.x - pos.x).to_degrees();
		let rel = if obs.turn == 0 || me.angle < 0 {
			0.0
		} else {
			norm_deg180(desired_deg - me.angle as f64)
		};

		let mut act = if rel.abs() > GIVE_UP_ANGLE {
			BotAction::Thrust(0)
		} else {
			let angle_factor = 1.0 - (rel.abs() / GIVE_UP_ANGLE);
			let dist_factor = if dist_cp > BRAKE_DIST {
				1.0
			} else {
				BRAKE_MIN_FACTOR + (1.0 - BRAKE_MIN_FACTOR) * (dist_cp / BRAKE_DIST)
			};
			let f = angle_factor.min(dist_factor.max(angle_factor * dist_factor));
			BotAction::Thrust((100.0 * f).round() as i64)
		};

		if (obs.turn == 0 || rel.abs() < BOOST_MAX_ANGLE) && dist_cp > BOOST_MIN_DIST {
			act = BotAction::Boost;
		}

		for enemy in &obs.pods[2..4] {
			let my_next = next_turn_pos(me);
			let enemy_next = next_turn_pos(enemy);
			let closing = (vel - vel_of(enemy)).norm();
			if my_next.dist(enemy_next) < SHIELD_DIST && closing > SHIELD_MIN_CLOSING {
				act = BotAction::Shield;
			}
		}

		Action {
			x: target.x.round() as i64,
			y: target.y.round() as i64,
			act,
		}
	}
}

impl Driver for SimpleBot {
	fn init(&mut self, track: &Track) {
		self.cps = track.cps.clone();
	}

	fn act(&mut self, obs: &Obs) -> Option<[Action; 2]> {
		Some([self.pod_action(obs, 0), self.pod_action(obs, 1)])
	}
}
