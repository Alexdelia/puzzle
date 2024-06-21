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

type ActionScore = [f64; ACTION_AMOUNT];

const PLAYER_NUMBER: usize = 3;
const GAME_AMOUNT: usize = 4;
const REG_SIZE: usize = 7;
const ACTION_AMOUNT: usize = 4;
const GAME_OVER: &str = "GAME_OVER";

struct Env {
    player_idx: usize,
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

            let Some(input) = Input::from_stdin() else {
                continue;
            };

            let game_score = game.dispatch(self, input);
            dbg!(&game_score);

            for (x, score) in game_score.iter().enumerate() {
                score_sum[x] += score;
            }
        }

        let highest_score = score_sum
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
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

impl From<char> for Action {
    fn from(value: char) -> Self {
        match value {
            'R' => Self::Right,
            'U' => Self::Up,
            'L' => Self::Left,
            'D' => Self::Down,
            _ => panic!("{value} is not a valid action"),
        }
    }
}

impl Input {
    fn from_stdin() -> Option<Self> {
        let line = Env::read_line();
        let inputs = line.split(' ').collect::<Vec<_>>();

        let gpu = inputs[0].trim().to_string();
        dbg!(&gpu);

        if gpu == GAME_OVER {
            return None;
        }

        let mut reg = [0; REG_SIZE];
        for i in 0..REG_SIZE {
            reg[i] = inputs[i + 1].trim().parse::<i32>().unwrap();
        }

        Some(Self { gpu, reg })
    }
}

impl Game {
    fn dispatch(&self, env: &Env, input: Input) -> ActionScore {
        match self {
            Self::HurdleRace => hurdle(env, input),
            Self::Archery => archery(env, input),
            Self::RollerSpeedSkating => [0.0; ACTION_AMOUNT],
            Self::Diving => diving(env, input),
        }
    }
}

fn hurdle(env: &Env, input: Input) -> ActionScore {
    let position = input.reg[env.player_idx] as usize;
    let hurdle = hurdle_in(&input.gpu, position);
    dbg!(hurdle);

    let mut action_score = [0.0; ACTION_AMOUNT];

    match hurdle {
        1 => {
            action_score[Action::Left as usize] = -2.0;
            action_score[Action::Down as usize] = -2.0;
            action_score[Action::Right as usize] = -2.0;
            action_score[Action::Up as usize] = 2.0;
        }
        2 => {
            action_score[Action::Left as usize] = 2.0;
            action_score[Action::Down as usize] = -1.0;
            action_score[Action::Right as usize] = -1.0;
            action_score[Action::Up as usize] = -1.0;
        }
        3 => {
            action_score[Action::Left as usize] = 1.0;
            action_score[Action::Down as usize] = 2.0;
            action_score[Action::Right as usize] = 0.0;
            action_score[Action::Up as usize] = 2.0;
        }
        _ => {
            action_score[Action::Left as usize] = 1.0;
            action_score[Action::Down as usize] = 2.0;
            action_score[Action::Right as usize] = 3.0;
            action_score[Action::Up as usize] = 2.0;
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

fn archery(env: &Env, input: Input) -> ActionScore {
    let mut action_score = [0.0; ACTION_AMOUNT];

    let wind = input
        .gpu
        .chars()
        .next()
        .unwrap()
        .to_digit(10)
        .unwrap_or_else(|| panic!("could not parse wind {}", input.gpu));

    let position = (input.reg[env.player_idx], input.reg[env.player_idx + 1]);
    let distance = euclidean_distance(position, (0, 0));

    for direction in [Action::Right, Action::Up, Action::Left, Action::Down] {
        let new_position = match direction {
            Action::Right => (position.0 + wind as i32, position.1),
            Action::Up => (position.0, position.1 - wind as i32),
            Action::Left => (position.0 - wind as i32, position.1),
            Action::Down => (position.0, position.1 + wind as i32),
        };
        let new_distance = euclidean_distance(new_position, (0, 0));

        let improvement = distance - new_distance;

        action_score[direction as usize] = improvement;
    }

    action_score
}

fn euclidean_distance(a: (i32, i32), b: (i32, i32)) -> f64 {
    (((b.0 - a.0) as f64).powi(2) + ((b.1 - a.1) as f64).powi(2)).sqrt()
}

fn diving(env: &Env, input: Input) -> ActionScore {
    let mut action_score = [0.0; ACTION_AMOUNT];

    let Some(goal) = input.gpu.chars().next() else {
        return action_score;
    };

    let goal = Action::from(goal);

    let mut my_score = 0;
    let mut op_score = [0; PLAYER_NUMBER - 1];

    {
        let mut op_score_index = 0;
        for i in 0..PLAYER_NUMBER {
            if i == env.player_idx {
                my_score = input.reg[i];
            } else {
                op_score[op_score_index] = input.reg[i];
                op_score_index += 1;
            }
        }
    }

    action_score[goal as usize] = if my_score > op_score[0] && my_score > op_score[1] {
        0.5
    } else if my_score > op_score[0] || my_score > op_score[1] {
        1.5
    } else {
        3.0
    };

    action_score
}

fn main() {
    let e = Env::init();

    loop {
        e.read_score();

        println!("{action}", action = e.solve());
    }
}
