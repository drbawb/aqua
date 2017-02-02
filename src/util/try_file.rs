use std::path::Path;

use iron::AroundMiddleware;
use iron::middleware::Handler;
use iron::prelude::*;
use iron::status;
use mime_guess;

/// This middleware attempts to serve a static file from
/// the `./static` directory if it is available.
///
/// If the file is found an early response is generated; otherwise
/// the request is processed by the original handler.
pub struct TryFile;

struct TryHandler {
    next: Box<Handler>
}

impl Handler for TryHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        // info!("looking for file: {:?}", req.url.path());
        let root_path = Path::new("./static").to_owned();
        let try_path  = req.url.path()
                               .iter()
                               .fold(root_path, |path, component| { path.join(component) });
     
        let file_exists = try_path.exists() && try_path.is_file();
        match file_exists {
            true  => {
                let mime_type = mime_guess::guess_mime_type(&try_path);
                Ok(Response::with((status::Ok, try_path))
                            .set(mime_type))
            },

            false => self.next.handle(req),
        }
    }
}

impl AroundMiddleware for TryFile {
    fn around(self, inner_mw: Box<Handler>) -> Box<Handler> {
        let handler = TryHandler { next: inner_mw };
        Box::new(handler)
    }
}
