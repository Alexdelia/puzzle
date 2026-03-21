use std::io;

const SOLUTION: &str = r#"
flag:output
"#;

fn read_line() -> String {
	let mut input_line = String::new();
	io::stdin().read_line(&mut input_line).unwrap();
	input_line.trim().to_string()
}

fn main() {
	let n = read_line().parse::<usize>().unwrap();

	let flag = read_line();

	for _ in 1..n {
		read_line();
	}

	for solution in SOLUTION.trim().lines() {
		let p = solution.split(':').collect::<Vec<_>>();
		assert_eq!(p.len(), 2);

		if p[0].trim() == flag {
			println!("{s}", s = p[1].trim());
			return;
		}
	}

	unreachable!();
}
