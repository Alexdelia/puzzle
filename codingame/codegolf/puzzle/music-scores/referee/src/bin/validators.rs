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
	pad_left: usize,
	#[arg(long, default_value_t = 0)]
	pad_right: usize,
	#[arg(long, default_value_t = 0)]
	pad_top: usize,
	#[arg(long, default_value_t = 0)]
	pad_bottom: usize,
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

fn note_segments(image: &Image) -> Result<Vec<Range<usize>>> {
	let staff_left = (0..image.width)
		.find(|&column| (0..image.height).any(|row| image.black(row, column)))
		.context("blank image")?;
	let staff_rows: Vec<usize> = (0..image.height)
		.filter(|&row| image.black(row, staff_left))
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
	let ledger_rows: Vec<usize> = staff_rows
		.iter()
		.filter(|&&row| row >= line_tops[4])
		.map(|&row| row + ledger_shift)
		.filter(|&row| row < image.height)
		.collect();
	let free_rows: Vec<usize> = (0..image.height)
		.filter(|&row| !image.black(row, staff_left) && !ledger_rows.contains(&row))
		.collect();
	let mut segments: Vec<Range<usize>> = Vec::new();
	for column in 0..image.width {
		if !free_rows.iter().any(|&row| image.black(row, column)) {
			continue;
		}
		match segments.last_mut() {
			Some(segment) if segment.end == column => segment.end += 1,
			_ => segments.push(column..column + 1),
		}
	}
	let ledger_black = |column: usize| ledger_rows.iter().any(|&row| image.black(row, column));
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

fn rearrange(image: &Image, segments: &[Range<usize>], order: &[usize], args: &Args) -> Image {
	let staff_only_column = segments[0].start - 1;
	let mut columns: Vec<Option<usize>> = vec![None; args.pad_left];
	let mut cursor = 0;
	for (slot, segment) in segments.iter().enumerate() {
		if slot > 0 && cursor == segment.start {
			columns.push(Some(staff_only_column));
		}
		columns.extend((cursor..segment.start).map(Some));
		columns.extend(segments[order[slot]].clone().map(Some));
		cursor = segment.end;
	}
	columns.extend((cursor..image.width).map(Some));
	columns.extend(std::iter::repeat_n(None, args.pad_right));
	let width = columns.len();
	let height = args.pad_top + image.height + args.pad_bottom;
	let mut pixels = vec![false; width * height];
	for row in 0..image.height {
		for (target, source) in columns.iter().enumerate() {
			if let Some(column) = source {
				pixels[(row + args.pad_top) * width + target] = image.black(row, *column);
			}
		}
	}
	Image {
		width,
		height,
		pixels,
	}
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
	let segments = note_segments(&image)?;
	ensure!(
		segments.len() == notes.len(),
		"{segments} column segments for {notes} notes",
		segments = segments.len(),
		notes = notes.len()
	);
	let order = permutation(args.order, notes.len(), args.seed);
	let result = rearrange(&image, &segments, &order, &args);
	ensure!(
		(101..5000).contains(&result.width) && (71..300).contains(&result.height),
		"size {width}x{height} out of constraints",
		width = result.width,
		height = result.height
	);
	let regrouped = note_segments(&result)?;
	ensure!(
		regrouped.len() == notes.len(),
		"rearranged image splits into {count} segments: {regrouped:?} from {segments:?}",
		count = regrouped.len()
	);
	let reordered: Vec<&str> = order.iter().map(|&index| notes[index]).collect();
	fs::write(args.target.with_extension("in"), result.encode())?;
	fs::write(
		args.target.with_extension("out"),
		format!("{notes}\n", notes = reordered.join(" ")),
	)?;
	Ok(())
}
