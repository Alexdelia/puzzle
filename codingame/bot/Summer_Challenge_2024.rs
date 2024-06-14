use std::{fmt, io};

enum Game {
    HurdleRace = 0,
    Archery = 1,
    RollerSpeedSkating = 2,
    Diving = 3,
}

#[derive(Copy, Clone)]
enum Action {
    Right = 0,
    Up = 1,
    Left = 2,
    Down = 3,
}

type Register = i32;

struct Input {
	gpu: String,
	reg: [Register; 7],
}

const PLAYER_NUMBER: usize = 3;
const GAME_AMOUNT: usize = 4;

struct Env {
    player_idx: usize,
}

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Right => write!(f, "RIGHT"),
            Self::Up => write!(f, "UP"),
            Self::Left => write!(f, "LEFT"),
            Self::Down => write!(f, "DOWN"),
        }
    }
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

    fn read_score(&self) {
        for _ in 0..PLAYER_NUMBER {
            dbg!(Self::read_line());
        }
    }

    fn solve(&self) -> Action {
        let mut hurdle = 0;

        for i in 0..GAME_AMOUNT {
            let game = Game::from(i);

            let line = Self::read_line();
            let inputs = line.split(' ').collect::<Vec<_>>();

            let gpu = inputs[0].trim().to_string();
            dbg!(&gpu);

			let reg = [
				inputs[1].trim().parse::<i32>().unwrap(),
				inputs[2].trim().parse::<i32>().unwrap(),
				inputs[3].trim().parse::<i32>().unwrap(),
				inputs[4].trim().parse::<i32>().unwrap(),
				inputs[5].trim().parse::<i32>().unwrap(),
				inputs[6].trim().parse::<i32>().unwrap(),
				inputs[7].trim().parse::<i32>().unwrap(),
			];

            dbg!(game.dispatch(line, reg));

            if i == 0 {
                let player_positions = [reg_0, reg_1, reg_2];
                let my_player_position = player_positions[self.player_idx] as usize;
                dbg!(my_player_position);

                hurdle = hurdle_in(&gpu, my_player_position);
            }
        }

        let choice = [Action::Right, Action::Up, Action::Left, Action::Down];

        choice[hurdle % choice.len()]
    }
}

impl From<usize> for Game {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::HurdleRace,
            1 => Self::Archery,
            2 => Self::RollerSpeedSkating,
            3 => Self::Diving,
            _ => panic!("{value} is not a valid game"),
        }
    }
}

impl Game {
    fn dispatch(&self, gpu: String, reg: [Register; 7]) -> [i8; 4] {
        match self {
            Self::HurdleRace => [0, 0, 0, 0],
            Self::Archery => [0, 0, 0, 0],
            Self::RollerSpeedSkating => [0, 0, 0, 0],
            Self::Diving => [0, 0, 0, 0],
        }
    }
}

impl Input

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
        e.read_score();

        println!("{action}", action = e.solve());
    }
}
