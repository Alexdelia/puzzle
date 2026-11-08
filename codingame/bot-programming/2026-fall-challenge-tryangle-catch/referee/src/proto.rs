use crate::game::Game;

pub const NEW_PATHS_LEAGUE: i32 = 3;
const INPUT_CAPACITY: usize = 4096;

fn push_int(out: &mut Vec<u8>, value: i64) {
	if (0..10).contains(&value) {
		out.push(b'0' + value as u8);
		return;
	}
	if (10..100).contains(&value) {
		out.extend_from_slice(&[b'0' + (value / 10) as u8, b'0' + (value % 10) as u8]);
		return;
	}
	let mut digits = [0u8; 21];
	let mut start = digits.len();
	let mut rest = value.unsigned_abs();
	loop {
		start -= 1;
		digits[start] = b'0' + (rest % 10) as u8;
		rest /= 10;
		if rest == 0 {
			break;
		}
	}
	if value < 0 {
		start -= 1;
		digits[start] = b'-';
	}
	out.extend_from_slice(&digits[start..]);
}

fn push_line<const N: usize>(out: &mut Vec<u8>, fields: [i64; N]) {
	for (index, field) in fields.into_iter().enumerate() {
		if index > 0 {
			out.push(b' ');
		}
		push_int(out, field);
	}
	out.push(b'\n');
}

pub fn init_lines(game: &Game, out: &mut Vec<u8>) {
	let board = &game.board;
	push_line(out, [board.len() as i64]);
	for (id, p) in board.points.iter().enumerate() {
		push_line(out, [id as i64, p.x.into(), p.y.into()]);
	}
}

pub fn linkable(game: &Game) -> Vec<(usize, usize)> {
	if game.league < NEW_PATHS_LEAGUE {
		return Vec::new();
	}
	let board = &game.board;
	let mut links = Vec::new();
	for a in 0..board.len() {
		for b in a + 1..board.len() {
			if !board.connected(a, b) && board.can_connect(a, b) {
				links.push((a, b));
			}
		}
	}
	links
}

pub fn turn_lines(game: &Game, player: usize, out: &mut Vec<u8>) {
	let board = &game.board;
	let opponent = 1 - player;
	let [mine, theirs] = [player, opponent].map(|p| game.players[p].score.into());
	push_line(out, [mine, theirs]);
	for (id, units) in board.units.iter().enumerate() {
		push_line(
			out,
			[id as i64, units[player].into(), units[opponent].into()],
		);
	}
	let edges = board.edges();
	push_line(out, [edges.len() as i64]);
	for (a, b) in edges {
		push_line(out, [a as i64, b as i64]);
	}
	push_line(out, [board.triangles.len() as i64]);
	for &t in &board.triangles {
		let triangle = board.triangle(t);
		let [a, b, c] = triangle.nodes.map(|n| n as i64);
		let owner = match triangle.owner {
			None => -1,
			Some(owner) if owner == player => 0,
			Some(_) => 1,
		};
		let [me, them] = [player, opponent].map(|p| i64::from(triangle.can_capture[p]));
		push_line(out, [a, b, c, owner, me, them]);
	}
	let links = linkable(game);
	push_line(out, [links.len() as i64]);
	for (a, b) in links {
		push_line(out, [a as i64, b as i64]);
	}
}

pub fn input(game: &Game, player: usize, initial: bool) -> String {
	let mut out = Vec::with_capacity(INPUT_CAPACITY);
	if initial {
		init_lines(game, &mut out);
	}
	turn_lines(game, player, &mut out);
	String::from_utf8(out).expect("input lines are ascii")
}
