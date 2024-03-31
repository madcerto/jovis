use std::fmt::Display;

use crate::scanner::error::ScannerError;

pub enum Error<'a> {
    IO(std::io::Error),
    Scanner(ScannerError<'a>),
    Parser((usize, &'a str)),
}

impl From<std::io::Error> for Error<'_> {
    fn from(value: std::io::Error) -> Self {
        Error::IO(value)
    }
}

impl From<ScannerError<'_>> for Error<'_> {
    fn from(value: ScannerError) -> Self {
        Error::Scanner(value)
    }
}

impl Display for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(e) => write!(f, "{e}"),
            Error::Scanner(ScannerError::Err(line, message)) | Error::Parser((line, message)) => {
                write!(f, "{message} at {line}")
            }
        }
    }
}
