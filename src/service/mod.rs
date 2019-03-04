pub mod error;
pub mod storage;

pub type Result<T> = std::result::Result<T, error::Error>;
