extern crate bit_vec;
extern crate bit_vec_serde;
extern crate ring;
extern crate serde;
extern crate streaming_iterator;

pub mod digest;
#[macro_use]
pub mod error;
pub mod level;
#[macro_use]
pub mod proof;
pub mod tree;

pub type Array = Vec<u8>;
pub type Result<T> = std::result::Result<T, error::Error>;
