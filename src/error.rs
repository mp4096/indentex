#[derive(Debug)]
pub enum IndentexError {
    Io(std::io::Error),
    InvalidExtension,
    WalkError(ignore::Error),
}

impl From<ignore::Error> for IndentexError {
    fn from(e: ignore::Error) -> IndentexError {
        IndentexError::WalkError(e)
    }
}

impl From<std::io::Error> for IndentexError {
    fn from(e: std::io::Error) -> IndentexError {
        IndentexError::Io(e)
    }
}

impl std::fmt::Display for IndentexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::error::Error;
        match *self {
            IndentexError::Io(ref e) => write!(f, "{}", e.description()),
            IndentexError::InvalidExtension => write!(f, "not a valid indentex file"),
            IndentexError::WalkError(ref e) => write!(f, "{}", e.description()),
        }
    }
}

// LCOV_EXCL_START
#[cfg(test)]
mod tests {
    use super::IndentexError;

    #[test]
    fn from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "foo");
        assert_eq!(format!("{}", IndentexError::from(io_error)), "foo");
    }

    #[test]
    fn from_ignore_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "bar");
        let ignore_error = ignore::Error::Io(io_error);
        assert_eq!(format!("{}", IndentexError::from(ignore_error)), "bar");
    }

    #[test]
    fn invalid_extension() {
        assert_eq!(
            format!("{}", IndentexError::InvalidExtension),
            "not a valid indentex file"
        );
    }
}
// LCOV_EXCL_STOP
