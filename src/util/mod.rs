use std::path::Path;
use std::sync::Arc;

use handlebars::Handlebars;
use iron::{BeforeMiddleware, AroundMiddleware, AfterMiddleware};
use iron::middleware::Handler;
use iron::prelude::*;
use iron::status;
use iron::typemap;
use mime_guess;
use time::precise_time_ns;

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


pub struct TemplateMiddleware {
    engine: Arc<Handlebars>    
}

impl TemplateMiddleware {
    pub fn new(engine: Handlebars) -> Self {
        TemplateMiddleware { engine: Arc::new(engine) }
    }
}

pub struct TemplateEngine;
impl typemap::Key for TemplateEngine { type Value = Arc<Handlebars>; }

impl BeforeMiddleware for TemplateMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<TemplateEngine>(self.engine.clone());
        Ok(())
    }
}

pub struct ResponseTime;
impl typemap::Key for ResponseTime { type Value = u64; }

impl BeforeMiddleware for ResponseTime {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<ResponseTime>(precise_time_ns());
        Ok(())
    }
}

impl AfterMiddleware for ResponseTime {
    fn after(&self, req: &mut Request, resp: Response) -> IronResult<Response> {
        let delta = precise_time_ns() - *req.extensions.get::<ResponseTime>().unwrap();
        println!("Request took: {}ms", (delta as f64) / 1000000.0);
        Ok(resp)
    }
}
