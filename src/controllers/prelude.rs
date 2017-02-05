pub use std::io::Cursor;
pub use conduit::{Request, Response, WriteBody};

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use aqua_web::mw::forms::{MultipartForm, FormField, SavedFile};
use crypto::digest::Digest;
use crypto::sha2::Sha256;

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

pub fn hash_file<P: AsRef<Path>>(path: P) -> Option<String> {
    println!("file was pretty coo, gonna hash it");
    let mut buf = vec![];

    info!("path exists? {}",  (path.as_ref()).exists());
    info!("path is file? {}", (path.as_ref()).is_file());

    File::open(path)
         .and_then(|mut file| { file.read_to_end(&mut buf) })
         .map(|size| {

        println!("read {} bytes", size);
        let mut digest = Sha256::new();
        digest.input(&mut buf);
        digest.result_str()
    }).ok()
}
