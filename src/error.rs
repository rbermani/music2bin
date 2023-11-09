use core::result;
use mulib::error::Error as MuLibErr;
use muxml::error::Error as MuError;
use repl_rs::Error as ReplError;
use thiserror::Error;

pub type Result<T> = result::Result<T, Error>;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("Repl crate error {0}")]
    Repl(ReplError),
    #[error("MusicXML crate Error {0}")]
    MuCrate(MuError),
    #[error("MuLib Crate Error {0}")]
    MuLib(MuLibErr),
    #[error("Data Out of Bounds")]
    OutofBounds,
    #[error("Unsupported Feature")]
    Unsupported,
    #[error("IO Kind {0}")]
    IoKind(String),
    #[error("Unnamed Error")]
    Unit,
    #[error("Missing Reader")]
    MissingReader,
    #[error("Parsing Error")]
    Parse,
    #[error("Encoding Error")]
    Encoding,
    #[error("Item Already Exists")]
    ItemExists,
    #[error("Not Initialized")]
    NotInitialized,
    #[error("Decoding Error")]
    Decoding,
    #[error("ParseIntError")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("StrumParse {0}")]
    Strum(#[from] strum::ParseError),
}

impl From<MuError> for Error {
    fn from(e: MuError) -> Self {
        Error::MuCrate(e)
    }
}

impl From<ReplError> for Error {
    fn from(e: ReplError) -> Self {
        Error::Repl(e)
    }
}

impl From<MuLibErr> for Error {
    fn from(e: MuLibErr) -> Self {
        Error::MuLib(e)
    }
}
