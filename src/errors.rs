use std::convert::From;
use std::fmt::Display;
use std::io;
use std::path::PathBuf;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum Error {
    InvalidPath { path: PathBuf },
    IOError { err: io::Error },
    NoKey { err: String },
    PGPError { err: pgp::errors::Error },
    UnsupportedMessageType { err: String },
    GeneralError { err: String },
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
            Self::NoKey { err } => {
                write!(f, "No suitable key found: '{}'.", err)
            }
            Self::PGPError { err } => {
                write!(f, "Encountered an unexpected PGP Error: '{}'.", err)
            }
            Self::UnsupportedMessageType { err } => {
                write!(f, "{}", err)
            }
            Self::GeneralError { err } => {
                write!(f, "{}", err)
            }
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IOError { err: error }
    }
}

impl From<pgp::errors::Error> for Error {
    fn from(error: pgp::errors::Error) -> Self {
        Error::PGPError { err: error }
    }
}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Self {
        Error::GeneralError { err: error.to_string() }
    }
}
