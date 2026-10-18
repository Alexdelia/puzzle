use std::{io, str::FromStr};

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FeatNumber {
	One = 1,
	Two = 2,
	Three = 3,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FeatShading {
	Outlined,
	Striped,
	Solid,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FeatColor {
	Red,
	Green,
	Purple,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FeatShape {
	Diamond,
	Oval,
	Squiggle,
}

impl FromStr for FeatNumber {
	type Err = ();

	fn from_str(input: &str) -> Result<FeatNumber, Self::Err> {
		match input {
			"1" => Ok(FeatNumber::One),
			"2" => Ok(FeatNumber::Two),
			"3" => Ok(FeatNumber::Three),
			_ => Err(()),
		}
	}
}

impl FromStr for FeatShading {
	type Err = ();

	fn from_str(input: &str) -> Result<FeatShading, Self::Err> {
		match input {
			"OUTLINED" => Ok(FeatShading::Outlined),
			"STRIPED" => Ok(FeatShading::Striped),
			"SOLID" => Ok(FeatShading::Solid),
			_ => Err(()),
		}
	}
}

impl FromStr for FeatColor {
	type Err = ();

	fn from_str(input: &str) -> Result<FeatColor, Self::Err> {
		match input {
			"RED" => Ok(FeatColor::Red),
			"GREEN" => Ok(FeatColor::Green),
			"PURPLE" => Ok(FeatColor::Purple),
			_ => Err(()),
		}
	}
}

impl FromStr for FeatShape {
	type Err = ();

	fn from_str(input: &str) -> Result<FeatShape, Self::Err> {
		match input {
			"DIAMOND" => Ok(FeatShape::Diamond),
			"OVAL" => Ok(FeatShape::Oval),
			"SQUIGGLE" => Ok(FeatShape::Squiggle),
			_ => Err(()),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Card {
	number: FeatNumber,
	shading: FeatShading,
	color: FeatColor,
	shape: FeatShape,
}

impl FromStr for Card {
	type Err = ();

	fn from_str(input: &str) -> Result<Card, Self::Err> {
		let parts: Vec<&str> = input.trim().split_whitespace().collect();
		if parts.len() != 4 {
			return Err(());
		}

		let number = parts[0].parse::<FeatNumber>()?;
		let shading = parts[1].parse::<FeatShading>()?;
		let color = parts[2].parse::<FeatColor>()?;
		let shape = parts[3].parse::<FeatShape>()?;

		Ok(Card {
			number,
			shading,
			color,
			shape,
		})
	}
}

fn read_line() -> String {
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	input_line
}

macro_rules! all_same_or_all_different {
	($a:expr, $b:expr, $c:expr) => {
		(($a == $b && $b == $c) || ($a != $b && $a != $c && $b != $c))
	};
}

fn is_set(hand: &[Card; 3]) -> bool {
	all_same_or_all_different!(hand[0].number, hand[1].number, hand[2].number)
		&& all_same_or_all_different!(hand[0].shading, hand[1].shading, hand[2].shading)
		&& all_same_or_all_different!(hand[0].color, hand[1].color, hand[2].color)
		&& all_same_or_all_different!(hand[0].shape, hand[1].shape, hand[2].shape)
}

fn is_set_in_hand(hand: &[Card]) -> bool {
	let hand_size = hand.len();

	for x in 0..hand_size {
		for y in (x + 1)..hand_size {
			for z in (y + 1)..hand_size {
				if is_set(&[hand[x], hand[y], hand[z]]) {
					return true;
				}
			}
		}
	}

	false
}

fn make_deck() -> Vec<Card> {
	let mut deck = Vec::new();

	for number in [FeatNumber::One, FeatNumber::Two, FeatNumber::Three].iter() {
		for shading in [
			FeatShading::Outlined,
			FeatShading::Striped,
			FeatShading::Solid,
		]
		.iter()
		{
			for color in [FeatColor::Red, FeatColor::Green, FeatColor::Purple].iter() {
				for shape in [FeatShape::Diamond, FeatShape::Oval, FeatShape::Squiggle].iter() {
					deck.push(Card {
						number: *number,
						shading: *shading,
						color: *color,
						shape: *shape,
					});
				}
			}
		}
	}

	deck
}

fn main() {
	let mut deck = make_deck();
	let mut hand = Vec::new();

	let n = parse_input!(read_line(), usize);
	for _ in 0..n {
		let card = read_line().parse::<Card>().expect("Invalid card input");

		let index = deck
			.iter()
			.position(|&c| c == card)
			.expect("Card not found in deck");
		deck.remove(index);

		hand.push(card);
	}

	let mut found_set_count = 0;

	for card in deck.iter() {
		hand.push(*card);
		if is_set_in_hand(&hand) {
			found_set_count += 1;
		}
		hand.pop();
	}

	let p = found_set_count as f64 / deck.len() as f64;
	println!("{p:.4}");
}
