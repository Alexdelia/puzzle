use fall_challenge_2023::{referencial, referencial_bool, Float};
use std::{collections::HashMap, io, ops::Range, str::FromStr};

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

macro_rules! read_parse_line {
    ($t:ident) => {{
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        parse_input!(input_line, $t)
    }};
}

type Score = u8;
type Turn = u8;

type MapSize = u16;
#[derive(Default, Clone, Copy)]
struct Coord {
    x: MapSize,
    y: MapSize,
}

type VectorPrecision = i32;
#[derive(Default)]
struct Vector {
    x: VectorPrecision,
    y: VectorPrecision,
}

type Id = u8;

const CREATURE_CAPACITY: usize = 12;
type CreatureSpecSize = u8;

const DRONE_CAPACITY: usize = 1;

const SCAN_DISTANCE: f64 = 800.0;

const MAX_TURN: Turn = 200;
const MAX_SCORE: Score = 100; // to check
const MAX_MAP_SIZE: MapSize = 10_000;
const MAX_BATTERY: Battery = 30;

const DRONE_ID_START: Id = 0;
const DRONE_ID_END: Id = DRONE_ID_START + (DRONE_CAPACITY as Id * 2) - 1;

const CREATURE_ID_START: Id = DRONE_ID_END + 1;
const CREATURE_ID_END: Id = CREATURE_ID_START + CREATURE_CAPACITY as Id - 1;

const PLAYER_COUNT: usize = 2;
const INPUT_VEC_SIZE: usize = 1	// turn
    + (1 * PLAYER_COUNT)	// score
    + (CREATURE_CAPACITY * PLAYER_COUNT)
    + (DRONE_CAPACITY * 9 * PLAYER_COUNT);

type Battery = u8;

struct Env {
    turn: Turn,
    me: Player,
    foe: Player,
    creature: HashMap<Id, Creature>,
    drone_radar: HashMap<Id, DroneRadar>,
}

struct Player {
    score: Score,
    id_scaned: Vec<Id>, // might be a hashset, depending what is the most optimized
    drone: HashMap<Id, Drone>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            score: 0,
            id_scaned: Vec::with_capacity(CREATURE_CAPACITY),
            drone: HashMap::with_capacity(DRONE_CAPACITY),
        }
    }
}

#[derive(Default)]
struct Creature {
    color: CreatureSpecSize,
    r#type: CreatureSpecSize,
    p: Coord,
    v: Vector,
}

#[derive(Default)]
struct Drone {
    p: Coord,
    emergency: bool,
    battery: Battery,
}

type DroneRadar = [Radar; CREATURE_CAPACITY];

/**
 * `TL`: `vertical`: `false`, `horizontal`: `false`
 * `TR`: `vertical`: `false`, `horizontal`: `true`
 * `BL`: `vertical`: `true`, `horizontal`: `false`
 * `BR`: `vertical`: `true`, `horizontal`: `true`
*/
struct Radar {
    vertical: bool,
    horizontal: bool,
}

impl FromStr for Radar {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TL" => Ok(Self {
                vertical: false,
                horizontal: false,
            }),
            "TR" => Ok(Self {
                vertical: false,
                horizontal: true,
            }),
            "BL" => Ok(Self {
                vertical: true,
                horizontal: false,
            }),
            "BR" => Ok(Self {
                vertical: true,
                horizontal: true,
            }),
            _ => Err(()),
        }
    }
}

impl Env {
    fn new() -> Self {
        let creature_count = read_parse_line!(Id) as usize;

        let mut creature = HashMap::with_capacity(CREATURE_CAPACITY);

        for _ in 0..creature_count {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(' ').collect::<Vec<_>>();

            let id = parse_input!(inputs[0], Id);

            creature.insert(
                id,
                Creature {
                    color: parse_input!(inputs[1], CreatureSpecSize),
                    r#type: parse_input!(inputs[2], CreatureSpecSize),
                    p: Coord::default(),
                    v: Vector::default(),
                },
            );

            eprintln!(
                "creature[{}]\tcolor: {} type: {}",
                id, creature[&id].color, creature[&id].r#type
            );
        }

        Self {
            turn: 0,
            me: Player::default(),
            foe: Player::default(),
            creature,
            drone_radar: HashMap::with_capacity(DRONE_CAPACITY),
        }
    }

    fn update(&mut self) {
        self.me.score = read_parse_line!(Score);
        self.foe.score = read_parse_line!(Score);

        self.me.update_scan();
        self.foe.update_scan();

        self.me.update_drone(self.turn == 0);
        self.foe.update_drone(self.turn == 0);

        let drone_scan_count = read_parse_line!(Id);
        for _ in 0..drone_scan_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(' ').collect::<Vec<_>>();

            let drone_id = parse_input!(inputs[0], Id);
            let creature_id = parse_input!(inputs[1], Id);

            eprintln!("drone[{drone_id}] scan creature[{creature_id}]");
        }

        let visible_creature_count = read_parse_line!(Id);
        for _ in 0..visible_creature_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(' ').collect::<Vec<_>>();

            let creature_id = parse_input!(inputs[0], Id);

            let creature = self.creature.get_mut(&creature_id).unwrap();

            creature.p.x = parse_input!(inputs[1], MapSize);
            creature.p.y = parse_input!(inputs[2], MapSize);
            creature.v.x = parse_input!(inputs[3], VectorPrecision);
            creature.v.y = parse_input!(inputs[4], VectorPrecision);

            /*
            eprintln!(
                "creature[{creature_id}]\tp: ({x}, {y}) v: ({vx}, {vy})",
                x = creature.p.x,
                y = creature.p.y,
                vx = creature.v.x,
                vy = creature.v.y,
            );
            */
        }

        let radar_blip_count = read_parse_line!(Id);
        for _ in 0..radar_blip_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();

            let drone_id = parse_input!(inputs[0], Id);
            let creature_id = parse_input!(inputs[1], Id);
            let radar = parse_input!(inputs[2], Radar);
        }
    }

    fn update_radar(&mut self) {
        let init = self.turn == 0;

        if init {
            self.drone_radar
                .insert(drone_id, [Radar; CREATURE_CAPACITY]);
        }

        let radar_blip_count = read_parse_line!(Id);
        for _ in 0..radar_blip_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();

            let drone_id = parse_input!(inputs[0], Id);
            let creature_id = parse_input!(inputs[1], Id);
            let radar = parse_input!(inputs[2], Radar);

            self.drone_radar.get_mut(&drone_id).unwrap()[creature_id - FISH_ID_START] = radar;
        }
    }

    fn output(&mut self) {
        eprintln!("turn: {}", self.turn);

        for (id, drone) in &self.me.drone {
            eprintln!(
                "drone[{id}]\tp: ({x}, {y}) emergency: {emergency} battery: {battery}",
                x = drone.p.x,
                y = drone.p.y,
                emergency = drone.emergency,
                battery = drone.battery,
            );

            let light = self.not_scanned_in_range(drone);

            if let Some(target) = self.closest_not_scanned(drone) {
                r#move(target.p, light);
            } else {
                wait(light);
            }
        }

        self.turn += 1;
    }

    fn input_vec(&self) -> Vec<Float> {
        let mut input = Vec::new();

        let out_range = 0.0..1.0;

        self.input_turn(&mut input, &out_range);
        self.input_score(&mut input, &out_range);
        self.input_scan(&mut input, &out_range);
        self.input_drone(&mut input, &out_range);
        // no drone scan for now
        self.input_drone_radar(&mut input, &out_range);

        assert_eq!(input.len(), INPUT_VEC_SIZE);

        input
    }

    fn input_turn(&self, input: &mut Vec<Float>, out_range: &Range<Float>) {
        let in_range = 0.0..MAX_TURN as Float;
        input.push(referencial(self.turn as Float, &in_range, &out_range));
    }

    fn input_score(&self, input: &mut Vec<Float>, out_range: &Range<Float>) {
        self.me.input_score(input, out_range);
        self.foe.input_score(input, out_range);
    }

    fn input_scan(&self, input: &mut Vec<Float>, out_range: &Range<Float>) {
        self.me.input_scan(input, out_range);
        self.foe.input_scan(input, out_range);
    }

    fn input_drone(&self, input: &mut Vec<Float>, out_range: &Range<Float>) {
        for id in DRONE_ID_START..=DRONE_ID_END {
            if let Some(drone) = self.me.drone.get(&id) {
                drone.input(true, input, out_range);
            } else if let Some(drone) = self.foe.drone.get(&id) {
                drone.input(false, input, out_range);
            } else {
                panic!("drone {id} does not exist");
            }
        }
    }

    fn input_drone_radar(&self, input: &mut Vec<Float>, out_range: &Range<Float>) {
        for id in DRONE_ID_START..=DRONE_ID_END {
            if let Some(radar) = self.drone_radar.get(&id) {
                for r in radar {
                    input.push(referencial_bool(r.vertical, out_range));
                    input.push(referencial_bool(r.horizontal, out_range));
                }
            }
        }
    }

    fn closest_not_scanned(&self, drone: &Drone) -> Option<&Creature> {
        let mut closest = None;
        let mut closest_dist = 0f64;

        for (id, creature) in &self.creature {
            if !self.me.id_scaned.contains(id) {
                let dist = dist(drone.p, creature.p);

                if closest.is_none() || dist < closest_dist {
                    closest = Some(creature);
                    closest_dist = dist;
                }
            }
        }

        closest
    }

    fn not_scanned_in_range(&self, drone: &Drone) -> bool {
        for (id, creature) in &self.creature {
            if !self.me.id_scaned.contains(id) && dist(drone.p, creature.p) < SCAN_DISTANCE {
                return true;
            }
        }

        false
    }
}

impl Player {
    fn update_scan(&mut self) {
        let count = read_parse_line!(Id);
        self.id_scaned.clear();
        for _ in 0..count as usize {
            self.id_scaned.push(read_parse_line!(Id));
        }
    }

    fn update_drone(&mut self, init: bool) {
        let count = read_parse_line!(Id);
        for _ in 0..count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(' ').collect::<Vec<_>>();

            let id = parse_input!(inputs[0], Id);

            if init {
                self.drone.insert(
                    id,
                    Drone {
                        p: Coord {
                            x: parse_input!(inputs[1], MapSize),
                            y: parse_input!(inputs[2], MapSize),
                        },
                        emergency: parse_input!(inputs[3], i32) == 1,
                        battery: parse_input!(inputs[4], Battery),
                    },
                );
            } else {
                let drone = self.drone.get_mut(&id).unwrap();

                drone.p.x = parse_input!(inputs[1], MapSize);
                drone.p.y = parse_input!(inputs[2], MapSize);
                drone.emergency = parse_input!(inputs[3], i32) == 1;
                drone.battery = parse_input!(inputs[4], Battery);
            }
        }
    }

    fn input_score(&self, input: &mut Vec<Float>, out_range: &Range<Float>) {
        let in_range = 0.0..MAX_SCORE as Float;
        input.push(referencial(self.score as Float, &in_range, out_range));
    }

    fn input_scan(&self, input: &mut Vec<Float>, out_range: &Range<Float>) {
        for id in FISH_ID_START..=FISH_ID_END {
            let scanned = self.id_scaned.contains(&id);
            input.push(referencial_bool(scanned, out_range));
        }
    }
}

impl Drone {
    fn input(&self, me: bool, input: &mut Vec<Float>, out_range: &Range<Float>) {
        input.push(referencial_bool(me, out_range));

        self.input_coord(input, out_range);
        self.input_emergency(input, out_range);
        self.input_battery(input, out_range);
    }

    fn input_coord(&self, input: &mut Vec<Float>, out_range: &Range<Float>) {
        self.p.input(input, out_range);
    }

    fn input_emergency(&self, input: &mut Vec<Float>, out_range: &Range<Float>) {
        input.push(referencial_bool(self.emergency, out_range));
    }

    fn input_battery(&self, input: &mut Vec<Float>, out_range: &Range<Float>) {
        let in_range = 0.0..MAX_BATTERY as Float;
        input.push(referencial(self.battery as Float, &in_range, out_range));

        let empty = self.battery == 0;
        input.push(referencial_bool(empty, out_range));

        let full = self.battery == MAX_BATTERY;
        input.push(referencial_bool(full, out_range));

        let three_quarter = self.battery >= MAX_BATTERY * 3 / 4;
        input.push(referencial_bool(three_quarter, out_range));
    }
}

impl Coord {
    fn input(&self, input: &mut Vec<Float>, out_range: &Range<Float>) {
        let in_range = 0.0..MAX_MAP_SIZE as Float;
        input.push(referencial(self.x as Float, &in_range, out_range));
        input.push(referencial(self.y as Float, &in_range, out_range));
    }
}

fn dist(p1: Coord, p2: Coord) -> f64 {
    let x1 = p1.x as i32;
    let y1 = p1.y as i32;
    let x2 = p2.x as i32;
    let y2 = p2.y as i32;

    (((x1 - x2).pow(2) + (y1 - y2).pow(2)) as f64).sqrt()
}

fn r#move(p: Coord, light: bool) {
    println!(
        "MOVE {x} {y} {light}",
        x = p.x,
        y = p.y,
        light = if light { 1 } else { 0 }
    );
}

fn wait(light: bool) {
    println!("WAIT {light}", light = if light { 1 } else { 0 });
}

fn main() {
    let mut e = Env::new();

    // game loop
    loop {
        e.update();

        e.output();
    }
}
