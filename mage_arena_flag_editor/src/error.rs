use std::error::Error as StdError;
use std::fmt::{Display, Formatter};

/// Mage Arena Flag Editor error.
#[derive(Debug)]
pub enum Error {
    /// An attempt to access a necessary resource failed.
    AccessFailure(String),

    /// An unexpected value was encountered.
    UnexpectedValue(String),

    /// An error occurred in an external dependency
    External(String)
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AccessFailure(msg) => write!(f, "access failure: {msg}"),
            Error::UnexpectedValue(msg) => write!(f, "unexpected value: {msg}"),
            Error::External(err) => write!(f, "external error: {err}"),
        }
    }
}

impl StdError for Error {}
