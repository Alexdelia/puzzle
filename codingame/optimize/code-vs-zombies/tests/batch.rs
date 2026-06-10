use code_vs_zombies::{InitialState, Referee};

fn initial() -> InitialState {
	InitialState {
		player: (3989, 3259),
		human_list: vec![(3647, 384), (60, 3262), (2391, 1601), (2363, 3422)],
		zombie_list: vec![
			(6485, 499),
			(7822, 446),
			(9202, 826),
			(11060, 253),
			(12568, 808),
			(14148, 650),
			(6571, 1893),
			(8484, 2013),
			(9669, 1968),
			(7570, 3338),
			(9780, 3611),
			(8360, 4767),
			(9804, 4154),
			(10935, 4977),
			(12310, 4614),
			(13891, 4302),
			(913, 5636),
			(2410, 5912),
			(3952, 6143),
			(4615, 5995),
			(6568, 6085),
			(8204, 5579),
			(9049, 5470),
			(30, 6798),
			(1798, 6682),
			(3247, 7664),
			(5005, 7319),
			(6415, 7094),
			(8159, 7447),
			(9550, 6847),
		],
	}
}

const SOLUTION: &[(i32, i32)] = &[
	(2402, 5512),
	(641, 4882),
	(39, 5598),
	(2812, 6192),
	(4839, 5278),
	(4545, 975),
	(4210, 755),
	(3875, 534),
	(3359, 662),
	(5325, 952),
	(4228, 3837),
	(4838, 3902),
	(6150, 1224),
	(7132, 1462),
	(8261, 1331),
];

#[test]
fn batch_identical_simulation_match() {
	let referee = Referee::new(&initial(), SOLUTION.len()).expect("init");

	let simulation_count = 1024;
	let solution_list: Vec<Vec<(i32, i32)>> =
		(0..simulation_count).map(|_| SOLUTION.to_vec()).collect();
	let score_list = referee.run(&solution_list).expect("run");
	assert_eq!(score_list.len(), simulation_count);
	for (i, s) in score_list.iter().enumerate() {
		assert_eq!(*s, 6460, "sim {i} got score {s}");
	}
}

#[test]
fn batch_different_sims_produce_different_score_list() {
	let referee = Referee::new(&initial(), SOLUTION.len()).expect("init");

	let standing_still: Vec<(i32, i32)> = (0..15).map(|_| (3989, 3259)).collect();
	let solution_list = vec![SOLUTION.to_vec(), standing_still];

	let score_list = referee.run(&solution_list).expect("run");
	assert_eq!(score_list.len(), 2);
	assert_eq!(score_list[0], 6460);
	assert_ne!(
		score_list[1], score_list[0],
		"standing still should produce a different score"
	);
}
