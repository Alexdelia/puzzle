extern crate enum_like;
extern crate enum_like_derive;
extern crate enum_vec;

pub mod game;

pub mod priority;

pub mod io {
    pub const FILE_RESULT: &str = ".2048_results.out";
    pub mod read;
    pub mod write;
}

pub mod utils;
