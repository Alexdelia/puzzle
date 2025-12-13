use std::collections::{HashMap, VecDeque};

use aocd::*;

struct FlagPoint {
	you: usize,
	out: usize,
	svr: usize,
	dac: usize,
	fft: usize,
}

fn parse(data: &str) -> (Vec<Vec<usize>>, FlagPoint) {
	let mut h = HashMap::<String, usize>::new();

	fn ensure_index(h: &mut HashMap<String, usize>, key: &str) -> usize {
		if let Some(&index) = h.get(key) {
			index
		} else {
			let index = h.len();
			h.insert(key.to_string(), index);
			index
		}
	}

	let data = data.trim().lines().collect::<Vec<&str>>();

	let mut graph = vec![Vec::new(); data.len() + 1];

	for line in data {
		let split: Vec<&str> = line.split(':').collect();

		let from = split[0].trim();
		let to: Vec<&str> = split[1].trim().split_whitespace().collect();

		let from_index = ensure_index(&mut h, from);
		assert!(from_index < graph.len());
		for &t in &to {
			let to_index = ensure_index(&mut h, t);
			assert!(to_index < graph.len());
			graph[from_index].push(to_index);
		}
	}

	let flag_point = FlagPoint {
		you: *h.get("you").expect("no 'you' node found"),
		out: *h.get("out").expect("no 'out' node found"),
		svr: *h.get("svr").expect("no 'svr' node found"),
		dac: *h.get("dac").expect("no 'dac' node found"),
		fft: *h.get("fft").expect("no 'fft' node found"),
	};

	(graph, flag_point)
}

fn count_path(graph: &[Vec<usize>], start: usize, end: usize) -> usize {
	let mut count = 0;

	let mut q = VecDeque::from([start]);
	while let Some(node) = q.pop_front() {
		for &neighbor in &graph[node] {
			if neighbor == end {
				count += 1;
				if count % 1_000 == 0 {
					dbg!(count);
				}
			} else {
				q.push_back(neighbor);
			}
		}
	}

	count
}

fn solve_p1(graph: &[Vec<usize>], f: &FlagPoint) -> usize {
	return count_path(graph, f.you, f.out);
}

fn solve_p2(graph: &[Vec<usize>], f: &FlagPoint) -> usize {
	let fft_start = count_path(graph, f.svr, f.fft);
	dbg!(fft_start);
	let dac_start = count_path(graph, f.svr, f.dac);
	dbg!(dac_start);

	let fft_to_dac = count_path(graph, f.fft, f.dac);
	dbg!(fft_to_dac);
	let dac_to_fft = count_path(graph, f.dac, f.fft);
	dbg!(dac_to_fft);

	let fft_end = count_path(graph, f.fft, f.out);
	dbg!(fft_end);
	let dac_end = count_path(graph, f.dac, f.out);
	dbg!(dac_end);

	let fft = fft_start * fft_to_dac * dac_end;
	let dac = dac_start * dac_to_fft * fft_end;
	fft + dac
}

fn solve(data: &str) -> (usize, usize) {
	let (graph, f) = parse(data);

	(solve_p1(&graph, &f), solve_p2(&graph, &f))
}

#[aocd(2025, 11)]
fn main() {
	let (p1, p2) = solve(&input!());
	println!("part 1:\t{p1}\npart 2:\t{p2}");
}

#[cfg(test)]
mod tests {
	use super::*;

	const TEST_DATA_PART_1: &str = r#"
aaa: you hhh
you: bbb ccc
bbb: ddd eee
ccc: ddd eee fff
ddd: ggg
eee: out
fff: out
ggg: out
hhh: ccc fff iii
iii: out
svr: aaa
dac: aaa
fft: aaa
"#;

	const TEST_DATA_PART_2: &str = r#"
svr: aaa bbb
aaa: fft
fft: ccc
bbb: tty
tty: ccc
ccc: ddd eee
ddd: hub
hub: fff
eee: dac
dac: fff
fff: ggg hhh
ggg: out
hhh: out
you: aaa
"#;

	#[test]
	fn test_example() {
		let p1_data = parse(TEST_DATA_PART_1);
		let p2_data = parse(TEST_DATA_PART_2);

		let expected = (5, 2);
		let got = (
			solve_p1(&p1_data.0, &p1_data.1),
			solve_p2(&p2_data.0, &p2_data.1),
		);
		assert_eq!(
			expected.0, got.0,
			"part 1\nexpected {}\ngot {}",
			expected.0, got.0
		);
		assert_eq!(
			expected.1, got.1,
			"part 2\nexpected {}\ngot\n{}",
			expected.1, got.1
		);
	}
}
