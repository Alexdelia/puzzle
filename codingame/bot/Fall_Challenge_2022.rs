use std::collections::HashSet;
use std::io;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

type Scrap = u8;
type Matter = u32;
type Unit = u8;

#[derive(Clone, PartialEq, Eq)]
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
    m_units: Vec<(usize, usize)>,
    o_units: Vec<(usize, usize)>,
    m_recycler: Vec<(usize, usize)>,
    o_recycler: Vec<(usize, usize)>,
}

impl Env {
    fn new(w: usize, h: usize) -> Self {
        Env {
            w,
            h,
            m_m: 0,
            o_m: 0,
            map: vec![vec![Tile::default(); w]; h],
            m_units: Vec::new(),
            o_units: Vec::new(),
            m_recycler: Vec::new(),
            o_recycler: Vec::new(),
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

        self.m_units.clear();
        self.o_units.clear();
        self.m_recycler.clear();
        self.o_recycler.clear();

        for x in 0..self.h {
            for y in 0..self.w {
                let mut input_line = String::new();
                io::stdin().read_line(&mut input_line).unwrap();

                self.map[x][y].update(&input_line);

                if self.map[x][y].owner == Owner::Me {
                    for _ in 0..self.map[x][y].unit {
                        self.m_units.push((x, y));
                    }
                    if self.map[x][y].recycler {
                        self.m_recycler.push((x, y));
                    }
                } else if self.map[x][y].owner == Owner::Op {
                    for _ in 0..self.map[x][y].unit {
                        self.o_units.push((x, y));
                    }
                    if self.map[x][y].recycler {
                        self.o_recycler.push((x, y));
                    }
                }
            }
        }
    }

    fn dist(&self, src: (usize, usize), dst: (usize, usize)) -> usize {
        ((src.0 as isize - dst.0 as isize).abs() + (src.1 as isize - dst.1 as isize).abs()) as usize
    }

    fn next_to_not_owned(&self, pos: (usize, usize)) -> bool {
        if pos.0 > 0
            && self.map[pos.0 - 1][pos.1].owner != Owner::Me
            && self.map[pos.0 - 1][pos.1].scrap > 0
        {
            return true;
        } else if pos.0 < self.h - 1
            && self.map[pos.0 + 1][pos.1].owner != Owner::Me
            && self.map[pos.0 + 1][pos.1].scrap > 0
        {
            return true;
        } else if pos.1 > 0
            && self.map[pos.0][pos.1 - 1].owner != Owner::Me
            && self.map[pos.0][pos.1 - 1].scrap > 0
        {
            return true;
        } else if pos.1 < self.w - 1
            && self.map[pos.0][pos.1 + 1].owner != Owner::Me
            && self.map[pos.0][pos.1 + 1].scrap > 0
        {
            return true;
        }
        false
    }

    fn build(&mut self, pos: (usize, usize)) {
        self.m_recycler.push(pos);
        self.map[pos.0][pos.1].recycler = true;
        self.map[pos.0][pos.1].can_build = false;
        self.map[pos.0][pos.1].can_spawn = false;
        self.m_m -= 10;
        print!("BUILD {} {};", pos.1, pos.0);
    }

    fn r#move(&mut self, src: (usize, usize), dst: (usize, usize)) {
        self.map[dst.0][dst.1].owner = Owner::Me;
        self.map[src.0][src.1].unit -= 1;
        if self.map[src.0][src.1].unit == 0 {
            self.map[src.0][src.1].can_build = true;
        }
        self.map[dst.0][dst.1].unit += 1;
        self.map[dst.0][dst.1].can_build = false;
        print!("MOVE 1 {} {} {} {};", src.1, src.0, dst.1, dst.0);
    }

    fn spawn(&mut self, pos: (usize, usize)) {
        // might put tile at Owner::Me
        self.map[pos.0][pos.1].unit += 1;
        self.m_m -= 10;
        print!("SPAWN 1 {} {};", pos.1, pos.0);
    }

    fn build_all(&mut self) {
        if self.o_recycler.len() == 0 {
            return;
        }

        while self.m_m >= 10 && self.m_recycler.len() <= self.o_recycler.len() {
            let mut most_scrap: (Scrap, usize, usize) = (0, 0, 0);

            for x in 0..self.h {
                for y in 0..self.w {
                    if self.map[x][y].owner == Owner::Me
                        && self.map[x][y].can_build
                        && self.map[x][y].scrap > most_scrap.0
                        && self.map[x][y].unit == 0
                    {
                        most_scrap = (self.map[x][y].scrap, x, y);
                    }
                }
            }

            self.build((most_scrap.1, most_scrap.2));
        }
    }

    fn move_all(&mut self) {
        while !self.m_units.is_empty() {
            let u = self.m_units.pop().unwrap();
            let mut closest: (usize, usize, usize) = (self.w + self.h, 0, 0);

            for x in 0..self.h {
                for y in 0..self.w {
                    if self.map[x][y].owner != Owner::Me
                        && self.map[x][y].scrap > 0
                        && !self.map[x][y].recycler
                    {
                        let dist = self.dist(u, (x, y));
                        if dist < closest.0 {
                            closest = (dist, x, y);
                        }
                    }
                }
            }

            self.r#move(u, (closest.1, closest.2));
        }
    }

    fn spawn_all(&mut self) {
        if self.m_m < 10 {
            return;
        }

        let mut set: HashSet<(usize, usize)> = HashSet::new();
        for x in 0..self.h {
            for y in 0..self.w {
                if self.map[x][y].owner == Owner::Me
                    && self.map[x][y].can_spawn
                    && self.map[x][y].unit == 0
                    && !self.map[x][y].recycler
                    && self.next_to_not_owned((x, y))
                {
                    set.insert((x, y));
                }
            }
        }

        while self.m_m >= 10 {
            // empty owned tile closest to center and next to not owned tile with scrap
            let mut closest: (usize, usize, usize) = (self.w + self.h, 0, 0);
            for (x, y) in set.iter() {
                let dist = self.dist((self.h / 2, self.w / 2), (*x, *y));
                if dist < closest.0 {
                    closest = (dist, *x, *y);
                }
            }

            if closest.0 == self.w + self.h {
                break;
            }

            self.spawn((closest.1, closest.2));
        }
    }
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

        e.build_all();
        e.move_all();
        e.spawn_all();

        println!();
    }
}
