use std::convert::From;
use std::fmt::Display;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    InvalidPath { path: PathBuf },
    IOError { err: io::Error },
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPath { path } => {
                write!(f, "Path '{}' is invalid", path.to_string_lossy())
            }
            Self::IOError { err } => {
                write!(f, "Encountered an unexpected IO Error: '{}'.", err)
            }
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IOError { err: error }
    }
}
