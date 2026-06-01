use crate::{
	dist::dist,
	referee::env::{CHECKPOINT_RADIUS, Coord},
};

/// based on https://github.com/Illedan/CGSearchRace/blob/master/SearchRace/src/main/java/com/codingame/game/Unit.java#L36
pub fn intersect(checkpoint: Coord, from: Coord, to: Coord) -> bool {
	if dist(from.x, from.y, checkpoint.x, checkpoint.y) <= CHECKPOINT_RADIUS {
		return true;
	}

	let vx = to.x - from.x;
	let vy = to.y - from.y;
	let fx = from.x - checkpoint.x;
	let fy = from.y - checkpoint.y;

	let a = vx * vx + vy * vy;
	if a <= 0.0 {
		return false;
	}

	let b = 2.0 * (fx * vx + fy * vy);
	let c = fx * fx + fy * fy - CHECKPOINT_RADIUS * CHECKPOINT_RADIUS;
	let discriminant = b * b - 4.0 * a * c;

	if discriminant < 0.0 {
		return false;
	}

	let t = (-b - discriminant.sqrt()) / (2.0 * a);
	t > 0.0 && t <= 1.0
}
