use std::collections::HashMap;
use std::io;
use std::str::from_utf8;

type Seed = u32;

const SIZE: usize = 4;
const PRINT_SIZE: usize = 9000;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

fn get_info() -> Seed {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let seed = parse_input!(input_line, Seed);

    io::stdin().read_line(&mut input_line).unwrap();

    for _ in 0..SIZE {
        io::stdin().read_line(&mut input_line).unwrap();
    }

    seed
}

fn pre_calc(seed: Seed) -> Option<&'static str> {
    let d: HashMap<Seed, &str> = HashMap::from([(0, "U")]);

    d.get(&seed).copied()
}

pub fn main() {
    let mut seed: Seed;

    seed = get_info();

    if let Some(x) = pre_calc(seed) {
        let cs = x
            .as_bytes()
            .chunks(PRINT_SIZE)
            .map(from_utf8)
            .collect::<Result<Vec<&str>, _>>()
            .unwrap();

        for s in cs {
            println!("{}", s);
            seed = get_info();
        }
    }

    loop {
        let m = ['U', 'D', 'L', 'R'];
        println!("{}", m[seed as usize % 4]);
        seed = get_info();
    }
}
