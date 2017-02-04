use std::collections::HashMap;
pub use std::io::Cursor;
pub use conduit::{Request, Response, WriteBody};

/// Send an `200 OK` response w/ mime: `TEXT/HTML`
pub fn respond_html<B>(body: B) -> Response 
where B: WriteBody+Send+'static {
    Response {
        status: (200, "OK"),
        headers: HashMap::new(),
        body: Box::new(body),
    }
}
