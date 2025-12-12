pub type State = u16;

pub fn parse_state(s: &str) -> State {
	assert!(s.len() <= 16, "State string '{s}' is too long");
	todo!()
}
