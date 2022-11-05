use std::io;

type Coord = u16;
type Id = u8;
type Dist = i32;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

struct Zone {
    x: Coord,
    y: Coord,
    owner: i8,
    to_beat: Id,
    d: Vec<Id>,
}

impl Zone {
    fn new(x: Coord, y: Coord, owner: i8) -> Zone {
        Zone {
            x,
            y,
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
    t_x: Coord,
    t_y: Coord,
}

impl Drone {
    fn new(x: Coord, y: Coord) -> Drone {
        Drone {
            x,
            y,
            t_x: x,
            t_y: y,
        }
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
    (x1 as i32 - x2 as i32).pow(2) + (y1 as i32 - y2 as i32).pow(2)
}

fn is_d_in_z(d: &Drone, z: &Zone) -> bool {
    get_distance(d.x, d.y, z.x, z.y) <= 100
}

fn is_c_in_c(x1: Coord, y1: Coord, x2: Coord, y2: Coord) -> bool {
    get_distance(x1, y1, x2, y2) <= 100
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
            z.push(Zone::new(x, y, -1));
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
                    let zid = self.get_nearest_zid(self.d[pid][did].x, self.d[pid][did].y);
                    self.d[pid][did].t_x = self.z[zid as usize].x + 1;
                    self.d[pid][did].t_y = self.z[zid as usize].y + 1;
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
            let dist = get_distance(x, y, self.z[zid].x, self.z[zid].y);
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
                    self.d[self.id][did as usize].x,
                    self.d[self.id][did as usize].y,
                );
                if dist < min_dist {
                    min_dist = dist;
                    min_id = did as Id;
                }
            }
        }
        min_id
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
                for d in &self.d[pid] {
                    for z in &mut self.z {
                        if is_d_in_z(d, z) {
                            z.to_beat += 1;
                            break;
                        }
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
    }

    fn create_queue(&self, force: bool) -> Vec<Vec<Id>> {
        let mut queue: Vec<Vec<Id>> = vec![vec![]; (self.n_d + 1) as usize];

        if !force {
            for zid in 0..self.n_z as usize {
                if self.z[zid].cost() > 0 {
                    queue[(self.z[zid].cost() - 1) as usize].push(zid as Id);
                }
            }
        } else {
            for zid in 0..self.n_z as usize {
                queue[self.z[zid].to_beat as usize].push(zid as Id);
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

        for l in &queue {
            for zid in l {
                // doesn't involve finding best match between free_d and all zid from queue[cost - 1] for now
                let did = self.get_nearest_did(
                    self.z[*zid as usize].x + 1,
                    self.z[*zid as usize].y + 1,
                    true,
                );
                if did == Id::MAX {
                    return changed;
                }
                self.d[self.id][did as usize].t_x = self.z[*zid as usize].x + 1;
                self.d[self.id][did as usize].t_y = self.z[*zid as usize].y + 1;
                changed = true;
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
        let stuck = e.update_target(false);
        // if all none of the target change, then need to relaunch with free_d being all drones in not owned zones
        // or already add d in not owned zones to free_d
        if stuck {
            e.update_free_d(true);
            e.update_target(true);
        }

        for d in &e.d[e.id] {
            println!("{} {}", d.t_x, d.t_y);
        }
    }
}
