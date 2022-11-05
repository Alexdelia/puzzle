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
                for _ in 0..self.n_d as usize {
                    let mut input_line = String::new();
                    io::stdin().read_line(&mut input_line).unwrap();
                    let inputs = input_line.split(' ').collect::<Vec<_>>();
                    self.d[pid].push(Drone::new(
                        parse_input!(inputs[0], Coord),
                        parse_input!(inputs[1], Coord),
                    ));
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

fn main() {
    let mut e = Env::new();

    loop {
        e.get_info();

        for did in 0..e.n_d as usize {
            let zid: usize = e.get_nearest_zid(e.d[e.id][did].x, e.d[e.id][did].y) as usize;
            println!("{} {}", e.z[zid].x, e.z[zid].y);
            // println!("{} {}", e.d[e.id][i].t_x, e.d[e.id][i].t_y);
        }
    }
}
