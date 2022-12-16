use std::collections::{BinaryHeap, HashMap, HashSet};
use std::env::args;

#[derive(Clone)]
struct Node {
    valve: String,
    time: i8,
    released: u16,
    visited: HashSet<String>,
    opened_all: bool,
}

impl Eq for Node {}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.priority() == other.priority()
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority().cmp(&other.priority())
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Node {
    fn new(
        valve: String,
        time: i8,
        released: u16,
        visited: HashSet<String>,
        pressured: &HashSet<String>,
    ) -> Self {
        Self {
            valve,
            time,
            released,
            opened_all: pressured.is_subset(&visited),
            visited,
        }
    }

    fn explore(&mut self, d: &HashMap<String, (u8, Vec<String>)>) {
        if !self.visited.contains(&self.valve) {
            self.released += self.time as u16 * d[&self.valve].0 as u16;
            self.visited.insert(self.valve.clone());
            self.time -= 1;
        }
        self.time -= 1;
    }

    fn priority(&self) -> u16 {
        self.released
    }
}

fn main() {
    if args().len() != 2 {
        panic!("usage: cargo run --bin y2022d16 <input file>");
    }

    let lines: Vec<String> = std::fs::read_to_string(args().nth(1).unwrap())
        .unwrap()
        .lines()
        .map(|s| s.to_string())
        .collect();

    let mut d: HashMap<String, (u8, Vec<String>)> = HashMap::new();

    for l in lines {
        let l = l
            .chars()
            .filter(|c| c.is_ascii_uppercase() || c.is_numeric() || c.is_ascii_whitespace())
            .collect::<String>();
        let l = l
            .split(' ')
            .filter(|s| !s.is_empty())
            .skip(1)
            .collect::<Vec<&str>>();

        let mut v = Vec::new();
        for i in l.iter().skip(2) {
            v.push(i.to_string());
        }

        d.insert(l[0].to_string(), (l[1].parse().unwrap(), v));
    }

    let mut q: BinaryHeap<Node> = BinaryHeap::from(vec![Node::new(
        "AA".to_string(),
        30,
        0,
        HashSet::new(),
        &d.keys().cloned().collect(),
    )]);

    let mut max = 0;
    let mut i: usize = 0;

    while let Some(cur) = q.pop() {
        i += 1;
        if i > 1_000 {
            print!("{}\t{}\r", q.len(), cur.released);
            i = 0;
        }

        if cur.time <= 0 || cur.opened_all {
            if cur.released > max {
                max = cur.released;
                println!("\nnew max:\t{}", max)
            }
            continue;
        }
        for v in &d[&cur.valve].1 {
            let mut n = cur.clone();
            n.valve = v.clone();
            n.explore(&d);
            q.push(n);
        }
    }

    println!("{}", max);
}
