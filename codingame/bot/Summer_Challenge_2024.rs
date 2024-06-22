use std::{fmt, io, str::FromStr};

#[derive(Debug)]
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

#[derive(Debug, Default, Clone, Copy)]
struct Score {
    total: u32,
    games: [GameMedal; GAME_AMOUNT],
}

#[derive(Debug, Default, Clone, Copy)]
struct GameMedal {
    gold: u8,
    silver: u8,
    bronze: u8,
}

type ActionScore = [f64; ACTION_AMOUNT];

#[derive(Debug, PartialEq, Eq)]
enum Rank {
    Gold,
    Silver,
    Bronze,
}

const PLAYER_NUMBER: usize = 3;
const GAME_AMOUNT: usize = 4;
const REG_SIZE: usize = 7;
const ACTION_AMOUNT: usize = 4;
const GAME_OVER: &str = "GAME_OVER";

struct Env {
    player_idx: usize,
    opponent_idx: [usize; PLAYER_NUMBER - 1],

    player_score: Score,
    opponent_score: [Score; PLAYER_NUMBER - 1],
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

        let mut opponent_idx = [0; PLAYER_NUMBER - 1];
        let mut opponent_idx_index = 0;
        for i in 0..PLAYER_NUMBER {
            if i != player_idx {
                opponent_idx[opponent_idx_index] = i;
                opponent_idx_index += 1;
            }
        }

        Self {
            player_idx,
            opponent_idx,

            player_score: Score::default(),
            opponent_score: [Score::default(); PLAYER_NUMBER - 1],
        }
    }

    fn read_score(&mut self) {
        let mut op_idx = 0;
        for i in 0..PLAYER_NUMBER {
            if i == self.player_idx {
                self.player_score = Self::read_line().parse().unwrap();
            } else {
                self.opponent_score[op_idx] = Self::read_line().parse().unwrap();
                op_idx += 1;
            }
        }
    }

    fn solve(&self) -> Action {
        let mut score_sum = ActionScore::default();

        for i in 0..GAME_AMOUNT {
            let game = Game::from(i);

            let Some(input) = Input::from_stdin() else {
                continue;
            };

            let (game_score, rank) = game.dispatch(self, input);
            let game_score = prioritize(self, game, game_score, rank);

            for (x, score) in game_score.iter().enumerate() {
                score_sum[x] += score
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

impl FromStr for Score {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut scores = s.trim().split(' ');

        let total = scores.next().unwrap().parse().unwrap();

        let mut games = [GameMedal::default(); GAME_AMOUNT];

        for i in 0..GAME_AMOUNT {
            games[i] = GameMedal {
                gold: scores.next().unwrap().parse().unwrap(),
                silver: scores.next().unwrap().parse().unwrap(),
                bronze: scores.next().unwrap().parse().unwrap(),
            };
        }

        Ok(Self { total, games })
    }
}

impl From<(f64, [f64; PLAYER_NUMBER - 1])> for Rank {
    fn from((score, op_score): (f64, [f64; PLAYER_NUMBER - 1])) -> Self {
        let win_one = score > op_score[0];
        let win_two = score > op_score[1];

        match (win_one, win_two) {
            (true, true) => Self::Gold,
            (true, false) | (false, true) => Self::Silver,
            (false, false) => Self::Bronze,
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
    fn dispatch(&self, env: &Env, input: Input) -> (ActionScore, Rank) {
        match self {
            Self::HurdleRace => hurdle(env, input),
            Self::Archery => archery(env, input),
            Self::RollerSpeedSkating => ([0.0; ACTION_AMOUNT], Rank::Bronze),
            Self::Diving => diving(env, input),
        }
    }
}

impl GameMedal {
    fn total(&self) -> u32 {
        (self.gold as u32) * 3 + (self.silver as u32)
    }
}

fn prioritize(env: &Env, game: Game, mut action_score: ActionScore, rank: Rank) -> ActionScore {
    let total = env.player_score.games[game as usize].total();

    eprintln!("g:{game:?} r:{rank:?} t:{total}");
    /*
    for (i, score) in action_score.iter().enumerate() {
        eprintln!("{action}:\t{score:.2}", action = Action::from(i));
    }
    */

    match rank {
        Rank::Gold => {
            for score in action_score.iter_mut() {
                *score /= 4.0
            }
        }
        Rank::Silver => {
            for score in action_score.iter_mut() {
                *score /= 2.0
            }
        }
        Rank::Bronze => (),
    }

    /*
    eprintln!("\nAfter rank adjustment");
    for (i, score) in action_score.iter().enumerate() {
        eprintln!("{action}:\t{score:.2}", action = Action::from(i));
    }
    */

    if env
        .player_score
        .games
        .iter()
        .all(|game| game.total() >= total)
    {
        for score in action_score.iter_mut() {
            *score *= 8.0;
        }
    }

    // eprintln!("\nAfter total adjustment");
    for (i, score) in action_score.iter().enumerate() {
        eprintln!("{action}:\t{score:.2}", action = Action::from(i));
    }

    action_score
}

fn hurdle(env: &Env, input: Input) -> (ActionScore, Rank) {
    let position = input.reg[env.player_idx] as usize;
    let hurdle = hurdle_in(&input.gpu, position);

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

    let rank = Rank::from((
        position as f64,
        [
            input.reg[env.opponent_idx[0]] as f64,
            input.reg[env.opponent_idx[1]] as f64,
        ],
    ));

    (action_score, rank)
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

fn archery(env: &Env, input: Input) -> (ActionScore, Rank) {
    let mut action_score = [0.0; ACTION_AMOUNT];

    let wind = input
        .gpu
        .chars()
        .next()
        .unwrap()
        .to_digit(10)
        .unwrap_or_else(|| panic!("could not parse wind {}", input.gpu));

    let position = (
        input.reg[env.player_idx * 2],
        input.reg[(env.player_idx * 2) + 1],
    );
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

    let rank = Rank::from((
        distance,
        [
            euclidean_distance(
                (
                    input.reg[env.opponent_idx[0] * 2],
                    input.reg[(env.opponent_idx[0] * 2) + 1],
                ),
                (0, 0),
            ),
            euclidean_distance(
                (
                    input.reg[env.opponent_idx[1] * 2],
                    input.reg[(env.opponent_idx[1] * 2) + 1],
                ),
                (0, 0),
            ),
        ],
    ));

    (action_score, rank)
}

fn euclidean_distance(a: (i32, i32), b: (i32, i32)) -> f64 {
    (((b.0 - a.0) as f64).powi(2) + ((b.1 - a.1) as f64).powi(2)).sqrt()
}

fn diving(env: &Env, input: Input) -> (ActionScore, Rank) {
    let mut action_score = [0.0; ACTION_AMOUNT];

    let Some(goal) = input.gpu.chars().next() else {
        return (action_score, Rank::Bronze);
    };

    let goal = Action::from(goal);

    action_score[goal as usize] = 3.0;

    let rank = Rank::from((
        input.reg[env.player_idx] as f64,
        [
            input.reg[env.opponent_idx[0]] as f64,
            input.reg[env.opponent_idx[1]] as f64,
        ],
    ));

    (action_score, rank)
}

fn main() {
    let mut e = Env::init();

    loop {
        e.read_score();

        println!("{action}", action = e.solve());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hurdle_in() {
        assert_eq!(hurdle_in("....#....", 0), 4);
        assert_eq!(hurdle_in("....#....", 1), 3);
        assert_eq!(hurdle_in("....#....", 2), 2);
        assert_eq!(hurdle_in("....#....", 3), 1);
        assert_eq!(hurdle_in("....#....", 4), 64);
        assert_eq!(hurdle_in("....#....", 5), 64);
    }

    #[test]
    fn test_euclidean_distance() {
        assert_eq!(euclidean_distance((0, 0), (0, 0)), 0.0);
        assert_eq!(euclidean_distance((1, 0), (0, 0)), 1.0);
        assert_eq!(euclidean_distance((0, 1), (0, 0)), 1.0);
        assert_eq!(euclidean_distance((1, 1), (0, 0)), 1.4142135623730951);
        assert_eq!(euclidean_distance((2, 2), (0, 0)), 2.8284271247461903);
        assert_eq!(euclidean_distance((-2, -2), (0, 0)), 2.8284271247461903);
    }

    #[test]
    fn test_from_str_score() {
        let score = "42 1 2 3 4 5 6 7 8 9 10 11 12".parse::<Score>().unwrap();

        assert_eq!(score.total, 42);

        assert_eq!(score.games[Game::HurdleRace as usize].gold, 1);
        assert_eq!(score.games[Game::HurdleRace as usize].silver, 2);
        assert_eq!(score.games[Game::HurdleRace as usize].bronze, 3);

        assert_eq!(score.games[Game::Archery as usize].gold, 4);
        assert_eq!(score.games[Game::Archery as usize].silver, 5);
        assert_eq!(score.games[Game::Archery as usize].bronze, 6);

        assert_eq!(score.games[Game::RollerSpeedSkating as usize].gold, 7);
        assert_eq!(score.games[Game::RollerSpeedSkating as usize].silver, 8);
        assert_eq!(score.games[Game::RollerSpeedSkating as usize].bronze, 9);

        assert_eq!(score.games[Game::Diving as usize].gold, 10);
        assert_eq!(score.games[Game::Diving as usize].silver, 11);
        assert_eq!(score.games[Game::Diving as usize].bronze, 12);

        let score = "0 0 0 0 0 0 0 0 0 0 0 0 0\n".parse::<Score>().unwrap();

        assert_eq!(score.total, 0);

        assert_eq!(score.games[Game::HurdleRace as usize].gold, 0);
        assert_eq!(score.games[Game::HurdleRace as usize].silver, 0);
        assert_eq!(score.games[Game::HurdleRace as usize].bronze, 0);

        assert_eq!(score.games[Game::Archery as usize].gold, 0);
        assert_eq!(score.games[Game::Archery as usize].silver, 0);
        assert_eq!(score.games[Game::Archery as usize].bronze, 0);

        assert_eq!(score.games[Game::RollerSpeedSkating as usize].gold, 0);
        assert_eq!(score.games[Game::RollerSpeedSkating as usize].silver, 0);
        assert_eq!(score.games[Game::RollerSpeedSkating as usize].bronze, 0);

        assert_eq!(score.games[Game::Diving as usize].gold, 0);
        assert_eq!(score.games[Game::Diving as usize].silver, 0);
        assert_eq!(score.games[Game::Diving as usize].bronze, 0);
    }

    #[test]
    fn test_parse_rank() {
        let rank = Rank::from((1.0, [0.0, 0.0]));
        assert_eq!(rank, Rank::Gold);

        let rank = Rank::from((1.0, [2.0, 0.0]));
        assert_eq!(rank, Rank::Silver);

        let rank = Rank::from((1.0, [2.0, 3.0]));
        assert_eq!(rank, Rank::Bronze);
    }

    #[test]
    fn test_game_medal_total() {
        let medal = GameMedal {
            gold: 1,
            silver: 2,
            bronze: 3,
        };

        assert_eq!(medal.total(), 1 * 3 + 2);

        let medal = GameMedal {
            gold: 0,
            silver: 0,
            bronze: 0,
        };

        assert_eq!(medal.total(), 0);

        let medal = GameMedal {
            gold: 42,
            silver: 8,
            bronze: 16,
        };

        assert_eq!(medal.total(), 42 * 3 + 8);
    }
}
