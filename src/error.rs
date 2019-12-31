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
        match *self {
            IndentexError::Io(ref e) => e.fmt(f),
            IndentexError::InvalidExtension => write!(f, "not a valid indentex file"),
            IndentexError::WalkError(ref e) => e.fmt(f),
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
        let io_error_debug = format!("{:?}", io_error);
        let err = IndentexError::from(io_error);
        assert_eq!(format!("{}", err), "foo");
        assert_eq!(format!("{:?}", err), format!("Io({})", io_error_debug));
    }

    #[test]
    fn from_ignore_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "bar");
        let ignore_error = ignore::Error::Io(io_error);
        let ignore_error_debug = format!("{:?}", ignore_error);
        let err = IndentexError::from(ignore_error);
        assert_eq!(format!("{}", err), "bar");
        assert_eq!(
            format!("{:?}", err),
            format!("WalkError({})", ignore_error_debug)
        );
    }

    #[test]
    fn invalid_extension() {
        let err = IndentexError::InvalidExtension;
        assert_eq!(format!("{}", err), "not a valid indentex file");
        assert_eq!(format!("{:?}", err), "InvalidExtension");
    }
}
// LCOV_EXCL_STOP
