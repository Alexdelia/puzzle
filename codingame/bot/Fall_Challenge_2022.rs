use std::collections::{HashSet, VecDeque};
use std::io;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

type Scrap = u8;
type Matter = u32;
type Unit = u8;
type Coord = (usize, usize);

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
    m_units: Vec<Coord>,
    o_units: Vec<Coord>,
    m_recycler: Vec<Coord>,
    o_recycler: Vec<Coord>,
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

    fn dist(&self, src: Coord, dst: Coord) -> usize {
        ((src.0 as isize - dst.0 as isize).abs() + (src.1 as isize - dst.1 as isize).abs()) as usize
    }

    fn neighbors(&self, pos: Coord) -> Vec<Coord> {
        let mut ret = Vec::new();

        if pos.0 > 0 && self.map[pos.0 - 1][pos.1].scrap > 0 {
            ret.push((pos.0 - 1, pos.1));
        }
        if pos.0 < self.h - 1 && self.map[pos.0 + 1][pos.1].scrap > 0 {
            ret.push((pos.0 + 1, pos.1));
        }
        if pos.1 > 0 && self.map[pos.0][pos.1 - 1].scrap > 0 {
            ret.push((pos.0, pos.1 - 1));
        }
        if pos.1 < self.w - 1 && self.map[pos.0][pos.1 + 1].scrap > 0 {
            ret.push((pos.0, pos.1 + 1));
        }

        ret
    }

    fn next_to_not_owned(&self, pos: Coord) -> bool {
        self.neighbors(pos)
            .iter()
            .any(|i| self.map[i.0][i.1].owner != Owner::Me)
    }

    fn next_to_op(&self, pos: Coord) -> bool {
        self.neighbors(pos)
            .iter()
            .any(|i| self.map[i.0][i.1].owner == Owner::Op)
    }

    fn find_contact_tiles(&self) -> Vec<Coord> {
        let mut ret: Vec<Coord> = Vec::new();
        let mut seen_m: HashSet<Coord> = HashSet::from_iter(self.m_units.iter().cloned());
        let mut seen_o: HashSet<Coord> = HashSet::from_iter(self.o_units.iter().cloned());
        let mut q_m: VecDeque<Coord> = VecDeque::from(self.m_units.clone());
        let mut q_o: VecDeque<Coord> = VecDeque::from(self.o_units.clone());

        while !q_m.is_empty() || !q_o.is_empty() {
            if let Some(cur) = q_m.pop_front() {
                if seen_o.contains(&cur) {
                    ret.push(cur);
                }
                for n in self.neighbors(cur) {
                    if !seen_m.contains(&n) {
                        seen_m.insert(n);
                        q_m.push_back(n);
                    }
                }
            }

            if let Some(cur) = q_o.pop_front() {
                if seen_m.contains(&cur) {
                    ret.push(cur);
                }
                for n in self.neighbors(cur) {
                    if !seen_o.contains(&n) {
                        seen_o.insert(n);
                        q_o.push_back(n);
                    }
                }
            }
        }

        ret
    }

    fn pop_closest(&self, src: &mut Vec<Coord>, dst: Coord) -> Option<Coord> {
        return src
            .iter()
            .enumerate()
            .min_by_key(|(_, c)| self.dist(**c, dst))
            .map(|(i, _)| src.swap_remove(i));
    }

    fn build(&mut self, pos: Coord) {
        self.m_recycler.push(pos);
        self.map[pos.0][pos.1].recycler = true;
        self.map[pos.0][pos.1].can_build = false;
        self.map[pos.0][pos.1].can_spawn = false;
        self.m_m -= 10;
        print!("BUILD {} {};", pos.1, pos.0);
    }

    fn r#move(&mut self, src: Coord, dst: Coord) {
        self.map[dst.0][dst.1].owner = Owner::Me;
        self.map[src.0][src.1].unit -= 1;
        if self.map[src.0][src.1].unit == 0 {
            self.map[src.0][src.1].can_build = true;
        }
        self.map[dst.0][dst.1].unit += 1;
        self.map[dst.0][dst.1].can_build = false;
        print!("MOVE 1 {} {} {} {};", src.1, src.0, dst.1, dst.0);
    }

    fn spawn(&mut self, pos: Coord) {
        // might put tile at Owner::Me
        self.map[pos.0][pos.1].unit += 1;
        self.m_m -= 10;
        print!("SPAWN 1 {} {};", pos.1, pos.0);
    }

    fn move_to_contact(&mut self, contact_tiles: &mut Vec<Coord>) {
        while !contact_tiles.is_empty() && !self.m_units.is_empty() {
            let tile = contact_tiles.pop().unwrap();
            let mut needed_units = if self.map[tile.0][tile.1].owner == Owner::Op {
                self.map[tile.0][tile.1].unit as usize + 1
            } else {
                1
            };

            while needed_units > 0 && !self.m_units.is_empty() {
                self.r#move(self.pop_closest(&mut self.m_units, tile).unwrap(), tile);
                needed_units -= 1;
            }
        }
    }

    fn build_all(&mut self) {
        if self.o_recycler.is_empty() {
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
        while let Some(u) = self.m_units.pop() {
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

        let mut set: HashSet<Coord> = HashSet::new();
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

            set.remove(&(closest.1, closest.2));
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

        let mut contact_tiles = e.find_contact_tiles();
        // attack (move) protect (spawn) and block (build) in contact
        e.move_to_contact(&mut contact_tiles);
        // spawn 1 more unit than op

        // e.build_all();
        // e.move_all();
        // e.spawn_all();

        println!();
    }
}
