use std::error::Error as StdError;
use std::fmt;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ExtNotAvailable,
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ExtNotAvailable => "aqua extension unavailable",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ExtNotAvailable => write!(f, "The requested middleware extension has not yet been \
                                             loaded for this pipeline. Please ensure a middleware \
                                             providing the type is registered before any pipeline steps \
                                             which require it."),
        }
    }
}
