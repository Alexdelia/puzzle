use crate::instance::{DEPOT, Instance};
use std::fmt;

pub type Tour = Vec<usize>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Fault {
	BadToken(String),
	EmptyTour(usize),
	DepotInTour(usize),
	UnknownSite { tour: usize, site: usize },
	VisitedTwice(usize),
	NotVisited(usize),
	OverCapacity { tour: usize, load: u32 },
}

impl fmt::Display for Fault {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Fault::BadToken(token) => write!(f, "bad token {token:?}"),
			Fault::EmptyTour(tour) => write!(f, "tour {tour} is empty"),
			Fault::DepotInTour(tour) => write!(f, "tour {tour} lists the depot"),
			Fault::UnknownSite { tour, site } => {
				write!(f, "tour {tour} visits unknown site {site}")
			}
			Fault::VisitedTwice(site) => write!(f, "site {site} visited twice"),
			Fault::NotVisited(site) => write!(f, "site {site} not visited"),
			Fault::OverCapacity { tour, load } => write!(f, "tour {tour} carries {load}"),
		}
	}
}

pub fn parse_tours(line: &str) -> Result<Vec<Tour>, Fault> {
	line.trim()
		.split(';')
		.enumerate()
		.map(|(tour, text)| {
			if text.is_empty() {
				return Err(Fault::EmptyTour(tour));
			}
			text.split(' ')
				.map(|token| {
					token
						.parse::<usize>()
						.map_err(|_| Fault::BadToken(token.to_string()))
				})
				.collect()
		})
		.collect()
}

pub fn write_tours(tours: &[Tour]) -> String {
	tours
		.iter()
		.map(|tour| {
			tour.iter()
				.map(usize::to_string)
				.collect::<Vec<_>>()
				.join(" ")
		})
		.collect::<Vec<_>>()
		.join(";")
}

pub fn tour_distance(instance: &Instance, tour: &[usize]) -> u32 {
	let stops = || tour.iter().copied();
	let legs = std::iter::once(DEPOT)
		.chain(stops())
		.zip(stops().chain(std::iter::once(DEPOT)));
	legs.map(|(from, to)| instance.distance(from, to)).sum()
}

pub fn check_tours(instance: &Instance, tours: &[Tour]) -> Result<(), Fault> {
	let mut visited = vec![false; instance.site_count()];
	for (index, tour) in tours.iter().enumerate() {
		let mut load = 0;
		for &site in tour {
			if site == DEPOT {
				return Err(Fault::DepotInTour(index));
			}
			if site >= instance.site_count() {
				return Err(Fault::UnknownSite { tour: index, site });
			}
			if visited[site] {
				return Err(Fault::VisitedTwice(site));
			}
			visited[site] = true;
			load += instance.sites[site].demand;
		}
		if load > instance.capacity {
			return Err(Fault::OverCapacity { tour: index, load });
		}
	}
	match visited.iter().skip(1).position(|&seen| !seen) {
		Some(missing) => Err(Fault::NotVisited(missing + 1)),
		None => Ok(()),
	}
}

pub fn score_tours(instance: &Instance, tours: &[Tour]) -> Result<u32, Fault> {
	check_tours(instance, tours)?;
	Ok(tours.iter().map(|tour| tour_distance(instance, tour)).sum())
}

pub fn score_line(instance: &Instance, line: &str) -> Result<u32, Fault> {
	score_tours(instance, &parse_tours(line)?)
}

#[cfg(test)]
mod tests {
	use super::*;

	const EXAMPLE: &str = "5\n10\n0 0 0 0\n1 0 10 3\n2 -10 10 3\n3 0 -10 3\n4 10 -10 3\n";

	fn score(line: &str) -> Result<u32, Fault> {
		score_line(&Instance::parse(EXAMPLE).unwrap(), line)
	}

	#[test]
	fn statement_valid_answers_use_rounded_legs() {
		assert_eq!(score("1 2 3;4"), Ok(10 + 10 + 22 + 10 + 14 + 14));
		assert_eq!(score("1 2;3 4"), Ok(68));
	}

	#[test]
	fn statement_invalid_answers() {
		assert_eq!(
			score("4 2 1 3"),
			Err(Fault::OverCapacity { tour: 0, load: 12 })
		);
		assert_eq!(score("1;2 4;3 2"), Err(Fault::VisitedTwice(2)));
		assert_eq!(score("1;3 4"), Err(Fault::NotVisited(2)));
	}

	#[test]
	fn malformed_answers() {
		assert_eq!(score("1 2;;3 4"), Err(Fault::EmptyTour(1)));
		assert_eq!(score("1 2;3 4;"), Err(Fault::EmptyTour(2)));
		assert_eq!(score("1  2;3 4"), Err(Fault::BadToken(String::new())));
		assert_eq!(score("1 x;3 4"), Err(Fault::BadToken("x".to_string())));
		assert_eq!(score("0 1 2;3 4"), Err(Fault::DepotInTour(0)));
		assert_eq!(
			score("1 2;3 5"),
			Err(Fault::UnknownSite { tour: 1, site: 5 })
		);
	}

	#[test]
	fn outer_whitespace_is_ignored() {
		assert_eq!(score("1 2;3 4\r"), Ok(68));
	}

	#[test]
	fn written_tours_parse_back() {
		let tours = vec![vec![1, 2], vec![4, 3]];
		assert_eq!(parse_tours(&write_tours(&tours)), Ok(tours));
	}

	#[test]
	fn every_test_instance_parses_and_echoes() {
		for dir in ["validator/test", "validator/submit"] {
			for entry in std::fs::read_dir(dir).unwrap() {
				let path = entry.unwrap().path();
				let text = std::fs::read_to_string(&path).unwrap();
				let instance = Instance::parse(&text).unwrap_or_else(|e| panic!("{path:?}: {e}"));
				assert_eq!(instance.input().trim_end(), text.trim_end(), "{path:?}");
			}
		}
	}
}
