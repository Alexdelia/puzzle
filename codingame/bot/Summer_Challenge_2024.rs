use std::io;

enum Game {
    HurdleRace = 0,
    Archery = 1,
    RollerSpeedSkating = 2,
    Diving = 3,
}

enum Action {
    Right = 0,
    Up = 1,
    Left = 2,
    Down = 3,
}

type Register = i32;

const PLAYER_NUMBER: usize = 3;
const GAME_AMOUNT: usize = 4;

struct Env {
    player_idx: usize,
}

struct GameInput {
    gpu: String,
    reg: [Register; 7],
}

impl Env {
    fn read_line() -> String {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input
    }

    fn init() -> Self {
        let player_idx = Self::read_line().trim().parse::<usize>().unwrap();

        let game_amount = Self::read_line().trim().parse::<usize>().unwrap();
        assert_eq!(game_amount, GAME_AMOUNT);

        Self { player_idx }
    }

    fn read_scores() {
        for i in 0..PLAYER_NUMBER {
            eprintln!("{i}: {}", Self::read_line().trim());
        }
    }

    fn read_game_input() -> GameInput {
        let inputs = Self::read_line().split(" ").collect::<Vec<_>>();

        let gpu = inputs[0].trim().to_string();
        let reg = [
            inputs[1].trim().parse::<Register>().unwrap(),
            inputs[2].trim().parse::<Register>().unwrap(),
            inputs[3].trim().parse::<Register>().unwrap(),
            inputs[4].trim().parse::<Register>().unwrap(),
            inputs[5].trim().parse::<Register>().unwrap(),
            inputs[6].trim().parse::<Register>().unwrap(),
            inputs[7].trim().parse::<Register>().unwrap(),
        ];
    }
}

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

fn hurdle_in(track: &str, index: usize) -> usize {
    let track = track.chars().collect::<Vec<char>>();

    let mut i = index + 1;
    while i < track.len() {
        if track[i] == '#' {
            return i - index;
        }
        i += 1;
    }
    64
}

fn main() {
    let e = Env::init();

    loop {
        Env::read_scores();

        let mut hurdle = 0;

        for i in 0..GAME_AMOUNT {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();

            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let gpu = inputs[0].trim().to_string();
            dbg!(&gpu);

            let reg_0 = parse_input!(inputs[1], i32);
            let reg_1 = parse_input!(inputs[2], i32);
            let reg_2 = parse_input!(inputs[3], i32);

            let _reg_3 = parse_input!(inputs[4], i32);
            let _reg_4 = parse_input!(inputs[5], i32);
            let _reg_5 = parse_input!(inputs[6], i32);

            let _reg_6 = parse_input!(inputs[7], i32);

            if i == 0 {
                let player_positions = [reg_0, reg_1, reg_2];
                let my_player_position = player_positions[e.player_idx] as usize;
                dbg!(my_player_position);

                hurdle = hurdle_in(&gpu, my_player_position);
            }
        }

        let choice = ["RIGHT", "UP", "LEFT", "DOWN"];

        println!("{}", choice[hurdle % choice.len()]);
    }
}
