pub mod compare;
pub mod interactive;
pub mod io_puzzle;
pub mod process;
pub mod suite;

pub use interactive::{Game, Interactive, Step};
pub use io_puzzle::IoPuzzle;
pub use suite::{Verdict, main};
