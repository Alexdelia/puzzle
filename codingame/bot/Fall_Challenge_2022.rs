use std::io;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

type Scrap = u8;
type Matter = u32;
type Unit = u8;

#[derive(Clone)]
enum Owner {
    None,
    Me,
    Op,
}

impl Owner {
    fn from(i: i8) -> Self {
        match i {
            1 => Owner::Me,
            0 => Owner::Op,
            _ => Owner::None,
        }
    }
}

#[derive(Clone)]
struct Tile {
    scrap: Scrap,
    owner: Owner,
    unit: Unit,
    recycler: bool,
    can_build: bool,
    can_spawn: bool,
    in_range_of_recycler: bool,
}

impl Default for Tile {
    fn default() -> Self {
        Tile {
            scrap: 0,
            owner: Owner::None,
            unit: 0,
            recycler: false,
            can_build: false,
            can_spawn: false,
            in_range_of_recycler: false,
        }
    }
}

impl Tile {
    fn update(&mut self, input_line: &str) {
        let inputs = input_line.split(' ').collect::<Vec<_>>();

        self.scrap = parse_input!(inputs[0], Scrap);
        self.owner = Owner::from(parse_input!(inputs[1], i8));
        self.unit = parse_input!(inputs[2], Unit);
        self.recycler = parse_input!(inputs[3], u8) == 1;
        self.can_build = parse_input!(inputs[4], u8) == 1;
        self.can_spawn = parse_input!(inputs[5], u8) == 1;
        self.in_range_of_recycler = parse_input!(inputs[6], u8) == 1;
    }
}

struct Env {
    w: usize,
    h: usize,
    m_m: Matter,
    o_m: Matter,
    map: Vec<Vec<Tile>>,
}

impl Env {
    fn new(w: usize, h: usize) -> Self {
        Env {
            w,
            h,
            m_m: 0,
            o_m: 0,
            map: vec![vec![Tile::default(); w]; h],
        }
    }

    fn get_input(&mut self) {
        {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(' ').collect::<Vec<_>>();

            self.m_m = parse_input!(inputs[0], Matter);
            self.o_m = parse_input!(inputs[1], Matter);
        }

        for x in 0..self.h {
            for y in 0..self.w {
                let mut input_line = String::new();
                io::stdin().read_line(&mut input_line).unwrap();

                self.map[x][y].update(&input_line);
            }
        }
    }

    fn build(&mut self) {}
}

fn main() {
    let mut e: Env;
    {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(' ').collect::<Vec<_>>();
        e = Env::new(
            parse_input!(inputs[0], usize),
            parse_input!(inputs[1], usize),
        );
    }

    loop {
        e.get_input();
        println!("WAIT");
    }
}
