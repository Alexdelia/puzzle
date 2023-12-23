use std::{collections::HashMap, io};

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
#[derive(Default)]
struct Coord {
    x: MapSize,
    y: MapSize,
}

type Id = u8;

const CREATURE_CAPACITY: usize = 12;
type CreatureSpecSize = u8;

const DRONE_CAPACITY: usize = 1;

type Battery = u8;

struct Env {
    turn: Turn,
    me: Player,
    foe: Player,
    creature: HashMap<Id, Creature>,
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
    v: Coord,
}

#[derive(Default)]
struct Drone {
    p: Coord,
    emergency: bool,
    battery: Battery,
}

impl Env {
    fn new() -> Self {
        let creature_count = read_parse_line!(Id) as usize;

        let mut creature = HashMap::with_capacity(CREATURE_CAPACITY);

        for _ in 0..creature_count {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();

            let id = parse_input!(inputs[0], Id);

            creature.insert(
                id,
                Creature {
                    color: parse_input!(inputs[1], CreatureSpecSize),
                    r#type: parse_input!(inputs[2], CreatureSpecSize),
                    p: Coord::default(),
                    v: Coord::default(),
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
            let inputs = input_line.split(" ").collect::<Vec<_>>();

            let drone_id = parse_input!(inputs[0], Id);
            let creature_id = parse_input!(inputs[1], Id);

            eprintln!("drone[{drone_id}] scan creature[{creature_id}]");
        }

        let visible_creature_count = read_parse_line!(Id);
        for _ in 0..visible_creature_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();

            let creature_id = parse_input!(inputs[0], Id);

            let creature = self.creature.get_mut(&creature_id).unwrap();

            creature.p.x = parse_input!(inputs[1], MapSize);
            creature.p.y = parse_input!(inputs[2], MapSize);
            creature.v.x = parse_input!(inputs[3], MapSize);
            creature.v.y = parse_input!(inputs[4], MapSize);

            eprintln!(
                "creature[{creature_id}]\tp: ({x}, {y}) v: ({vx}, {vy})",
                x = creature.p.x,
                y = creature.p.y,
                vx = creature.v.x,
                vy = creature.v.y,
            );
        }

        let radar_blip_count = read_parse_line!(Id);
        for _ in 0..radar_blip_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();

            let drone_id = parse_input!(inputs[0], Id);
            let creature_id = parse_input!(inputs[1], Id);
            let radar = inputs[2].trim().to_string();

            eprintln!("drone[{drone_id}] radar creature[{creature_id}] {radar}");
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

            // Write an action using println!("message...");
            // To debug: eprintln!("Debug message...");

            // println!("WAIT 1"); // MOVE <x> <y> <light (1|0)> | WAIT <light (1|0)>
            println!("MOVE 4000 0 0");
        }

        self.turn += 1;
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
            let inputs = input_line.split(" ").collect::<Vec<_>>();

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
}

fn main() {
    let mut e = Env::new();

    // game loop
    loop {
        e.update();

        e.output();
    }
}
