use crate::referee::env::Coord;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Segment {
	pub a: Coord,
	pub b: Coord,
}
