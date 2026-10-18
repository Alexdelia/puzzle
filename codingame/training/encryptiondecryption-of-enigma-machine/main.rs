use std::io;

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

struct Enigma {
	shift: u8,
	rotor: [Rotor; 3],
}

struct Rotor([u8; 26]);

impl From<String> for Rotor {
	fn from(s: String) -> Self {
		let mut rotor = [0; 26];

		for (i, c) in s.chars().enumerate() {
			rotor[i] = c as u8 - b'A';
		}

		Rotor(rotor)
	}
}

impl Enigma {
	fn new(shift: u8, rotor: &[String]) -> Enigma {
		Enigma {
			shift,
			rotor: [
				rotor[0].clone().into(),
				rotor[1].clone().into(),
				rotor[2].clone().into(),
			],
		}
	}

	fn encode(&self, message: String) -> String {
		let mut encoded = String::new();
		let mut shift = self.shift;

		for c in message.chars() {
			encoded.push(self.encode_char(c, shift));
			shift += 1;
		}

		encoded
	}

	fn encode_char(&self, c: char, shift: u8) -> char {
		if c.is_ascii_uppercase() {
			let mut c = (c as u8 - b'A' + shift) % 26;

			for rotor in self.rotor.iter() {
				c = rotor.0[c as usize];
			}

			(c + b'A') as char
		} else {
			c
		}
	}

	fn decode(&self, message: String) -> String {
		let mut decoded = String::new();
		let mut shift = self.shift;

		for c in message.chars() {
			decoded.push(self.decode_char(c, shift));
			shift += 1;
		}

		decoded
	}

	fn decode_char(&self, c: char, shift: u8) -> char {
		if c.is_ascii_uppercase() {
			let mut c = c as u8 - b'A';

			for rotor in self.rotor.iter().rev() {
				c = rotor.0.iter().position(|&x| x == c).unwrap() as u8;
			}

			c = (c + 130 - shift) % 26;

			(c + b'A') as char
		} else {
			c
		}
	}
}

fn main() {
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let op = input_line.trim_matches('\n').to_string();

	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let shift = parse_input!(input_line, u8);

	let mut rotor = Vec::new();

	for _ in 0..3 {
		let mut input_line = String::new();
		io::stdin().read_line(&mut input_line).unwrap();
		rotor.push(input_line.trim_matches('\n').to_string());
	}

	let enigma = Enigma::new(shift, &rotor);

	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	let message = input_line.trim_matches('\n').to_string();

	match op.as_str() {
		"ENCODE" => println!("{}", enigma.encode(message)),
		"DECODE" => println!("{}", enigma.decode(message)),
		_ => panic!("Unknown operation"),
	}
}
