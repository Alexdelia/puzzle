use std::io::stdin;

fn main() {
    let mut line = String::new();
    stdin().read_line(&mut line).unwrap();
    let sum: i64 = line.split_whitespace().map(|n| n.parse::<i64>().unwrap()).sum();
    println!("{sum}");
}
