use std::fmt::Write;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
	Safe,
	Tree,
	House,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct CellParams {
	pub cut_duration: u32,
	pub fire_duration: i8,
	pub value: u32,
}

#[derive(Clone, Debug)]
pub struct Map {
	pub width: usize,
	pub height: usize,
	pub tree: CellParams,
	pub house: CellParams,
	pub start: usize,
	pub cells: Vec<Cell>,
}

impl Map {
	pub fn parse(text: &str) -> Result<Map, String> {
		let mut lines = text.lines();
		let mut numbers = |what: &str, count: usize| -> Result<Vec<i64>, String> {
			let line = lines.next().ok_or(format!("missing {what}"))?;
			let parsed = line
				.split_whitespace()
				.map(|token| token.parse::<i64>())
				.collect::<Result<Vec<_>, _>>()
				.map_err(|e| format!("{what}: {e}"))?;
			if parsed.len() != count {
				return Err(format!("{what}: expected {count} numbers, got {line:?}"));
			}
			Ok(parsed)
		};
		let cell_params = |numbers: Vec<i64>| CellParams {
			cut_duration: numbers[0] as u32,
			fire_duration: numbers[1] as i8,
			value: numbers[2] as u32,
		};
		let tree = cell_params(numbers("tree params", 3)?);
		let house = cell_params(numbers("house params", 3)?);
		let size = numbers("size", 2)?;
		let start = numbers("fire start", 2)?;
		let (width, height) = (size[0] as usize, size[1] as usize);
		let cells = lines
			.take(height)
			.flat_map(|row| row.trim_end().chars().take(width))
			.map(|c| match c {
				'#' => Ok(Cell::Safe),
				'.' => Ok(Cell::Tree),
				'X' => Ok(Cell::House),
				other => Err(format!("unknown cell {other:?}")),
			})
			.collect::<Result<Vec<_>, _>>()?;
		if cells.len() != width * height {
			return Err(format!(
				"grid has {count} cells, expected {width}*{height}",
				count = cells.len()
			));
		}
		let map = Map {
			width,
			height,
			tree,
			house,
			start: start[1] as usize * width + start[0] as usize,
			cells,
		};
		if let Some(i) =
			(0..width * height).find(|&i| map.is_border(i) && map.cells[i] != Cell::Safe)
		{
			return Err(format!(
				"border cell {coords:?} is not safe",
				coords = map.coords(i)
			));
		}
		Ok(map)
	}

	pub fn params(&self, i: usize) -> CellParams {
		match self.cells[i] {
			Cell::Safe => CellParams::default(),
			Cell::Tree => self.tree,
			Cell::House => self.house,
		}
	}

	pub fn index(&self, x: usize, y: usize) -> usize {
		y * self.width + x
	}

	pub fn coords(&self, i: usize) -> (usize, usize) {
		(i % self.width, i / self.width)
	}

	pub fn neighbours(&self, i: usize) -> [usize; 4] {
		[i - self.width, i - 1, i + 1, i + self.width]
	}

	pub fn total_value(&self) -> u32 {
		(0..self.cells.len()).map(|i| self.params(i).value).sum()
	}

	pub fn init_input(&self) -> String {
		let mut input = String::new();
		for p in [self.tree, self.house] {
			writeln!(
				input,
				"{cut} {fire} {value}",
				cut = p.cut_duration,
				fire = p.fire_duration,
				value = p.value
			)
			.unwrap();
		}
		let (start_x, start_y) = self.coords(self.start);
		writeln!(
			input,
			"{width} {height}\n{start_x} {start_y}",
			width = self.width,
			height = self.height
		)
		.unwrap();
		for row in self.cells.chunks(self.width) {
			let symbols = row.iter().map(|cell| match cell {
				Cell::Safe => '#',
				Cell::Tree => '.',
				Cell::House => 'X',
			});
			input.extend(symbols);
			input.push('\n');
		}
		input
	}

	fn is_border(&self, i: usize) -> bool {
		let (x, y) = self.coords(i);
		x == 0 || y == 0 || x + 1 == self.width || y + 1 == self.height
	}
}
