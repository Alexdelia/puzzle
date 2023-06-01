use std::cmp::min;
use std::collections::{HashSet, VecDeque};
use std::io::stdin;
use std::str::FromStr;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

type Ressource = u32;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
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
    my_ant: Ressource,
    opp_ant: Ressource,
}

struct Env {
    cell: Vec<Cell>,
    init_crystal: Ressource,
    remain_crystal: Ressource,
    remain_ant: Ressource,
    n_base: usize,
    my_base: Vec<usize>,
    opp_base: Vec<usize>,
    my_score: Ressource,
    opp_score: Ressource,
    my_ant: Ressource,
    opp_ant: Ressource,
    beacon: HashSet<usize>,
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

#[inline]
fn gain_type(gain: (Ressource, Ressource), r#type: Option<CellType>) -> Ressource {
    match r#type {
        Some(CellType::Crystal) => gain.0,
        Some(CellType::Egg) => gain.1,
        _ => gain.0 + gain.1,
    }
}

impl Env {
    fn new() -> Self {
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let n_cell = parse_input!(buf, usize);

        let mut cell: Vec<Cell> = Vec::with_capacity(n_cell);
        let mut init_crystal = 0;

        for _ in 0..n_cell {
            let mut buf = String::new();
            stdin().read_line(&mut buf).unwrap();

            cell.push(buf.parse::<Cell>().unwrap());
            init_crystal += cell.last().unwrap().ressource;
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
            init_crystal,
            remain_crystal: init_crystal,
            remain_ant: 0,
            n_base,
            my_base,
            opp_base,
            my_score: 0,
            opp_score: 0,
            my_ant: 0,
            opp_ant: 0,
            beacon: HashSet::new(),
        }
    }

    fn update(&mut self) {
        self.remain_crystal = 0;
        self.remain_ant = 0;

        for i in 0..self.cell.len() {
            let mut buf = String::new();
            stdin().read_line(&mut buf).unwrap();

            let mut sw = buf.split_whitespace();

            self.cell[i].ressource = sw.next().unwrap().parse::<Ressource>().unwrap();
            match self.cell[i].r#type {
                CellType::Crystal => self.remain_crystal += self.cell[i].ressource,
                CellType::Egg => self.remain_ant += self.cell[i].ressource,
                _ => (),
            }

            self.cell[i].my_ant = sw.next().unwrap().parse::<Ressource>().unwrap();
            self.my_ant += self.cell[i].my_ant;
            self.cell[i].opp_ant = sw.next().unwrap().parse::<Ressource>().unwrap();
            self.opp_ant += self.cell[i].opp_ant;
        }
    }

    fn output(&self) {
        let mut output = String::new();

        for b in &self.beacon {
            output.push_str(&format!("BEACON {b};"));
        }

        if output.is_empty() {
            output.push_str("WAIT;");
        }

        let in_game_ant = self.my_ant + self.opp_ant;
        println!(
            "{output} MESSAGE 💎 {}%  |  🐜 {}%  |  🧙 {}% - 👤 {}%",
            self.remain_crystal * 100 / self.init_crystal,
            self.remain_ant * 100 / in_game_ant,
            self.my_ant * 100 / in_game_ant,
            self.opp_ant * 100 / in_game_ant,
        );
    }

    /// return (Crystal, Ant)
    fn gain(&self, beacon: &HashSet<usize>, ant: Ressource) -> (Ressource, Ressource) {
        let weakest = ant / beacon.len() as Ressource;
        let mut crystal = 0;
        let mut egg = 0;

        // does not yet calculate ant battle

        for i in beacon {
            match self.cell[*i].r#type {
                CellType::Crystal => crystal += min(self.cell[*i].ressource, weakest),
                CellType::Egg => egg += min(self.cell[*i].ressource, weakest),
                _ => (),
            }
        }

        (crystal, egg)
    }

    // return (Ressource, path (+ Ressource found))
    fn beacon_flood(&self, r#type: Option<CellType>) -> Vec<(usize, HashSet<usize>)> {
        let mut queue: VecDeque<Vec<usize>> = VecDeque::new();
        let mut visited = vec![false; self.cell.len()];

        for i in self.beacon.iter() {
            queue.push_back(vec![*i]);
            visited[*i] = true;
        }

        let mut found: Vec<(usize, HashSet<usize>)> = Vec::new();

        while let Some(path) = queue.pop_front() {
            let index = *path.last().unwrap();

            if self.cell[index].ressource > 0
                && r#type
                    .as_ref()
                    .map_or(true, |t| self.cell[index].r#type == *t)
            {
                found.push((index, path.into_iter().collect()));
                continue;
            }

            if !found.is_empty() && path.len() >= found[0].1.len() {
                break;
            }

            for i in 0..6 {
                if let Some(i) = self.cell[index].neighbor[i] {
                    if !visited[i] {
                        let mut path = path.clone();
                        path.push(i);
                        queue.push_back(path);
                        visited[i] = true;
                    }
                }
            }
        }

        found
    }

    fn best_beacon_list(
        &self,
        beacon: Vec<HashSet<usize>>,
        only_calc: Option<CellType>,
        force: bool,
    ) -> Option<HashSet<usize>> {
        let current_gain = gain_type(self.gain(&self.beacon, self.my_ant), only_calc);

        let mut best_gain: Option<Ressource> = None;
        let mut best_beacon: Option<HashSet<usize>> = None;

        for b in beacon {
            // calculate gain of self.beacon + b
            let tmp_beacon: HashSet<usize> = self.beacon.union(&b).cloned().collect();
            let gain = gain_type(self.gain(&tmp_beacon, self.my_ant), only_calc);

            if (gain >= current_gain || force)
                && (best_gain.is_none() || gain >= best_gain.unwrap())
            {
                best_gain = Some(gain);
                best_beacon = Some(b.clone());
            }
        }

        best_beacon
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

    fn compute_beacon(&mut self, clear_beacon: bool) {
        if clear_beacon {
            self.beacon.clear();
        }

        let mut gain = 0;

        while gain >= gain_type(self.gain(&self.beacon, self.my_ant), None) {
            let mut beacon: Vec<HashSet<usize>> = Vec::new();

            for b in self.beacon_flood(None) {
                // add to beacon, union of b.1 and ressource_group of b.0
                beacon.push(b.1);
                beacon.last_mut().unwrap().extend(self.ressource_group(b.0));
            }

            // get best beacon list gain
            self.best_beacon_list(beacon, None, false);
        }
    }
}

fn main() {
    let mut env = Env::new();

    loop {
        env.update();

        env.compute_beacon(true);

        env.output();
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
