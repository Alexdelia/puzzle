use std::fs;
use std::ops::Range;
use std::path::PathBuf;

use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, ValueEnum)]
enum Order {
	Keep,
	Reverse,
	Shuffle,
}

#[derive(Parser)]
struct Args {
	#[arg(long)]
	source: PathBuf,
	#[arg(long)]
	target: PathBuf,
	#[arg(long, value_enum, default_value = "keep")]
	order: Order,
	#[arg(long, default_value_t = 1)]
	seed: u64,
	#[arg(long, default_value_t = 0)]
	translate: isize,
}

struct Image {
	width: usize,
	height: usize,
	pixels: Vec<bool>,
}

impl Image {
	fn parse(text: &str) -> Result<Self> {
		let mut lines = text.lines();
		let size: Vec<usize> = lines
			.next()
			.context("missing size line")?
			.split_whitespace()
			.map(str::parse)
			.collect::<Result<_, _>>()?;
		let [width, height] = size[..] else {
			bail!("size line must hold width and height");
		};
		let tokens: Vec<&str> = lines
			.next()
			.context("missing pixel line")?
			.split_whitespace()
			.collect();
		let mut pixels = Vec::with_capacity(width * height);
		for pair in tokens.chunks(2) {
			let [color, count] = pair else {
				bail!("odd token count");
			};
			let count: usize = count.parse()?;
			pixels.extend(std::iter::repeat_n(*color == "B", count));
		}
		ensure!(
			pixels.len() == width * height,
			"pixel count {count} differs from {width}x{height}",
			count = pixels.len()
		);
		Ok(Self {
			width,
			height,
			pixels,
		})
	}

	fn black(&self, row: usize, column: usize) -> bool {
		self.pixels[row * self.width + column]
	}

	fn set(&mut self, row: usize, column: usize, black: bool) {
		self.pixels[row * self.width + column] = black;
	}

	fn blank(&self) -> Self {
		Self {
			width: self.width,
			height: self.height,
			pixels: vec![false; self.width * self.height],
		}
	}

	fn encode(&self) -> String {
		let mut runs: Vec<(bool, usize)> = Vec::new();
		for &pixel in &self.pixels {
			match runs.last_mut() {
				Some((color, count)) if *color == pixel => *count += 1,
				_ => runs.push((pixel, 1)),
			}
		}
		let body: Vec<String> = runs
			.iter()
			.map(|&(black, count)| {
				format!("{color} {count}", color = if black { 'B' } else { 'W' })
			})
			.collect();
		format!(
			"{width} {height}\n{body}\n",
			width = self.width,
			height = self.height,
			body = body.join(" ")
		)
	}
}

struct Layout {
	x0: usize,
	staff_row: Vec<bool>,
	ledger_row: Vec<bool>,
	left: Vec<usize>,
	right: Vec<usize>,
	rightmost: usize,
}

impl Layout {
	fn detect(image: &Image) -> Result<Self> {
		let x0 = (0..image.width)
			.find(|&column| (0..image.height).any(|row| image.black(row, column)))
			.context("blank image")?;
		let staff_rows: Vec<usize> = (0..image.height)
			.filter(|&row| image.black(row, x0))
			.collect();
		let line_tops: Vec<usize> = staff_rows
			.iter()
			.enumerate()
			.filter(|&(index, &row)| index == 0 || staff_rows[index - 1] + 1 != row)
			.map(|(_, &row)| row)
			.collect();
		ensure!(
			line_tops.len() == 5,
			"{count} staff lines",
			count = line_tops.len()
		);
		let ledger_shift = line_tops[4] - line_tops[3];
		let mut staff_row = vec![false; image.height];
		for &row in &staff_rows {
			staff_row[row] = true;
		}
		let mut ledger_row = vec![false; image.height];
		for &row in &staff_rows {
			let ledger = row + ledger_shift;
			if row >= line_tops[4] && ledger < image.height {
				ledger_row[ledger] = true;
			}
		}
		let mut left = vec![image.width; image.height];
		let mut right = vec![0; image.height];
		let mut rightmost = 0;
		for &row in &staff_rows {
			if let Some(column) = (0..image.width).find(|&column| image.black(row, column)) {
				left[row] = column;
			}
			if let Some(column) = (0..image.width)
				.rev()
				.find(|&column| image.black(row, column))
			{
				right[row] = column;
				rightmost = rightmost.max(column);
			}
		}
		Ok(Self {
			x0,
			staff_row,
			ledger_row,
			left,
			right,
			rightmost,
		})
	}
}

fn note_segments(image: &Image, layout: &Layout) -> Result<Vec<Range<usize>>> {
	let mut segments: Vec<Range<usize>> = Vec::new();
	for column in 0..image.width {
		let occupied = (0..image.height).any(|row| {
			!layout.staff_row[row] && !layout.ledger_row[row] && image.black(row, column)
		});
		if !occupied {
			continue;
		}
		match segments.last_mut() {
			Some(segment) if segment.end == column => segment.end += 1,
			_ => segments.push(column..column + 1),
		}
	}
	let ledger_black = |column: usize| {
		(0..image.height).any(|row| layout.ledger_row[row] && image.black(row, column))
	};
	let mut bounds = vec![0];
	bounds.extend(
		segments
			.iter()
			.flat_map(|segment| [segment.start, segment.end]),
	);
	bounds.push(image.width);
	for (index, gap) in bounds.chunks(2).enumerate() {
		let [start, end] = gap[..] else {
			unreachable!()
		};
		let leading = (start..end)
			.take_while(|&column| ledger_black(column))
			.count();
		let trailing = (start..end)
			.rev()
			.take_while(|&column| ledger_black(column))
			.count();
		let (to_left, to_right) = if leading == end - start {
			let half = (end - start) / 2;
			(half, end - start - half)
		} else {
			(leading, trailing)
		};
		if index > 0 {
			segments[index - 1].end += to_left;
		}
		if index < segments.len() {
			segments[index].start -= to_right;
		}
	}
	Ok(segments)
}

fn permutation(order: Order, count: usize, seed: u64) -> Vec<usize> {
	let mut indices: Vec<usize> = (0..count).collect();
	match order {
		Order::Keep => {}
		Order::Reverse => indices.reverse(),
		Order::Shuffle => {
			let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
			for last in (1..count).rev() {
				state ^= state << 13;
				state ^= state >> 7;
				state ^= state << 17;
				let pick = (state % (last as u64 + 1)) as usize;
				indices.swap(last, pick);
			}
		}
	}
	indices
}

fn augment(
	slot: usize,
	candidates: &[Vec<usize>],
	seen: &mut [bool],
	matched: &mut [usize],
) -> bool {
	for &note in &candidates[slot] {
		if seen[note] {
			continue;
		}
		seen[note] = true;
		if matched[note] == usize::MAX || augment(matched[note], candidates, seen, matched) {
			matched[note] = slot;
			return true;
		}
	}
	false
}

fn assign(
	segments: &[Range<usize>],
	order: &[usize],
	rightmost: usize,
	width: usize,
) -> Result<Vec<usize>> {
	let count = segments.len();
	let widths: Vec<usize> = segments
		.iter()
		.map(|segment| segment.end - segment.start)
		.collect();
	let mut candidates: Vec<Vec<usize>> = Vec::with_capacity(count);
	for (slot, segment) in segments.iter().enumerate() {
		let end = if slot + 1 < count {
			segments[slot + 1].start
		} else {
			width.min(rightmost + 1)
		};
		let slot_width = end - segment.start;
		let requested = order[slot];
		let mut preferred = Vec::with_capacity(count);
		preferred.push(requested);
		preferred.extend((0..count).filter(|&note| note != requested));
		let fitting: Vec<usize> = preferred
			.into_iter()
			.filter(|&note| {
				if slot + 1 == count {
					widths[note] <= slot_width
				} else {
					widths[note] < slot_width
				}
			})
			.collect();
		ensure!(
			!fitting.is_empty(),
			"no note fits slot {slot} of width {slot_width}"
		);
		candidates.push(fitting);
	}
	let mut matched = vec![usize::MAX; count];
	for slot in 0..count {
		let mut seen = vec![false; count];
		ensure!(
			augment(slot, &candidates, &mut seen, &mut matched),
			"no assignment for slot {slot}"
		);
	}
	let mut assignment = vec![usize::MAX; count];
	for (note, &slot) in matched.iter().enumerate() {
		ensure!(slot != usize::MAX, "unassigned note {note}");
		assignment[slot] = note;
	}
	Ok(assignment)
}

fn translated(
	image: &Image,
	layout: &Layout,
	segments: &[Range<usize>],
	delta: isize,
) -> Result<Vec<(usize, usize)>> {
	let mut slots = Vec::with_capacity(segments.len());
	for segment in segments {
		let start = segment
			.start
			.checked_add_signed(delta)
			.context("negative target column")?;
		let end = segment
			.end
			.checked_add_signed(delta)
			.context("negative target column")?;
		ensure!(
			start > layout.x0,
			"note starts at {start}, not after staff column {x0}",
			x0 = layout.x0
		);
		ensure!(
			end <= layout.rightmost + 1 && end <= image.width,
			"note ends at {end}, beyond staff {right}",
			right = layout.rightmost
		);
		slots.push((start, end));
	}
	Ok(slots)
}

fn rebuild(
	image: &Image,
	layout: &Layout,
	segments: &[Range<usize>],
	slots: &[(usize, usize)],
	assignment: &[usize],
) -> Result<Image> {
	let mut result = image.blank();
	for row in 0..image.height {
		if !layout.staff_row[row] {
			continue;
		}
		for column in layout.left[row]..=layout.right[row] {
			result.set(row, column, true);
		}
	}
	for (slot, &note) in assignment.iter().enumerate() {
		if note == usize::MAX {
			continue;
		}
		let (start, end) = slots[slot];
		let source = &segments[note];
		ensure!(
			start + source.len() <= end,
			"note {note} of width {note_width} does not fit slot {slot}",
			note_width = source.len()
		);
		for row in 0..image.height {
			for column in source.clone() {
				result.set(row, start + column - source.start, image.black(row, column));
			}
		}
	}
	Ok(result)
}

fn main() -> Result<()> {
	let args = Args::parse();
	let input = fs::read_to_string(&args.source)
		.with_context(|| format!("read {source}", source = args.source.display()))?;
	let expected_file = args.source.with_extension("out");
	let expected = fs::read_to_string(&expected_file)
		.with_context(|| format!("read {file}", file = expected_file.display()))?;
	let notes: Vec<&str> = expected.split_whitespace().collect();
	let image = Image::parse(&input)?;
	let layout = Layout::detect(&image)?;
	let segments = note_segments(&image, &layout)?;
	ensure!(
		segments.len() == notes.len(),
		"{count} column segments for {note_count} notes",
		count = segments.len(),
		note_count = notes.len()
	);
	let (slots, assignment) = if args.translate == 0 {
		let order = permutation(args.order, notes.len(), args.seed);
		let assignment = assign(&segments, &order, layout.rightmost, image.width)?;
		let mut slots = Vec::with_capacity(segments.len());
		for (slot, segment) in segments.iter().enumerate() {
			let end = if slot + 1 < segments.len() {
				segments[slot + 1].start
			} else {
				image.width
			};
			slots.push((segment.start, end));
		}
		(slots, assignment)
	} else {
		let slots = translated(&image, &layout, &segments, args.translate)?;
		let assignment: Vec<usize> = (0..segments.len()).collect();
		(slots, assignment)
	};
	let result = rebuild(&image, &layout, &segments, &slots, &assignment)?;
	let regrouped = note_segments(&result, &layout)?;
	ensure!(
		regrouped.len() == notes.len(),
		"rebuilt image splits into {count} segments",
		count = regrouped.len()
	);
	ensure!(
		(101..5000).contains(&result.width) && (71..300).contains(&result.height),
		"size {width}x{height} out of constraints",
		width = result.width,
		height = result.height
	);
	let reordered: Vec<&str> = assignment.iter().map(|&note| notes[note]).collect();
	fs::write(args.target.with_extension("in"), result.encode())?;
	fs::write(
		args.target.with_extension("out"),
		format!("{notes}\n", notes = reordered.join(" ")),
	)?;
	Ok(())
}
