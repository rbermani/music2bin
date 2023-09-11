use core::result;
use thiserror::Error;
pub type Result<T> = result::Result<T, Error>;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("Unsupported Feature")]
    Unsupported,
    #[error("IO Kind {0}")]
    IoKind(String),
    #[error("Unnamed Error")]
    Unit,
    #[error("Data Out of Bounds")]
    OutofBounds,
    #[error("Missing Reader")]
    MissingReader,
    #[error("Parsing Error")]
    Parse,
    #[error("Encoding Error")]
    EncodingError,
    #[error("Decoding Error")]
    DecodingError,
    #[error("ParseIntError")]
    ParseInt(#[from] std::num::ParseIntError),
}
