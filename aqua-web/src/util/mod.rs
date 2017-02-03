use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::Read;

use conduit::{self, Response, WriteBody};
use hyper::status::StatusCode;

pub type Result<T> = ::std::result::Result<T, MemeFailure>;

fn status(code: StatusCode) -> (u32, &'static str) {
    (code.to_u16() as u32, code.canonical_reason().unwrap_or(""))
}

pub fn response<R: Read + Send + 'static>(code: StatusCode, buf: R) -> Response {
    Response {
        status: status(StatusCode::Ok),
        headers: HashMap::new(),
        body: Box::new(buf),
    }
}

#[derive(Debug)]
pub enum MemeFailure {
    NoLulz(String),
}

impl fmt::Display for MemeFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
        match *self {
            MemeFailure::NoLulz(ref msg) => write!(f,"{}", msg)?,
        }

        Ok(())
    }
}

impl Error for MemeFailure {
    fn description(&self) -> &str {
        match *self {
            MemeFailure::NoLulz(ref msg) => &msg[..]
        }
    }

    fn cause(&self) -> Option<&Error> { Some(self) }
}
