use fall_challenge_2023::{referencial, referencial_bool, Float, Network};
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

const CREATURE_COLOR_COUNT: CreatureSpecSize = 4;
const CREATURE_TYPE_COUNT: CreatureSpecSize = 3;

const SCAN_DISTANCE: f64 = 800.0;

const MAX_TURN: Turn = 200;
const MAX_SCORE: Score = 100; // to check
const MAX_MAP_SIZE: MapSize = 10_000;
const MAX_CREATURE_SPEED: VectorPrecision = 21; // to check
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

const OUTPUT_VEC_SIZE: usize = DRONE_CAPACITY * 3;

const OUT_RANGE: Range<Float> = 0.0..1.0;

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
    visible: bool,
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
#[derive(Default, Clone, Copy)]
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
                    visible: false,
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
        let init = self.turn == 0;

        self.me.score = read_parse_line!(Score);
        self.foe.score = read_parse_line!(Score);

        self.me.update_scan();
        self.foe.update_scan();

        self.me.update_drone(init);
        self.foe.update_drone(init);

        self.update_drone_scan();

        self.update_visible_creature();

        self.update_radar(init);
    }

    fn update_drone_scan(&mut self) {
        let drone_scan_count = read_parse_line!(Id);
        for _ in 0..drone_scan_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(' ').collect::<Vec<_>>();

            let drone_id = parse_input!(inputs[0], Id);
            let creature_id = parse_input!(inputs[1], Id);

            eprintln!("drone[{drone_id}] scan creature[{creature_id}]");
        }
    }

    fn update_visible_creature(&mut self) {
        for (_, creature) in &mut self.creature {
            creature.visible = false;

            let min = 0;
            let max = MAX_MAP_SIZE as VectorPrecision - 1;

            creature.p.x =
                (creature.p.x as VectorPrecision + creature.v.x).clamp(min, max) as MapSize;
            creature.p.y =
                (creature.p.y as VectorPrecision + creature.v.y).clamp(min, max) as MapSize;
        }

        let visible_creature_count = read_parse_line!(Id);
        for _ in 0..visible_creature_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(' ').collect::<Vec<_>>();

            let creature_id = parse_input!(inputs[0], Id);

            let creature = self.creature.get_mut(&creature_id).unwrap();

            creature.visible = true;

            creature.p.x = parse_input!(inputs[1], MapSize);
            creature.p.y = parse_input!(inputs[2], MapSize);
            creature.v.x = parse_input!(inputs[3], VectorPrecision);
            creature.v.y = parse_input!(inputs[4], VectorPrecision);

            eprintln!(
                "creature[{creature_id}]\tp: ({x}, {y}) v: ({vx}, {vy})",
                x = creature.p.x,
                y = creature.p.y,
                vx = creature.v.x,
                vy = creature.v.y,
            );
        }
    }

    fn update_radar(&mut self, init: bool) {
        let radar_blip_count = read_parse_line!(Id);
        for _ in 0..radar_blip_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();

            let drone_id = parse_input!(inputs[0], Id);
            let creature_id = parse_input!(inputs[1], Id);
            let radar = parse_input!(inputs[2], Radar);

            if init {
                if !self.drone_radar.contains_key(&drone_id) {
                    self.drone_radar
                        .insert(drone_id, [Radar::default(); CREATURE_CAPACITY]);
                }
            }

            self.drone_radar.get_mut(&drone_id).unwrap()
                [(creature_id - CREATURE_ID_START) as usize] = radar;
        }

        // posibly TODO:
        // calculate expected position of creature, based on all drone radar
    }

    fn output(&mut self, network: &Network) {
        eprintln!("turn: {}", self.turn);

        let input_vec = self.input_vec();

        let output_vec = network.forward(input_vec);
        assert_eq!(output_vec.len(), OUTPUT_VEC_SIZE);

        let mut i = 0;

        for id in DRONE_ID_START..=DRONE_ID_END {
            let Some(drone) = self.me.drone.get(&id) else {
                continue;
            };

            eprintln!(
                "drone[{id}]\tp: ({x}, {y}) emergency: {emergency} battery: {battery}",
                x = drone.p.x,
                y = drone.p.y,
                emergency = drone.emergency,
                battery = drone.battery,
            );

            let x = output_vec[i * 3 + 0];
            let y = output_vec[i * 3 + 1];
            let light = output_vec[i * 3 + 2];

            let min = 0.0;
            let max = MAX_MAP_SIZE as Float;

            let x = (x * max).round().clamp(min, max) as MapSize;
            let y = (y * max).round().clamp(min, max) as MapSize;

            let light = light > 0.5;

            r#move(Coord { x, y }, light);

            i += 1;
        }

        self.turn += 1;
    }

    fn input_vec(&self) -> Vec<Float> {
        let mut input = Vec::new();

        self.input_turn(&mut input);
        self.input_score(&mut input);
        self.input_scan(&mut input);
        self.input_drone(&mut input);
        self.input_creature(&mut input);
        // no drone scan for now
        self.input_drone_radar(&mut input);

        assert_eq!(input.len(), INPUT_VEC_SIZE);

        input
    }

    fn input_turn(&self, input: &mut Vec<Float>) {
        let in_range = 0.0..MAX_TURN as Float;
        input.push(referencial(self.turn as Float, &in_range, &OUT_RANGE));
    }

    fn input_score(&self, input: &mut Vec<Float>) {
        self.me.input_score(input);
        self.foe.input_score(input);
    }

    fn input_scan(&self, input: &mut Vec<Float>) {
        self.me.input_scan(input);
        self.foe.input_scan(input);
    }

    fn input_drone(&self, input: &mut Vec<Float>) {
        for id in DRONE_ID_START..=DRONE_ID_END {
            if let Some(drone) = self.me.drone.get(&id) {
                drone.input(true, input);
            } else if let Some(drone) = self.foe.drone.get(&id) {
                drone.input(false, input);
            } else {
                panic!("drone {id} does not exist");
            }
        }
    }

    fn input_creature(&self, input: &mut Vec<Float>) {
        for id in CREATURE_ID_START..=CREATURE_ID_END {
            self.creature.get(&id).unwrap().input(input);
        }
    }

    fn input_drone_radar(&self, input: &mut Vec<Float>) {
        for id in DRONE_ID_START..=DRONE_ID_END {
            if let Some(radar) = self.drone_radar.get(&id) {
                for r in radar {
                    input.push(referencial_bool(r.vertical, &OUT_RANGE));
                    input.push(referencial_bool(r.horizontal, &OUT_RANGE));
                }
            }
        }
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

    fn input_score(&self, input: &mut Vec<Float>) {
        let in_range = 0.0..MAX_SCORE as Float;
        input.push(referencial(self.score as Float, &in_range, &OUT_RANGE));
    }

    fn input_scan(&self, input: &mut Vec<Float>) {
        for id in CREATURE_ID_START..=CREATURE_ID_END {
            let scanned = self.id_scaned.contains(&id);
            input.push(referencial_bool(scanned, &OUT_RANGE));
        }
    }
}

impl Creature {
    fn input(&self, input: &mut Vec<Float>) {
        self.input_color(input);
        self.input_type(input);
        self.input_coord(input);
        self.input_vector(input);
        self.input_visible(input);
    }

    fn input_color(&self, input: &mut Vec<Float>) {
        let in_range = 0.0..CREATURE_COLOR_COUNT as Float;
        input.push(referencial(self.color as Float, &in_range, &OUT_RANGE));
    }

    fn input_type(&self, input: &mut Vec<Float>) {
        let in_range = 0.0..CREATURE_TYPE_COUNT as Float;
        input.push(referencial(self.r#type as Float, &in_range, &OUT_RANGE));
    }

    fn input_coord(&self, input: &mut Vec<Float>) {
        self.p.input(input);
    }

    fn input_vector(&self, input: &mut Vec<Float>) {
        self.v.input(input);
    }

    fn input_visible(&self, input: &mut Vec<Float>) {
        input.push(referencial_bool(self.visible, &OUT_RANGE));
    }
}

impl Drone {
    fn input(&self, me: bool, input: &mut Vec<Float>) {
        input.push(referencial_bool(me, &OUT_RANGE));

        self.input_coord(input);
        self.input_emergency(input);
        self.input_battery(input);
    }

    fn input_coord(&self, input: &mut Vec<Float>) {
        self.p.input(input);
    }

    fn input_emergency(&self, input: &mut Vec<Float>) {
        input.push(referencial_bool(self.emergency, &OUT_RANGE));
    }

    fn input_battery(&self, input: &mut Vec<Float>) {
        let in_range = 0.0..MAX_BATTERY as Float;
        input.push(referencial(self.battery as Float, &in_range, &OUT_RANGE));

        let empty = self.battery == 0;
        input.push(referencial_bool(empty, &OUT_RANGE));

        let full = self.battery == MAX_BATTERY;
        input.push(referencial_bool(full, &OUT_RANGE));

        let three_quarter = self.battery >= MAX_BATTERY * 3 / 4;
        input.push(referencial_bool(three_quarter, &OUT_RANGE));
    }
}

impl Coord {
    fn input(&self, input: &mut Vec<Float>) {
        let in_range = 0.0..MAX_MAP_SIZE as Float;
        input.push(referencial(self.x as Float, &in_range, &OUT_RANGE));
        input.push(referencial(self.y as Float, &in_range, &OUT_RANGE));
    }
}

impl Vector {
    fn input(&self, input: &mut Vec<Float>) {
        let in_range = -MAX_CREATURE_SPEED as Float..MAX_CREATURE_SPEED as Float;
        input.push(referencial(self.x as Float, &in_range, &OUT_RANGE));
        input.push(referencial(self.y as Float, &in_range, &OUT_RANGE));
    }
}

/*
fn dist(p1: Coord, p2: Coord) -> f64 {
    let x1 = p1.x as i32;
    let y1 = p1.y as i32;
    let x2 = p2.x as i32;
    let y2 = p2.y as i32;

    (((x1 - x2).pow(2) + (y1 - y2).pow(2)) as f64).sqrt()
}
*/

fn r#move(p: Coord, light: bool) {
    println!(
        "MOVE {x} {y} {light}",
        x = p.x,
        y = p.y,
        light = if light { 1 } else { 0 }
    );
}

/*
fn wait(light: bool) {
    println!("WAIT {light}", light = if light { 1 } else { 0 });
}
*/

fn network() -> Network {
    todo!()
}

fn main() {
    let mut e = Env::new();

    let network = network();

    // game loop
    loop {
        e.update();

        e.output(&network);
    }
}
