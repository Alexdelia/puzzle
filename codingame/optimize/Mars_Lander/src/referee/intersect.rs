use crate::{
	referee::env::{Axis, Coord},
	segment::Segment,
};

pub fn intersect(a: &Segment, b: &Segment) -> bool {
	let d1 = direction(&a.a, &a.b, &b.a);
	let d2 = direction(&a.a, &a.b, &b.b);

	if (d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0) {
		return true;
	}

	let d3 = direction(&b.a, &b.b, &a.a);
	let d4 = direction(&b.a, &b.b, &a.b);

	(d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0)
}

#[inline]
fn direction(a: &Coord, b: &Coord, c: &Coord) -> Axis {
	(b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}
