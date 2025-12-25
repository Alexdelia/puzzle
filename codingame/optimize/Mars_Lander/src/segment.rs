use crate::referee::env::Coord;

#[derive(Clone, Copy, Debug)]
pub struct Segment {
	pub a: Coord,
	pub b: Coord,
}
