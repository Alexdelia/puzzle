use crate::game::{Action, Game};
use crate::map::Map;
use std::fs;

pub fn read_cuts(map: &Map, path: &str) -> Vec<usize> {
	fs::read_to_string(path)
		.unwrap_or_default()
		.lines()
		.filter_map(|line| {
			let mut numbers = line.split_whitespace().map(|n| n.parse::<usize>().ok());
			Some(map.index(numbers.next()??, numbers.next()??))
		})
		.collect()
}

pub fn write_cuts(map: &Map, path: &str, cuts: &[usize]) {
	let text: String = cuts
		.iter()
		.map(|&i| {
			let (x, y) = map.coords(i);
			format!("{x} {y}\n")
		})
		.collect();
	fs::write(path, text).unwrap();
}

pub fn burnt_region(map: &Map, cuts: &[usize]) -> Vec<bool> {
	let mut game = Game::new(map);
	for &i in cuts {
		while game.cooldown() > 0 {
			game.play(Action::Wait).unwrap();
		}
		game.play(Action::Cut(i)).unwrap();
	}
	game.burn_out();
	game.progress().iter().map(|&p| p >= 0).collect()
}
