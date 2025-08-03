use std::error::Error as StdError;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    Unsupported(&'static str),
    IllegalParameter(&'static str),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Unsupported(msg) => write!(f, "unsupported: {msg}"),
            Error::IllegalParameter(msg) => write!(f, "illegal parameter: {msg}"), 
        }
    }
}

impl StdError for Error {}
