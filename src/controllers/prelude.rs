pub use std::io::Cursor;
pub use conduit::{Request, Response, WriteBody};

use std::collections::HashMap;

use aqua_web::mw::forms::{MultipartForm, FormField, SavedFile};

/// Send an `200 OK` response w/ mime: `TEXT/HTML`
pub fn respond_html<B>(body: B) -> Response 
where B: WriteBody+Send+'static {
    Response {
        status: (200, "OK"),
        headers: HashMap::new(),
        body: Box::new(body),
    }
}

/// Extracts a file from a multipart form if the key exists & it is a file
pub fn extract_file(form: &mut MultipartForm, field: &str) -> Option<SavedFile> {
    match form.entries.remove(field) {
        Some(FormField::File(file)) => Some(file),
        Some(_) => { warn!("file expected, but got string"); None },
        None    => { warn!("file expected, but not present"); None },
    }
}
