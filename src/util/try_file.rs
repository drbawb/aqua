use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use aqua_web::mw::{Outcome, Segment, Wrapper};
use conduit::{Request, Response};
use mime_guess;

/// This middleware attempts to serve a static file from
/// the `./static` directory if it is available.
///
/// If the file is found an early response is generated; otherwise
/// the request is processed by the original handler.
pub struct TryFile;

struct TryHandler {
    next: Box<Segment>
}

impl Segment for TryHandler {
    fn invoke(&self, req: &mut Request) -> Outcome {
        // NOTE: limits lexical borrow of `req`
        let try_path = {
            if req.path().contains("./") || req.path().contains("../") {
                panic!("hey man, that's just not cool... {}", req.path());
            }

            PathBuf::from(format!("./static{}", req.path()))
        };

        println!("checking path: {:?}", try_path);
        let file_exists = try_path.exists() && try_path.is_file();
        match file_exists {
            true  => {
                let mime_type = mime_guess::guess_mime_type(&try_path);
                let mut headers = HashMap::new();
                headers.insert("content-type".to_string(), vec![mime_type.to_string()]);

                Outcome::Complete(Response {
                    status: (200, "OK"),
                    headers: headers,
                    body: Box::new(File::open(try_path).unwrap())
                })

                // Ok(Response::with((status::Ok, try_path))
                //             .set(mime_type))
            },

            false => self.next.invoke(req),
        }
    }
}

impl Wrapper for TryFile {
    fn around(self, inner_mw: Box<Segment>) -> Box<Segment> {
        let handler = TryHandler { next: inner_mw };
        Box::new(handler)
    }
}
