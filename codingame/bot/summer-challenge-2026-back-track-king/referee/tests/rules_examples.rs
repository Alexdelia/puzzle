use btk::game::{Connections, Rules, State};
use btk::map::{Map, PLAIN, Town};

fn example_map() -> Map {
	Map::new(
		5,
		4,
		vec![PLAIN; 20],
		vec![0; 20],
		vec![
			Town {
				x: 0,
				y: 0,
				desired: vec![1, 2],
			},
			Town {
				x: 4,
				y: 3,
				desired: vec![],
			},
			Town {
				x: 4,
				y: 0,
				desired: vec![],
			},
		],
	)
}

fn lay(map: &Map, state: &mut State, owner: i8, cells: &[(usize, usize)]) {
	for &(x, y) in cells {
		state.owner[map.idx(x, y)] = owner;
	}
}

fn path_of(map: &Map, conn: &Connections, src: u8, dst: u8) -> Vec<(usize, usize)> {
	let pi = conn.pairs.iter().position(|&p| p == (src, dst)).unwrap();
	assert!(conn.live[pi], "connection {src}-{dst} should be active");
	conn.paths[pi].iter().map(|&c| map.xy(c as usize)).collect()
}

#[test]
fn example_one() {
	let map = example_map();
	let mut state = State::new(&map);
	lay(&map, &mut state, 0, &[(0, 1), (0, 2), (0, 3), (4, 1)]);
	lay(&map, &mut state, 1, &[(1, 3), (2, 3), (3, 3), (4, 2)]);

	let mut conn = Connections::new(&map);
	conn.recompute(&map, &state);

	assert_eq!(
		path_of(&map, &conn, 0, 1),
		[
			(0, 0),
			(0, 1),
			(0, 2),
			(0, 3),
			(1, 3),
			(2, 3),
			(3, 3),
			(4, 3)
		]
	);
	assert_eq!(
		path_of(&map, &conn, 0, 2),
		[
			(0, 0),
			(0, 1),
			(0, 2),
			(0, 3),
			(1, 3),
			(2, 3),
			(3, 3),
			(4, 3),
			(4, 2),
			(4, 1),
			(4, 0)
		]
	);
	assert_eq!(conn.gains(&state, &Rules::default()), [7, 7]);
}

#[test]
fn example_two() {
	let map = example_map();
	let mut state = State::new(&map);
	lay(
		&map,
		&mut state,
		0,
		&[(0, 1), (0, 2), (0, 3), (4, 1), (1, 0), (2, 0), (3, 0)],
	);
	lay(&map, &mut state, 1, &[(1, 3), (2, 3), (3, 3), (4, 2)]);

	let mut conn = Connections::new(&map);
	conn.recompute(&map, &state);

	assert_eq!(
		path_of(&map, &conn, 0, 2),
		[(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
	);
	assert_eq!(
		path_of(&map, &conn, 0, 1),
		[
			(0, 0),
			(1, 0),
			(2, 0),
			(3, 0),
			(4, 0),
			(4, 1),
			(4, 2),
			(4, 3)
		]
	);
	assert_eq!(conn.gains(&state, &Rules::default()), [7, 1]);
}
