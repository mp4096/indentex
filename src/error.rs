use ignore;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum IndentexError {
    Io(io::Error),
    InvalidExtension,
    WalkError(ignore::Error),
    TranspileError,
}

impl From<ignore::Error> for IndentexError {
    fn from(e: ignore::Error) -> IndentexError {
        IndentexError::WalkError(e)
    }
}

impl From<io::Error> for IndentexError {
    fn from(e: io::Error) -> IndentexError {
        IndentexError::Io(e)
    }
}

impl fmt::Display for IndentexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        match *self {
            IndentexError::Io(ref e) => write!(f, "{}", e.description()),
            IndentexError::InvalidExtension => write!(f, "not a valid indentex file"),
            IndentexError::WalkError(ref e) => write!(f, "{}", e.description()),
            IndentexError::TranspileError => write!(f, "invalid indentex text"),
        }
    }
}
