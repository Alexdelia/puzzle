use crate::referee::env::Axis;

pub fn dist(ax: Axis, ay: Axis, bx: Axis, by: Axis) -> f64 {
	((ax - bx).powi(2) + (ay - by).powi(2)).sqrt()
}
