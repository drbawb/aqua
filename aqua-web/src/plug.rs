use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, Cursor, Write};
use std::path::Path;

use conduit::{Handler, Request, Response};

type HeaderMap = HashMap<String, Vec<String>>;

// TODO: use this as a generic parameter for type safety?
// e.g: see hyper's "Fresh" response
/// Represents the state of the connection
#[derive(Debug,Eq,PartialEq)]
enum RespState {
    Waiting,
    Chunked,
    Sent,
}

/// A plug represents any method which can be applied to a connection.
///
/// Plugs may use the connection to generate a response. Other plugs
/// are purely advisory and merely provide additional context to the connection.
///
/// Plugs may use the request's extension storage to store these plugins in
/// a type-safe manner.
///
/// NOTE: Plugs *must* be safe to be moved across threads, that is to say they 
/// are `Send + Sync` in Rust parlance.
///
pub trait Plug: Send+Sync+'static {
    fn call(&self, conn: &mut Conn);
}

/// The connection includes a response scratch-buffer, response headers,
/// and the incoming request which needs to be handled.
///
/// This structure provides utility functions for generating responses
/// and cooperating with other middleware in the pipeline.
pub struct Conn<'r> {
    is_halting: bool,
    state: RespState,
    status_code: u16,
    headers: HeaderMap,
    resp: Cursor<Vec<u8>>,
    
    req: &'r mut Request,
    callbacks: Vec<Box<Plug>>,
}

impl<'r> Conn<'r> {
    fn new(req: &'r mut Request) -> Self {
        Conn {
            state:        RespState::Waiting,
            status_code:  200,
            headers:      HashMap::new(),
            resp:         Cursor::new(vec![]),
           
            is_halting:  false,
            req:         req,
            callbacks:   vec![],
        }
    }

    /// Halts the current pipeline, further plugs will not be run.
    pub fn halt(&mut self) { self.is_halting = true; }

    /// Registers a callback to be fired before the request is sent
    ///
    /// The response body cursor is rewound to the beginning before each
    /// individual callback is fired.
    ///
    pub fn register_before_send<P: Plug>(&mut self, callback: P) {
        self.callbacks.push(Box::new(callback));
    }

    pub fn send_file<P: AsRef<Path>>(&mut self, _status: u16, path: P) 
    where P: ::std::fmt::Debug {
        match File::open(&path) {
            // TODO: unnecessary copy
            Ok(ref mut file) => {
                self.status_code = 200;
                self.state = RespState::Sent;
                io::copy(file, &mut self.resp);
            },

            Err(msg) => {
                warn!("Could not open file {:?} for response", path);
                self.resp.write(b"unexpected server error: could not open file.")
                    .expect("could not write resp to buffer");

                self.status_code = 500;
                self.state = RespState::Sent;
            }
        }
    }

    /// Writes a response to this `Conn`'s buffer and sets the connection state
    /// to RespState::Sent so that further writes will fail ...
    pub fn send_resp(&mut self, _status: u16, body: &str) {
        assert_eq!(self.state, RespState::Waiting);
        
        self.resp.write(body.as_bytes())
            .expect("could not write resp to buffer");

        self.state = RespState::Sent;
    }

    /// Borrows the underlying request object immutably
    pub fn req(&self) -> &Request { &*self.req }

    /// Borrows the underlying request object mutably
    pub fn req_mut(&mut self) -> &mut Request { self.req }
}

/// The hyperbipeline is a series of `Plug`s which can be used
/// to modify the connection as it moves through the system.
///
/// Plugs are called in the order they were added, if the end of
/// the pipeline is reached without a response being set the pipeline
/// will return an `HTTP 500` error w/ the body `no handler found`.
///
/// As such it is imperative that the last plug in your pipeline
/// has a catch-all response to prevent this message from being shown
/// to the end-user.
///
pub struct Pipeline {
    stack: Vec<Box<Plug>>,
}

impl Pipeline {
    /// Creates an empty request-handling pipeline
    pub fn new() -> Self {
        Pipeline { stack: vec![] }
    }

    /// Connects a plug to the end of the pipeline
    pub fn register<P: Plug>(&mut self, plug: P) { 
        self.stack.push(Box::new(plug));
    }
}

impl Handler for Pipeline {
    // TODO: don't panic at unexepcted end of chain ...
    /// A pipeline is handled by running it to completion
    ///
    /// Afterwards any callbacks scheduled at runtime are then run in
    /// the order they were registered.
    ///
    /// Finally if a response has been generated at this point: it is
    /// returned to the client.
    ///
    fn call(&self, req: &mut Request) -> Result<Response, Box<Error + Send>> {
        let mut conn = Conn::new(req);
        for plug in &self.stack { 
            plug.call(&mut conn); 
            if conn.is_halting { break; }
        }

        // TODO: mem::swap dance to take ownership of the callbacks
        let mut callbacks = vec![];
        ::std::mem::swap(&mut conn.callbacks, &mut callbacks);
        for callback in &callbacks { callback.call(&mut conn); }

        // TODO: finish other response modes
        // generate the response based on what the user asked us to do
        match conn.state {
            RespState::Waiting => panic!("pipeline did not generate a response?"),
            RespState::Chunked => panic!("chunked responses not supported yet"),
            RespState::Sent => {
                conn.resp.set_position(0);
                let response = Response {
                    // TODO: status code => canonical reason per RFC
                    status: (200, "OK"),
                    headers: conn.headers,
                    body: Box::new(conn.resp),
                };

                Ok(response)
            },
        }
    }
}

impl Plug for Pipeline {
    /// Pipelines can also be plugged together, this is useful for
    /// creating large chunks of pipelines which need to be swapped out
    /// based on the incoming request.
    ///
    /// For e.g: you might have two different pipelines for handling requests
    /// for HTML from a browser agent, versus JSON from an HTTP client.
    ///
    /// If a pipeline is invoked as a plug: it simply iterates through its
    /// internal plugs and calls each one in the order they were added.
    fn call(&self, conn: &mut Conn) {
        for plug in &self.stack { plug.call(conn); }
    }
}

impl<F> Plug for F 
where F: Send + Sync + 'static + Fn(&mut Conn) {
    fn call(&self, conn: &mut Conn) { (*self)(conn) }
}
