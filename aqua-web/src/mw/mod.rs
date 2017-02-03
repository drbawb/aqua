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

/// A segment represents a visitor which will receive a request
/// along with a partial response and transform them in some way.
///
/// Segments cooperate with each other by way of specifying some 
/// `Outcome` which tells the stack how it should proceed.
///
pub trait Segment {
    fn invoke(&self, req: &mut Request) -> Outcome;
}

impl<F> Segment for F 
where F: Fn(&mut Request) -> Outcome {
    fn invoke(&self, req: &mut Request) -> Outcome { (*self)(req) }
}

/// This stack represents a request pipeline which will be handled
/// by the conduit web gateway.
///
/// Requests are routed through the stack in order, segments of the
/// stack must implement the Segment trait.
///
/// Note that stack segments are applied cooperatively, that is to say
/// that any given segment may halt the pipeline. Furthermore these segments
/// are visited in the order they were added. As such the programmer must consider
/// the order of dependencies in their stack at the time it is created.
/// 
pub struct Stack {
    handlers: Vec<Box<Segment + Send + Sync>>,
}

impl Stack {
    /// Creates an empty stack which will only yield empty responses
    pub fn new() -> Self {
        Stack { handlers: vec![] }
    }

    /// Pushes a segment onto the end of the stack
    pub fn add_segment<S: Segment+Send+Sync+'static>(&mut self, segment: S) {
        self.handlers.push(Box::new(segment))
    }
}


impl Handler for Stack {
    fn call(&self, req: &mut Request) -> Result<Response, Box<Error + Send>> {
        for segment in &self.handlers {
            match segment.invoke(req) {
                Outcome::Proceed() => println!("segment proceed ..."),
                Outcome::Halt(status, msg)  => return Ok(halt_stack(status,msg)),
                Outcome::Complete(response) => return Ok(response),
            }
        }

        Ok(Response {
            status: (200, "unimplemented"),
            headers: HashMap::new(),
            body: Box::new(Cursor::new("nice job! empty stack GET!")),
        })
    }
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
