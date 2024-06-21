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
    reg: [Register; REG_SIZE],
}

type ActionScore = [i32; ACTION_AMOUNT];

const PLAYER_NUMBER: usize = 3;
const GAME_AMOUNT: usize = 4;
const REG_SIZE: usize = 7;
const ACTION_AMOUNT: usize = 4;

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
        let mut score_sum = ActionScore::default();

        for i in 0..GAME_AMOUNT {
            let game = Game::from(i);

            let input = Input::from_stdin();

            let game_score = game.dispatch(self, input);
            dbg!(&game_score);

            for (x, score) in game_score.iter().enumerate() {
                score_sum[x] += score;
            }
        }

        let highest_score = score_sum
            .iter()
            .enumerate()
            .max_by_key(|x| x.1)
            .expect("No max score")
            .0;

        Action::from(highest_score)
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

impl From<usize> for Action {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Right,
            1 => Self::Up,
            2 => Self::Left,
            3 => Self::Down,
            _ => panic!("{value} is not a valid action"),
        }
    }
}

impl Input {
    fn from_stdin() -> Self {
        let line = Env::read_line();
        let inputs = line.split(' ').collect::<Vec<_>>();

        let gpu = inputs[0].trim().to_string();
        dbg!(&gpu);

        let mut reg = [0; REG_SIZE];
        for i in 0..REG_SIZE {
            reg[i] = inputs[i + 1].trim().parse::<i32>().unwrap();
        }

        Self { gpu, reg }
    }
}

impl Game {
    fn dispatch(&self, env: &Env, input: Input) -> ActionScore {
        match self {
            Self::HurdleRace => hurdle(env, input),
            Self::Archery => [0, 0, 0, 0],
            Self::RollerSpeedSkating => [0, 0, 0, 0],
            Self::Diving => [0, 0, 0, 0],
        }
    }
}

fn hurdle(env: &Env, input: Input) -> ActionScore {
    let position = input.reg[env.player_idx] as usize;
    let hurdle = hurdle_in(&input.gpu, position);

    let mut action_score = [0; ACTION_AMOUNT];

    match hurdle {
        1 => {
            action_score[Action::Left as usize] = -2;
            action_score[Action::Down as usize] = -2;
            action_score[Action::Right as usize] = -2;
            action_score[Action::Up as usize] = 2;
        }
        2 => {
            action_score[Action::Left as usize] = 2;
            action_score[Action::Down as usize] = -1;
            action_score[Action::Right as usize] = -1;
            action_score[Action::Up as usize] = -1;
        }
        3 => {
            action_score[Action::Left as usize] = 1;
            action_score[Action::Down as usize] = 2;
            action_score[Action::Right as usize] = 0;
            action_score[Action::Up as usize] = 2;
        }
        _ => {
            action_score[Action::Left as usize] = 1;
            action_score[Action::Down as usize] = 2;
            action_score[Action::Right as usize] = 3;
            action_score[Action::Up as usize] = 2;
        }
    }

    action_score
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
        e.read_score();

        println!("{action}", action = e.solve());
    }
}
