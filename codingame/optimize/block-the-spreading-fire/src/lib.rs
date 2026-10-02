pub mod game;
pub mod map;

pub use game::{Action, Fault, Game, NO_FIRE, SAFE, score_plan};
pub use map::{Cell, CellParams, Map};
