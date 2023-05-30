use std::collections::VecDeque;
use std::io::stdin;
use std::str::FromStr;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

type Ressource = u32;
type Ant = u32;

#[derive(Debug, Eq, PartialEq)]
enum CellType {
    None,
    Egg,
    Crystal,
}

#[derive(Debug, Eq, PartialEq)]
struct Cell {
    r#type: CellType,
    ressource: Ressource,
    neighbor: [Option<usize>; 6],
    my_ant: Ant,
    opp_ant: Ant,
}

enum Action {
    Beacon(usize, usize),
    Line(usize, usize, usize),
    // Wait,
    // Message(String),
}

struct Env {
    cell: Vec<Cell>,
    n_base: usize,
    my_base: Vec<usize>,
    opp_base: Vec<usize>,
    my_score: Ressource,
    opp_score: Ressource,
    my_ant: Ant,
    opp_ant: Ant,
    action: Vec<Action>,
}

impl FromStr for CellType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Self::None),
            "1" => Ok(Self::Egg),
            "2" => Ok(Self::Crystal),
            _ => Err(()),
        }
    }
}

impl FromStr for Cell {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut i = s.split_whitespace();

        Ok(Self {
            r#type: i.next().unwrap().parse::<CellType>().unwrap(),
            ressource: i.next().unwrap().parse::<Ressource>().unwrap(),
            neighbor: [
                i.next().unwrap().parse::<usize>().ok(),
                i.next().unwrap().parse::<usize>().ok(),
                i.next().unwrap().parse::<usize>().ok(),
                i.next().unwrap().parse::<usize>().ok(),
                i.next().unwrap().parse::<usize>().ok(),
                i.next().unwrap().parse::<usize>().ok(),
            ],
            my_ant: 0,
            opp_ant: 0,
        })
    }
}

impl Env {
    fn new() -> Self {
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let n_cell = parse_input!(buf, usize);

        let mut cell: Vec<Cell> = Vec::with_capacity(n_cell);

        for _ in 0..n_cell {
            let mut buf = String::new();
            stdin().read_line(&mut buf).unwrap();

            cell.push(buf.parse::<Cell>().unwrap());
        }

        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let n_base = parse_input!(buf, usize);

        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let my_base = buf
            .split_whitespace()
            .map(|s| s.parse::<usize>().unwrap())
            .collect();

        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let opp_base = buf
            .split_whitespace()
            .map(|s| s.parse::<usize>().unwrap())
            .collect();

        Env {
            cell,
            n_base,
            my_base,
            opp_base,
            action: Vec::new(),
            my_score: 0,
            opp_score: 0,
            my_ant: 0,
            opp_ant: 0,
        }
    }

    fn update(&mut self, clear_action: bool) {
        if clear_action {
            self.action.clear();
        }

        for i in 0..self.cell.len() {
            let mut buf = String::new();
            stdin().read_line(&mut buf).unwrap();

            let mut sw = buf.split_whitespace();

            self.cell[i].ressource = sw.next().unwrap().parse::<Ressource>().unwrap();
            self.cell[i].my_ant = sw.next().unwrap().parse::<Ant>().unwrap();
            self.my_ant += self.cell[i].my_ant;
            self.cell[i].opp_ant = sw.next().unwrap().parse::<Ant>().unwrap();
            self.opp_ant += self.cell[i].opp_ant;
        }
    }

    #[inline]
    fn act_beacon(&mut self, index: usize, strength: usize) {
        self.action.push(Action::Beacon(index, strength));
    }

    #[inline]
    fn act_line(&mut self, index1: usize, index2: usize, strength: usize) {
        self.action.push(Action::Line(index1, index2, strength));
    }

    fn act_output(&self) {
        let mut output = String::new();

        for i in 0..self.action.len() {
            match self.action[i] {
                Action::Beacon(index, strength) => {
                    output.push_str(&format!("BEACON {index} {strength};"))
                }
                Action::Line(index1, index2, strength) => {
                    output.push_str(&format!("LINE {index1} {index2} {strength};"))
                }
            }
        }

        if output.is_empty() {
            println!("WAIT");
        } else {
            println!("{output}");
        }
    }

    /// find closest ressource from given index in hexagonal grid
    /// return index of cell and distance
    fn closest_ressource(&self, index: usize) -> Option<(usize, usize)> {
        let mut queue = VecDeque::new();
        let mut visited = vec![false; self.cell.len()];

        queue.push_back((index, 0));
        visited[index] = true;

        while let Some((index, distance)) = queue.pop_front() {
            if self.cell[index].ressource > 0 {
                return Some((index, distance));
            }

            for i in 0..6 {
                if let Some(i) = self.cell[index].neighbor[i] {
                    if !visited[i] {
                        queue.push_back((i, distance + 1));
                        visited[i] = true;
                    }
                }
            }
        }

        None
    }

    fn ressource_group(&self, index: usize) -> Vec<usize> {
        let mut queue = VecDeque::new();
        let mut visited = vec![false; self.cell.len()];
        let mut group = Vec::new();

        queue.push_back(index);
        visited[index] = true;

        while let Some(index) = queue.pop_front() {
            if self.cell[index].ressource > 0 {
                group.push(index);
            }

            for i in 0..6 {
                if let Some(i) = self.cell[index].neighbor[i] {
                    if !visited[i] && self.cell[i].ressource > 0 {
                        queue.push_back(i);
                        visited[i] = true;
                    }
                }
            }
        }

        group
    }

    fn depleted_group(&self, group: &[usize]) -> bool {
        for index in group {
            if self.cell[*index].ressource > 0 {
                return false;
            }
        }

        true
    }
}

fn main() {
    let mut env = Env::new();

    let mut group: Vec<usize> = Vec::new();

    loop {
        let depleted = env.depleted_group(&group);

        env.update(depleted);

        if depleted {
            group.clear();

            if let Some((index, _)) = env.closest_ressource(env.my_base[0]) {
                env.act_line(env.my_base[0], index, 1);
                group.push(index);

                for index in env.ressource_group(index) {
                    env.act_beacon(index, 1);
                    group.push(index);
                }
            }
        }

        env.act_output();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cell_parse() {
        assert_eq!(
            "0 0 1 2 3 4 5 6".parse::<Cell>(),
            Ok(Cell {
                r#type: CellType::None,
                ressource: 0,
                neighbor: [Some(1), Some(2), Some(3), Some(4), Some(5), Some(6)],
                my_ant: 0,
                opp_ant: 0
            })
        );

        assert_eq!(
            "1 0 -1 -1 8 -1 -1 -1".parse::<Cell>(),
            Ok(Cell {
                r#type: CellType::Egg,
                ressource: 0,
                neighbor: [None, None, Some(8), None, None, None],
                my_ant: 0,
                opp_ant: 0
            })
        );

        assert_eq!(
            "2 42 13 14 15 2 1 0".parse::<Cell>(),
            Ok(Cell {
                r#type: CellType::Crystal,
                ressource: 42,
                neighbor: [Some(13), Some(14), Some(15), Some(2), Some(1), Some(0)],
                my_ant: 0,
                opp_ant: 0
            })
        );
    }
}
