use std::collections::{HashSet, VecDeque};
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

    fn find_direct_contact(&self) -> (Vec<Coord>, Vec<Coord>) {
        let mut me: HashSet<Coord> = HashSet::new();
        let mut op: HashSet<Coord> = HashSet::new();

        for x in 0..self.h {
            for y in 0..self.w {
                if self.map[x][y].owner == Owner::Me {
                    for n in self.neighbors((x, y)) {
                        if self.map[n.0][n.1].owner == Owner::Op {
                            me.insert((x, y));
                        }
                    }
                } else if self.map[x][y].owner == Owner::Op {
                    for n in self.neighbors((x, y)) {
                        if self.map[n.0][n.1].owner == Owner::Me {
                            op.insert((x, y));
                        }
                    }
                }
            }
        }

        (me.into_iter().collect(), op.into_iter().collect())
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

    fn r#move(&mut self, src: Coord, dst: Coord, n: Unit, can_build: bool) {
        self.map[dst.0][dst.1].owner = Owner::Me;
        self.map[src.0][src.1].unit -= n;
        if can_build && self.map[src.0][src.1].unit == 0 {
            self.map[src.0][src.1].can_build = true;
        }
        self.map[dst.0][dst.1].unit += n;
        self.map[dst.0][dst.1].can_build = false;
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

    fn attack(&mut self, me_tile: &mut Vec<Coord>) {
        me_tile.retain(|t| {
            if self.map[t.0][t.1].owner != Owner::Op {
                return true;
            }

            let mut can_move: Vec<(Coord, Unit)> = Vec::new();
            let mut sum: Unit = 0;

            for n in self.neighbors(*tile) {
                if self.map[n.0][n.1].owner == Owner::Me && self.map[n.0][n.1].unit > 0 {
                    can_move.push((n, self.map[n.0][n.1].unit));
                    sum += self.map[n.0][n.1].unit;
                }
            }

            if sum > self.map[tile.0][tile.1].unit {
                for (pos, n) in can_move.iter() {
                    self.r#move(*pos, *tile, *n, false);
                }
                return false;
            }

            true
        });
    }

    fn protect(&mut self, contact_tiles: &mut Vec<Coord>) {
        let mut n_block = 0;
        for tile in contact_tiles.iter() {
            if self.map[tile.0][tile.1].unit == 0
                && self.map[tile.0][tile.1].can_build
                && self.next_to_op(*tile)
            {
                n_block += 1;
            }
        }

        contact_tiles.retain(|tile| {
            if self.m_m as usize <= 10 * n_block {
                return true;
            }

            let mut sum: Unit = 0;

            for n in self.neighbors(*tile) {
                if self.map[n.0][n.1].owner == Owner::Op && self.map[n.0][n.1].unit > 0 {
                    sum += self.map[n.0][n.1].unit;
                }
            }

            let needed: i32 = (sum + 1) as i32 - self.map[tile.0][tile.1].unit as i32;
            if needed > 0 && self.m_m as i32 - (n_block as i32 - 1) * 10 >= needed * 10 {
                self.spawn(*tile, needed as Unit);
                n_block -= 1;
                return false;
            }

            true
        });
    }

    fn block(&mut self, contact_tiles: &[Coord]) {
        for tile in contact_tiles.iter() {
            if self.m_m >= 10
                && self.map[tile.0][tile.1].unit == 0
                && self.map[tile.0][tile.1].can_build
                && self.next_to_op(*tile)
            {
                self.build(*tile);
            } else {
                for n in self.neighbors(*tile) {
                    if self.map[n.0][n.1].owner == Owner::Me && self.map[n.0][n.1].unit > 0 {
                        self.r#move(n, *tile, self.map[n.0][n.1].unit, false);
                        break;
                    }
                }
            }
        }
    }

    fn contact(&mut self, contact_tiles: &mut Vec<Coord>) {
        self.attack(contact_tiles);
        let (mut me_ct, mut other_ct): (Vec<Coord>, Vec<Coord>) = contact_tiles
            .iter()
            .partition(|tile| self.map[tile.0][tile.1].owner == Owner::Me);
        me_ct.retain(|tile| self.next_to_op(*tile));
        me_ct.sort_by(|a, b| self.map[a.0][a.1].unit.cmp(&self.map[b.0][b.1].unit));
        self.protect(&mut me_ct);
        self.block(&me_ct);

        other_ct.sort_by(|a, b| self.map[a.0][a.1].unit.cmp(&self.map[b.0][b.1].unit));
        *contact_tiles = other_ct;
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
                let closest = pop_closest(&mut self.m_units, tile).unwrap();
                self.r#move(closest, tile, 1, false);
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
                        let dist = dist(u, (x, y));
                        if dist < closest.0 {
                            closest = (dist, x, y);
                        }
                    }
                }
            }

            self.r#move(u, (closest.1, closest.2), 1, false);
        }
    }

    fn spawn_all(&mut self) {
        if self.m_m < 10 && self.m_units.len() < self.o_units.len() {
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

        while self.m_m >= 10 && self.m_units.len() < self.o_units.len() + 1 {
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
}

fn dist(src: Coord, dst: Coord) -> usize {
    ((src.0 as isize - dst.0 as isize).abs() + (src.1 as isize - dst.1 as isize).abs()) as usize
}

fn pop_closest(src: &mut Vec<Coord>, dst: Coord) -> Option<Coord> {
    src.iter()
        .enumerate()
        .min_by_key(|(_, i)| dist(**i, dst))
        .map(|(i, _)| i)
        .map(|i| src.swap_remove(i))
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

        // need to find contact tiles for me to op with direct connection to attack
        let (mut me_direct_contact, mut op_direct_contact): (Vec<Coord>, Vec<Coord>) = e.find_direct_contact();
		e.attack(
        // then me to op with direct connection to protect and block
        // if none in both
        // contact tile from me to op to move to contact
        // might need to group unit by chunk of tile
        let mut contact_tiles = e.find_contact();
        if contact_tiles.is_empty() {
            e.move_all();
        }

        dbg!(contact_tiles.len());
        // dbg!(&contact_tiles);
        e.contact(&mut contact_tiles);
        dbg!(contact_tiles.len());
        e.move_to_contact(&mut contact_tiles);
        dbg!(contact_tiles.len());

        // e.build_all();
        // e.move_all();
        e.spawn_all();

        if !e.action {
            print!("WAIT");
        }
        println!();
    }
}
