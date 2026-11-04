use std::fmt;

pub(crate) type Error = Box<dyn std::error::Error>;
pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum ParseError {
    General
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::General => write!(f, "ERROR"),
        }
    }
}

impl std::error::Error for ParseError {}
