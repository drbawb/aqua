use std::collections::HashMap;
use std::error::Error;
use std::io::Cursor;

pub use self::route::Route;
pub use self::router::Router;

use conduit::{Handler, Request, Response};

pub mod regexp;
pub mod route;
pub mod router;

pub enum Outcome { 
    Proceed(),
    Halt(u16, String),
    Complete(Response),
}

pub trait Wrapper {
    fn around(self, handler: Box<Segment>) -> Box<Segment>;
}

pub struct Chain {
    handler: Option<Box<Segment>>,
}

impl Chain {
    pub fn new<H: Segment>(handler: H) -> Self {
        Chain {
            handler: Some(Box::new(handler)),
        }
    }

    pub fn with<W: Wrapper>(&mut self, handler: W) -> &mut Self {
        let mut inner = self.handler.take().unwrap();
        inner = handler.around(inner);
        self.handler = Some(inner);
        self
    }
}

impl Handler for Chain {
    // TODO: don't panic at unexepcted end of chain ...
    fn call(&self, req: &mut Request) -> Result<Response, Box<Error + Send>> {
        match self.handler.as_ref().unwrap().invoke(req) {
            Outcome::Complete(resp)   => Ok(resp),
            Outcome::Halt(status,msg) => Ok(halt_stack(status, msg)),
            _ => panic!("middleware stack did not terminate... dats bad.")
        }
    }
}


/// A segment represents a visitor which will receive a request
/// along with a partial response and transform them in some way.
///
/// Segments cooperate with each other by way of specifying some 
/// `Outcome` which tells the stack how it should proceed.
///
pub trait Segment: Send + Sync + 'static {
    fn invoke(&self, req: &mut Request) -> Outcome;
}

impl<F> Segment for F 
where F: Send + Sync + 'static + Fn(&mut Request) -> Outcome {
    fn invoke(&self, req: &mut Request) -> Outcome { (*self)(req) }
}

/// Immediately generates a simple response, this should only be
/// used if processing cannot continue in an ordinary fashion ...
fn halt_stack(status: u16, reason: String) -> Response {
    Response {
        // TODO: status code => canonical reason per RFC
        status: (status as u32, "no reason given"),
        headers: HashMap::new(),
        body: Box::new(Cursor::new(reason.into_bytes())),
    }
}
