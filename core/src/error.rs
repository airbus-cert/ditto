use std::fmt::{Display, Formatter};
use std::str::Utf8Error;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum Error {
    NodeNotFound(String),
    ConditionsNotMeet(String),
    EngineNotFound(String),
    Utf8Error(Utf8Error),
    FromUtf8Error(FromUtf8Error),
    InvalidProbability(f32),
}

impl From<Utf8Error> for Error {
    fn from(e: Utf8Error) -> Error {
        Error::Utf8Error(e)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Error {
        Error::FromUtf8Error(e)
    }
}

pub type DittoResult<T> = Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NodeNotFound(name) => write!(f, "Node not found: {name}"),
            Error::ConditionsNotMeet(message) => write!(f, "Conditions not met: {}", message),
            Error::EngineNotFound(name) => write!(f, "Engine not found: {name}"),
            Error::Utf8Error(e) => write!(f, "UTF-8 error: {e}"),
            Error::FromUtf8Error(e) => write!(f, "From UTF-8 error: {e}"),
            Error::InvalidProbability(p) => write!(f, "Invalid probability: {:.0}%", p * 100.0),
        }
    }
}
