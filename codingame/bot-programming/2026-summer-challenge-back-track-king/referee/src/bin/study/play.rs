use btk::arena::{MatchResult, run_match};
use btk::driver::make_driver;
use btk::game::{DEFAULT_LEAGUE, Game};
use btk::grid::Grid;
use std::time::Duration;

pub fn play(
	grid: &Grid,
	specs: [&str; 2],
	timeout: u64,
) -> Result<(Vec<[String; 2]>, MatchResult), String> {
	let wait = Duration::from_millis(timeout);
	let mut first = make_driver(specs[0], wait)?;
	let mut second = make_driver(specs[1], wait)?;
	let mut game = Game::new(grid.clone(), DEFAULT_LEAGUE);
	let mut answers: Vec<[String; 2]> = Vec::new();
	let result = {
		let mut record = |_: &Game, _: &_, said: &[String; 2], _| answers.push(said.clone());
		run_match(
			&mut game,
			[first.as_mut(), second.as_mut()],
			None,
			Some(&mut record),
		)
	};
	Ok((answers, result))
}
