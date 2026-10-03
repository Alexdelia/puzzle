pub mod instance;
pub mod tours;

pub use instance::{DEPOT, Instance, Site};
pub use tours::{
	Fault, Tour, check_tours, parse_tours, score_line, score_tours, tour_distance, write_tours,
};
