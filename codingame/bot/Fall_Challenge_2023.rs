use std::io;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

macro_rules! read_parse_line {
    ($t:ident) => {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        parse_input!(input_line, $t);
    };
}

type Score = u8;

type MapSize = u16;
#[derive(Default)]
struct Coord {
    x: MapSize,
    y: MapSize,
}

type CreatureSize = u8;
type CreatureSpecSize = u8;

type DroneSize = u8;
type Battery = u8;

struct Env {
    me: Player,
    foe: Player,
    creature: Vec<Creature>,
}

#[derive(Default)]
struct Player {
    score: Score,
    id_scaned: Vec<CreatureSize>,
    drone: Vec<Drone>,
}

#[derive(Default)]
struct Creature {
    id: CreatureSize,
    color: CreatureSpecSize,
    r#type: CreatureSpecSize,
    p: Coord,
    v: Coord,
}

#[derive(Default)]
struct Drone {
    id: DroneSize,
    p: Coord,
    emergency: bool,
    battery: Battery,
}

impl Env {
    fn new() -> Self {
        let mut creature = Vec::new();

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let creature_count = parse_input!(input_line, CreatureSize);

        let creature_count = read_parse_line!(CreatureSize);

        for _ in 0..creature_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();

            creature.push(Creature {
                id: parse_input!(inputs[0], CreatureSize),
                color: parse_input!(inputs[1], CreatureSpecSize),
                r#type: parse_input!(inputs[2], CreatureSpecSize),
                p: Coord::default(),
                v: Coord::default(),
            });
        }

        Self {
            me: Player::default(),
            foe: Player::default(),
            creature,
        }
    }

    fn update() {}
}

/**
 * Score points by scanning valuable fish faster than your opponent.
 **/
fn main() {
    let mut e = Env::new();

    // game loop
    loop {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let my_score = parse_input!(input_line, i32);
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let foe_score = parse_input!(input_line, i32);
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let my_scan_count = parse_input!(input_line, i32);
        for i in 0..my_scan_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let creature_id = parse_input!(input_line, i32);
        }
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let foe_scan_count = parse_input!(input_line, i32);
        for i in 0..foe_scan_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let creature_id = parse_input!(input_line, i32);
        }
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let my_drone_count = parse_input!(input_line, i32);
        for i in 0..my_drone_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let drone_id = parse_input!(inputs[0], i32);
            let drone_x = parse_input!(inputs[1], i32);
            let drone_y = parse_input!(inputs[2], i32);
            let emergency = parse_input!(inputs[3], i32);
            let battery = parse_input!(inputs[4], i32);
        }
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let foe_drone_count = parse_input!(input_line, i32);
        for i in 0..foe_drone_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let drone_id = parse_input!(inputs[0], i32);
            let drone_x = parse_input!(inputs[1], i32);
            let drone_y = parse_input!(inputs[2], i32);
            let emergency = parse_input!(inputs[3], i32);
            let battery = parse_input!(inputs[4], i32);
        }
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let drone_scan_count = parse_input!(input_line, i32);
        for i in 0..drone_scan_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let drone_id = parse_input!(inputs[0], i32);
            let creature_id = parse_input!(inputs[1], i32);
        }
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let visible_creature_count = parse_input!(input_line, i32);
        for i in 0..visible_creature_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let creature_id = parse_input!(inputs[0], i32);
            let creature_x = parse_input!(inputs[1], i32);
            let creature_y = parse_input!(inputs[2], i32);
            let creature_vx = parse_input!(inputs[3], i32);
            let creature_vy = parse_input!(inputs[4], i32);
        }
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let radar_blip_count = parse_input!(input_line, i32);
        for i in 0..radar_blip_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let drone_id = parse_input!(inputs[0], i32);
            let creature_id = parse_input!(inputs[1], i32);
            let radar = inputs[2].trim().to_string();
        }
        for i in 0..my_drone_count as usize {
            // Write an action using println!("message...");
            // To debug: eprintln!("Debug message...");

            // println!("WAIT 1"); // MOVE <x> <y> <light (1|0)> | WAIT <light (1|0)>
            println!("MOVE 4000 0 0");
        }
    }
}
