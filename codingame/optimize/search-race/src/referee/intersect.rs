use crate::{
	dist::dist,
	referee::env::{CHECKPOINT_RADIUS, Coord},
};

/// based on https://github.com/Illedan/CGSearchRace/blob/master/SearchRace/src/main/java/com/codingame/game/Unit.java#L36
pub fn intersect(checkpoint: Coord, from: Coord, to: Coord) -> bool {
	if dist(from.x, from.y, checkpoint.x, checkpoint.y) <= CHECKPOINT_RADIUS {
		return true;
	}

	let d = Coord {
		x: to.x - from.x,
		y: to.y - from.y,
	};
	let f = Coord {
		x: from.x - checkpoint.x,
		y: from.y - checkpoint.y,
	};

	let a = d.x * d.x + d.y * d.y;
	let b = 2.0 * (f.x * d.x + f.y * d.y);
	let c = f.x * f.x + f.y * f.y - CHECKPOINT_RADIUS * CHECKPOINT_RADIUS;

	let discriminant = b * b - 4.0 * a * c;
	if discriminant < 0.0 {
		return false;
	}

	let sqrt_discriminant = discriminant.sqrt();
	let t = (-b - sqrt_discriminant) / (2.0 * a);

	t <= 0.0
}
