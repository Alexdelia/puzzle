pub mod board;
pub mod log;
pub mod probe;
pub mod record;
pub mod run;

pub use board::{render, town_char};
pub use log::{Log, LoggedTown, parse, read};
pub use probe::Probe;
pub use record::{ActivePair, Claim, PairPay, Replay, Totals, TurnRecord, ZoneLook};
pub use run::{run, run_probed};

pub type Pair = (usize, usize);

pub fn pair_name(pair: Pair) -> String {
	format!("{}->{}", town_char(pair.0), town_char(pair.1))
}
