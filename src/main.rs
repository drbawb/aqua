#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate log;
#[macro_use] extern crate mime;
#[macro_use] extern crate serde_derive;

extern crate aqua_web;
extern crate conduit;
extern crate conduit_hyper;
extern crate crypto;
extern crate dotenv;
extern crate env_logger;
extern crate handlebars;
extern crate mime_guess;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate serde_json;
extern crate time;

mod controllers;
mod models;
mod schema;
mod util;
mod views;

use aqua_web::mw::{MultipartParser, Router};
use aqua_web::plug;
use conduit::Method;
use conduit_hyper::Server;
use dotenv::dotenv;


fn main() {
    // load configuration from .env
    dotenv().expect("must provide .env file, see README (TODO: haha jk)");
    env_logger::init().expect("could not initialize console logging");

    // these are application extensions which our controllers expect to be present
    let mut extensions = plug::Pipeline::new();
    extensions.register(util::db::DbMiddleware::new());
    extensions.register(util::template::TemplateMiddleware::new());

    // the main entry point into our application
    let mut router = Router::new();
    router.add_route(Method::Get, "/dash", controllers::dash::index);
    router.add_route(Method::Post, "/entries/upload", controllers::dash::submit);

    // the endpoint provides basic HTTP massaging before our router is invoked
    // with the current request data ...
    let mut endpoint = plug::Pipeline::new();
    endpoint.register(util::timer::plug);
    endpoint.register(util::try_file::TryFileMiddleware);
    endpoint.register(MultipartParser);
    endpoint.register(extensions);
    endpoint.register(router);

    Server::http("0.0.0.0:3000")
        .expect("could not start http server")
        .handle(endpoint)
        .expect("could not attach handler");
}
