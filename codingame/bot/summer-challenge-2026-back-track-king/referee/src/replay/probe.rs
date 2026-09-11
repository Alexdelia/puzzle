use crate::driver::Driver;
use crate::game::Game;
use crate::grid::Grid;
use crate::proto::{init_lines, turn_lines};

pub struct Probe<'a> {
	pub side: usize,
	pub driver: &'a mut dyn Driver,
	pub answers: Vec<String>,
	pub broke: Option<String>,
}

impl<'a> Probe<'a> {
	pub fn new(side: usize, driver: &'a mut dyn Driver) -> Self {
		Probe {
			side,
			driver,
			answers: Vec::new(),
			broke: None,
		}
	}

	pub fn start(&mut self, grid: &Grid, frame: &mut String) {
		init_lines(grid, self.side, frame);
		if let Err(why) = self.driver.send(frame) {
			self.broke = Some(why);
		}
	}

	pub fn ask(&mut self, game: &Game, frame: &mut String) {
		if self.broke.is_some() {
			return;
		}
		turn_lines(game, self.side, frame);
		match self.driver.send(frame).and_then(|()| self.driver.answer()) {
			Ok(line) => self.answers.push(line),
			Err(why) => self.broke = Some(why),
		}
	}
}
