use std::collections::{HashSet, VecDeque};
use std::convert::TryInto;
use std::io;
use std::iter::FromIterator;

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
    action: bool,
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
            action: false,
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

        self.action = false;
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

    fn neighbors(&self, pos: Coord) -> Vec<Coord> {
        let mut ret = Vec::new();

        if pos.0 > 0 && self.map[pos.0 - 1][pos.1].scrap > 0 && !self.map[pos.0 - 1][pos.1].recycler
        {
            ret.push((pos.0 - 1, pos.1));
        }
        if pos.0 < self.h - 1
            && self.map[pos.0 + 1][pos.1].scrap > 0
            && !self.map[pos.0 + 1][pos.1].recycler
        {
            ret.push((pos.0 + 1, pos.1));
        }
        if pos.1 > 0 && self.map[pos.0][pos.1 - 1].scrap > 0 && !self.map[pos.0][pos.1 - 1].recycler
        {
            ret.push((pos.0, pos.1 - 1));
        }
        if pos.1 < self.w - 1
            && self.map[pos.0][pos.1 + 1].scrap > 0
            && !self.map[pos.0][pos.1 + 1].recycler
        {
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

    fn find_contact(&self) -> Vec<Coord> {
        let mut ret: HashSet<Coord> = HashSet::new();
        let mut seen_m: HashSet<Coord> = HashSet::from_iter(self.m_units.iter().cloned());
        let mut seen_o: HashSet<Coord> = HashSet::from_iter(self.o_units.iter().cloned());
        let mut q_m: VecDeque<Coord> = VecDeque::from(self.m_units.clone());
        let mut q_o: VecDeque<Coord> = VecDeque::from(self.o_units.clone());

        while !q_m.is_empty() || !q_o.is_empty() {
            if let Some(cur) = q_m.pop_front() {
                if seen_o.contains(&cur) {
                    ret.insert(cur);
                }
                for n in self.neighbors(cur) {
                    if !seen_m.contains(&n) {
                        seen_m.insert(n);
                        if seen_o.contains(&n) {
                            ret.insert(n);
                        } else {
                            q_m.push_back(n);
                        }
                    }
                }
            }

            if let Some(cur) = q_o.pop_front() {
                if seen_m.contains(&cur) {
                    ret.insert(cur);
                }
                for n in self.neighbors(cur) {
                    if !seen_o.contains(&n) {
                        seen_o.insert(n);
                        if seen_m.contains(&n) {
                            ret.insert(n);
                        } else {
                            q_o.push_back(n);
                        }
                    }
                }
            }
        }

        ret.into_iter().collect()
    }

    fn find_direct_contact(&self, src: Owner, dst: Owner) -> Vec<(Coord, Coord)> {
        let mut s: HashSet<(Coord, Coord)> = HashSet::new();

        for x in 0..self.h {
            for y in 0..self.w {
                if self.map[x][y].owner == src {
                    for n in self.neighbors((x, y)) {
                        if self.map[n.0][n.1].owner == dst {
                            s.insert(((x, y), n));
                        }
                    }
                }
            }
        }

        s.into_iter().collect()
    }

    fn build(&mut self, pos: Coord) {
        self.m_recycler.push(pos);
        self.map[pos.0][pos.1].recycler = true;
        self.map[pos.0][pos.1].can_build = false;
        self.map[pos.0][pos.1].can_spawn = false;
        self.m_m -= 10;
        print!("BUILD {} {};", pos.1, pos.0);
        self.action = true;
    }

    fn r#move(&mut self, src: Coord, dst: Coord, n: Unit) {
        self.map[dst.0][dst.1].owner = Owner::Me;
        self.map[src.0][src.1].unit -= n;
        // if self.map[src.0][src.1].unit == 0 {
        //     self.map[src.0][src.1].can_build = true;
        // }
        self.map[dst.0][dst.1].unit += n;
        self.map[dst.0][dst.1].can_build = false;
        self.map[dst.0][dst.1].can_spawn = false;
        let mut n_moved = 0;
        self.m_units.retain(|i| {
            if n_moved < n && *i == src {
                n_moved += 1;
                false
            } else {
                true
            }
        });

        print!(
            "MOVE {n} {sy} {sx} {dy} {dx};",
            n = n,
            sy = src.1,
            sx = src.0,
            dy = dst.1,
            dx = dst.0
        );
        self.action = true;
    }

    fn spawn(&mut self, pos: Coord, n: Unit) {
        // might put tile at Owner::Me
        self.map[pos.0][pos.1].unit += n;
        self.m_m -= 10 * n as Matter;
        print!("SPAWN {n} {y} {x};", n = n, y = pos.1, x = pos.0);
        self.action = true;
    }

    fn attack(&mut self, direct_contact: &mut Vec<(Coord, Coord)>) {
        direct_contact.retain(|(m, o)| {
            if self.map[m.0][m.1].unit > self.map[o.0][o.1].unit {
                self.r#move(*m, *o, self.map[o.0][o.1].unit + 1);
                false
            } else {
                true
            }
        });
    }

    fn protect(&mut self, direct_contact: &mut Vec<(Coord, Coord)>, block: bool) {
        let mut n_block = 0;
        if block {
            for (m, o) in direct_contact.iter() {
                if self.map[m.0][m.1].unit == 0
                    && self.map[m.0][m.1].can_build
                    && self.map[o.0][o.1].unit > 0
                {
                    n_block += 1;
                }
            }
        }

        direct_contact.sort_by(|a, b| {
            needed(self.map[a.0 .0][a.0 .1].unit, self.map[a.1 .0][a.1 .1].unit).cmp(&needed(
                self.map[b.0 .0][b.0 .1].unit,
                self.map[b.1 .0][b.1 .1].unit,
            ))
        });

        direct_contact.retain(|(m, o)| {
            let needed = needed(self.map[m.0][m.1].unit, self.map[o.0][o.1].unit);

            if (self.m_m as i32 - (needed as i32 * 10)) <= 10 * n_block || needed == 0 {
                true
            } else {
                if block && self.map[m.0][m.1].unit == 0 && self.map[m.0][m.1].can_build {
                    n_block -= 1;
                }
                self.spawn(*m, needed);
                false
            }
        });
    }

    fn block(&mut self, direct_contact: &[(Coord, Coord)]) {
        for (m, o) in direct_contact.iter() {
            if self.m_m >= 10
                && self.map[m.0][m.1].unit == 0
                && self.map[m.0][m.1].can_build
                && self.map[o.0][o.1].unit > 0
            {
                self.build(*m);
            }
        }
    }

    fn move_to_contact(&mut self, contact_tiles: &mut Vec<Coord>) {
        let mut cp = Vec::new();

        if contact_tiles.len() > self.m_units.len() {
            cp = contact_tiles.clone();
        }

        while !contact_tiles.is_empty() && !self.m_units.is_empty() {
            let t = contact_tiles.pop().unwrap();
            let mut closest: (Coord, usize) = ((0, 0), self.h + self.w);

            for u in self.m_units.iter() {
                let d = dist(*u, t);
                if d < closest.1 {
                    closest = (*u, d);
                }
            }

            self.r#move(closest.0, t, 1);
        }

        if cp.is_empty() {
            return;
        }

        while !self.m_units.is_empty() {
            let u = self.m_units[0];
            let mut closest: (Coord, usize) = ((0, 0), self.h + self.w);

            for t in cp.iter() {
                let d = dist(u, *t);
                if d < closest.1 {
                    closest = (*t, d);
                }
            }

            self.r#move(u, closest.0, 1);
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
                        let dist = dist(u, (x, y));
                        if dist < closest.0 {
                            closest = (dist, x, y);
                        }
                    }
                }
            }

            self.r#move(u, (closest.1, closest.2), 1);
        }
    }

    fn spawn_all(&mut self) {
        if self.m_m < 10 || self.m_units.len() > self.o_units.len() {
            return;
        }

        let mut set: HashSet<Coord> = HashSet::new();
        for x in 0..self.h {
            for y in 0..self.w {
                if self.map[x][y].owner == Owner::Me
                    && self.map[x][y].can_spawn
                    && self.map[x][y].scrap > 0
                    && !self.map[x][y].recycler
                    && self.next_to_not_owned((x, y))
                {
                    set.insert((x, y));
                }
            }
        }

        while self.m_m >= 10 && self.m_units.len() <= self.o_units.len() {
            // empty owned tile closest to center and next to not owned tile with scrap
            let mut closest: (usize, usize, usize) = (self.w + self.h, 0, 0);
            for (x, y) in set.iter() {
                let dist = dist((self.h / 2, self.w / 2), (*x, *y));
                if dist < closest.0 {
                    closest = (dist, *x, *y);
                }
            }

            if closest.0 == self.w + self.h {
                break;
            }

            set.remove(&(closest.1, closest.2));
            self.spawn((closest.1, closest.2), 1);
        }
    }

    fn direct_fight(&mut self, block: bool) -> bool {
        let mut direct_contact: Vec<(Coord, Coord)> =
            self.find_direct_contact(Owner::Me, Owner::Op);
        dbg!(direct_contact.len());

        if direct_contact.is_empty() {
            return false;
        } else if direct_contact.len() == 1 {
            self.final_fight(direct_contact[0].1);
            return true;
        }

        self.attack(&mut direct_contact);
        self.protect(&mut direct_contact, block);
        if block {
            self.block(&direct_contact);
        }
        dbg!(direct_contact.len());

        true
    }

    fn direct_explore(&mut self) -> bool {
        let mut gray_direct_contact: Vec<(Coord, Coord)> =
            self.find_direct_contact(Owner::Me, Owner::None);
        dbg!(gray_direct_contact.len());

        if gray_direct_contact.is_empty() {
            return false;
        }

        self.attack(&mut gray_direct_contact);
        self.protect(&mut gray_direct_contact, false);
        dbg!(gray_direct_contact.len());

        true
    }

    fn final_fight(&mut self, contact: Coord) {
        // find closest owned tile to contact
        let mut closest: (Coord, usize) = ((0, 0), self.h + self.w);
        for x in 0..self.h {
            for y in 0..self.w {
                if self.map[x][y].owner == Owner::Me && self.map[x][y].can_spawn {
                    let dist = dist((x, y), contact);
                    if dist < closest.1 {
                        closest = ((x, y), dist);
                    }
                }
            }
        }
        self.spawn(closest.0, (self.m_m / 10).try_into().unwrap());

        for u in self.m_units.clone() {
            self.r#move(u, contact, 1);
        }
    }
}

fn dist(src: Coord, dst: Coord) -> usize {
    ((src.0 as isize - dst.0 as isize).abs() + (src.1 as isize - dst.1 as isize).abs()) as usize
}

fn needed(me: Unit, op: Unit) -> Unit {
    if op == 0 && me == 0 {
        1
    } else {
        let n: i32 = (op as i32 + 1) - me as i32;
        if n < 0 {
            0
        } else {
            n as Unit
        }
    }
}

/*
fn pop_closest(src: &mut Vec<Coord>, dst: Coord) -> Option<Coord> {
    src.iter()
        .enumerate()
        .min_by_key(|(_, i)| dist(**i, dst))
        .map(|(i, _)| i)
        .map(|i| src.swap_remove(i))
}
*/

fn end_of_loop(action: bool) {
    if !action {
        print!("WAIT");
    }
    println!();
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

        let mut contact = e.find_contact();
        dbg!(contact.len());

        if contact.len() == 2 && dist(contact[0], contact[1]) == 1 {
            e.final_fight(
                contact[if e.map[contact[0].0][contact[0].1].owner == Owner::Op {
                    0
                } else {
                    1
                }],
            );
            end_of_loop(e.action);
            continue;
        }

        if e.direct_fight(true) {
            e.direct_explore();
            contact = e.find_contact();
            dbg!(contact.len());
        }

        if !contact.is_empty() {
            e.move_to_contact(&mut contact);
        } else {
            e.direct_explore();
        }
        dbg!(contact.len());

        // e.build_all();	// will implement a new build all
        // e.move_all();
        e.spawn_all();

        end_of_loop(e.action);
    }
}
