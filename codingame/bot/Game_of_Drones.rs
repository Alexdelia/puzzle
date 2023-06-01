use std::io;

type Coord = u16;
type Id = u8;
type Dist = f32;

const WIDE: Coord = 4000;
const HIGH: Coord = 1800;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

struct Zone {
    x: Coord,
    y: Coord,
    c_x: Coord,
    c_y: Coord,
    owner: i8,
    to_beat: Id,
    d: Vec<Id>,
}

impl Zone {
    fn new(x: Coord, y: Coord, c_x: Coord, c_y: Coord, owner: i8) -> Zone {
        Zone {
            x,
            y,
            c_x,
            c_y,
            owner,
            to_beat: 0,
            d: vec![],
        }
    }

    fn cost(&self) -> i32 {
        (self.to_beat as usize - self.d.len() + 1) as i32
    }
}

struct Drone {
    x: Coord,
    y: Coord,
    target: Id,
}

impl Drone {
    fn new(x: Coord, y: Coord) -> Drone {
        Drone { x, y, target: 0 }
    }
}

struct Env {
    n_p: Id,
    id: usize,
    n_d: Id,
    n_z: Id,
    d: Vec<Vec<Drone>>,
    z: Vec<Zone>,
    free_d: Vec<Id>,
}

#[inline]
fn get_distance(x1: Coord, y1: Coord, x2: Coord, y2: Coord) -> Dist {
    (((x2 as i32 - x1 as i32).pow(2) + (y2 as i32 - y1 as i32).pow(2)) as f32).sqrt() as Dist
}

fn is_d_in_z(d: &Drone, z: &Zone) -> bool {
    get_distance(d.x, d.y, z.x, z.y) <= 100.0
}

fn is_c_in_c(x1: Coord, y1: Coord, x2: Coord, y2: Coord) -> bool {
    get_distance(x1, y1, x2, y2) <= 100.0
}

fn get_cz_to_center(x: Coord, y: Coord) -> (Coord, Coord) {
    // we get a zone coord, need to find point in zone closest to center, with zone radius == 100
    let w = WIDE / 2;
    let h = HIGH / 2;
    let t = 98.0 / get_distance(x, y, w, h);
    (
        ((1.0 - t) * x as f32 + t * w as f32) as Coord,
        ((1.0 - t) * y as f32 + t * h as f32) as Coord,
    )
}

impl Env {
    fn new() -> Env {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(' ').collect::<Vec<_>>();
        let n_p = parse_input!(inputs[0], Id); // number of players in the game (2 to 4 players)
        let id = parse_input!(inputs[1], usize); // ID of your player (0, 1, 2, or 3)
        let n_d = parse_input!(inputs[2], Id); // number of drones in each team (3 to 11)
        let n_z = parse_input!(inputs[3], Id); // number of zones on the map (4 to 8)

        let mut z: Vec<Zone> = Vec::with_capacity(n_z as usize);
        for _ in 0..n_z as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(' ').collect::<Vec<_>>();
            let x = parse_input!(inputs[0], Coord);
            let y = parse_input!(inputs[1], Coord);
            let (c_x, c_y) = get_cz_to_center(x, y);
            z.push(Zone::new(x, y, c_x, c_y, -1));
        }

        Env {
            n_p,
            id,
            n_d,
            n_z,
            d: vec![vec![], vec![], vec![], vec![]],
            z,
            free_d: vec![],
        }
    }

    fn get_info(&mut self) {
        for zid in 0..self.n_z as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            self.z[zid].owner = parse_input!(input_line, i8);
        }

        if self.d[self.id].is_empty() {
            for pid in 0..self.n_p as usize {
                for did in 0..self.n_d as usize {
                    let mut input_line = String::new();
                    io::stdin().read_line(&mut input_line).unwrap();
                    let inputs = input_line.split(' ').collect::<Vec<_>>();
                    self.d[pid].push(Drone::new(
                        parse_input!(inputs[0], Coord),
                        parse_input!(inputs[1], Coord),
                    ));
                    self.d[pid][did].target =
                        self.get_nearest_zid(self.d[pid][did].x, self.d[pid][did].y);
                }
            }
        } else {
            for pid in 0..self.n_p as usize {
                for did in 0..self.n_d as usize {
                    let mut input_line = String::new();
                    io::stdin().read_line(&mut input_line).unwrap();
                    let inputs = input_line.split(' ').collect::<Vec<_>>();
                    self.d[pid][did].x = parse_input!(inputs[0], Coord);
                    self.d[pid][did].y = parse_input!(inputs[1], Coord);
                }
            }
        }
    }

    fn get_nearest_zid(&self, x: Coord, y: Coord) -> Id {
        let mut min_dist: Dist = Dist::MAX;
        let mut min_id: Id = Id::MAX;
        for zid in 0..self.n_z as usize {
            let dist = get_distance(x, y, self.z[zid].c_x, self.z[zid].c_y);
            if dist < min_dist {
                min_dist = dist;
                min_id = zid as Id;
            }
        }
        min_id
    }

    fn get_nearest_did(&self, x: Coord, y: Coord, free: bool) -> Id {
        let mut min_dist: Dist = Dist::MAX;
        let mut min_id: Id = Id::MAX;
        if free {
            for did in &self.free_d {
                let dist = get_distance(
                    x,
                    y,
                    self.d[self.id][*did as usize].x,
                    self.d[self.id][*did as usize].y,
                );
                if dist < min_dist {
                    min_dist = dist;
                    min_id = *did;
                }
            }
        } else {
            for did in 0..self.n_d as usize {
                let dist = get_distance(
                    x,
                    y,
                    self.d[self.id][did].x,
                    self.d[self.id][did].y,
                );
                if dist < min_dist {
                    min_dist = dist;
                    min_id = did as Id;
                }
            }
        }
        min_id
    }

    fn n_d_at_target(&self, n: Id) -> bool {
        let mut t: Id = 0;
        for d in &self.d[self.id] {
            if d.x == self.z[d.target as usize].c_x && d.y == self.z[d.target as usize].c_y {
                t += 1;
                if t >= n {
                    return true;
                }
            }
        }
        false
    }

    fn update_d_in_z(&mut self) {
        for zid in 0..self.n_z as usize {
            self.z[zid].d.clear();
            self.z[zid].to_beat = 0;
        }
        for pid in 0..self.n_p as usize {
            if pid == self.id {
                for did in 0..self.n_d as usize {
                    for z in &mut self.z {
                        if is_d_in_z(&self.d[pid][did], z) {
                            z.d.push(did as Id);
                            break;
                        }
                    }
                }
            } else {
                for z in &mut self.z {
                    let mut to_beat = 0;
                    for d in &self.d[pid] {
                        if is_d_in_z(d, z) {
                            to_beat += 1;
                            break;
                        }
                    }
                    if to_beat > z.to_beat {
                        z.to_beat = to_beat;
                    }
                }
            }
        }
    }

    fn update_free_d(&mut self, force: bool) {
        self.free_d.clear();

        // for d in &self.d[self.id] {
        // 	if d.t_x == d.x && d.t_y == d.y {
        // 		self.free_d.push(d.id);
        // 	}
        // }

        if !force {
            // remove drones in owned zones until there is to_beat + 1 drones left in the zone
            for z in &mut self.z {
                if z.owner == self.id as i8 {
                    while z.d.len() > z.to_beat as usize + 1 {
                        let d = z.d.pop().unwrap();
                        self.free_d.push(d);
                    }
                }
            }
        } else {
            for z in &self.z {
                if z.owner != self.id as i8 {
                    for d in &z.d {
                        self.free_d.push(*d);
                    }
                }
            }
        }

        // add drones on the way to a owned zone
        for zid in 0..self.n_z as usize {
            if self.z[zid].owner == self.id as i8 {
                for did in 0..self.n_d as usize {
                    if !self.free_d.contains(&(did as Id))
                        && !is_d_in_z(&self.d[self.id][did], &self.z[zid])
                        && self.d[self.id][did].target == zid as Id
                    {
                        self.free_d.push(did as Id);
                    }
                }
            }
        }
    }

    fn create_queue(&self, force: bool) -> Vec<Vec<Id>> {
        let mut queue: Vec<Vec<Id>> = vec![vec![]; (self.n_d + 2) as usize];

        if !force {
            for zid in 0..self.n_z as usize {
                if self.z[zid].owner != self.id as i8 && self.z[zid].cost() > 0 {
                    queue[(self.z[zid].cost()) as usize].push(zid as Id);
                }
            }
        } else {
            for zid in 0..self.n_z as usize {
                if self.z[zid].owner != self.id as i8 {
                    if self.z[zid].to_beat == 0 {
                        queue[0].push(zid as Id);
                    } else if self.z[zid].cost() > 0 {
                        queue[(self.z[zid].cost()) as usize].push(zid as Id);
                    }
                }
            }
        }

        queue
    }

    fn update_target(&mut self, force: bool) -> bool {
        if self.free_d.is_empty() {
            return false;
        }

        let queue: Vec<Vec<Id>> = self.create_queue(force);
        let mut changed = false;

        for (i, l) in queue.iter().enumerate() {
            for zid in l {
                let mut cost = i + 2;
                while cost > 0 {
                    // doesn't involve finding best match between free_d and all zid from queue[cost - 1] for now
                    let did = self.get_nearest_did(
                        self.z[*zid as usize].c_x,
                        self.z[*zid as usize].c_y,
                        true,
                    );
                    if did == Id::MAX {
                        return changed;
                    }
                    self.d[self.id][did as usize].target = *zid;
                    self.free_d.retain(|&x| x != did);
                    if self.free_d.is_empty() {
                        return true;
                    }
                    changed = true;
                    cost -= 1;
                }
            }
        }

        changed
    }
}

fn main() {
    let mut e = Env::new();

    loop {
        e.get_info();

        e.update_d_in_z();
        e.update_free_d(false);
        eprintln!("free_d: {:?}", e.free_d);
        let stuck = !e.update_target(false) && e.n_d_at_target(e.n_d / 2 as Id);
        // if all none of the target change, then need to relaunch with free_d being all drones in not owned zones
        // or already add d in not owned zones to free_d
        if stuck {
            eprintln!("stuck");
            e.update_free_d(true);
            eprintln!("free_d: {:?}", e.free_d);
            e.update_target(true);
        }

        // add feature:
        // add to free_d all d that have for target a zone that is owned by me

        for d in &e.d[e.id] {
            println!(
                "{} {}",
                e.z[d.target as usize].c_x, e.z[d.target as usize].c_y
            );
        }
    }
}
